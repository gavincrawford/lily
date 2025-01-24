//! Implements all memory-related functions for the interpreter.
//! This includes getting and setting variables, as well as garbage collection.

// TODO the implementations of the functions in this file do not allow for nested modules to exist
//      they're also direct copies of each other, with a few snippets changed. this could be made
//      more elegant, using inner functions that cover both cases

use super::*;
use crate::parser::{IDKind, ID};

pub mod drop;
pub mod svtable;
pub mod variable;

impl<'a> Interpreter<'a> {
    /// Gets the value of a variable.
    pub fn get(&self, id: &ID) -> &Variable {
        match id.get_kind() {
            IDKind::Literal(id) => {
                // step down scopes until a variable is found
                for scope in self.modules.get(&self.mod_id).unwrap().iter().rev() {
                    if scope.contains_key(&id) {
                        return scope.get(&id).unwrap();
                    }
                }
            }
            IDKind::Member { parent, member } => {
                match (&*parent, &*member) {
                    (IDKind::Literal(pid), IDKind::Literal(mid)) => {
                        for scope in self.modules.get(&pid.to_owned()).unwrap().iter().rev() {
                            let mid = mid.to_owned();
                            if scope.contains_key(&mid) {
                                return scope.get(&mid).unwrap();
                            }
                        }
                    }
                    _ => {
                        panic!();
                    }
                };
            }
        }
        panic!("no variable found.");
    }

    /// Gets the value of a variable, and clones it in the process.
    pub fn get_owned(&self, id: &ID) -> Variable<'a> {
        match id.get_kind() {
            IDKind::Literal(id) => {
                // step down scopes until a variable is found
                for scope in self.modules.get(&self.mod_id).unwrap().iter().rev() {
                    if scope.contains_key(&id) {
                        return scope.get(&id).unwrap().to_owned();
                    }
                }
            }
            IDKind::Member { parent, member } => {
                match (&*parent, &*member) {
                    (IDKind::Literal(pid), IDKind::Literal(mid)) => {
                        for scope in self.modules.get(&pid.to_owned()).unwrap().iter().rev() {
                            let mid = mid.to_owned();
                            if scope.contains_key(&mid) {
                                return scope.get(&mid).unwrap().to_owned();
                            }
                        }
                    }
                    _ => {
                        panic!();
                    }
                };
            }
        }
        panic!("no variable found.");
    }

    /// Declares a new variable.
    pub fn declare(&mut self, id: &ID, value: Variable<'a>) {
        match id.get_kind() {
            IDKind::Literal(id) => {
                // add new scope if required
                let module = self.modules.get_mut(&self.mod_id).unwrap();
                while module.scopes() <= self.scope {
                    module.add_scope();
                }

                // if this variable already exists in this scope, panic
                let var_map = module.get_scope(self.scope).unwrap();
                if var_map.contains_key(&id) {
                    panic!("variable '{}' already exists.", id);
                }

                // otherwise, insert
                var_map.insert(id, value);
            }
            IDKind::Member { parent, member } => {
                match (&*parent, &*member) {
                    (IDKind::Literal(pid), IDKind::Literal(mid)) => {
                        // add new scope if required
                        let module = self.modules.get_mut(&pid.to_owned()).unwrap();
                        while module.scopes() <= self.scope {
                            module.add_scope();
                        }

                        // if this variable already exists in this scope, panic
                        let var_map = module.get_scope(self.scope).unwrap();
                        let mid = mid.to_owned();
                        if var_map.contains_key(&mid) {
                            panic!("variable '{}' already exists.", &mid);
                        }

                        // otherwise, insert
                        var_map.insert(mid, value);
                    }
                    _ => {
                        panic!();
                    }
                };
            }
        }
    }

    /// Assigns to an existing variable.
    pub fn assign(&mut self, id: &ID, value: Variable<'a>) {
        match id.get_kind() {
            IDKind::Literal(id) => {
                let module = self.modules.get_mut(&self.mod_id).unwrap();
                let mut scope_idx = self.scope;
                for (idx, scope) in module.iter().enumerate() {
                    if scope.contains_key(&id) {
                        scope_idx = idx;
                    }
                }
                let var_map = module.get_scope(scope_idx);
                var_map.unwrap().insert(id, value);
            }
            IDKind::Member { parent, member } => {
                match (&*parent, &*member) {
                    (IDKind::Literal(pid), IDKind::Literal(mid)) => {
                        let mid = mid.to_owned();
                        let module = self.modules.get_mut(&pid.to_owned()).unwrap();
                        let mut scope_idx = self.scope;
                        for (idx, scope) in module.iter().enumerate() {
                            if scope.contains_key(&mid) {
                                scope_idx = idx;
                            }
                        }
                        let var_map = module.get_scope(scope_idx);
                        var_map.unwrap().insert(mid, value);
                    }
                    _ => {
                        panic!();
                    }
                };
            }
        }
    }
}
