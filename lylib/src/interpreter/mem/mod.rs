//! Implements all memory-related functions for the interpreter.
//! This includes getting and setting variables.

use super::*;
use crate::{errors::MemoryError, interner::Symbol};
use anyhow::{Context, Result};

pub mod drop;
pub mod flatrcmap;
pub mod svtable;
pub mod variable;

/// This trait can be added to any type to give it the ability to be accessed by identifier.
pub(crate) trait MemoryInterface {
    fn get_owned(&self, id: Symbol) -> Result<Variable, MemoryError>;
    fn get_ref(&self, id: Symbol) -> Result<Rc<RefCell<Variable>>, MemoryError>;
    fn get_module(&self, id: Symbol) -> Result<Rc<RefCell<SVTable>>, MemoryError>;
    fn declare(&mut self, id: Symbol, value: Variable, scope: usize) -> Result<(), MemoryError>;
    fn assign(&mut self, id: Symbol, value: Variable, scope: usize) -> Result<(), MemoryError>;
}

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Helper function to get the target and variable name from an ID.
    ///
    /// Some identifiers reference variables within stacks of modules, and this function resolves
    /// these long chains of reference into the relevant target and variable name.
    fn resolve_access_target(&self, id: &ID) -> Result<(Rc<RefCell<dyn MemoryInterface>>, Symbol)> {
        // get current context (module)
        let mut module: Rc<RefCell<dyn MemoryInterface>> = match &self.context {
            Some(context) => context.clone(),
            None => self.memory.clone(),
        };

        // get variable id, stepping down if required
        let id = match id {
            ID::Symbol(sym) => *sym,
            ID::Index(val) => *val,
            ID::Member { .. } => {
                let path = id.to_path_symbolic();
                for &item in &path[0..(path.len() - 1)] {
                    // try module lookup first; any error means "not a module" — fall through
                    // to struct/list deref below
                    let module_result = module.borrow().get_module(item);

                    // if this is a simple module, use that and continue
                    if let Ok(v) = module_result {
                        module = v;
                        continue;
                    }

                    // otherwise, this is a structure or list deref, so we have to find its SVT
                    let item_ref = module.borrow().get_ref(item)?;

                    match &*item_ref.borrow() {
                        // expose inner scope for instances
                        Variable::Owned(ASTNode::Instance { kind: _, svt }) => module = svt.clone(),

                        // all other literals return their parent variable
                        Variable::Owned(_) => {
                            module = item_ref.clone();
                        }

                        _ => {}
                    };
                }
                *path.last().unwrap()
            }
        };

        Ok((module, id))
    }

    /// Gets the value of a variable, and clones it in the process.
    #[inline]
    pub(crate) fn get(&self, id: &ID) -> Result<Variable> {
        // get absolute module and ID
        let (module, resolved_id) = self.resolve_access_target(id)?;

        // borrow statically to read value
        let handle = module.borrow();

        // return value
        let result = handle.get_owned(resolved_id);
        drop(handle);

        // struct/module methosds run with local `self.context`, but also need to be able to access
        // root memory from `self.memory`-- retry there if we can't find this variable
        if result.is_err()
            && self.context.is_some()
            && matches!(id, ID::Symbol(_))
            && let Ok(value) = self.memory.borrow().get_owned(resolved_id)
        {
            return Ok(value);
        }

        result.with_context(|| format!("failed to read {id:?}"))
    }

    /// Declares a new variable.
    #[inline]
    pub(crate) fn declare(&mut self, id: &ID, value: Variable) -> Result<()> {
        // indexing into a variable can't safely write through `resolve_access_target`'s walk, since
        // the variable `Rc` may be shared. instead, deep-clone, mutate in isolation, and replace
        if let ID::Member { parent, member } = id
            && let ID::Index(index) = member.as_ref()
        {
            let mut current = self.get(parent)?;
            current
                .declare(*index, value, self.scope_id)
                .with_context(|| format!("failed to declare {id:?}"))?;
            return self.assign(parent, current);
        }

        // get absolute module and ID
        let (module, resolved_id) = self.resolve_access_target(id)?;

        // borrow module mutably to make changes
        let mut module = module.borrow_mut();

        // declare value
        module
            .declare(resolved_id, value, self.scope_id)
            .with_context(|| format!("failed to declare {id:?}"))
    }

    /// Assigns to an existing variable.
    #[inline]
    pub(crate) fn assign(&mut self, id: &ID, value: Variable) -> Result<()> {
        // see the comment in `declare` above-- identical pattern
        if let ID::Member { parent, member } = id
            && let ID::Index(index) = member.as_ref()
        {
            let mut current = self.get(parent)?;
            current
                .assign(*index, value, self.scope_id)
                .with_context(|| format!("failed to assign {id:?}"))?;
            return self.assign(parent, current);
        }

        // get absolute module and ID
        let (module, resolved_id) = self.resolve_access_target(id)?;

        // borrow module mutably to make changes
        let mut module = module.borrow_mut();

        // assign value
        module
            .assign(resolved_id, value, self.scope_id)
            .with_context(|| format!("failed to assign {id:?}"))
    }
}
