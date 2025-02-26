//! Implementations for drop functions. These basically serve as a very basic garbage collector.

use super::Interpreter;

impl Interpreter {
    /// Drops all out-of-scope variables.
    pub fn drop(&mut self) {
        if let Some(mod_pointer) = &self.mod_id {
            let mut module = mod_pointer.borrow_mut();
            let mut scope_n = 0;
            module.inner_mut().retain(|_| {
                let in_scope = scope_n <= self.scope_id;
                scope_n += 1;
                in_scope
            });
        } else {
            let mut scope_n = 0;
            self.memory.borrow_mut().inner_mut().retain(|_| {
                let in_scope = scope_n <= self.scope_id;
                scope_n += 1;
                in_scope
            });
        }
    }

    /// Drops all variables in the current scope.
    pub fn drop_here(&mut self) {
        if let Some(mod_pointer) = &self.mod_id {
            let mut module = mod_pointer.borrow_mut();
            if let Some(this_scope) = module.get_scope(self.scope_id) {
                this_scope.clear();
            }
        } else {
            if let Some(this_scope) = self.memory.borrow_mut().get_scope(self.scope_id) {
                this_scope.clear();
            }
        }
    }
}
