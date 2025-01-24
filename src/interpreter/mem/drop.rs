//! Implementations for drop functions. These basically serve as a very basic garbage collector.

use super::Interpreter;

impl<'a> Interpreter<'a> {
    /// Drops all out-of-scope variables.
    pub fn drop(&mut self) {
        let module = self.modules.get_mut(&self.mod_id).unwrap();
        let mut scope_n = 0;
        module.inner_mut().retain(|_| {
            let in_scope = scope_n <= self.scope;
            scope_n += 1;
            in_scope
        });
    }

    /// Drops all variables in the current scope.
    pub fn drop_here(&mut self) {
        let module = self.modules.get_mut(&self.mod_id).unwrap();
        if let Some(this_scope) = module.get_scope(self.scope) {
            this_scope.clear();
        }
    }
}
