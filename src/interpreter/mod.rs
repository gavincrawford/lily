//! The interpreter executes an abstract syntax tree.

mod tests;

use crate::{lexer::Token, parser::ASTNode};
use std::collections::HashMap;

pub struct Interpreter {
    pub global_scope: HashMap<String, Box<ASTNode>>,
}
impl Interpreter {
    pub fn new() -> Self {
        Self {
            global_scope: HashMap::new(),
        }
    }

    /// Executes an AST block, typically the head.
    pub fn execute(&mut self, ast: ASTNode) {
        if let ASTNode::Block(statements) = ast {
            for statement in statements {
                self.execute_expr(statement);
            }
        } else {
            panic!("AST segment is not a block.");
        }
    }

    /// Gets the value of a variable.
    #[allow(unused)] // only used in tests
    fn get(&self, id: String) -> Token {
        let var = self.global_scope.get(&id).unwrap();
        let var = var.clone();
        match *var {
            ASTNode::Literal(t) => t,
            _ => {
                panic!("invalid AST node in global scope.");
            }
        }
    }

    /// Executes an individual expression.
    fn execute_expr(&mut self, statement: ASTNode) -> Option<ASTNode> {
        match statement {
            ASTNode::Variable { id, value } => {
                let resolved_expr = self.execute_expr(*value).unwrap();
                self.global_scope.insert(id, Box::from(resolved_expr));
                None
            }
            ASTNode::Op { lhs, op, rhs } => {
                let a = self.execute_expr(*lhs);
                let b = self.execute_expr(*rhs);
                if let (
                    Some(ASTNode::Literal(Token::Number(a))),
                    Some(ASTNode::Literal(Token::Number(b))),
                ) = (a, b)
                {
                    match op {
                        Token::Add => return Some(ASTNode::Literal(Token::Number(a + b))),
                        Token::Sub => return Some(ASTNode::Literal(Token::Number(a - b))),
                        Token::Mul => return Some(ASTNode::Literal(Token::Number(a * b))),
                        Token::Div => return Some(ASTNode::Literal(Token::Number(a / b))),
                        _ => {
                            panic!("operator not implemented.");
                        }
                    }
                } else {
                    return None;
                }
            }
            ASTNode::Literal(ref t) => {
                if let Token::Identifier(identifier) = t {
                    // get variable value if applicable
                    Some(ASTNode::Literal(self.get(identifier.clone())))
                } else {
                    // otherwise, return raw literal
                    Some(statement)
                }
            }
            _ => {
                todo!()
            }
        }
    }
}
