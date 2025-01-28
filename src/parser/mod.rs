//! The parser converts lexed tokens into an abstract syntax tree.

use crate::lexer::{Lexer, Token};
use std::{env, fs::File, io::Read, path::PathBuf, rc::Rc};

pub mod id;
pub use id::*;

mod tests;

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    /// Represents a block of statements, grouped in a scope.
    Block(Vec<Rc<ASTNode>>),
    /// Holds a block, but represents a separate module.
    Module {
        alias: Option<String>,
        body: Rc<ASTNode>,
    },

    Index {
        id: ID,
        index: Rc<ASTNode>,
    },
    Assign {
        id: ID,
        value: Rc<ASTNode>,
    },
    Declare {
        id: ID,
        value: Rc<ASTNode>,
    },
    Function {
        id: ID,
        arguments: Vec<String>,
        body: Rc<ASTNode>,
    },
    FunctionCall {
        id: ID,
        arguments: Vec<Rc<ASTNode>>,
    },
    Conditional {
        condition: Rc<ASTNode>,
        if_body: Rc<ASTNode>,
        else_body: Rc<ASTNode>,
    },
    Loop {
        condition: Rc<ASTNode>,
        body: Rc<ASTNode>,
    },
    Op {
        lhs: Rc<ASTNode>,
        op: Token,
        rhs: Rc<ASTNode>,
    },
    Return(Rc<ASTNode>),
    Literal(Token),
    List(Vec<Token>),
}

impl ASTNode {
    pub fn inner_to_owned(rc: &Rc<ASTNode>) -> ASTNode {
        (&*rc.clone()).clone()
    }
}

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

    /// Panics if the next token is not `expected`.
    fn expect(&mut self, expected: Token) {
        match self.next() {
            Some(token) if token == expected => {
                return;
            }
            Some(token) => {
                panic!("found {:?}, expected {:?}", token, expected);
            }
            _ => {
                panic!("unexpected EOF")
            }
        }
    }

    /// Parses all tokens into a program.
    pub fn parse(&mut self) -> Rc<ASTNode> {
        self.parse_with_imports(vec![])
    }

    /// Parses all tokens with hidden module imports.
    pub fn parse_with_imports(&mut self, imports: Vec<Rc<ASTNode>>) -> Rc<ASTNode> {
        let mut statements = vec![];
        while let Some(token) = self.peek() {
            if *token == Token::BlockEnd {
                // consume block ends and expect endline
                self.next();
                self.expect(Token::Endl);
                break;
            } else if *token == Token::Else {
                // also counts as a block end for conditionals
                break;
            } else if *token == Token::Endl {
                // consume endlines
                self.next();
            } else {
                // otherwise, parse the next statement
                statements.push(self.parse_statement());
            }
        }
        ASTNode::Block([imports, statements].concat()).into()
    }

    /// Parses a statement.
    fn parse_statement(&mut self) -> Rc<ASTNode> {
        match self.peek() {
            Some(Token::Import) => self.parse_import(),
            Some(Token::Let) => self.parse_decl_var(),
            Some(Token::If) => self.parse_cond(),
            Some(Token::Function) => self.parse_decl_fn(),
            Some(Token::While) => self.parse_while(),
            Some(Token::Identifier(_)) => match self.peek_n(1) {
                Some(Token::ParenOpen) => self.parse_call_fn(),
                Some(Token::BracketOpen) => self.parse_index(),
                _ => self.parse_assign_var(),
            },
            Some(Token::Return) => self.parse_return(),
            _ => {
                panic!("expected statement, found {:?}.", self.peek().unwrap());
            }
        }
    }

    /// Parses imports.
    fn parse_import(&mut self) -> Rc<ASTNode> {
        self.expect(Token::Import);
        if let Some(Token::Str(path)) = self.next() {
            // get full path
            let mut path = self.path.join(PathBuf::from(path));
            if !path.exists() {
                panic!("module not found at '{}'", path.display());
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
                    panic!("expected identifier as alias, found {:?}", self.peek());
                }
            }

            // read the file to be imported to a buffer
            let mut buffer = String::new();
            // TODO handle unwrap
            File::open(path.to_owned())
                .unwrap()
                .read_to_string(&mut buffer)
                .unwrap();

            // lex buffer into tokens
            let tokens = Lexer::new().lex(buffer);

            // create a parser and point it to the file's parent directory
            let mut parser = Self::new(tokens);
            path.pop();
            parser.set_pwd(path);

            // parse the module
            let module = parser.parse();
            ASTNode::Module {
                alias,
                body: module.into(),
            }
            .into()
        } else {
            panic!("expected path after import.");
        }
    }

    /// Parses a conditional expression.
    fn parse_cond(&mut self) -> Rc<ASTNode> {
        // consume if token
        self.expect(Token::If);

        // get if expression and if body block
        let expr = self.parse_expr(true);
        let if_body = self.parse();

        // process else body block, if present
        let mut else_body = ASTNode::Block(vec![]).into();
        if let Some(Token::Else) = self.peek() {
            self.next();
            else_body = self.parse();
        }

        ASTNode::Conditional {
            condition: expr,
            if_body,
            else_body,
        }
        .into()
    }

    /// Parses a list index.
    fn parse_index(&mut self) -> Rc<ASTNode> {
        if let Some(Token::Identifier(id)) = self.next() {
            // if id is found, parse index value
            self.expect(Token::BracketOpen);
            let index = self.parse_expr(false);
            self.expect(Token::BracketClose);

            // return index block
            ASTNode::Index {
                id: ID::new(id),
                index,
            }
            .into()
        } else {
            panic!("expected identifier to index.");
        }
    }

    /// Parses a while loop.
    fn parse_while(&mut self) -> Rc<ASTNode> {
        self.expect(Token::While);
        ASTNode::Loop {
            condition: self.parse_expr(true),
            body: self.parse(),
        }
        .into()
    }

    /// Parses a function declaration.
    fn parse_decl_fn(&mut self) -> Rc<ASTNode> {
        self.expect(Token::Function);
        let next = self.next();
        if let Some(Token::Identifier(name)) = next {
            // gather arguments
            let mut args = vec![];
            while let Some(Token::Identifier(arg)) = self.peek() {
                args.push(arg.clone());
                self.next();
            }
            self.expect(Token::BlockStart);
            ASTNode::Function {
                id: ID::new(name),
                body: self.parse(),
                arguments: args,
            }
            .into()
        } else {
            panic!("expected identifier, found {:?}", next);
        }
    }

    /// Parses a function call.
    fn parse_call_fn(&mut self) -> Rc<ASTNode> {
        // parse identifier
        let id;
        if let Some(Token::Identifier(fn_id)) = self.next() {
            id = fn_id;
        } else {
            panic!("function identifier not found.");
        }

        // parse arguments
        self.expect(Token::ParenOpen);
        let mut args = vec![];
        loop {
            match self.peek() {
                Some(Token::ParenClose) => {
                    self.next();
                    break;
                }
                Some(_) => {
                    args.push(self.parse_expr(false));
                }
                _ => {
                    panic!("unexpected token in argument position");
                }
            }
        }

        ASTNode::FunctionCall {
            id: ID::new(id),
            arguments: args,
        }
        .into()
    }

    /// Parses a return statement.
    fn parse_return(&mut self) -> Rc<ASTNode> {
        self.expect(Token::Return);
        ASTNode::Return(self.parse_expr(true)).into()
    }

    /// Parses a variable assignment.
    fn parse_assign_var(&mut self) -> Rc<ASTNode> {
        let next = self.next();
        if let Some(Token::Identifier(id)) = next {
            self.expect(Token::Equal);
            ASTNode::Assign {
                id: ID::new(id),
                value: self.parse_expr(true),
            }
            .into()
        } else {
            panic!("expected identifier, found {:?}", next);
        }
    }

    /// Parses a variable declaration.
    fn parse_decl_var(&mut self) -> Rc<ASTNode> {
        self.expect(Token::Let);
        let next = self.next();
        if let Some(Token::Identifier(id)) = next {
            self.expect(Token::Equal);
            ASTNode::Declare {
                id: ID::new(id),
                value: self.parse_expr(true),
            }
            .into()
        } else {
            panic!("expected identifier, found {:?}", next);
        }
    }

    /// Parses raw expressions, such as math or comparisons.
    fn parse_expr(&mut self, consume_delimiters: bool) -> Rc<ASTNode> {
        // tracks if a paren has been opened for error messages
        let parens_open;

        // evaluate primary value
        let primary;
        match self.peek() {
            Some(Token::ParenOpen) => {
                // if parenthesis are present, parse them as an expression
                self.next();
                parens_open = true;
                primary = self.parse_expr(true);
            }
            _ => {
                // otherwise, parse as a primary/literal
                parens_open = false;
                primary = self.parse_primary();
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
            | Some(Token::LogicalEq) => ASTNode::Op {
                lhs: primary,
                op: self.next().unwrap(), // safety: peek
                rhs: self.parse_expr(parens_open || consume_delimiters),
            }
            .into(),
            Some(Token::ParenClose) | Some(Token::BracketClose) => {
                if consume_delimiters {
                    self.next();
                }
                primary
            }
            Some(Token::Endl) | Some(Token::BlockStart) | Some(Token::Comma) => {
                self.next();
                primary
            }
            _ => {
                if parens_open {
                    panic!("unclosed delimiter found.");
                } else {
                    panic!("unexpected member of expression.")
                }
            }
        }
    }

    /// Parses primaries, such as literals and function calls.
    fn parse_primary(&mut self) -> Rc<ASTNode> {
        match self.peek() {
            // process negative numbers
            Some(Token::Sub) => {
                let next = self.peek_n(1).unwrap().to_owned();
                if let Token::Number(value) = next {
                    // consume both values
                    self.next();
                    self.next();

                    // negate literal and return
                    ASTNode::Literal(Token::Number(-1. * (value.to_owned()))).into()
                } else {
                    panic!("expected number after '-', found {:?}.", self.peek());
                }
            }

            // literals
            Some(Token::Number(_))
            | Some(Token::Str(_))
            | Some(Token::Bool(_))
            | Some(Token::Char(_)) => {
                ASTNode::Literal(self.next().expect("expected literal, found EOF.")).into()
            }

            // lists
            Some(Token::BracketOpen) => self.parse_list(),

            // variables, function calls
            Some(Token::Identifier(_)) => match self.peek_n(1) {
                Some(Token::ParenOpen) => {
                    // if the future token is a parenthesis, this is a function call
                    self.parse_call_fn().into()
                }
                Some(Token::BracketOpen) => {
                    // if the future token is a bracket, this is an index
                    self.parse_index().into()
                }
                _ => {
                    // otherwise, it's safe to assume that the token is a literal
                    ASTNode::Literal(self.next().expect("expected literal, found EOF.")).into()
                }
            },
            _ => {
                todo!()
            }
        }
    }

    /// Parses lists.
    fn parse_list(&mut self) -> Rc<ASTNode> {
        // consume open bracket
        self.expect(Token::BracketOpen);

        // parse items individually
        let mut items = vec![];
        loop {
            // end when bracket close is reached
            if let Some(Token::BracketClose) = self.peek() {
                break;
            }

            // add item to the list
            if let ASTNode::Literal(value) = &*self.parse_expr(false) {
                items.push(value.clone());
            }
        }

        ASTNode::List(items).into()
    }
}
