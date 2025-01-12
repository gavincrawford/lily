//! The parser converts lexed tokens into an abstract syntax tree.

use crate::lexer::Token;

mod tests;

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    Block(Vec<ASTNode>),
    Assign {
        id: String,
        value: Box<ASTNode>,
    },
    Function {
        id: String,
        arguments: Vec<String>,
        body: Box<ASTNode>,
    },
    Conditional {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
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
        while let Some(token) = self.peek() {
            if *token == Token::BlockEnd {
                // consume block end and expect endline
                self.next();
                self.expect(Token::Endl);
                break;
            } else if *token == Token::Endl {
                // consume endlines
                self.next();
            }
            statements.push(self.parse_statement());
        }
        ASTNode::Block(statements)
    }

    /// Parses a statement.
    fn parse_statement(&mut self) -> ASTNode {
        match self.peek() {
            Some(Token::Let) => self.parse_decl_var(),
            Some(Token::If) => self.parse_cond(),
            Some(Token::Function) => self.parse_decl_fn(),
            Some(Token::Identifier(_)) => self.parse_assign_var(),
            _ => {
                todo!();
            }
        }
    }

    /// Parses a conditional expression.
    fn parse_cond(&mut self) -> ASTNode {
        self.expect(Token::If);
        let expr = self.parse_expr();
        ASTNode::Conditional {
            condition: expr,
            body: Box::from(self.parse()),
        }
    }

    /// Parses a function declaration.
    fn parse_decl_fn(&mut self) -> ASTNode {
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
                body: Box::from(self.parse()),
                arguments: args,
            }
        } else {
            panic!("expected identifier, found {:?}", next);
        }
    }

    /// Parses a variable assignment.
    fn parse_assign_var(&mut self) -> ASTNode {
        let next = self.next();
        if let Some(Token::Identifier(name)) = next {
            self.expect(Token::Equal);
            ASTNode::Assign {
                id: name,
                value: self.parse_expr(),
            }
        } else {
            panic!("expected identifier, found {:?}", next);
        }
    }

    /// Parses a variable declaration.
    fn parse_decl_var(&mut self) -> ASTNode {
        self.expect(Token::Let);
        let next = self.next();
        if let Some(Token::Identifier(name)) = next {
            self.expect(Token::Equal);
            ASTNode::Assign {
                id: name,
                value: self.parse_expr(),
            }
        } else {
            panic!("expected identifier, found {:?}", next);
        }
    }

    /// Parses raw expressions, such was math or comparisons.
    fn parse_expr(&mut self) -> Box<ASTNode> {
        let primary;
        if let Some(Token::ParenOpen) = self.peek() {
            // if parenthesis are present, parse them as an expression
            self.next();
            primary = self.parse_expr();
        } else {
            // otherwise, parse as a primary/literal
            primary = self.parse_primary();
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
            | Some(Token::LogicalGe) => Box::from(ASTNode::Op {
                lhs: primary,
                op: self.next().unwrap(),
                rhs: self.parse_expr(),
            }),
            Some(Token::Endl) | Some(Token::BlockStart) | Some(Token::ParenClose) => {
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
            Some(Token::Number(_))
            | Some(Token::Str(_))
            | Some(Token::Identifier(_))
            | Some(Token::Bool(_)) => Box::from(ASTNode::Literal(self.next().unwrap())),
            _ => {
                todo!()
            }
        }
    }
}
