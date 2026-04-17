//! Implements all memory-related functions for the interpreter.
//! This includes getting and setting variables.

use super::*;
use crate::errors::MemoryError;
use anyhow::Result;

pub mod drop;
pub mod flatrcmap;
pub mod svtable;
pub mod variable;

/// This trait can be added to any type to give it the ability to be accessed by identifier.
pub(crate) trait MemoryInterface {
    fn get_owned(&self, id: usize) -> Result<Variable, MemoryError>;
    fn get_ref(&self, id: usize) -> Result<Rc<RefCell<Variable>>, MemoryError>;
    fn get_module(&self, id: usize) -> Result<Rc<RefCell<SVTable>>, MemoryError>;
    fn declare(&mut self, id: usize, value: Variable, scope: usize) -> Result<(), MemoryError>;
    fn assign(&mut self, id: usize, value: Variable, scope: usize) -> Result<(), MemoryError>;
}

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Helper function to get the target and variable name from an ID.
    ///
    /// Some identifiers reference variables within stacks of modules, and this function resolves
    /// these long chains of reference into the relevant target and variable name.
    fn resolve_access_target(&self, id: &ID) -> Result<(Rc<RefCell<dyn MemoryInterface>>, usize)> {
        // get current context (module)
        let mut module: Rc<RefCell<dyn MemoryInterface>> = match &self.context {
            Some(context) => context.clone(),
            None => self.memory.clone(),
        };

        // get variable id, stepping down if required
        let id = match id.get_kind() {
            IDKind::Symbol(sym) => sym,
            IDKind::Literal(val) => val,
            IDKind::Member {
                parent: _,
                member: _,
            } => {
                let path = id.to_path();
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
        let (module, id) = self.resolve_access_target(id)?;

        // borrow statically to read value
        let handle = module.borrow();

        // return value
        Ok(handle.get_owned(id)?)
    }

    /// Declares a new variable.
    #[inline]
    pub(crate) fn declare(&mut self, id: &ID, value: Variable) -> Result<()> {
        // get absolute module and ID
        let (module, id) = self.resolve_access_target(id)?;

        // borrow module mutably to make changes
        let mut module = module.borrow_mut();

        // declare value
        Ok(module.declare(id, value, self.scope_id)?)
    }

    /// Assigns to an existing variable.
    #[inline]
    pub(crate) fn assign(&mut self, id: &ID, value: Variable) -> Result<()> {
        // get absolute module and ID
        let (module, id) = self.resolve_access_target(id)?;

        // borrow module mutably to make changes
        let mut module = module.borrow_mut();

        // assign value
        Ok(module.assign(id, value, self.scope_id)?)
    }
}
