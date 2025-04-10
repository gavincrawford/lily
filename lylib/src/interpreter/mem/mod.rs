//! Implements all memory-related functions for the interpreter.
//! This includes getting and setting variables, as well as garbage collection.

use super::*;
use anyhow::{bail, Result};

pub mod drop;
pub mod svtable;
pub mod variable;

impl Interpreter {
    /// Helper function to get the absolute module and variable name from an ID.
    ///
    /// Some identifiers reference variables within stacks of modules, and this function resolves
    /// these long chains of reference into the relevant module and variable name, respectively.
    fn resolve_identifier(&self, id: &ID) -> Result<(Rc<RefCell<SVTable>>, String)> {
        // get relevant module pointer
        let mut module = match &self.mod_id {
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
                for item in &path[0..(path.len() - 1)] {
                    // get mutable module ref
                    let module_copy = &*module.clone();
                    let mut module_ref = module_copy.borrow_mut();

                    if let Ok(v) = module_ref.get_module(item) {
                        // if this is a simple module, return that and continue
                        module = v;
                    } else {
                        // otherwise, this is a structure deref, so we have to find its SVT
                        for (name, value) in module_ref.get_scope(0).unwrap() {
                            if let Variable::Owned(var) = &*value.borrow() {
                                match (var, item == name) {
                                    (
                                        ASTNode::Instance {
                                            kind: _,
                                            id: _,
                                            svt,
                                        },
                                        true,
                                    ) => module = svt.clone(),
                                    _ => {}
                                }
                            }
                        }
                    };
                }
                path.last().unwrap().to_owned()
            }
        };

        Ok((module, id))
    }

    /// Gets the value of a variable.
    pub fn get(&self, id: &ID) -> Result<Rc<RefCell<Variable>>> {
        // get absolute module and ID
        let (module, id) = self.resolve_identifier(id)?;

        // find id in any scope
        for scope in (&*module).borrow().iter().rev() {
            if scope.contains_key(&id) {
                let variable = scope.get(&id).unwrap();
                return Ok(variable.to_owned());
            }
        }

        // if no value is found, bail
        bail!("failed to get value {:?}", id)
    }

    /// Gets the value of a variable, and clones it in the process.
    pub fn get_owned(&self, id: &ID) -> Result<Variable> {
        // get absolute module and ID
        let (module, id) = self.resolve_identifier(id)?;

        // find id in any scope
        for scope in (&*module).borrow().iter().rev() {
            if scope.contains_key(&id) {
                let variable = scope.get(&id).unwrap();
                return Ok((&**variable).borrow().clone());
            }
        }

        // if no value is found, bail
        bail!("failed to get owned value {:?}", id)
    }

    /// Declares a new variable.
    pub fn declare(&mut self, id: &ID, value: Variable) -> Result<()> {
        // get absolute module and ID
        let (module, id) = self.resolve_identifier(id)?;

        // borrow module mutably to make changes
        let mut module = module.borrow_mut();

        // add scopes if necessary
        while module.scopes() <= self.scope_id {
            module.add_scope();
        }

        // get variable map and insert new value. if the value already exists, bail
        let var_map = module
            .get_scope(self.scope_id)
            .context(format!("cannot delcare at scope {}", self.scope_id,))?;
        if let Some(_) = var_map.insert(id.clone(), RefCell::new(value).into()) {
            bail!("variable '{}' already exists", id);
        }
        Ok(())
    }

    /// Assigns to an existing variable.
    pub fn assign(&mut self, id: &ID, value: Variable) -> Result<()> {
        // get absolute module and ID
        let (module, id) = self.resolve_identifier(id)?;

        // borrow module mutably to make changes
        let mut module = module.borrow_mut();

        // get currently selected scope id
        let mut scope_idx = self.scope_id;
        for (idx, scope) in module.iter().enumerate() {
            if scope.contains_key(&id) {
                scope_idx = idx;
            }
        }

        // get variable map at specified scope id
        let var_map = module
            .get_scope(scope_idx)
            .context(format!("cannot assign at scope {}", scope_idx,))?;

        // insert new value
        var_map.insert(id, RefCell::new(value).into());
        Ok(())
    }
}
