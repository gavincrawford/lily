//! The parser converts lexed tokens into an abstract syntax tree.

use crate::interpreter::{Variable, ID};
use crate::lexer::{Lexer, Token};
use anyhow::{bail, Context, Result};
use std::{env, fs::File, io::Read, path::PathBuf, rc::Rc};

pub mod astnode;
pub use astnode::*;
mod tests;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    path: PathBuf,
}

impl Parser {
    /// Creates a new parser over `tokens`.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            path: env::current_dir().unwrap(),
        }
    }

    /// Sets the current working directory, used to set relative location of imports.
    pub fn set_pwd(&mut self, path: PathBuf) {
        self.path = path;
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
            Some(token) if token == expected => Ok(()),
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
        // process all possible base statements
        let result = match self.peek() {
            Some(Token::Import) => self.parse_import(),
            Some(Token::Let) => self.parse_decl_var(),
            Some(Token::If) => self.parse_cond(),
            Some(Token::Function) => self.parse_decl_fn(),
            Some(Token::Struct) => self.parse_decl_struct(),
            Some(Token::While) => self.parse_while(),
            Some(Token::Identifier(_)) => self.parse_expr(None),
            Some(Token::Return) => self.parse_return(),
            _ => {
                bail!("expected statement, found {:?}", self.peek().unwrap());
            }
        };

        // return result with added context
        result.context("failed to parse statement")
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
            File::open(&path)
                .context("failed to created file buffer")?
                .read_to_string(&mut buffer)
                .context("failed to read file data")?;

            // lex buffer into tokens
            let tokens = Lexer::new()
                .lex(buffer)
                .context("failed to lex imported file")?;

            // create a parser and point it to the file's parent directory
            let mut parser = Self::new(tokens);
            path.pop();
            parser.set_pwd(path);

            // parse the module
            let body = parser.parse().context("failed to parse module body")?;
            Ok(ASTNode::Module { alias, body }.into())
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
        if let Some(Token::Else) = self.peek() {
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
        if let ASTNode::FunctionCall {
            target: _,
            arguments: _,
        } = &*stmnt
        {
            Ok(stmnt)
        } else {
            bail!("failed to parse instantiation of structure")
        }
    }

    /// Parses a structure declaration.
    fn parse_decl_struct(&mut self) -> Result<Rc<ASTNode>> {
        self.expect(Token::Struct)?;
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
        match next {
            Some(Token::Identifier(name)) => {
                // gather arguments
                let mut arguments = vec![];
                while let Some(Token::Identifier(arg)) = self.peek() {
                    arguments.push(arg.clone());
                    self.next();
                }
                self.expect(Token::BlockStart)?;
                Ok(ASTNode::Function {
                    id: ID::new(name),
                    body: self.parse().context("failed to parse function body")?,
                    arguments,
                }
                .into())
            }
            _ => {
                bail!("expected identifier, found {:?}", next);
            }
        }
    }

    /// Parses a function call.
    fn parse_call_fn(&mut self, target: Rc<ASTNode>) -> Result<Rc<ASTNode>> {
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
                    args.push(
                        self.parse_expr(Some(Token::Comma))
                            .context("failed to parse argument")?,
                    );
                }
                _ => {
                    break;
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
        Ok(ASTNode::Return(
            self.parse_expr(None)
                .context("failed to parse return value")?,
        )
        .into())
    }

    /// Parses assignment to any target.
    fn parse_assignment(&mut self, target: Rc<ASTNode>) -> Result<Rc<ASTNode>> {
        // parse value
        self.expect(Token::Equal)?;
        let value = self
            .parse_expr(None)
            .context("failed to parse assignment value")?;

        // return node
        Ok(ASTNode::Assign { target, value }.into())
    }

    /// Parses a variable declaration.
    fn parse_decl_var(&mut self) -> Result<Rc<ASTNode>> {
        // parse id and value
        self.expect(Token::Let)?;
        let target = self
            .parse_expr(Some(Token::Equal))
            .context("failed to parse declaration target")?;
        let value = self
            .parse_expr(None)
            .context("failed to parse declaration value")?;

        // return node
        Ok(ASTNode::Declare { target, value }.into())
    }

    /// Parses expressions, such as operators, indices, function calls, etc.
    fn parse_expr(&mut self, expect: Option<Token>) -> Result<Rc<ASTNode>> {
        // evaluate primary value
        let mut primary = match self.peek() {
            Some(Token::ParenOpen) => {
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
            if let Some(ref token) = expect {
                if self.peek().unwrap() == token {
                    self.expect(expect.unwrap())?;
                    break;
                }
            }

            // match operator
            let result = match self.peek() {
                // math and logical operators
                Some(Token::Add)
                | Some(Token::Sub)
                | Some(Token::Mul)
                | Some(Token::Div)
                | Some(Token::Floor)
                | Some(Token::Pow)
                | Some(Token::LogicalL)
                | Some(Token::LogicalLe)
                | Some(Token::LogicalG)
                | Some(Token::LogicalGe)
                | Some(Token::LogicalEq)
                | Some(Token::LogicalNeq)
                | Some(Token::LogicalAnd)
                | Some(Token::LogicalOr) => Ok(ASTNode::Op {
                    lhs: primary.clone(),
                    op: self.next().unwrap(), // safety: peek
                    rhs: self
                        .parse_expr(None)
                        .context("failed to parse member of op")?,
                }
                .into()),

                // function calls
                Some(Token::ParenOpen) => self.parse_call_fn(primary.clone()),

                // indexes
                Some(Token::BracketOpen) => self.parse_index(primary.clone()),

                // assignments
                Some(Token::Equal) => self.parse_assignment(primary.clone()),

                // break for all others
                Some(Token::Endl) | Some(Token::BlockStart) | None => {
                    self.next();
                    break;
                }
                _ => {
                    break;
                }
            };

            primary = result?;
        }

        Ok(primary)
    }

    /// Parses literal primaries.
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
                    Ok(ASTNode::Literal(Token::Number(-(value.to_owned()))).into())
                } else {
                    bail!("expected number after '-', found {:?}", self.peek());
                }
            }

            // literals
            Some(Token::Identifier(_))
            | Some(Token::Number(_))
            | Some(Token::Str(_))
            | Some(Token::Bool(_))
            | Some(Token::Char(_)) => {
                Ok(ASTNode::Literal(self.next().expect("expected literal, found EOF")).into())
            }

            // logical not
            Some(Token::LogicalNot) => {
                // consumes the `!` and creates a one-sided operator
                self.next();
                Ok(ASTNode::Op {
                    lhs: self.parse_expr(None).unwrap(),
                    op: Token::LogicalNot,
                    rhs: ASTNode::Literal(Token::Undefined).into(),
                }
                .into())
            }

            // lists
            Some(Token::BracketOpen) => self.parse_list().context("failed to parse list"),

            // structure instances
            Some(Token::New) => self
                .parse_struct_instance()
                .context("failed to parse new structure instance"),

            None => {
                bail!("invalid primary: EOF");
            }
            _ => {
                bail!("invalid primary '{:?}'", self.peek().unwrap());
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

            // get resolved item
            let item = self
                .parse_expr(Some(Token::Comma))
                .context("failed to parse list item")?;

            // add item to the list
            items.push(Variable::Owned(ASTNode::inner_to_owned(&item)).into())
        }

        Ok(ASTNode::List(items).into())
    }
}
