//! The interpreter executes an abstract syntax tree.

mod builtins;
mod execute_function;
mod id;
mod mem;
mod node_to_id;
mod resolve_refs;
mod tests;

use crate::{lexer::Token, parser::ASTNode, *};
use anyhow::{bail, Context, Result};
use std::{
    cell::RefCell,
    io::{Read, Write},
    rc::Rc,
};

pub(crate) use id::*;
pub(crate) use mem::{svtable::SVTable, variable::*, MemoryInterface};

/// The interpreter executes Abstract Syntax Trees (ASTs) and manages program state.
#[derive(Debug)]
pub struct Interpreter<Out: Write, In: Read> {
    /// Base-scope memory table. Tracks all locals.
    pub memory: Rc<RefCell<SVTable>>,
    /// Current memory context.
    /// `Some` when interpreter is working with another module's local memory.
    /// `None` when interpreter is working in base-scope memory.
    context: Option<Rc<RefCell<SVTable>>>,
    /// Scope level.
    scope_id: usize,
    /// Output buffer. Typically `stdout`.
    output: Out,
    /// Input buffer. Typically `stdin`.
    input: In,
}
impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Creates a new interpreter with default builtins.
    pub fn new(input: In, output: Out) -> Self {
        let mut i = Self {
            memory: Rc::new(RefCell::new(SVTable::new())),
            context: None,
            scope_id: 0,
            output,
            input,
        };
        i.inject_builtins()
            .context("failed to add builtins")
            .unwrap();
        i
    }

    /// Gets a reference to the internal input reader.
    pub fn input(&mut self) -> &mut In {
        &mut self.input
    }

    /// Gets a reference to the internal output writer.
    pub fn output(&mut self) -> &mut Out {
        &mut self.output
    }

    /// Executes a closure with a temporary memory context, restoring the previous context after
    /// execution has completed. Propagates all errors.
    #[inline]
    fn with_context<T, F>(&mut self, temp_context: Option<Rc<RefCell<SVTable>>>, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let previous_context = self.context.clone();
        self.context = temp_context;
        let result = f(self);
        self.context = previous_context;
        result
    }

    /// Executes an AST segment, typically the head. Returns `Some` when a return block is reached.
    pub fn execute(&mut self, ast: Rc<ASTNode>) -> Result<Option<Rc<ASTNode>>> {
        if let ASTNode::Block(statements) = &*ast {
            // if this segment is a block, execute all of its statements
            for statement in statements {
                if let Some(ret_value) = self
                    .execute_expr(statement)
                    .context("failed to evaluate expression")?
                {
                    if self.scope_id == 0 {
                        bail!("cannot return as base scope");
                    }
                    return Ok(Some(ret_value));
                }
            }
        } else {
            // otherwise, execute the segment by itself
            self.execute_expr(&ast)
                .context("failed to execute expression")?;
        }
        Ok(None)
    }

    /// Executes an individual expression.
    fn execute_expr(&mut self, statement: &Rc<ASTNode>) -> Result<Option<Rc<ASTNode>>> {
        let statement = statement.clone();
        match &*statement {
            ASTNode::Literal(Token::Identifier(sym)) => {
                // resovle variable and return literal value
                if let Variable::Owned(var) = self.get(&sym.as_id())? {
                    return Ok(Some(var.into()));
                }
                Ok(None)
            }
            ASTNode::Literal(_) | ASTNode::List(_) | ASTNode::Instance { .. } => {
                // return raw literal without resolving
                Ok(Some(statement))
            }
            ASTNode::Assign { target, value } => {
                // resolve target & expression
                let resolved_target = &self
                    .node_to_id(target.clone())
                    .context("failed to evaluate assignment target")?;
                let resolved_expr = self
                    .execute_expr(value)
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
                let resolved_target = &self
                    .node_to_id(target.clone())
                    .context("failed to evaluate declaration target")?;
                let resolved_expr = self
                    .execute_expr(value)
                    .context("failed to evaluate declaration value")?
                    .unwrap();

                // declare variable
                self.declare(
                    resolved_target,
                    Variable::Owned(ASTNode::inner_to_owned(&resolved_expr)),
                )?;
                Ok(None)
            }
            ASTNode::Op { lhs, op, rhs } => {
                if let (Ok(Some(a)), Ok(Some(b))) = (self.execute_expr(lhs), self.execute_expr(rhs))
                {
                    use Token::*;

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

                    // math & numeric equality
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

                    // bi-directional string concatenation
                    opmatch!(
                        match op, &*a, &*b => Str(l), r if
                        Add => Str(l.clone() + &*format!("{r}"))
                    );
                    opmatch!(
                        match op, &*a, &*b => l, Str(r) if
                        Add => Str(format!("{l}") + r)
                    );

                    // and & or
                    opmatch!(
                        match op, &*a, &*b => Bool(l), Bool(r) if
                        LogicalAnd => Bool(*l && *r),
                        LogicalOr => Bool(*l || *r)
                    );

                    // equality
                    opmatch!(
                        match op, &*a, &*b => l, r if
                        LogicalEq => Bool(l == r),
                        LogicalNeq => Bool(l != r)
                    );
                    bail!("operator not implemented ({} {:#?} {})", &*a, op, &*b)
                } else {
                    bail!("failed to evaluate operands")
                }
            }
            ASTNode::UnaryOp { target, op } => match op {
                // increment/decrement operations need special handling
                Token::Increment | Token::Decrement => {
                    // TODO: i'm pretty sure this doesn't work with dot notation or anything
                    // like that. that's a later fix, though
                    if let ASTNode::Literal(Token::Identifier(sym)) = &**target {
                        // get variable
                        let id = sym.as_id();
                        if let Variable::Owned(current_value) = self.get(&id)? {
                            if let ASTNode::Literal(Token::Number(n)) = current_value {
                                // get new assignment value
                                let new_value = match op {
                                    Token::Increment => Token::Number(n + 1.0),
                                    Token::Decrement => Token::Number(n - 1.0),
                                    _ => unreachable!(),
                                };
                                self.assign(&id, Variable::Owned(ASTNode::Literal(new_value)))?;
                            }
                        }
                    } else {
                        bail!("invalid increment/decrement target: {target:?}");
                    }
                    Ok(None)
                }

                // other unary operations need the target to be evaluated first
                _ => {
                    if let Ok(Some(target_result)) = self.execute_expr(target) {
                        match (op, target_result.as_ref()) {
                            // negative numbers
                            (Token::Sub, ASTNode::Literal(Token::Number(n))) => {
                                Ok(Some(Rc::new(ASTNode::Literal(Token::Number(-n)))))
                            }
                            // logical not
                            (Token::LogicalNot, ASTNode::Literal(Token::Bool(b))) => {
                                Ok(Some(Rc::new(ASTNode::Literal(Token::Bool(!b)))))
                            }
                            // bail for others
                            _ => {
                                bail!(
                                    "unsupported unary operation: {:?} on {:?}",
                                    op,
                                    target_result
                                );
                            }
                        }
                    } else {
                        bail!("failed to evaluate unary operand");
                    }
                }
            },
            ASTNode::Function {
                ref id,
                arguments: ref _arguments,
                body: ref _body,
            } => {
                self.declare(id, Variable::Function(statement.clone()))?;
                Ok(None)
            }
            ASTNode::FunctionCall { target, arguments } => {
                // get target variable and check if we need to set instance context
                let (variable, instance_context) = match &**target {
                    ASTNode::Literal(Token::Identifier(sym)) => (self.get(&sym.as_id())?, None),
                    ASTNode::Deref { parent, child: _ } => {
                        let id = self.node_to_id(target.clone())?;
                        let variable = self.get(&id)?;

                        // Check if this is an instance method call
                        let instance_context = match &**parent {
                            ASTNode::Literal(Token::Identifier(parent_sym)) => {
                                // Try to get the parent variable, but don't fail if it doesn't exist
                                // We need to handle this case because imported modules don't add
                                // context *here*, they allow access to it through recursive
                                // resolution in `resolve_access_target`
                                if let Ok(parent_var) = self.get(&parent_sym.as_id()) {
                                    match (&parent_var, &variable) {
                                        (
                                            Variable::Owned(ASTNode::Instance { .. }),
                                            Variable::Function(_),
                                        ) => Some(parent_var),
                                        _ => None,
                                    }
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        };

                        (variable, instance_context)
                    }
                    other => bail!("cannot call {:#?}", other),
                };

                // Resolve values before passing them as arguments. We do this so that the
                // arguments are already in their most basic form-- math expressions become single
                // numbers, variables become owned values, etc.
                let mut resolved_args = vec![];
                for arg in arguments {
                    resolved_args.push(
                        self.execute_expr(arg)
                            .context("failed to evaluate argument in extern")?
                            .unwrap_or(lit!(Token::Undefined))
                            .clone(),
                    );
                }

                let result = match variable {
                    // this branch should trigger on external functions
                    Variable::Extern(closure) => {
                        // call closure with i/o handles
                        closure(&mut self.output, &mut self.input, &resolved_args)
                    }

                    // this branch should trigger on raw, local functions
                    Variable::Function(function) => {
                        if let Some(Variable::Owned(ASTNode::Instance { svt, .. })) =
                            instance_context
                        {
                            // if we found a valid instance context, use it as memory space
                            self.with_context(Some(svt), |interpreter| {
                                interpreter.execute_function(&resolved_args, function)
                            })
                        } else {
                            // otherwise, use previously set memory space
                            self.execute_function(&resolved_args, function)
                        }
                    }

                    // this branch should trigger when constructors are called
                    Variable::Type(ref structure) => {
                        // get template as refcell
                        let svt = Rc::new(RefCell::new(
                            structure
                                .create_struct_template()
                                .context("failed to create structure template")?,
                        ));

                        // if there is a defined constructor, run it
                        if let Some(v) = structure.constructor() {
                            self.with_context(Some(svt.clone()), |interpreter| {
                                interpreter.execute_function(&resolved_args, v)
                            })?;
                        }

                        Ok(Some(
                            ASTNode::Instance {
                                kind: variable.into(),
                                svt: svt,
                            }
                            .into(),
                        ))
                    }

                    // catch others
                    _ => {
                        bail!("no function `{:#?}` found", target);
                    }
                };

                result
            }
            ASTNode::Struct { id, body: _ } => {
                self.declare(id, Variable::Type(statement.clone()))
                    .context("failed to declare type for structure")?;
                Ok(None)
            }
            ASTNode::Conditional {
                condition,
                if_body,
                else_body,
            } => {
                // evaluate condition
                let condition = self
                    .execute_expr(condition)?
                    .context("failed to evaluate condition")?;

                // increase scope level
                self.scope_id += 1;

                // execute if-body if statement is true. otherwise, execute else body
                if let Some(result) = self.execute(match condition.is_truthy() {
                    true => if_body.clone(),
                    false => else_body.clone(),
                })? {
                    self.drop_scope();
                    return Ok(Some(result));
                }

                // after finishing, drop the scope
                self.drop_scope();
                Ok(None)
            }
            ASTNode::Loop { condition, body } => {
                // create result buffer, default none
                let mut result = None;

                // increase scope level and execute body
                self.scope_id += 1;
                while let Some(condition) = self.execute_expr(condition)? {
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
                self.drop_scope();
                Ok(result)
            }
            ASTNode::Index { target, index } => {
                // get index as a usize
                let usize_idx = self
                    .execute_expr(index)
                    .context(format!("failed to evaluate index value ({index})"))?
                    .context("index cannot be undefined")?
                    .as_index()?;

                // get the target of this index
                let target = self
                    .execute_expr(target)
                    .context("failed to evaluate index target")?
                    .unwrap();

                // find item if applicable, bail otherwise
                match &*target {
                    ASTNode::List(items) => {
                        if let Variable::Owned(value) = &*items
                            .get(usize_idx)
                            .context("list item does not exist")?
                            .borrow()
                        {
                            return Ok(Some(value.clone().into()));
                        }
                        bail!("expected list item to be an owned value");
                    }
                    ASTNode::Literal(Token::Str(string)) => {
                        // get the char at the provided index, bail if it is not found
                        let ch = string.chars().nth(usize_idx).context(format!(
                            "no character exists at {usize_idx} in string '{string}'"
                        ))?;

                        // return the cloned character
                        Ok(Some(lit!(Token::Char(ch))))
                    }
                    _ => {
                        bail!("expected list as index target");
                    }
                }
            }
            ASTNode::Deref { parent, child } => {
                // NOTE: we should really just figure out how to `self.get` values with IDs that
                // represent a function call, but that might get a bit messy

                // get applicable memory entry
                let variable = if let Ok(deref_id) = self.node_to_id(statement.clone()) {
                    // for simple derefs, convert directly
                    self.get(&deref_id)?
                } else {
                    // for complex derefs (like `parent().child`), evaluate parts
                    let parent = self
                        .execute_expr(parent)?
                        .context("deref parent cannot be undefined")?;

                    // now handle member access on the result
                    if let ASTNode::Literal(Token::Identifier(member_id)) = &**child {
                        match &*parent {
                            ASTNode::Instance { svt, .. } => svt.borrow().get_owned(*member_id)?,
                            _ => bail!("cannot dereference member of {:#?}", parent),
                        }
                    } else {
                        bail!("deref child must be an identifier")
                    }
                };

                // convert variable back to AST node
                match variable {
                    Variable::Owned(node) => Ok(Some(Rc::new(node))),
                    Variable::Function(func) => Ok(Some(func)),
                    _ => bail!(format!("cannot convert {:#?} to valid node", variable)),
                }
            }
            ASTNode::Return(ref expr) => {
                // resolve expression
                let expr = self
                    .execute_expr(expr)
                    .context("failed to evaluate return expression")?
                    .context("expected return value")?;

                // if there are references, flatten them
                let expr = self
                    .resolve_refs(ASTNode::inner_to_owned(&expr))
                    .context("could not flatten references")?;

                Ok(Some(expr))
            }
            ASTNode::Module { alias, body } => {
                // TODO: it'd be nice to have the file path stuck in here somewhere for debugging
                // information...

                let ctx = match alias {
                    // if alias exists, create named module and execute in its context
                    Some(sym) => {
                        if let Some(current_context) = self.context.clone() {
                            Some(current_context.borrow_mut().add_module(*sym))
                        } else {
                            Some(self.memory.borrow_mut().add_module(*sym))
                        }
                    }

                    // otherwise, run in anonymous (current) context
                    None => None,
                };

                self.with_context(ctx, |interpreter| {
                    interpreter.execute(body.clone()).context(format!(
                        "failed to evaluate module '{}'",
                        (*alias).unwrap_or(intern!("anonymous"))
                    ))
                })?;
                Ok(None)
            }
            _ => {
                todo!()
            }
        }
    }
}
