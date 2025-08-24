//! Implementations for drop functions. These basically serve as a very basic garbage collector.

use super::*;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Drops all out-of-scope variables and drops down a scope.
    pub(crate) fn drop_scope(&mut self) {
        // decrease scope level
        self.scope_id -= 1;

        // remove out of scope variables
        if let Some(mod_pointer) = &self.context {
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
    pub(crate) fn drop_here(&mut self) {
        if let Some(mod_pointer) = &self.context {
            let mut module = mod_pointer.borrow_mut();
            if let Some(this_scope) = module.get_scope(self.scope_id) {
                this_scope.clear();
            }
        } else if let Some(this_scope) = self.memory.borrow_mut().get_scope(self.scope_id) {
            this_scope.clear();
        }
    }
}
