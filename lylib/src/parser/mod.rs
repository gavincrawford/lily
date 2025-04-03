//! The parser converts lexed tokens into an abstract syntax tree.

use crate::lexer::{Lexer, Token};
use anyhow::{bail, Context, Result};
use std::{env, fs::File, io::Read, path::PathBuf, rc::Rc};

pub mod astnode;
pub use astnode::*;
pub mod id;
pub use id::*;
mod tests;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    path: Box<PathBuf>,
}

impl Parser {
    /// Creates a new parser over `tokens`.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            path: Box::new(env::current_dir().unwrap()),
        }
    }

    /// Sets the current working directory, used to set relative location of imports.
    pub fn set_pwd(&mut self, path: PathBuf) {
        self.path = Box::new(path);
    }

    /// Peek at the next token.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Peek `n` positions ahead.
    fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.position + n)
    }

    /// Get and return the next token.
    fn next(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            self.position += 1;
            Some(self.tokens[self.position - 1].clone())
        } else {
            None
        }
    }

    /// Throws an error if the next token is not `expected`.
    fn expect(&mut self, expected: Token) -> Result<()> {
        match self.next() {
            Some(token) if token == expected => {
                return Ok(());
            }
            Some(token) => {
                bail!("found {:?}, expected {:?}", token, expected);
            }
            _ => {
                bail!("unexpected EOF")
            }
        }
    }

    /// Parses all tokens into a program.
    pub fn parse(&mut self) -> Result<Rc<ASTNode>> {
        self.parse_with_imports(vec![])
    }

    /// Parses all tokens with hidden module imports.
    pub fn parse_with_imports(&mut self, imports: Vec<Rc<ASTNode>>) -> Result<Rc<ASTNode>> {
        let mut statements = vec![];
        while let Some(token) = self.peek() {
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
                // otherwise, parse the next statement
                statements.push(self.parse_statement()?);
            }
        }
        Ok(ASTNode::Block([imports, statements].concat()).into())
    }

    /// Parses a statement.
    fn parse_statement(&mut self) -> Result<Rc<ASTNode>> {
        match self.peek() {
            Some(Token::Import) => self.parse_import(),
            Some(Token::Let) => self.parse_decl_var(),
            Some(Token::If) => self.parse_cond(),
            Some(Token::Function) => self.parse_decl_fn(),
            Some(Token::Struct) => self.parse_decl_struct(),
            Some(Token::While) => self.parse_while(),
            Some(Token::Identifier(_)) => match self.peek_n(1) {
                Some(Token::ParenOpen) => self.parse_call_fn(),
                Some(Token::BracketOpen) => self.parse_index(),
                _ => self.parse_assign_var(),
            },
            Some(Token::Return) => self.parse_return(),
            _ => {
                bail!("expected statement, found {:?}", self.peek().unwrap());
            }
        }
    }

    /// Parses imports.
    fn parse_import(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::Import)?;
        if let Some(Token::Str(path)) = self.next() {
            // get full path
            let mut path = self.path.join(PathBuf::from(path));
            if !path.exists() {
                bail!("module not found at '{}'", path.display());
            }

            // check if alias is provided
            let mut alias = None;
            if let Some(Token::As) = self.peek() {
                self.next();
                if let Some(Token::Identifier(alias_str)) = self.peek() {
                    // if an identifier is found, it is our alias
                    alias = Some(alias_str.to_owned());
                    self.next();
                } else {
                    // if something other than an identifier is provided, this import is malformed
                    bail!("expected identifier as alias, found {:?}", self.peek());
                }
            }

            // read the file to be imported to a buffer
            let mut buffer = String::new();
            File::open(path.to_owned())
                .context("failed to created file buffer")?
                .read_to_string(&mut buffer)
                .context("failed to read file data")?;

            // lex buffer into tokens
            let tokens = Lexer::new().lex(buffer)?;

            // create a parser and point it to the file's parent directory
            let mut parser = Self::new(tokens);
            path.pop();
            parser.set_pwd(path);

            // parse the module
            let module = parser.parse().context("failed to parse module body")?;
            Ok(ASTNode::Module {
                alias,
                body: module.into(),
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
        let expr = self.parse_expr(true).context("failed to parse condition")?;
        let if_body = self.parse().context("failed to parse if-body")?;

        // process else body block, if present
        let mut else_body = ASTNode::Block(vec![]).into();
        if let Some(Token::Else) = self.peek() {
            self.next();
            else_body = self.parse().context("failed to parse else-body")?;
        }

        Ok(ASTNode::Conditional {
            condition: expr,
            if_body,
            else_body,
        }
        .into())
    }

    /// Parses a list index.
    fn parse_index(&mut self) -> Result<Rc<ASTNode>> {
        if let Some(Token::Identifier(id)) = self.next() {
            // if id is found, parse index value
            self.expect(Token::BracketOpen)?;
            let index = self
                .parse_expr(false)
                .context("failed to parse list index")?;
            self.expect(Token::BracketClose)?;

            // return index block
            Ok(ASTNode::Index {
                id: ID::new(id),
                index,
            }
            .into())
        } else {
            bail!("expected identifier to index");
        }
    }

    /// Parses a while loop.
    fn parse_while(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::While)?;
        Ok(ASTNode::Loop {
            condition: self
                .parse_expr(true)
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

        // parse as a function call to be handled at runtime
        self.parse_call_fn()
            .context("failed to parse constructor call")
    }

    /// Parses a structure declaration.
    fn parse_decl_struct(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::Struct)?;
        // TODO apply this style to function decl
        match self.next() {
            Some(Token::Identifier(name)) => {
                self.expect(Token::BlockStart)?;
                Ok(ASTNode::Struct {
                    id: ID::new(name),
                    body: self.parse()?,
                }
                .into())
            }
            other => {
                bail!("expected identifier, found {:?}", other)
            }
        }
    }

    /// Parses a function declaration.
    fn parse_decl_fn(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::Function)?;
        let next = self.next();
        if let Some(Token::Identifier(name)) = next {
            // gather arguments
            let mut args = vec![];
            while let Some(Token::Identifier(arg)) = self.peek() {
                args.push(arg.clone());
                self.next();
            }
            self.expect(Token::BlockStart)?;
            Ok(ASTNode::Function {
                id: ID::new(name),
                body: self.parse().context("failed to parse function body")?,
                arguments: args,
            }
            .into())
        } else {
            bail!("expected identifier, found {:?}", next);
        }
    }

    /// Parses a function call.
    fn parse_call_fn(&mut self) -> Result<Rc<ASTNode>> {
        // parse identifier
        let id;
        if let Some(Token::Identifier(fn_id)) = self.next() {
            id = fn_id;
        } else {
            bail!("function identifier not found");
        }

        // parse arguments
        self.expect(Token::ParenOpen)?;
        let mut args = vec![];
        loop {
            match self.peek() {
                Some(Token::ParenClose) => {
                    self.next();
                    break;
                }
                Some(_) => {
                    args.push(self.parse_expr(false).context("failed to parse argument")?);
                }
                _ => {
                    break;
                }
            }
        }

        Ok(ASTNode::FunctionCall {
            id: ID::new(id),
            arguments: args,
        }
        .into())
    }

    /// Parses a return statement.
    fn parse_return(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::Return)?;
        Ok(ASTNode::Return(
            self.parse_expr(true)
                .context("failed to parse return value")?,
        )
        .into())
    }

    /// Parses a variable assignment.
    fn parse_assign_var(&mut self) -> Result<Rc<ASTNode>> {
        let next = self.next();
        if let Some(Token::Identifier(id)) = next {
            self.expect(Token::Equal)?;
            Ok(ASTNode::Assign {
                id: ID::new(id),
                value: self.parse_expr(true)?,
            }
            .into())
        } else {
            bail!("expected identifier, found {:?}", next);
        }
    }

    /// Parses a variable declaration.
    fn parse_decl_var(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::Let)?;
        let next = self.next();
        if let Some(Token::Identifier(id)) = next {
            self.expect(Token::Equal)?;
            Ok(ASTNode::Declare {
                id: ID::new(id),
                value: self.parse_expr(true)?,
            }
            .into())
        } else {
            bail!("expected identifier, found {:?}", next);
        }
    }

    /// Parses raw expressions, such as math or comparisons.
    fn parse_expr(&mut self, consume_delimiters: bool) -> Result<Rc<ASTNode>> {
        // tracks if a paren has been opened for error messages
        let parens_open;

        // evaluate primary value
        let primary: Rc<ASTNode>;
        match self.peek() {
            Some(Token::ParenOpen) => {
                // if parenthesis are present, parse them as an expression
                self.next();
                parens_open = true;
                primary = self
                    .parse_expr(true)
                    .context("failed to parse parenthesised expression")?;
            }
            _ => {
                // otherwise, parse as a primary/literal
                parens_open = false;
                primary = self.parse_primary().context("failed to parse expression")?;
            }
        }

        // match operator
        match self.peek() {
            Some(Token::Add)
            | Some(Token::Sub)
            | Some(Token::Mul)
            | Some(Token::Div)
            | Some(Token::Pow)
            | Some(Token::LogicalL)
            | Some(Token::LogicalLe)
            | Some(Token::LogicalG)
            | Some(Token::LogicalGe)
            | Some(Token::LogicalEq) => Ok(ASTNode::Op {
                lhs: primary,
                op: self.next().unwrap(), // safety: peek
                rhs: self
                    .parse_expr(parens_open || consume_delimiters)
                    .context("failed to parse member of op")?,
            }
            .into()),
            Some(Token::ParenClose) | Some(Token::BracketClose) => {
                if consume_delimiters {
                    self.next();
                }
                Ok(primary)
            }
            Some(Token::Endl) | Some(Token::BlockStart) | Some(Token::Comma) => {
                self.next();
                Ok(primary)
            }
            _ => {
                if parens_open {
                    bail!("unclosed delimiter found");
                }

                // XXX
                // returning the primary here is a half-ass fix for the way that parenthesis are
                // handled. the consume_delimiters functionality causes lots of issues and
                // conflicts with funciton calls.
                Ok(primary)
            }
        }
    }

    /// Parses primaries, such as literals and function calls.
    fn parse_primary(&mut self) -> Result<Rc<ASTNode>> {
        match self.peek() {
            // process negative numbers
            Some(Token::Sub) => {
                let next = self.peek_n(1).unwrap().to_owned();
                if let Token::Number(value) = next {
                    // consume both values
                    self.next();
                    self.next();

                    // negate literal and return
                    Ok(ASTNode::Literal(Token::Number(-1. * (value.to_owned()))).into())
                } else {
                    bail!("expected number after '-', found {:?}", self.peek());
                }
            }

            // literals
            Some(Token::Number(_))
            | Some(Token::Str(_))
            | Some(Token::Bool(_))
            | Some(Token::Char(_)) => {
                Ok(ASTNode::Literal(self.next().expect("expected literal, found EOF")).into())
            }

            // lists
            Some(Token::BracketOpen) => self.parse_list().context("failed to parse list"),

            // variables, function calls
            Some(Token::Identifier(_)) => match self.peek_n(1) {
                Some(Token::ParenOpen) => {
                    // if the future token is a parenthesis, this is a function call
                    self.parse_call_fn()
                        .context("failed to parse function call")
                }
                Some(Token::BracketOpen) => {
                    // if the future token is a bracket, this is an index
                    self.parse_index().context("failed to parse index operator")
                }
                _ => {
                    // otherwise, it's safe to assume that the token is a literal
                    Ok(ASTNode::Literal(self.next().expect("expected literal, found EOF")).into())
                }
            },

            // structure instances
            Some(Token::New) => self
                .parse_struct_instance()
                .context("failed to parse new structure instance"),

            _ => {
                todo!()
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
            match self.peek() {
                Some(Token::BracketClose) => {
                    // break on bracket close, indicating list end
                    self.next();
                    break;
                }
                Some(Token::Endl) => {
                    // continue if list is interrupted by endline
                    self.next();
                    continue;
                }
                _ => {}
            }

            // add item to the list
            items.push(Rc::from(
                self.parse_expr(false)
                    .context("failed to parse list item")?,
            ))
        }

        Ok(ASTNode::List(items).into())
    }
}
