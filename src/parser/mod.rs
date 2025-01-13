//! The parser converts lexed tokens into an abstract syntax tree.

use crate::lexer::Token;
use std::rc::Rc;

mod tests;

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    Block(Vec<Rc<ASTNode>>),
    Assign {
        id: String,
        value: Rc<ASTNode>,
    },
    Function {
        id: String,
        arguments: Vec<String>,
        body: Rc<ASTNode>,
    },
    FunctionCall {
        id: String,
        arguments: Vec<Rc<ASTNode>>,
    },
    Conditional {
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
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}
impl Parser {
    /// Creates a new parser over `tokens`.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
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
        let mut statements = vec![];
        while let Some(token) = self.peek() {
            if *token == Token::BlockEnd {
                // consume block end and expect endline
                self.next();
                self.expect(Token::Endl);
                break;
            } else if *token == Token::Endl {
                // consume endlines
                self.next();
            } else {
                // otherwise, parse the next statement
                statements.push(self.parse_statement());
            }
        }
        ASTNode::Block(statements).into()
    }

    /// Parses a statement.
    fn parse_statement(&mut self) -> Rc<ASTNode> {
        match self.peek() {
            Some(Token::Let) => self.parse_decl_var(),
            Some(Token::If) => self.parse_cond(),
            Some(Token::Function) => self.parse_decl_fn(),
            Some(Token::Identifier(_)) => {
                if let Some(Token::ParenOpen) = self.peek_n(1) {
                    // handle function calls
                    self.parse_call_fn()
                } else {
                    // handle variable assignments
                    self.parse_assign_var()
                }
            }
            Some(Token::Return) => self.parse_return(),
            _ => {
                todo!();
            }
        }
    }

    /// Parses a conditional expression.
    fn parse_cond(&mut self) -> Rc<ASTNode> {
        self.expect(Token::If);
        let expr = self.parse_expr(true);
        ASTNode::Conditional {
            condition: expr,
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
            self.expect(Token::Endl);
            ASTNode::Function {
                id: name,
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
                    todo!();
                }
            }
        }

        ASTNode::FunctionCall {
            id,
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
        if let Some(Token::Identifier(name)) = next {
            self.expect(Token::Equal);
            ASTNode::Assign {
                id: name,
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
        if let Some(Token::Identifier(name)) = next {
            self.expect(Token::Equal);
            ASTNode::Assign {
                id: name,
                value: self.parse_expr(true),
            }
            .into()
        } else {
            panic!("expected identifier, found {:?}", next);
        }
    }

    /// Parses raw expressions, such as math or comparisons.
    // TODO the whole `consume_parens` thing seems janky. find another way?
    fn parse_expr(&mut self, consume_parens: bool) -> Rc<ASTNode> {
        let primary;
        match self.peek() {
            Some(Token::ParenOpen) => {
                // if parenthesis are present, parse them as an expression
                self.next();
                primary = self.parse_expr(true);
            }
            _ => {
                // otherwise, parse as a primary/literal
                primary = self.parse_primary();
            }
        }

        // match operator
        match self.peek() {
            Some(Token::Add)
            | Some(Token::Sub)
            | Some(Token::Mul)
            | Some(Token::Div)
            | Some(Token::LogicalL)
            | Some(Token::LogicalLe)
            | Some(Token::LogicalG)
            | Some(Token::LogicalGe) => ASTNode::Op {
                lhs: primary,
                op: self.next().unwrap(),
                rhs: self.parse_expr(true),
            }
            .into(),
            Some(Token::ParenClose) => {
                if consume_parens {
                    self.next();
                }
                primary
            }
            Some(Token::Endl) | Some(Token::BlockStart) | Some(Token::Comma) => {
                self.next();
                primary
            }
            _ => {
                todo!();
            }
        }
    }

    /// Parses primaries, such as literals and function calls.
    fn parse_primary(&mut self) -> Rc<ASTNode> {
        match self.peek() {
            Some(Token::Number(_)) | Some(Token::Str(_)) | Some(Token::Bool(_)) => {
                ASTNode::Literal(self.next().unwrap()).into()
            }
            Some(Token::Identifier(_)) => {
                if let Some(Token::ParenOpen) = self.peek_n(1) {
                    // if the future token is a parenthesis, this is a function call
                    self.parse_call_fn().into()
                } else {
                    // otherwise, it's safe to assume that the token is a literal
                    ASTNode::Literal(self.next().unwrap()).into()
                }
            }
            _ => {
                todo!()
            }
        }
    }
}
