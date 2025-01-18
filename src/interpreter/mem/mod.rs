//! Implements all memory-related functions for the interpreter.
//! This includes getting and setting variables, as well as garbage collection.

use super::*;

pub mod drop;
pub mod variable;

impl<'a> Interpreter<'a> {
    /// Gets the value of a variable.
    pub fn get(&self, id: impl Into<String>) -> &Variable {
        let id = id.into();

        // step down scopes until a variable is found
        for scope in self.variables.iter().rev() {
            if scope.contains_key(&id) {
                return scope.get(&id).unwrap();
            }
        }
        panic!("no variable found.");
    }

    /// Gets the value of a variable, and clones it in the process.
    pub fn get_owned(&self, id: impl Into<String>) -> Variable<'a> {
        let id = id.into();

        // step down scopes until a variable is found
        for scope in self.variables.iter().rev() {
            if scope.contains_key(&id) {
                return scope.get(&id).unwrap().to_owned();
            }
        }
        panic!("no variable found.");
    }

    /// Declares a new variable.
    pub fn declare(&mut self, id: impl Into<String>, value: Variable<'a>) {
        let id = id.into();

        // add new scope if required
        while self.variables.len() <= self.scope {
            self.variables.push(HashMap::new());
        }

        // if this variable already exists in this scope, panic
        let var_map = self.variables.get_mut(self.scope).unwrap();
        if var_map.contains_key(&id) {
            panic!("variable '{}' already exists.", id);
        }

        // otherwise, insert
        var_map.insert(id, value);
    }

    /// Assigns to an existing variable.
    pub fn assign(&mut self, id: impl Into<String>, value: Variable<'a>) {
        let id = id.into();

        let mut scope_idx = self.scope;
        for (idx, scope) in self.variables.iter().enumerate() {
            if scope.contains_key(&id) {
                scope_idx = idx;
            }
        }
        let var_map = self.variables.get_mut(scope_idx);
        var_map.unwrap().insert(id, value);
    }
}
