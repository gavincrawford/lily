//! Implements the SVTable, or the scoped-variable table.

use super::{flatrcmap::FlatRcMap, *};
use crate::interner::Symbol;
use anyhow::Result;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, fmt::Display, rc::Rc, slice::Iter};

/// Scoped-variable table. Holds values with respect to their variable names.
#[derive(Debug, PartialEq)]
pub struct SVTable {
    /// Holds all the scope frames, each of which hold their respective variables.
    table: Vec<FlatRcMap<Variable>>,
    /// Holds all the modules defined at this SVTable's scope.
    modules: FxHashMap<Symbol, Rc<RefCell<SVTable>>>,
}

impl Clone for SVTable {
    /// Shallow clone the SVTable, sharing variable `Rc` references with the original.
    ///
    /// This is safe because SVTable cloning only occurs when creating struct instances from
    /// their templates (see `ASTNode::template()`). Templates are constant source data that
    /// are never mutated after parsing, so shared references won't be written through.
    /// Assignments to instance fields go through `assign`, which always inserts a new `Rc`
    /// rather than mutating a potentially-shared one (copy-on-write).
    fn clone(&self) -> Self {
        Self {
            table: self.table.iter().map(|map| map.shallow_clone()).collect(),
            modules: self
                .modules
                .iter()
                .map(|(&id, module)| (id, Rc::new(RefCell::new(module.borrow().clone()))))
                .collect(),
        }
    }
}

impl Default for SVTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SVTable {
    /// Creates a new scoped-variable table with a default scope.
    #[inline]
    pub fn new() -> Self {
        Self {
            table: vec![FlatRcMap::default()],
            modules: FxHashMap::default(),
        }
    }

    /// Returns the iterator to the internal list of frames.
    #[inline]
    pub(crate) fn iter(&self) -> Iter<'_, FlatRcMap<Variable>> {
        self.table.iter()
    }

    /// Returns the inner list of frames, mutable.
    #[inline]
    pub(crate) fn inner_mut(&mut self) -> &mut Vec<FlatRcMap<Variable>> {
        &mut self.table
    }

    /// Adds a new module. Returns a reference to the newly created module.
    #[inline]
    pub fn add_module(&mut self, name: Symbol) -> Rc<RefCell<SVTable>> {
        self.modules
            .entry(name)
            .or_insert_with(|| Rc::new(RefCell::new(SVTable::default())))
            .clone()
    }

    /// Adds a new scope.
    #[inline]
    pub fn add_scope(&mut self) {
        self.table.push(FlatRcMap::default());
    }

    /// Gets a scope map. Mutable by default.
    #[inline]
    pub(crate) fn get_scope(
        &mut self,
        index: usize,
    ) -> Result<&mut FlatRcMap<Variable>, MemoryError> {
        match self.table.get_mut(index) {
            Some(table) => Ok(table),
            None => Err(MemoryError::NoScope(index)),
        }
    }

    /// Returns the number of scopes in this table.
    #[inline]
    pub fn scopes(&self) -> usize {
        self.table.len()
    }
}

impl SVTable {
    /// Helper method to find a variable in any scope, returns the found variable reference.
    #[inline]
    fn find_variable(&self, id: Symbol) -> Option<Rc<RefCell<Variable>>> {
        for scope in self.iter().rev() {
            if let Some(variable) = scope.get(id) {
                return Some(variable);
            }
        }
        None
    }
}

impl MemoryInterface for SVTable {
    #[inline]
    fn get_owned(&self, id: Symbol) -> Result<Variable, MemoryError> {
        match self.find_variable(id) {
            Some(variable) => Ok(variable.borrow().clone()),
            None => Err(MemoryError::VariableRead(resolve!(id))),
        }
    }

    #[inline]
    fn get_ref(&self, id: Symbol) -> Result<Rc<RefCell<Variable>>, MemoryError> {
        match self.find_variable(id) {
            Some(variable) => Ok(variable.clone()),
            None => Err(MemoryError::VariableRead(resolve!(id))),
        }
    }

    #[inline]
    fn get_module(&self, id: Symbol) -> Result<Rc<RefCell<SVTable>>, MemoryError> {
        self.modules
            .get(&id)
            .cloned()
            .ok_or_else(|| MemoryError::NoModule(resolve!(id)))
    }

    #[inline]
    fn declare(&mut self, id: Symbol, value: Variable, scope: usize) -> Result<(), MemoryError> {
        // add scopes if necessary
        while self.scopes() <= scope {
            self.add_scope();
        }

        // get variable map and insert new value
        let var_map = self.get_scope(scope)?;
        var_map.insert(id, Rc::new(RefCell::new(value)));
        Ok(())
    }

    #[inline]
    fn assign(&mut self, id: Symbol, value: Variable, scope: usize) -> Result<(), MemoryError> {
        // find which scope index contains the variable
        let target_scope = self
            .table
            .iter()
            .enumerate()
            .rev()
            .find_map(|(idx, scope)| if scope.contains(id) { Some(idx) } else { None });

        if let Some(scope_idx) = target_scope {
            // always insert a new Rc, safe even if the old Rc is shared (COW)
            self.table[scope_idx].insert(id, Rc::new(RefCell::new(value)));
            return Ok(());
        }

        // otherwise, manual insert. this is used for dynamic structure/module assignment, such as:
        // s.x = 0 # where `s` does not contain `x`
        let var_map = self.get_scope(scope)?;
        var_map.insert(id, Rc::new(RefCell::new(value)));
        Ok(())
    }
}

impl Display for SVTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn prettify(node: Rc<ASTNode>) -> String {
            match &*node {
                ASTNode::Literal(Token::Identifier(id)) => resolve!(*id),
                ASTNode::Literal(token) => format!("{token:#?}"),
                ASTNode::Op { lhs, op, rhs } => format!(
                    "{} {:#?} {}",
                    prettify(lhs.clone()),
                    op,
                    prettify(rhs.clone())
                ),
                ASTNode::Block(lines) => lines
                    .iter()
                    .map(|ln| prettify(ln.clone()))
                    .collect::<Vec<String>>()
                    .join(", ")
                    .to_string(),
                ASTNode::Return(value) => prettify(value.clone()),
                ASTNode::Function {
                    id,
                    arguments,
                    body,
                } => format!(
                    "{}({}) => {}",
                    id.to_path_kinds()
                        .iter()
                        .map(|kind| match kind {
                            IDKind::Symbol(sym) => resolve!(*sym),
                            IDKind::Literal(val) => val.to_string(),
                            IDKind::Member { .. } => {
                                unreachable!("member should be flattened by to_path_kinds")
                            }
                        })
                        .collect::<Vec<String>>()
                        .join("."),
                    arguments
                        .iter()
                        .map(|id| resolve!(*id))
                        .collect::<Vec<String>>()
                        .join(", "),
                    prettify(body.clone())
                ),
                other => format!("{other:#?}"),
            }
        }

        // log scopes progressively
        for (scope_idx, scope) in self.table.iter().enumerate() {
            // log scope level
            writeln!(f, "scope {scope_idx}")?;

            // iterate through scope values, sorted by key name
            let mut keys = scope.keys();
            keys.sort();
            for key in keys {
                // obtain debug string respective to variable value
                let value = scope.get(key).unwrap();
                let dbg_ln = match &*value.borrow() {
                    Variable::Owned(node) => prettify(node.to_owned().into()).to_string(),
                    Variable::Function(reference) => format!("&{}", prettify(reference.clone())),
                    Variable::Extern(_) => "EXTERN".to_string(),
                    Variable::Type(instance) => format!("struct {}", prettify(instance.clone())),
                };

                // tab out endlines to keep indents, and print it
                let dbg_ln = dbg_ln.replace("\n", "\n\t");
                writeln!(f, "\t{} = {dbg_ln}", resolve!(key))?;
            }
        }
        Ok(())
    }
}
