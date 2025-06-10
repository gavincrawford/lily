//! The interpreter executes an abstract syntax tree.

mod builtins;
mod execute_function;
mod id;
mod mem;
mod resolve_refs;
mod tests;

use crate::{lexer::Token, parser::ASTNode};
use anyhow::{bail, Context, Result};
use std::{
    cell::RefCell,
    io::{Read, Write},
    rc::Rc,
};

pub(crate) use id::*;
pub(crate) use mem::{svtable::SVTable, variable::*};

pub struct Interpreter<Out: Write, In: Read> {
    /// Memory structure. Tracks variables and modules.
    pub memory: Rc<RefCell<SVTable>>,
    /// Current module.
    mod_id: Option<Rc<RefCell<SVTable>>>,
    /// Scope level.
    scope_id: usize,
    /// Output buffer. Typically `stdout`.
    output: Rc<RefCell<Out>>,
    /// Input buffer. Typically `stdin`.
    input: Rc<RefCell<In>>,
}
impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Creates a new interpreter with default builtins.
    pub fn new(input: In, output: Out) -> Self {
        let mut i = Self {
            memory: Rc::new(RefCell::new(SVTable::new())),
            mod_id: None,
            scope_id: 0,
            output: Rc::new(RefCell::new(output)),
            input: Rc::new(RefCell::new(input)),
        };
        i.inject_builtins()
            .context("failed to add builtins")
            .unwrap();
        i
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
            ASTNode::Assign { target, value } => {
                // resolve target & expression
                let resolved_target = &ID::node_to_id(target.clone())
                    .context("failed to evaluate assignment target")?;
                let resolved_expr = self
                    .execute_expr(value.clone())
                    .context("failed to evaluate assignment value")?
                    .unwrap();

                // assign variable
                self.assign(
                    resolved_target,
                    Variable::Owned(ASTNode::inner_to_owned(&resolved_expr)),
                )?;
                Ok(None)
            }
            ASTNode::Declare { target, value } => {
                // resolve target & expression
                let resolved_target = &ID::node_to_id(target.clone())
                    .context("failed to evaluate declaration target")?;
                let resolved_expr = self
                    .execute_expr(value.clone())
                    .context("failed to evaluate declaration value")?
                    .unwrap();

                // declare variable
                self.declare(
                    resolved_target,
                    Variable::Owned(ASTNode::inner_to_owned(&resolved_expr)),
                )?;
                Ok(None)
            }
            ASTNode::Function {
                ref id,
                arguments: ref _arguments,
                body: ref _body,
            } => {
                self.declare(id, Variable::Function(statement.to_owned()))?;
                Ok(None)
            }
            ASTNode::FunctionCall { target, arguments } => {
                // get function reference, bail if none found
                if let ASTNode::Literal(Token::Identifier(id)) = &**target {
                    let id = ID::new(id);
                    let variable = self.get(&id)?;
                    match variable {
                        // this branch should trigger on external functions
                        Variable::Extern(closure) => {
                            return closure(
                                self.output.clone(),
                                self.input.clone(),
                                &arguments
                                    .iter()
                                    .map(|arg| {
                                        // TODO no unwrap, especially here
                                        self.execute_expr(arg.clone()).unwrap().unwrap().to_owned()
                                    })
                                    .collect::<Vec<Rc<ASTNode>>>(),
                            );
                        }

                        // this branch should trigger on raw, local functions
                        Variable::Function(function) => {
                            if let ASTNode::Function {
                                id: _,
                                arguments: _,
                                body: _,
                            } = &*function
                            {
                                return Ok(self.execute_function(arguments, function)?);
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
                                self.execute_function(arguments, v)?;

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
                    macro_rules! opmatch {
                        (match $op:expr, $lhs:expr, $rhs:expr => $locallhs:pat, $localrhs:pat if $($pat:pat => $res:expr),*) => {
                            match ($op, $lhs, $rhs) {
                                $(($pat, ASTNode::Literal($locallhs), ASTNode::Literal($localrhs)) => {
                                    return Ok(Some(Rc::new(ASTNode::Literal($res))))
                                })*
                                _ => {},
                            }
                        };
                    }

                    use Token::*;
                    opmatch!(
                        match op, &*a, &*b => Number(l), Number(r) if
                        Add => Number(l + r),
                        Sub => Number(l - r),
                        Mul => Number(l * r),
                        Div => Number(l / r),
                        Floor => Number((l / r).floor()),
                        Pow => Number(l.powf(*r)),
                        LogicalG => Bool(l > r),
                        LogicalGe => Bool(l >= r),
                        LogicalL => Bool(l < r),
                        LogicalLe => Bool(l <= r)
                    );
                    opmatch!(
                        match op, &*a, &*b => Str(l), Str(r) if
                        Add => Str(l.clone() + r)
                    );
                    opmatch!(
                        match op, &*a, &*b => l, r if
                        LogicalEq => Bool(l == r),
                        LogicalNeq => Bool(l != r)
                    );
                    bail!("operator not implemented")
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
                    if condition.is_truthy() {
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
                        }
                    }
                    // after finishing, decrease scope level and drop locals
                    self.scope_id -= 1;
                    self.drop();
                }
                Ok(None)
            }
            ASTNode::Loop { condition, body } => {
                // create result buffer, default none
                let mut result = None;

                // increase scope level and execute body
                self.scope_id += 1;
                while let Some(condition) = self.execute_expr(condition.clone())? {
                    // if condition is true, execute body
                    if condition.is_truthy() {
                        result = self.execute(body.clone())?;

                        // if a value was returned, break
                        if result.is_some() {
                            break;
                        }
                    } else {
                        break;
                    }

                    // after each execution of the loop, clear values at this scope
                    self.drop_here();
                }

                // after finishing, decrease scope level and drop locals
                self.scope_id -= 1;
                self.drop();
                Ok(result)
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
                    // guard numbers outside of range
                    if *n < 0. {
                        bail!("index values must be non-negative");
                    } else if *n > usize::MAX as f32 {
                        bail!("index value larger than {}", usize::MAX);
                    }

                    // convert index to usize for later use
                    usize_idx = n.to_owned() as usize;
                } else {
                    panic!("index must be positive and a number");
                }

                // get list
                let list = self
                    .execute_expr(target.to_owned())
                    .context("failed to evaluate index target")?
                    .unwrap();

                // find list item if applicable, bail otherwise
                if let ASTNode::List(list_table) = &*list {
                    let mut items = list_table.borrow_mut();
                    // TODO ew. helpers?
                    if let Variable::Owned(value) = &*items
                        .get_scope(0)
                        .unwrap()
                        .get(&usize_idx.to_string())
                        .unwrap_or(&Rc::new(RefCell::new(Variable::Owned(ASTNode::Literal(
                            Token::Undefined,
                        )))))
                        .borrow()
                    {
                        return Ok(Some(value.to_owned().into()));
                    }
                    panic!();
                } else {
                    bail!("expected list as index target");
                }
            }
            ASTNode::Literal(ref t) => {
                if let Token::Identifier(identifier) = t {
                    // if this literal is an identifier, return the internal value
                    if let Variable::Owned(var) = self.get(&ID::new(identifier))? {
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
