//! The parser converts lexed tokens into an abstract syntax tree.

use crate::errors::ParserError;
use crate::interpreter::{AsID, ID, MemoryInterface, SVTable, Variable};
use crate::lexer::{Lexer, SpannedToken, Token};
use anyhow::{Context, Result, bail};
use std::collections::VecDeque;
use std::{env, fs::File, io::Read, path::PathBuf, rc::Rc};

pub mod astnode;
pub use astnode::*;
mod tests;

/// The parser converts a sequence of tokens into an Abstract Syntax Tree (AST).
pub struct Parser {
    tokens: VecDeque<SpannedToken>,
    path: PathBuf,
}

impl Parser {
    /// Creates a new parser over `tokens`.
    pub fn new(tokens: Vec<SpannedToken>) -> Result<Self> {
        match env::current_dir() {
            Ok(path) => Ok(Self {
                tokens: tokens.into(),
                path,
            }),
            Err(_) => bail!("could not open working directory."),
        }
    }

    /// Sets the current working directory, used to set relative location of imports.
    pub fn set_pwd(&mut self, path: PathBuf) {
        self.path = path;
    }

    /// Peek at the next token. Returns `Err` on EOF.
    fn peek(&self) -> Result<&Token> {
        self.tokens
            .front()
            .map(SpannedToken::kind)
            .context("unexpected EOF")
    }

    /// Peek `n` positions ahead. Returns `Err` on EOF.
    fn peek_n(&self, n: usize) -> Result<&Token> {
        self.tokens
            .get(n)
            .map(SpannedToken::kind)
            .context("unexpected EOF")
    }

    /// Peek at the line number of the next token. Returns `Err` on EOF.
    fn peek_line(&self) -> Result<usize> {
        self.tokens
            .front()
            .map(SpannedToken::line)
            .context("unexpected EOF")
    }

    /// Get and return the next token.
    fn next(&mut self) -> Option<Token> {
        self.tokens.pop_front().map(Token::from)
    }

    /// Throws an error if the next token is not `expected`.
    /// Line attribution is handled by the statement-level context wrap in `parse_with_imports`,
    /// so the bail message itself does not need to repeat the line.
    fn expect(&mut self, expected: Token) -> Result<()> {
        match self.next() {
            Some(token) if token == expected => Ok(()),
            Some(token) => {
                bail!("found {token:?}, expected {expected:?}");
            }
            _ => {
                bail!("unexpected EOF")
            }
        }
    }

    /// Returns the precedence level of an operator (higher number = higher precedence)
    fn get_precedence(op: &Token) -> u8 {
        match op {
            Token::LogicalOr => 1,
            Token::LogicalAnd => 2,
            Token::LogicalEq | Token::LogicalNeq => 3,
            Token::LogicalL | Token::LogicalLe | Token::LogicalG | Token::LogicalGe => 4,
            Token::Add | Token::Sub => 5,
            Token::Mul | Token::Div | Token::Floor => 6,
            Token::Pow => 7,
            Token::Increment | Token::Decrement => 8,
            _ => 0,
        }
    }

    /// Parses until a block end is found. (EOF, return, etc.)
    pub fn parse(&mut self) -> Result<Rc<ASTNode>> {
        // parse without any imports
        self.parse_with_imports(vec![])
    }

    /// Parses all tokens with hidden module imports.
    pub fn parse_with_imports(&mut self, imports: Vec<Rc<ASTNode>>) -> Result<Rc<ASTNode>> {
        let mut statements = vec![];
        while let Ok(token) = self.peek() {
            if *token == Token::BlockEnd {
                // consume block ends and expect endline
                self.next();
                self.expect(Token::Endl)?;
                break;
            } else if *token == Token::Else {
                // also counts as a block end for conditionals
                break;
            } else if *token == Token::Endl {
                // consume endlines
                self.next();
            } else {
                // capture line at statement boundary so all parser errors get a line
                // attached uniformly via context, regardless of which inner bail fires
                let line = self.peek_line()?;
                statements.push(self.parse_statement().context(format!("on line {line}"))?);
            }
        }
        Ok(ASTNode::Block([imports, statements].concat()).into())
    }

    /// Parses a statement.
    ///
    /// Branches that correspond to a distinct statement kind wrap their failure in a
    /// `ParserError` variant, which preserves the underlying `anyhow::Error` as a source. The
    /// remaining branches (expressions, breaks) propagate their `anyhow::Error`
    /// directly since they have no statement-level decoration to add.
    fn parse_statement(&mut self) -> Result<Rc<ASTNode>, ParserError> {
        let peek = self.peek()?;
        match peek {
            Token::Import => self.parse_import().map_err(ParserError::Import),
            Token::Let => self.parse_decl_var().map_err(ParserError::Declaration),
            Token::If => self.parse_cond().map_err(ParserError::Conditional),
            Token::Function => self.parse_decl_fn().map_err(ParserError::FunctionDecl),
            Token::Struct => self.parse_decl_struct().map_err(ParserError::StructDecl),
            Token::While => self.parse_while().map_err(ParserError::While),
            Token::Identifier(_) => Ok(self.parse_expr(None)?),
            Token::Return => self.parse_return().map_err(ParserError::Return),
            Token::Break => Ok(self.parse_break()?),
            Token::ParenOpen => Ok(self.parse_expr(None)?),
            _ => Err(anyhow::anyhow!("expected statement, found {peek:?}").into()),
        }
    }

    /// Parses breaks.
    fn parse_break(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::Break)?;
        Ok(ASTNode::Break.into())
    }

    /// Given a module path, returns the full parsed AST.
    /// This function should only be used internally, as an encapsulation of the file read, lex, and
    /// parse logic that's used.
    fn parse_module(&self, mut path: PathBuf) -> Result<Rc<ASTNode>> {
        // read the file to be imported to a buffer
        let mut buffer = String::new();
        File::open(&path)
            .context("failed to create file buffer")?
            .read_to_string(&mut buffer)
            .context("failed to read file data")?;

        // lex buffer into tokens
        let tokens = Lexer::default()
            .lex_spanned(buffer)
            .context("failed to lex imported file")?;

        // create a parser and point it to the file's parent directory temporarily
        let mut parser = Self::new(tokens)?;
        path.pop();
        let temp = parser.path.clone();
        parser.set_pwd(path.clone());

        // parse the module
        let body = parser.parse().context("failed to parse module body")?;

        // reset old parser working directory
        parser.set_pwd(temp);

        Ok(body)
    }

    /// Parses import statements.
    fn parse_import(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::Import)?;
        if let Some(Token::Str(path)) = self.next() {
            // get full path
            let path = self.path.join(PathBuf::from(path));
            if !path.exists() {
                bail!("module not found ({})", path.display());
            }

            // check if alias is provided
            let mut alias = None;
            if let Token::As = self.peek()? {
                // consume keyword
                self.next();

                // attempt to find alias as an identifier
                if let Token::Identifier(alias_id) = self.peek()? {
                    // if an identifier is found, it becomes our alias
                    alias = Some(alias_id.as_id());
                    self.next();
                } else {
                    // if something other than an identifier is provided, this import is malformed
                    bail!("expected identifier as alias, found {:?}", self.peek());
                }
            }

            // parse file & get AST
            let body = self
                .parse_module(path.clone())
                .context(format!("parsing import '{}'", path.display()))?;

            // TODO: more extensive import tests. will require *lots* of files, though

            Ok(ASTNode::Module {
                alias,
                path: Some(path),
                body,
            }
            .into())
        } else {
            bail!("expected path after import");
        }
    }

    /// Parses a conditional expression.
    fn parse_cond(&mut self) -> Result<Rc<ASTNode>> {
        // consume if token
        self.expect(Token::If)?;

        // get if expression and if body block
        let condition = self.parse_expr(None).context("failed to parse condition")?;
        let if_body = self.parse().context("failed to parse if-body")?;

        // process else body block, if present
        let mut else_body = ASTNode::Block(vec![]).into();
        if let Ok(Token::Else) = self.peek() {
            self.next();
            else_body = self.parse().context("failed to parse else-body")?;
        }

        Ok(ASTNode::Conditional {
            condition,
            if_body,
            else_body,
        }
        .into())
    }

    /// Parses a list index.
    fn parse_index(&mut self, target: Rc<ASTNode>) -> Result<Rc<ASTNode>> {
        // if id is found, parse index value
        self.expect(Token::BracketOpen)?;
        let index = self
            .parse_expr(Some(Token::BracketClose))
            .context("failed to parse list index")?;

        // return newly made index node
        Ok(ASTNode::Index { target, index }.into())
    }

    /// Parses a deref operation.
    fn parse_deref(&mut self, parent: Rc<ASTNode>) -> Result<Rc<ASTNode>> {
        self.expect(Token::Dot)?;

        // expect an identifier after the dot
        let child = match self.next() {
            Some(Token::Identifier(id)) => ASTNode::Identifier(ID::new_sym(id)).into(),
            Some(token) => bail!("expected identifier after '.', found {token:?}"),
            None => bail!("unexpected EOF after '.'"),
        };

        Ok(ASTNode::Deref { parent, child }.into())
    }

    /// Parses a while loop.
    fn parse_while(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::While)?;
        Ok(ASTNode::Loop {
            condition: self
                .parse_expr(None)
                .context("failed to parse loop condition")?,
            body: self.parse().context("failed to parse loop body")?,
        }
        .into())
    }

    /// Parses the creation of structure instances, which are simply function calls with an extra
    /// keyword tacked on to the front.
    fn parse_struct_instance(&mut self) -> Result<Rc<ASTNode>> {
        // consume new keyword
        self.expect(Token::New)?;

        // parse as a function call. if none is found, bail
        let stmnt = self.parse_expr(None)?;
        if let ASTNode::FunctionCall { .. } = &*stmnt {
            Ok(stmnt)
        } else {
            bail!("failed to parse instantiation of structure")
        }
    }

    /// Parses a structure declaration.
    fn parse_decl_struct(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::Struct)?;
        match self.next() {
            Some(Token::Identifier(sym)) => {
                // expect endl before struct body
                self.expect(Token::Endl)?;

                // parse body in its entirety
                let body = self.parse()?;

                // find default fields
                let mut default_fields = vec![];
                let ASTNode::Block(body_nodes) = &*body else {
                    unreachable!();
                };
                for node in body_nodes {
                    match &**node {
                        // if the member is a structure variable, add an owned value
                        ASTNode::Declare { target, value } => {
                            // if this field is literal, add it, bail otherwise
                            let ASTNode::Identifier(id) = &**target else {
                                bail!("invalid default field '{target:?}'");
                            };
                            default_fields
                                .push((id, Variable::Owned(ASTNode::inner_to_owned(value))));
                        }

                        // if the member is a function, add a reference to it
                        ASTNode::Function { id, .. } => {
                            default_fields.push((id, Variable::Function(node.clone())))
                        }

                        other => {
                            bail!("unexpected structure field: {other:?}")
                        }
                    }
                }

                // create a new variable table and instantiate default values
                let mut template = SVTable::default();
                for (target, value) in default_fields {
                    // get the first value in the interned path
                    let id = *target.to_path_symbolic().first().unwrap();

                    // add it to the table
                    template.declare(id, value, 0)?;
                }

                // create structure & provide the new template
                let node = ASTNode::Struct {
                    id: ID::new_sym(sym),
                    body,
                    template,
                };
                Ok(node.into())
            }
            other => {
                bail!("expected identifier, found {other:?}")
            }
        }
    }

    /// Parses a function declaration.
    fn parse_decl_fn(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::Function)?;
        let next = self.next();
        if let Some(Token::Identifier(sym)) = next {
            // gather arguments
            let mut arguments = vec![];
            while let Token::Identifier(arg) = self.peek()? {
                arguments.push(ID::new_sym(*arg));
                self.next();
            }

            // consume block start
            self.expect(Token::BlockStart)?;

            Ok(ASTNode::Function {
                id: ID::new_sym(sym),
                body: self.parse().context("failed to parse function body")?,
                arguments,
            }
            .into())
        } else {
            bail!("expected identifier, found {next:?}");
        }
    }

    /// Parses a function call.
    fn parse_call_fn(&mut self, target: Rc<ASTNode>) -> Result<Rc<ASTNode>> {
        // parse arguments
        self.expect(Token::ParenOpen)?;
        let mut args = vec![];
        loop {
            match self.peek()? {
                // If this is a close paren, arguments are over
                Token::ParenClose => {
                    self.next();
                    break;
                }
                // Otherwise, evaluate this argument and add it to the list
                _ => {
                    args.push(
                        self.parse_expr(Some(Token::Comma))
                            .context("failed to parse argument")?,
                    );
                }
            }
        }

        Ok(ASTNode::FunctionCall {
            target,
            arguments: args,
        }
        .into())
    }

    /// Parses a return statement.
    fn parse_return(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::Return)?;
        Ok(ASTNode::Return(self.parse_expr(None)?).into())
    }

    /// Parses assignment to any target.
    fn parse_assignment(&mut self, target: Rc<ASTNode>) -> Result<Rc<ASTNode>> {
        // parse value
        self.expect(Token::Equal)?;
        let value = self.parse_expr(None)?;

        // return node
        Ok(ASTNode::Assign { target, value }.into())
    }

    /// Parses a variable declaration.
    fn parse_decl_var(&mut self) -> Result<Rc<ASTNode>> {
        // parse id and value
        self.expect(Token::Let)?;
        let target = self.parse_expr(Some(Token::Equal))?;
        let value = self.parse_expr(None)?;

        // return node
        Ok(ASTNode::Declare { target, value }.into())
    }

    /// Parses expressions, such as operators, indices, function calls, etc.
    fn parse_expr(&mut self, expect: Option<Token>) -> Result<Rc<ASTNode>> {
        // evaluate primary value
        let mut primary = match self.peek()? {
            Token::ParenOpen => {
                self.next();
                self.parse_expr(Some(Token::ParenClose))
                    .context("failed to parse parenthesised expression")?
            }
            _ => self
                .parse_primary()
                .context("failed to parse primary expression")?,
        };

        // keep looping until we've found the largest possible primary
        loop {
            // if we hit the expected token, break
            if let Some(ref token) = expect
                && self.peek()? == token
            {
                // run the token through `expect` to provide an error message if it doesn't
                // match what we think that it should be
                self.expect(expect.unwrap())?;
                break;
            }

            // match operator with precedence handling
            primary = match self.peek() {
                // operators
                Ok(token) if token.is_operator() => {
                    let op = self.next().unwrap(); // safety: peek
                    let rhs = self
                        .parse_operator(Self::get_precedence(&op))
                        .context(format!("failed to parse operator: '{op}'"))?;
                    ASTNode::Op {
                        lhs: primary,
                        op,
                        rhs,
                    }
                    .into()
                }

                // function calls
                Ok(Token::ParenOpen) => self.parse_call_fn(primary)?,

                // indexes
                Ok(Token::BracketOpen) => self.parse_index(primary)?,

                // deref operations
                Ok(Token::Dot) => self.parse_deref(primary)?,

                // assignments
                Ok(Token::Equal) => self.parse_assignment(primary)?,

                // postfix increment/decrement (x++)
                // other unaries are handled in `parse_primary`
                Ok(Token::Increment) | Ok(Token::Decrement) => {
                    let op = self.next().unwrap(); // safety: peek
                    ASTNode::UnaryOp {
                        target: primary,
                        op,
                    }
                    .into()
                }

                // break for all others
                Ok(Token::Endl) | Ok(Token::BlockStart) => {
                    self.next();
                    break;
                }
                _ => {
                    break;
                }
            };
        }

        Ok(primary)
    }

    /// Parses operators with precedence climbing
    fn parse_operator(&mut self, min_precedence: u8) -> Result<Rc<ASTNode>> {
        // Expand left-hand side first
        let mut left = match self.peek()? {
            Token::ParenOpen => {
                self.next();
                self.parse_expr(Some(Token::ParenClose))
                    .context("failed to parse parenthesised expression")?
            }
            _ => self
                .parse_primary()
                .context("failed to parse left operand")?,
        };

        // Handle high precedence operations
        // (Such as deref, function calls, inc/dec, and indexing)
        loop {
            match self.peek()? {
                Token::Dot => {
                    left = self.parse_deref(left)?;
                }
                Token::ParenOpen => {
                    left = self.parse_call_fn(left)?;
                }
                Token::BracketOpen => {
                    left = self.parse_index(left)?;
                }
                Token::Increment | Token::Decrement => {
                    let op = self.next().unwrap(); // safety: peek
                    left = ASTNode::UnaryOp { target: left, op }.into();
                }
                _ => break,
            }
        }

        while let Ok(next) = self.peek() {
            // If the precedence of the `peek`ed token is lower than the minimum, or it isn't an
            // operator at all, break
            // This means we've gotten to a point where the next token does *not* take precedence
            if !next.is_operator() || Self::get_precedence(next) < min_precedence {
                break;
            }

            // Check for non-operator tokens that should break the precedence parsing
            match next {
                Token::Equal | Token::Endl | Token::BlockStart => break,
                _ => {}
            }

            // Evaluate right side recursively, iterating precedence each time. This effectively
            // groups higher precedence operations that are *after* this one.
            let op = self.next().unwrap();
            let right = self
                .parse_operator(Self::get_precedence(&op) + 1)
                .context("failed to parse right operand")?;

            left = ASTNode::Op {
                lhs: left,
                op,
                rhs: right,
            }
            .into();
        }

        Ok(left)
    }

    /// Parses literal primaries.
    fn parse_primary(&mut self) -> Result<Rc<ASTNode>> {
        match self.peek()? {
            // process negative expressions
            Token::Sub => {
                match *self.peek_n(1)? {
                    // This is a literal negative
                    Token::Number(value) => {
                        // Consume both value and negative operator
                        self.next();
                        self.next();

                        Ok(ASTNode::Literal(Token::Number(-value)).into())
                    }

                    // This is a unary negative expression
                    _ => {
                        // Consume negative, evaluate target
                        self.next();
                        let target = self
                            .parse_operator(0)
                            .context("failed to parse unary operand")?;

                        Ok(ASTNode::UnaryOp {
                            target,
                            op: Token::Sub,
                        }
                        .into())
                    }
                }
            }

            // Literals
            t if t.is_literal() => {
                Ok(ASTNode::Literal(self.next().context("expected literal, found EOF")?).into())
            }

            // Identifiers
            Token::Identifier(sym) => {
                let id = ID::new_sym(*sym);
                self.next().context("expected identifier, found EOF")?;
                Ok(ASTNode::Identifier(id).into())
            }

            // Prefix unary (`!`)
            // Increment/decrement are postfix only, handled in `parse_expr`.
            Token::LogicalNot => {
                // consume unary prefix & take ownership
                let op = self.next().context("expected unary operator, found EOF")?;
                let precedence = Self::get_precedence(&op);

                Ok(ASTNode::UnaryOp {
                    target: self
                        .parse_operator(precedence)
                        .context("failed to parse unary operator")?,
                    op,
                }
                .into())
            }

            // Lists
            Token::BracketOpen => self.parse_list().context("failed to parse list"),

            // Structure instances
            Token::New => self
                .parse_struct_instance()
                .context("failed to parse new structure instance"),

            _ => {
                bail!(
                    "invalid primary expression: '{:?}'",
                    self.peek().unwrap_or(&Token::Undefined)
                );
            }
        }
    }

    /// Parses lists.
    fn parse_list(&mut self) -> Result<Rc<ASTNode>> {
        // consume open bracket
        self.expect(Token::BracketOpen)?;

        // parse items individually
        let mut items = vec![];
        loop {
            // check for exceptions
            match self.peek()? {
                Token::BracketClose => {
                    // break on bracket close, indicating list end
                    self.next();
                    break;
                }
                Token::Endl => {
                    // continue if list is interrupted by endline
                    self.next();
                    continue;
                }
                _ => {}
            }

            // get resolved item
            let item = self
                .parse_expr(Some(Token::Comma))
                .context("failed to parse list item")
                .context(format!("in list: {items:?}"))?;

            // add item to the list
            items.push(Variable::Owned(ASTNode::inner_to_owned(&item)).into())
        }

        Ok(ASTNode::List(items).into())
    }
}
