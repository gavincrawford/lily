//! Implements all memory-related functions for the interpreter.
//! This includes getting and setting variables, as well as garbage collection.

// TODO i think that this may be setting variables to the lowest scopes first, preventing
// recursion from working as intended. this approach is better, though, and passes tests
//
// we'd have to figure out somehow when the let keyword is being used, as that decides
// wether or not we go and write over lower-scope vars

use super::*;

impl<'a> Interpreter<'a> {
    /// Gets the value of a variable.
    pub fn get(&self, id: String) -> Token {
        // step down scopes until a variable is found
        for scope in self.variables.iter().rev() {
            if scope.contains_key(&id) {
                match scope.get(&id).unwrap() {
                    ASTNode::Literal(t) => return t.clone(),
                    _ => {
                        panic!("invalid AST node in variable storage.");
                    }
                };
            }
        }
        panic!("no variable found.");
    }

    /// Declares a new variable.
    pub fn declare(&mut self, id: String, value: ASTNode) {
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
    pub fn assign(&mut self, id: String, value: ASTNode) {
        let mut scope_idx = self.scope;
        for (idx, scope) in self.variables.iter().enumerate() {
            if scope.contains_key(&id) {
                scope_idx = idx;
            }
        }
        let var_map = self.variables.get_mut(scope_idx);
        var_map.unwrap().insert(id, value);
    }

    /// Gets the value of a function.
    pub fn get_fn(&self, id: String) -> &'a Rc<ASTNode> {
        // step down scopes until a function is found
        for scope in self.functions.iter().rev() {
            if scope.contains_key(&id) {
                return scope.get(&id).unwrap();
            }
        }
        panic!("no function found.");
    }

    /// Sets the value of a function.
    pub fn set_fn(&mut self, id: String, value: &'a Rc<ASTNode>) {
        // add new scope if required
        while self.functions.len() <= self.scope {
            self.functions.push(HashMap::new());
        }

        // if this function already exists in this scope, panic
        let fn_map = self.functions.get_mut(self.scope).unwrap();
        if fn_map.contains_key(&id) {
            panic!("function '{}' already exists.", id);
        }

        // otherwise, insert
        fn_map.insert(id, value);
    }

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
