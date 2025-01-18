//! The interpreter executes an abstract syntax tree.

mod mem;
mod tests;

use crate::{lexer::Token, parser::ASTNode};
use std::{collections::HashMap, rc::Rc};

pub struct Interpreter<'a> {
    /// Variable storage.
    pub variables: Vec<HashMap<String, ASTNode>>,
    /// Function table.
    pub functions: Vec<HashMap<String, &'a Rc<ASTNode>>>,
    /// Scope level.
    scope: usize,
}
impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        Self {
            variables: Vec::with_capacity(8),
            functions: Vec::with_capacity(8),
            scope: 0,
        }
    }

    /// Executes an AST segment, typically the head. Returns `Some` when a return block is reached.
    pub fn execute(&mut self, ast: &'a Rc<ASTNode>) -> Option<Rc<ASTNode>> {
        if let ASTNode::Block(statements) = &**ast {
            // if this segment is a block, execute all of its statements
            for statement in statements {
                if let Some(ret_value) = self.execute_expr(statement) {
                    if self.scope > 0 {
                        return Some(ret_value);
                    } else {
                        // TODO this still doesn't prevent return from being called inside a
                        // conditional, so maybe add a syntax error for that too
                        panic!("return cannot be called outside of a function.");
                    }
                }
            }
        } else {
            // otherwise, execute the segment by itself
            self.execute_expr(ast);
        }
        None
    }

    /// Executes an individual expression.
    fn execute_expr(&mut self, statement: &'a Rc<ASTNode>) -> Option<Rc<ASTNode>> {
        match &**statement {
            ASTNode::Assign { id, value } => {
                let resolved_expr = &self
                    .execute_expr(&value)
                    .expect("expected expression after variable assignment.");
                self.assign(id.clone(), (&*resolved_expr.clone()).clone());
                None
            }
            ASTNode::Declare { id, value } => {
                let resolved_expr = &self
                    .execute_expr(&value)
                    .expect("expected expression after variable declaration.");
                self.declare(id.clone(), (&*resolved_expr.clone()).clone());
                None
            }
            ASTNode::Function {
                ref id,
                arguments: ref _arguments,
                body: ref _body,
            } => {
                self.set_fn(id.clone(), &*statement);
                None
            }
            ASTNode::FunctionCall {
                id,
                arguments: call_args,
            } => {
                // execute function
                let function = self.get_fn(id.clone());
                if let ASTNode::Function {
                    id: _id,
                    arguments: fn_args,
                    body,
                } = &**function
                {
                    // push arguments
                    assert_eq!(call_args.len(), fn_args.len());
                    self.scope += 1;
                    for (idx, arg) in fn_args.iter().enumerate() {
                        let arg_expr = call_args.get(idx).unwrap(); // safety: assertion
                        let resolved_expr = self.execute_expr(arg_expr).unwrap().clone();
                        self.declare(arg.clone(), (&*resolved_expr.clone()).clone());
                    }

                    // if no return, drop scoped variables anyway
                    let result = self.execute(body);
                    self.scope -= 1;
                    self.drop();
                    return result;
                }
                None
            }
            ASTNode::Op { lhs, op, rhs } => {
                if let (Some(a), Some(b)) = (self.execute_expr(&lhs), self.execute_expr(&rhs)) {
                    if let (
                        ASTNode::Literal(Token::Number(a)),
                        ASTNode::Literal(Token::Number(b)),
                    ) = (&*a, &*b)
                    {
                        match op {
                            // math operators
                            Token::Add => {
                                return Some(ASTNode::Literal(Token::Number(a + b)).into())
                            }
                            Token::Sub => {
                                return Some(ASTNode::Literal(Token::Number(a - b)).into())
                            }
                            Token::Mul => {
                                return Some(ASTNode::Literal(Token::Number(a * b)).into())
                            }
                            Token::Div => {
                                return Some(ASTNode::Literal(Token::Number(a / b)).into())
                            }
                            // logical operators
                            Token::LogicalG => {
                                return Some(ASTNode::Literal(Token::Bool(a > b)).into())
                            }
                            Token::LogicalGe => {
                                return Some(ASTNode::Literal(Token::Bool(a >= b)).into())
                            }
                            Token::LogicalL => {
                                return Some(ASTNode::Literal(Token::Bool(a < b)).into())
                            }
                            Token::LogicalLe => {
                                return Some(ASTNode::Literal(Token::Bool(a <= b)).into())
                            }
                            _ => {
                                panic!("operator not implemented.");
                            }
                        }
                    }
                    return None;
                } else {
                    return None;
                }
            }
            ASTNode::Conditional {
                condition,
                if_body,
                else_body,
            } => {
                if let Some(condition) = self.execute_expr(&condition) {
                    // increase scope level and execute body statements
                    self.scope += 1;
                    if let ASTNode::Literal(Token::Bool(true)) = *condition {
                        if let Some(result) = self.execute(if_body) {
                            self.scope -= 1;
                            self.drop();
                            return Some(result);
                        }
                    } else {
                        if let Some(result) = self.execute(else_body) {
                            self.scope -= 1;
                            self.drop();
                            return Some(result);
                        } else {
                        }
                    }
                    // after finishing, decrease scope level and drop locals
                    self.scope -= 1;
                    self.drop();
                }
                None
            }
            ASTNode::Loop { condition, body } => {
                // increase scope level and execute body
                self.scope += 1;
                while let Some(condition) = self.execute_expr(&condition) {
                    // run loop body
                    if let ASTNode::Literal(Token::Bool(true)) = *condition {
                        self.execute(body);
                    } else {
                        break;
                    }

                    // drop any variables created inside
                    self.drop_here();
                }
                // after finishing, decrease scope level and drop locals
                self.scope -= 1;
                self.drop();
                None
            }
            ASTNode::Literal(ref t) => {
                if let Token::Identifier(identifier) = t {
                    // get variable value if applicable
                    Some(ASTNode::Literal(self.get(identifier.clone())).into())
                } else {
                    // otherwise, return raw literal
                    Some(statement.clone())
                }
            }
            ASTNode::Return(ref expr) => Some(
                self.execute_expr(expr)
                    .expect("expected return expression."),
            ),
            _ => {
                todo!()
            }
        }
    }
}
