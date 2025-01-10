//! The parser converts lexed tokens into an abstract syntax tree.

use crate::lexer::Token;

mod tests;

#[derive(Debug, PartialEq)]
pub enum ASTNode {
    Program(Vec<ASTNode>),
    Variable {
        id: String,
        value: Box<ASTNode>,
    },
    Op {
        lhs: Box<ASTNode>,
        op: Token,
        rhs: Box<ASTNode>,
    },
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
    pub fn parse(&mut self) -> ASTNode {
        let mut statements = vec![];
        while self.peek().is_some() {
            statements.push(self.parse_statement());
        }
        ASTNode::Program(statements)
    }

    /// Parses a statement.
    fn parse_statement(&mut self) -> ASTNode {
        match self.peek() {
            Some(Token::Let) => self.parse_decl(),
            _ => {
                todo!();
            }
        }
    }

    /// Parses a variable declaration.
    fn parse_decl(&mut self) -> ASTNode {
        self.expect(Token::Let);
        let next = self.next();
        if let Some(Token::Identifier(name)) = next {
            self.expect(Token::Equal);
            ASTNode::Variable {
                id: name,
                value: self.parse_expr(),
            }
        } else {
            panic!("expected identifier, found {:?}", next);
        }
    }

    /// Parses raw expressions, such was math or comparisons.
    fn parse_expr(&mut self) -> Box<ASTNode> {
        let primary = self.parse_primary();
        match self.peek() {
            Some(Token::Add)
            | Some(Token::Sub)
            | Some(Token::Mul)
            | Some(Token::Div)
            | Some(Token::LogicalL)
            | Some(Token::LogicalLe)
            | Some(Token::LogicalG)
            | Some(Token::LogicalGe) => Box::from(ASTNode::Op {
                lhs: primary,
                op: self.next().unwrap(),
                rhs: self.parse_expr(),
            }),
            Some(Token::Endl) => {
                self.next();
                primary
            }
            _ => {
                todo!();
            }
        }
    }

    /// Parses primaries, such as literals.
    fn parse_primary(&mut self) -> Box<ASTNode> {
        match self.peek() {
            Some(Token::Number(_)) | Some(Token::Str(_)) | Some(Token::Identifier(_)) => {
                Box::from(ASTNode::Literal(self.next().unwrap()))
            }
            _ => {
                todo!()
            }
        }
    }
}
