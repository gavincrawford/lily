//! Implements all memory-related functions for the interpreter.
//! This includes getting and setting variables.

use super::*;
use anyhow::Result;

pub mod drop;
pub mod svtable;
pub mod variable;

/// This trait can be added to any type to give it the ability to be accessed by identifier.
pub(crate) trait MemoryInterface {
    fn get_owned(&self, id: usize) -> Result<Variable>;
    fn get_ref(&self, id: usize) -> Result<Rc<RefCell<Variable>>>;
    fn get_module(&self, id: usize) -> Result<Rc<RefCell<SVTable>>>;
    fn declare(&mut self, id: usize, value: Variable, scope: usize) -> Result<()>;
    fn assign(&mut self, id: usize, value: Variable, scope: usize) -> Result<()>;
}

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Helper function to get the target and variable name from an ID.
    ///
    /// Some identifiers reference variables within stacks of modules, and this function resolves
    /// these long chains of reference into the relevant target and variable name.
    fn resolve_access_target(&self, id: &ID) -> Result<(Rc<RefCell<dyn MemoryInterface>>, usize)> {
        // get relevant module pointer
        let mut module: Rc<RefCell<dyn MemoryInterface>> = match &self.mod_id {
            Some(mod_id) => mod_id.clone(),
            None => self.memory.clone(),
        };

        // get variable id, stepping down if required
        let id = match id.get_kind() {
            IDKind::Literal(id) => id,
            IDKind::Member {
                parent: _,
                member: _,
            } => {
                let path = id.to_path();
                for &item in &path[0..(path.len() - 1)] {
                    // try to get module first, then check if it's a struct/list access
                    let module_result = {
                        let module_ref = module.borrow();
                        module_ref.get_module(item)
                    };

                    if let Ok(v) = module_result {
                        // if this is a simple module, use that and continue
                        module = v;
                    } else {
                        // otherwise, this is a structure or list deref, so we have to find its SVT
                        let item_ref = {
                            let module_ref = module.borrow();
                            module_ref.get_ref(item).unwrap()
                        };

                        match &*item_ref.borrow() {
                            Variable::Owned(ASTNode::Instance {
                                kind: _,
                                id: _,
                                svt,
                            }) => module = svt.clone(),
                            Variable::Owned(node) if matches!(node, ASTNode::List(_)) => {
                                module = item_ref.clone();
                            }
                            _ => {}
                        };
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
        handle.get_owned(id)
    }

    /// Declares a new variable.
    #[inline]
    pub(crate) fn declare(&mut self, id: &ID, value: Variable) -> Result<()> {
        // get absolute module and ID
        let (module, id) = self.resolve_access_target(id)?;

        // borrow module mutably to make changes
        let mut module = module.borrow_mut();

        // declare value
        module.declare(id, value, self.scope_id)
    }

    /// Assigns to an existing variable.
    #[inline]
    pub(crate) fn assign(&mut self, id: &ID, value: Variable) -> Result<()> {
        // get absolute module and ID
        let (module, id) = self.resolve_access_target(id)?;

        // borrow module mutably to make changes
        let mut module = module.borrow_mut();

        // assign value
        module.assign(id, value, self.scope_id)
    }
}
