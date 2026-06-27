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
            module.inner_mut().truncate(self.scope_id + 1);
        } else {
            self.memory
                .borrow_mut()
                .inner_mut()
                .truncate(self.scope_id + 1);
        }
    }

    /// Drops all variables in the current scope.
    pub(crate) fn drop_here(&mut self) {
        if let Some(mod_pointer) = &self.context {
            let mut module = mod_pointer.borrow_mut();
            if let Ok(this_scope) = module.get_scope(self.scope_id) {
                this_scope.clear();
            }
        } else if let Ok(this_scope) = self.memory.borrow_mut().get_scope(self.scope_id) {
            this_scope.clear();
        }
    }
}
