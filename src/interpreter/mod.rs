//! The interpreter executes an abstract syntax tree.

mod mem;
mod tests;

use crate::{
    lexer::Token,
    parser::{ASTNode, ID},
};
use anyhow::{bail, Context, Result};
use mem::{svtable::SVTable, variable::Variable};
use std::{collections::HashMap, rc::Rc};

pub struct Interpreter<'a> {
    /// Module storage. Variables at base scope are stored in the `$` module.
    pub modules: HashMap<String, SVTable<'a>>,
    /// Current module name.
    mod_id: String,
    /// Scope level.
    scope: usize,
}
impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        // create a new module map with default scope
        let mut modules = HashMap::new();
        modules.insert("$".into(), SVTable::new());

        // return new interpreter
        Self {
            modules,
            mod_id: "$".into(),
            scope: 0,
        }
    }

    /// Executes an AST segment, typically the head. Returns `Some` when a return block is reached.
    pub fn execute(&mut self, ast: &'a Rc<ASTNode>) -> Result<Option<Rc<ASTNode>>> {
        if let ASTNode::Block(statements) = &**ast {
            // if this segment is a block, execute all of its statements
            for statement in statements {
                if let Some(ret_value) = self
                    .execute_expr(statement)
                    .context("failed to evaluate expression")?
                {
                    if self.scope > 0 {
                        return Ok(Some(ret_value));
                    } else {
                        // TODO this still doesn't prevent return from being called inside a
                        // conditional, so maybe add a syntax error for that too
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
    fn execute_expr(&mut self, statement: &'a Rc<ASTNode>) -> Result<Option<Rc<ASTNode>>> {
        match &**statement {
            ASTNode::Assign { id, value } => {
                let resolved_expr = &self
                    .execute_expr(&value)
                    .context("expected expression after variable assignment")?
                    .unwrap();
                self.assign(id, Variable::Owned((*resolved_expr.to_owned()).to_owned()));
                Ok(None)
            }
            ASTNode::Declare { id, value } => {
                let resolved_expr = &self
                    .execute_expr(&value)
                    .context("expected expression after variable declaration")?
                    .unwrap();
                self.declare(id, Variable::Owned(ASTNode::inner_to_owned(&resolved_expr)));
                Ok(None)
            }
            ASTNode::Function {
                ref id,
                arguments: ref _arguments,
                body: ref _body,
            } => {
                self.declare(id, Variable::Reference(&*statement));
                Ok(None)
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
                            let resolved_expr = self
                                .execute_expr(arg_expr)
                                .context("failed to evaluate argument")?
                                .unwrap()
                                .to_owned();
                            self.declare(
                                &ID::new(arg),
                                Variable::Owned(ASTNode::inner_to_owned(&resolved_expr)),
                            );
                        }

                        // if no return, drop scoped variables anyway
                        let result = self.execute(body)?;
                        self.scope -= 1;
                        self.drop();
                        return Ok(result);
                    }
                }
                Ok(None)
            }
            ASTNode::Op { lhs, op, rhs } => {
                if let (Ok(Some(a)), Ok(Some(b))) =
                    (self.execute_expr(&lhs), self.execute_expr(&rhs))
                {
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
                    .execute_expr(&condition)
                    .context("failed to evaluate condition")?
                {
                    // increase scope level and execute body statements
                    self.scope += 1;
                    if let ASTNode::Literal(Token::Bool(true)) = *condition {
                        if let Some(result) = self.execute(if_body)? {
                            self.scope -= 1;
                            self.drop();
                            return Ok(Some(result));
                        }
                    } else {
                        if let Some(result) = self.execute(else_body)? {
                            self.scope -= 1;
                            self.drop();
                            return Ok(Some(result));
                        } else {
                        }
                    }
                    // after finishing, decrease scope level and drop locals
                    self.scope -= 1;
                    self.drop();
                }
                Ok(None)
            }
            ASTNode::Loop { condition, body } => {
                // increase scope level and execute body
                self.scope += 1;
                while let Some(condition) = self.execute_expr(&condition)? {
                    // run loop body
                    if let ASTNode::Literal(Token::Bool(true)) = *condition {
                        self.execute(body)?;
                    } else {
                        break;
                    }

                    // drop any variables created inside
                    self.drop_here();
                }
                // after finishing, decrease scope level and drop locals
                self.scope -= 1;
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
                    .execute_expr(index)
                    .context("failed to evaluate index value")?
                    .unwrap()
                {
                    usize_idx = n.to_owned() as usize;
                } else {
                    panic!("index must be positive and a number.");
                }

                // get value from list
                if let Variable::Owned(list) = self.get(id) {
                    if let ASTNode::List(tokens) = &*list {
                        return Ok(Some(
                            ASTNode::Literal(
                                tokens
                                    .get(usize_idx.to_owned() as usize)
                                    .expect("index out of bounds.")
                                    .to_owned(),
                            )
                            .into(),
                        ));
                    }
                }

                // if return hasn't been reached, panic
                panic!("invalid index expression.");
            }
            ASTNode::Literal(ref t) => {
                if let Token::Identifier(identifier) = t {
                    if let Variable::Owned(var) = self.get(&ID::new(identifier)) {
                        // reutrn owned variables
                        return Ok(Some(var.to_owned().into()));
                    }
                    Ok(None)
                } else {
                    // otherwise, return raw literal
                    Ok(Some(statement.to_owned()))
                }
            }
            ASTNode::Return(ref expr) => Ok(Some(
                self.execute_expr(expr)
                    .context("failed to evaluate return expression")?
                    .expect("expected return expression."),
            )),
            ASTNode::Module { alias, body } => {
                // TODO keep proper track of these things. i think it might be adventageous to just
                // merge the variable table with the module table and have the default scope be
                // named something reserved

                if let Some(mod_name) = alias {
                    // insert named modules
                    self.modules.insert(mod_name.to_owned(), SVTable::new());
                    let temp = self.mod_id.to_owned();
                    self.mod_id = mod_name.to_owned();

                    // execute body
                    self.execute(&*body)
                        .context("failed to evaluate module body")?;
                    self.mod_id = temp;
                } else {
                    // insert named modules
                    let temp = self.mod_id.to_owned();
                    self.mod_id = String::from("$");

                    // execute body
                    self.execute(&*body)
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
