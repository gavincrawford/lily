//! Implements all memory-related functions for the interpreter.
//! This includes getting and setting variables, as well as garbage collection.

use super::*;
use crate::parser::{IDKind, ID};
use anyhow::{bail, Result};

pub mod drop;
pub mod svtable;
pub mod variable;

impl<'a> Interpreter<'a> {
    // TODO get functions could be simplified by unifying variable fetch from any module

    /// Gets the value of a variable.
    pub fn get(&self, id: &ID) -> Result<Rc<RefCell<Variable>>> {
        match id.get_kind() {
            IDKind::Literal(id) => {
                let module = match &self.mod_id {
                    Some(mod_id) => mod_id.borrow(),
                    None => (&self.memory).borrow(),
                };
                for scope in module.iter().rev() {
                    if scope.contains_key(&id) {
                        let variable = scope.get(&id).unwrap();
                        return Ok(variable.to_owned());
                    }
                }
            }
            IDKind::Member {
                parent: _,
                member: _,
            } => {
                let path = id.to_path();
                let mut module = self.memory.clone();
                for item in &path[0..(path.len() - 1)] {
                    let module_copy = module.clone();
                    module = module_copy
                        .borrow()
                        .get_module(item)
                        .context("failed to get value")
                        .unwrap();
                }
                let id = path.last().unwrap().to_owned();
                for scope in module.borrow().iter().rev() {
                    if scope.contains_key(&id) {
                        let variable = scope.get(&id).unwrap();
                        return Ok(variable.to_owned());
                    }
                }
            }
        }
        bail!("failed to get value {:?}", id)
    }

    /// Gets the value of a variable, and clones it in the process.
    pub fn get_owned(&self, id: &ID) -> Result<Variable> {
        match id.get_kind() {
            IDKind::Literal(id) => {
                let module = match &self.mod_id {
                    Some(mod_id) => mod_id.borrow(),
                    None => (&self.memory).borrow(),
                };
                for scope in module.iter().rev() {
                    if scope.contains_key(&id) {
                        let variable = scope.get(&id).unwrap();
                        return Ok(variable.borrow().clone());
                    }
                }
            }
            IDKind::Member {
                parent: _,
                member: _,
            } => {
                let path = id.to_path();
                // PERF this is likely slow due to three different clones, even of pointers
                let mut module = self.mod_id.clone().unwrap_or(self.memory.clone()).clone();
                for item in &path[0..(path.len() - 1)] {
                    let module_copy = module.clone();
                    module = module_copy
                        .borrow()
                        .get_module(item)
                        .context("failed to get value")
                        .unwrap();
                }
                let id = path.last().unwrap().to_owned();
                for scope in module.borrow().iter().rev() {
                    if scope.contains_key(&id) {
                        let variable = scope.get(&id).unwrap();
                        return Ok(variable.borrow().clone());
                    }
                }
            }
        }
        bail!("failed to get owned value {:?}", id)
    }

    /// Declares a new variable.
    pub fn declare(&mut self, id: &ID, value: Variable) -> Result<()> {
        match id.get_kind() {
            IDKind::Literal(id) => {
                let mut module = match &self.mod_id {
                    Some(mod_id) => mod_id.borrow_mut(),
                    None => (&self.memory).borrow_mut(),
                };

                // add scopes if necessary
                while module.scopes() <= self.scope_id {
                    module.add_scope();
                }

                // if this variable already exists in this scope, bail
                let var_map = module
                    .get_scope(self.scope_id)
                    .context(format!("cannot delcare at scope {}", self.scope_id,))
                    .unwrap();
                if var_map.contains_key(&id) {
                    bail!("variable '{}' already exists", id);
                }

                // otherwise, insert
                var_map.insert(id, RefCell::new(value).into());
            }
            IDKind::Member {
                parent: _,
                member: _,
            } => {
                // TODO implement member declaration
                todo!("member declaration not implemented");
            }
        }
        Ok(())
    }

    /// Assigns to an existing variable.
    pub fn assign(&mut self, id: &ID, value: Variable) -> Result<()> {
        match id.get_kind() {
            IDKind::Literal(id) => {
                let mut module = match &self.mod_id {
                    Some(mod_id) => mod_id.borrow_mut(),
                    None => (&self.memory).borrow_mut(),
                };

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
                    .context(format!("cannot assign at scope {}", scope_idx,))
                    .unwrap();

                // insert new value
                var_map.insert(id, RefCell::new(value).into());
            }
            IDKind::Member {
                parent: _,
                member: _,
            } => {
                // TODO implement member assignment
                todo!("member assignment not implemented");
            }
        }
        Ok(())
    }
}
