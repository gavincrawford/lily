//! The interpreter executes an abstract syntax tree.

mod mem;
mod tests;

use crate::{lexer::Token, parser::ASTNode};
use mem::variable::Variable;
use std::{collections::HashMap, rc::Rc};

pub struct Interpreter<'a> {
    /// Variable storage.
    pub variables: Vec<HashMap<String, Variable<'a>>>,
    /// Function table.
    pub functions: Vec<HashMap<String, &'a Rc<ASTNode>>>,
    /// Module prefix, if applicable.
    module: Option<String>,
    /// Scope level.
    scope: usize,
}
impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        Self {
            variables: Vec::with_capacity(8),
            functions: Vec::with_capacity(8),
            module: None,
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
                self.assign(id, Variable::Owned((*resolved_expr.to_owned()).to_owned()));
                None
            }
            ASTNode::Declare { id, value } => {
                let resolved_expr = &self
                    .execute_expr(&value)
                    .expect("expected expression after variable declaration.");
                self.declare(id, Variable::Owned(ASTNode::inner_to_owned(&resolved_expr)));
                None
            }
            ASTNode::Function {
                ref id,
                arguments: ref _arguments,
                body: ref _body,
            } => {
                if let Some(prefix) = &self.module {
                    let old_scope = self.scope;
                    self.scope = 0;
                    self.declare(
                        [prefix.to_owned(), id.to_owned()].join("."),
                        Variable::Reference(&*statement),
                    );
                    self.scope = old_scope;
                }
                self.declare(id, Variable::Reference(&*statement));
                None
            }
            ASTNode::FunctionCall {
                id,
                arguments: call_args,
            } => {
                // execute function
                let variable = self.get_owned(id);
                if let Variable::Reference(function) = variable {
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
                            let resolved_expr = self.execute_expr(arg_expr).unwrap().to_owned();
                            self.declare(
                                arg,
                                Variable::Owned(ASTNode::inner_to_owned(&resolved_expr)),
                            );
                        }

                        // if no return, drop scoped variables anyway
                        let result = self.execute(body);
                        self.scope -= 1;
                        self.drop();
                        return result;
                    }
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
                            Token::Pow => {
                                return Some(ASTNode::Literal(Token::Number(a.powf(*b))).into())
                            }

                            // logical operators
                            Token::LogicalEq => {
                                return Some(ASTNode::Literal(Token::Bool(a == b)).into())
                            }
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
            ASTNode::List(_) => {
                // return self
                return Some(statement.to_owned());
            }
            ASTNode::Index { id, index } => {
                // get index as a usize
                let usize_idx;
                if let ASTNode::Literal(Token::Number(n)) = &*self.execute_expr(index).unwrap() {
                    usize_idx = n.to_owned() as usize;
                } else {
                    panic!("index must be positive and a number.");
                }

                // get value from list
                if let Variable::Owned(list) = self.get(id) {
                    if let ASTNode::List(tokens) = &*list {
                        return Some(
                            ASTNode::Literal(
                                tokens
                                    .get(usize_idx.to_owned() as usize)
                                    .expect("index out of bounds.")
                                    .to_owned(),
                            )
                            .into(),
                        );
                    }
                }

                // if return hasn't been reached, panic
                panic!("invalid index expression.");
            }
            ASTNode::Literal(ref t) => {
                if let Token::Identifier(identifier) = t {
                    if let Variable::Owned(var) = self.get(identifier) {
                        // reutrn owned variables
                        return Some(var.to_owned().into());
                    }
                    None
                } else {
                    // otherwise, return raw literal
                    Some(statement.to_owned())
                }
            }
            ASTNode::Return(ref expr) => Some(
                self.execute_expr(expr)
                    .expect("expected return expression."),
            ),
            ASTNode::Module { alias, body } => {
                // add alias if applicable
                self.module = alias.to_owned();

                // execute modules to add them to the scope
                self.execute(&*body);

                // remove alias
                self.module = None;

                None
            }
            _ => {
                todo!()
            }
        }
    }
}
