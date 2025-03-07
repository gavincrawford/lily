//! The interpreter executes an abstract syntax tree.

mod mem;
mod resolve_refs;
mod tests;

use crate::{
    lexer::Token,
    parser::{ASTNode, ID},
};
use anyhow::{bail, Context, Result};
use mem::{svtable::SVTable, variable::Variable};
use std::{cell::RefCell, rc::Rc};

pub struct Interpreter {
    /// Memory structure. Tracks variables and modules.
    pub memory: Rc<RefCell<SVTable>>,
    /// Current module.
    mod_id: Option<Rc<RefCell<SVTable>>>,
    /// Scope level.
    scope_id: usize,
}
impl Interpreter {
    pub fn new() -> Self {
        // return new interpreter
        Self {
            memory: Rc::new(RefCell::new(SVTable::new())),
            mod_id: None,
            scope_id: 0,
        }
    }

    /// Executes an AST segment, typically the head. Returns `Some` when a return block is reached.
    pub fn execute(&mut self, ast: Rc<ASTNode>) -> Result<Option<Rc<ASTNode>>> {
        if let ASTNode::Block(statements) = &*ast {
            // if this segment is a block, execute all of its statements
            for statement in statements {
                if let Some(ret_value) = self
                    .execute_expr(statement.clone())
                    .context("failed to evaluate expression")?
                {
                    if self.scope_id > 0 {
                        return Ok(Some(ret_value));
                    } else {
                        bail!("return cannot be called outside of a function");
                    }
                }
            }
        } else {
            // otherwise, execute the segment by itself
            self.execute_expr(ast)?;
        }
        Ok(None)
    }

    /// Executes an individual expression.
    fn execute_expr(&mut self, statement: Rc<ASTNode>) -> Result<Option<Rc<ASTNode>>> {
        match &*statement {
            ASTNode::Assign { id, value } => {
                let resolved_expr = &self
                    .execute_expr(value.clone())
                    .context("failed to evaluate assignment value")?
                    .unwrap();
                self.assign(id, Variable::Owned((*resolved_expr.to_owned()).to_owned()))?;
                Ok(None)
            }
            ASTNode::Declare { id, value } => {
                let resolved_expr = &self
                    .execute_expr(value.clone())
                    .context("failed to evaluate declaration value")?
                    .unwrap();
                self.declare(id, Variable::Owned(ASTNode::inner_to_owned(&resolved_expr)))?;
                Ok(None)
            }
            ASTNode::Function {
                ref id,
                arguments: ref _arguments,
                body: ref _body,
            } => {
                self.declare(id, Variable::Reference(statement.to_owned()))?;
                Ok(None)
            }
            ASTNode::FunctionCall {
                id,
                arguments: call_args,
            } => {
                // execute function
                let variable = self.get_owned(id)?;
                if let Variable::Reference(function) = variable {
                    if let ASTNode::Function {
                        id: _id,
                        arguments: fn_args,
                        body,
                    } = &*function
                    {
                        // push arguments
                        assert_eq!(call_args.len(), fn_args.len());
                        self.scope_id += 1;
                        for (idx, arg) in fn_args.iter().enumerate() {
                            let arg_expr = call_args.get(idx).unwrap(); // safety: assertion
                            let resolved_expr = self
                                .execute_expr(arg_expr.clone())
                                .context("failed to evaluate argument")?
                                .unwrap()
                                .to_owned();
                            self.declare(
                                &ID::new(arg),
                                Variable::Owned(ASTNode::inner_to_owned(&resolved_expr)),
                            )?;
                        }

                        // if no return, drop scoped variables anyway
                        let result = self.execute(body.clone())?;
                        self.scope_id -= 1;
                        self.drop();
                        return Ok(result);
                    }
                }
                Ok(None)
            }
            ASTNode::Op { lhs, op, rhs } => {
                if let (Ok(Some(a)), Ok(Some(b))) = (
                    self.execute_expr(lhs.clone()),
                    self.execute_expr(rhs.clone()),
                ) {
                    if let (
                        ASTNode::Literal(Token::Number(a)),
                        ASTNode::Literal(Token::Number(b)),
                    ) = (&*a, &*b)
                    {
                        match op {
                            // math operators
                            Token::Add => {
                                return Ok(Some(ASTNode::Literal(Token::Number(a + b)).into()))
                            }
                            Token::Sub => {
                                return Ok(Some(ASTNode::Literal(Token::Number(a - b)).into()))
                            }
                            Token::Mul => {
                                return Ok(Some(ASTNode::Literal(Token::Number(a * b)).into()))
                            }
                            Token::Div => {
                                return Ok(Some(ASTNode::Literal(Token::Number(a / b)).into()))
                            }
                            Token::Pow => {
                                return Ok(Some(ASTNode::Literal(Token::Number(a.powf(*b))).into()))
                            }

                            // logical operators
                            Token::LogicalEq => {
                                return Ok(Some(ASTNode::Literal(Token::Bool(a == b)).into()))
                            }
                            Token::LogicalNeq => {
                                return Ok(Some(ASTNode::Literal(Token::Bool(a != b)).into()))
                            }
                            Token::LogicalG => {
                                return Ok(Some(ASTNode::Literal(Token::Bool(a > b)).into()))
                            }
                            Token::LogicalGe => {
                                return Ok(Some(ASTNode::Literal(Token::Bool(a >= b)).into()))
                            }
                            Token::LogicalL => {
                                return Ok(Some(ASTNode::Literal(Token::Bool(a < b)).into()))
                            }
                            Token::LogicalLe => {
                                return Ok(Some(ASTNode::Literal(Token::Bool(a <= b)).into()))
                            }
                            _ => {
                                panic!("operator not implemented.");
                            }
                        }
                    }
                    return Ok(None);
                } else {
                    return Ok(None);
                }
            }
            ASTNode::Conditional {
                condition,
                if_body,
                else_body,
            } => {
                if let Some(condition) = self
                    .execute_expr(condition.clone())
                    .context("failed to evaluate condition")?
                {
                    // increase scope level and execute body statements
                    self.scope_id += 1;
                    if let ASTNode::Literal(Token::Bool(true)) = *condition {
                        if let Some(result) = self.execute(if_body.clone())? {
                            self.scope_id -= 1;
                            self.drop();
                            return Ok(Some(result));
                        }
                    } else {
                        if let Some(result) = self.execute(else_body.clone())? {
                            self.scope_id -= 1;
                            self.drop();
                            return Ok(Some(result));
                        } else {
                        }
                    }
                    // after finishing, decrease scope level and drop locals
                    self.scope_id -= 1;
                    self.drop();
                }
                Ok(None)
            }
            ASTNode::Loop { condition, body } => {
                // increase scope level and execute body
                self.scope_id += 1;
                while let Some(condition) = self.execute_expr(condition.clone())? {
                    // run loop body
                    if let ASTNode::Literal(Token::Bool(true)) = *condition {
                        self.execute(body.clone())?;
                    } else {
                        break;
                    }

                    // drop any variables created inside
                    self.drop_here();
                }
                // after finishing, decrease scope level and drop locals
                self.scope_id -= 1;
                self.drop();
                Ok(None)
            }
            ASTNode::List(_) => {
                // return self
                return Ok(Some(statement.to_owned()));
            }
            ASTNode::Index { id, index } => {
                // get index as a usize
                let usize_idx;
                if let ASTNode::Literal(Token::Number(n)) = &*self
                    .execute_expr(index.clone())
                    .context("failed to evaluate index value")?
                    .unwrap()
                {
                    usize_idx = n.to_owned() as usize;
                } else {
                    panic!("index must be positive and a number.");
                }

                // get value from list
                let list = &*(self.get(id)?);
                if let Variable::Owned(ASTNode::List(tokens)) = &*list.borrow() {
                    return Ok(Some(
                        tokens
                            .get(usize_idx.to_owned() as usize)
                            .expect("index out of bounds.")
                            .to_owned(),
                    ));
                }

                // if return hasn't been reached, panic
                panic!("invalid index expression.");
            }
            ASTNode::Literal(ref t) => {
                if let Token::Identifier(identifier) = t {
                    // PERF implement with borrow
                    if let Variable::Owned(var) = self.get_owned(&ID::new(identifier))? {
                        // reutrn owned variables
                        return Ok(Some(var.to_owned().into()));
                    }
                    Ok(None)
                } else {
                    // otherwise, return raw literal
                    Ok(Some(statement.to_owned()))
                }
            }
            ASTNode::Return(ref expr) => {
                // resolve expression
                let expr = self
                    .execute_expr(expr.clone())
                    .context("failed to evaluate return expression")?
                    .expect("expected return expression");

                // if there are indicies, flatten them
                let expr = match *expr {
                    ASTNode::Index { id: _, index: _ } => self
                        .execute_expr(expr)
                        .context("could not flatten index")?
                        .unwrap(),
                    ASTNode::List(_) => {
                        self.resolve_refs(ASTNode::inner_to_owned(&expr))
                            .context("could not flatten list")?;
                        expr.into()
                    }
                    _ => expr.clone(),
                };

                Ok(Some(expr))
            }
            ASTNode::Module { alias, body } => {
                if let Some(mod_name) = alias {
                    // insert named modules
                    let temp = self.mod_id.to_owned();
                    if let Some(mod_pointer) = temp.to_owned() {
                        self.mod_id =
                            Some(mod_pointer.borrow_mut().add_module(mod_name.to_owned()));
                    } else {
                        self.mod_id =
                            Some(self.memory.borrow_mut().add_module(mod_name.to_owned()));
                    }

                    // execute body
                    self.execute(body.clone())
                        .context("failed to evaluate module body")?;
                    self.mod_id = temp;
                } else {
                    // insert unnamed modules
                    let temp = self.mod_id.to_owned();
                    self.mod_id = None;

                    // execute body
                    self.execute(body.clone())
                        .context("failed to evaluate module body")?;
                    self.mod_id = temp;
                }
                Ok(None)
            }
            _ => {
                todo!()
            }
        }
    }
}
