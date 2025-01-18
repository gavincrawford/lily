//! Implementations for drop functions. These basically serve as a very basic garbage collector.

use super::Interpreter;

impl<'a> Interpreter<'a> {
    /// Drops all out-of-scope variables.
    pub fn drop(&mut self) {
        // drop out-of-scope variable tables
        let mut scope_n = 0;
        self.variables.retain(|_| {
            let in_scope = scope_n <= self.scope;
            scope_n += 1;
            in_scope
        });

        // drop out-of-scope function tables
        let mut scope_n = 0;
        self.functions.retain(|_| {
            let in_scope = scope_n <= self.scope;
            scope_n += 1;
            in_scope
        });
    }

    /// Drops all variables in the current scope.
    pub fn drop_here(&mut self) {
        if let Some(this_scope) = self.variables.get_mut(self.scope) {
            this_scope.clear();
        }
    }
}
