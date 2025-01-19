//! The parser converts lexed tokens into an abstract syntax tree.

use crate::lexer::Token;
use std::rc::Rc;

mod tests;

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    Block(Vec<Rc<ASTNode>>),
    Index {
        id: String,
        index: Rc<ASTNode>,
    },
    Assign {
        id: String,
        value: Rc<ASTNode>,
    },
    Declare {
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
            // TODO you might be able to merge the first two of these statements into one, and let
            // other functions take care of consuming their block ends. haven't tried it though, so
            // it may fail terribly
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
        ASTNode::Block(statements).into()
    }

    /// Parses a statement.
    fn parse_statement(&mut self) -> Rc<ASTNode> {
        match self.peek() {
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

            // if the index is a non-number, panic
            if let ASTNode::Literal(Token::Number(_)) = &*index {
                ASTNode::Index { id, index }.into()
            } else {
                panic!("index must be a number.");
            }
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
                Some(Token::Endl) | None => {
                    // if an endline is found, there aren't any more arguments
                    // TODO i kinda feel like this shouldn't happen here, given the end paren
                    // should do this job just fine. pretty strange
                    break;
                }
                Some(Token::ParenClose) => {
                    self.next();
                    break;
                }
                Some(_) => {
                    args.push(self.parse_expr(false));
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
            ASTNode::Declare {
                id: name,
                value: self.parse_expr(true),
            }
            .into()
        } else {
            panic!("expected identifier, found {:?}", next);
        }
    }

    /// Parses raw expressions, such as math or comparisons.
    // TODO the whole `consume_delimiters` thing seems janky. find another way?
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
                rhs: self.parse_expr(true),
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
            Some(Token::Number(_))
            | Some(Token::Str(_))
            | Some(Token::Bool(_))
            | Some(Token::Char(_)) => {
                ASTNode::Literal(self.next().expect("expected literal, found EOF.")).into()
            }
            Some(Token::BracketOpen) => self.parse_list(),
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
        // TODO limit the size of arrays here to avoid getting stuck
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
