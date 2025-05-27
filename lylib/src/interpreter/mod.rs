//! The interpreter executes an abstract syntax tree.

mod execute_function;
mod id;
mod mem;
mod resolve_refs;
mod tests;

use crate::{lexer::Token, parser::ASTNode};
use anyhow::{bail, Context, Result};
use std::{cell::RefCell, rc::Rc};

pub use id::*;
pub use mem::{svtable::SVTable, variable::*};

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
                    return Ok(Some(ret_value));
                }
            }
        } else {
            // otherwise, execute the segment by itself
            self.execute_expr(ast)
                .context("failed to execute expression")?;
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
                target,
                arguments: call_args,
            } => {
                // get function reference, bail if none found
                if let ASTNode::Literal(Token::Identifier(id)) = &**target {
                    let id = ID::new(id);
                    let variable = self.get_owned(&id)?;
                    match variable {
                        // this branch should trigger on raw, local functions
                        Variable::Reference(function) => {
                            if let ASTNode::Function {
                                id: _,
                                arguments: _,
                                body: _,
                            } = &*function
                            {
                                return Ok(self.execute_function(call_args, function)?);
                            } else {
                                bail!("attempted to call non-function");
                            }
                        }

                        // this branch should trigger when constructors are called
                        Variable::Type(ref structure) => match structure.constructor() {
                            Some(v) => {
                                // get default svt
                                let svt = structure
                                    .default_svt()
                                    .context("cannot add struct default variables")
                                    .unwrap();

                                // use the new structure svt as module for this constructor
                                let svt = Rc::new(RefCell::new(svt));
                                let temp = self.mod_id.clone();
                                self.mod_id = Some(svt.clone());

                                // execute function
                                self.execute_function(call_args, v)?;

                                // reset module ID
                                self.mod_id = temp;

                                // return newly made instance
                                return Ok(Some(
                                    ASTNode::Instance {
                                        kind: variable.into(),
                                        id: id.clone(),
                                        svt,
                                    }
                                    .into(),
                                ));
                            }
                            None => {
                                // get default svt
                                let svt = structure
                                    .default_svt()
                                    .context("cannot add struct default variables")
                                    .unwrap();

                                // return newly made instance
                                return Ok(Some(
                                    ASTNode::Instance {
                                        kind: variable.into(),
                                        id: id.clone(),
                                        svt: RefCell::new(svt).into(),
                                    }
                                    .into(),
                                ));
                            }
                        },

                        // catch others
                        _ => {
                            bail!("no function `{:?}` found", target);
                        }
                    };
                }
                bail!("malformed function target")
            }
            ASTNode::Struct { id, body: _ } => {
                self.declare(id, Variable::Type(statement.to_owned()))
                    .context("failed to declare type for structure")?;
                Ok(None)
            }
            ASTNode::Op { lhs, op, rhs } => {
                if let (Ok(Some(a)), Ok(Some(b))) = (
                    self.execute_expr(lhs.clone()),
                    self.execute_expr(rhs.clone()),
                ) {
                    use ASTNode::*;
                    use Token::*;
                    match (op, &*a, &*b) {
                        // math operators
                        (Add, Literal(Number(a)), Literal(Number(b))) => {
                            return Ok(Some(Literal(Number(a + b)).into()))
                        }
                        (Sub, Literal(Number(a)), Literal(Number(b))) => {
                            return Ok(Some(Literal(Number(a - b)).into()))
                        }
                        (Div, Literal(Number(a)), Literal(Number(b))) => {
                            return Ok(Some(Literal(Number(a / b)).into()))
                        }
                        (Mul, Literal(Number(a)), Literal(Number(b))) => {
                            return Ok(Some(Literal(Number(a * b)).into()))
                        }
                        (Pow, Literal(Number(a)), Literal(Number(b))) => {
                            return Ok(Some(Literal(Number(a.powf(*b))).into()))
                        }

                        // logical operators
                        (LogicalEq, Literal(Number(a)), Literal(Number(b))) => {
                            return Ok(Some(Literal(Bool(a == b)).into()))
                        }
                        (LogicalNeq, Literal(Number(a)), Literal(Number(b))) => {
                            return Ok(Some(Literal(Bool(a != b)).into()))
                        }
                        (LogicalG, Literal(Number(a)), Literal(Number(b))) => {
                            return Ok(Some(Literal(Bool(a > b)).into()))
                        }
                        (LogicalGe, Literal(Number(a)), Literal(Number(b))) => {
                            return Ok(Some(Literal(Bool(a >= b)).into()))
                        }
                        (LogicalL, Literal(Number(a)), Literal(Number(b))) => {
                            return Ok(Some(Literal(Bool(a < b)).into()))
                        }
                        (LogicalLe, Literal(Number(a)), Literal(Number(b))) => {
                            return Ok(Some(Literal(Bool(a <= b)).into()))
                        }
                        _ => {
                            bail!("operator not implemented")
                        }
                    }
                } else {
                    bail!("failed to evaluate operands")
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
            ASTNode::Index { target, index } => {
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

                // get list
                let list = self
                    .execute_expr(target.to_owned())
                    .context("failed to evaluate index target")?
                    .unwrap();

                // find list item if applicable, bail otherwise
                if let ASTNode::List(tokens) = &*list {
                    return Ok(Some(
                        tokens
                            .get(usize_idx.to_owned() as usize)
                            .expect("index out of bounds.")
                            .to_owned(),
                    ));
                } else {
                    bail!("expected list as index target");
                }
            }
            ASTNode::Literal(ref t) => {
                if let Token::Identifier(identifier) = t {
                    // if this literal is an identifier, return the internal value
                    if let Variable::Owned(var) = self.get_owned(&ID::new(identifier))? {
                        return Ok(Some(var.into()));
                    }
                    Ok(None)
                } else {
                    // otherwise, return raw literal without destructuring
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
                let expr = self
                    .resolve_refs(ASTNode::inner_to_owned(&expr))
                    .context("could not flatten references")?;

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
