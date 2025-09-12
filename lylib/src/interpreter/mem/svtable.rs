//! Implements the SVTable, or the scoped-variable table.

use super::*;
use anyhow::{bail, Result};
use rustc_hash::FxHashMap;
use std::{cell::RefCell, fmt::Display, rc::Rc, slice::Iter};

/// Scoped-variable table. Holds values with respect to their variable names.
/// Internally, uses a `FxHashMap` providing fast but less secure access.
#[derive(Debug, PartialEq)]
pub struct SVTable {
    /// Holds all the scope frames, each of which hold their respective variables.
    table: Vec<FxHashMap<usize, Rc<RefCell<Variable>>>>,
    /// Holds all the modules defined at this SVTable's scope.
    modules: FxHashMap<usize, Rc<RefCell<SVTable>>>,
}

impl SVTable {
    /// Creates a new scoped-variable table with a default scope.
    #[inline]
    pub fn new() -> Self {
        Self {
            table: vec![FxHashMap::default()],
            modules: FxHashMap::default(),
        }
    }

    /// Returns the iterator to the internal list of frames.
    #[inline]
    pub fn iter(&self) -> Iter<'_, FxHashMap<usize, Rc<RefCell<Variable>>>> {
        self.table.iter()
    }

    /// Returns the inner list of frames.
    #[inline]
    pub fn inner(&self) -> &Vec<FxHashMap<usize, Rc<RefCell<Variable>>>> {
        &self.table
    }

    /// Returns the inner list of frames, mutable.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut Vec<FxHashMap<usize, Rc<RefCell<Variable>>>> {
        &mut self.table
    }

    /// Adds a new module. Returns a reference to the newly created module.
    #[inline]
    pub fn add_module(&mut self, name: usize) -> Rc<RefCell<SVTable>> {
        self.modules
            .entry(name)
            .or_insert_with(|| RefCell::new(SVTable::new()).into())
            .clone()
    }

    /// Gets a module by name. Returns an immutable reference to the module if found.
    #[inline]
    pub fn get_module(&self, name: usize) -> Result<Rc<RefCell<SVTable>>> {
        self.modules
            .get(&name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("failed to find module '{}'", resolve!(name)))
    }

    /// Adds a new scope.
    #[inline]
    pub fn add_scope(&mut self) {
        self.table.push(FxHashMap::default());
    }

    /// Gets a scope map. Mutable by default.
    #[inline]
    pub fn get_scope(
        &mut self,
        index: usize,
    ) -> Option<&mut FxHashMap<usize, Rc<RefCell<Variable>>>> {
        self.table.get_mut(index)
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
    fn find_variable(&self, id: usize) -> Option<&Rc<RefCell<Variable>>> {
        for scope in self.iter().rev() {
            if let Some(variable) = scope.get(&id) {
                return Some(variable);
            }
        }
        None
    }

    /// Helper method to find a variable in any scope with mutable access, returns the found variable reference.
    #[inline]
    fn find_variable_mut(&mut self, id: usize) -> Option<&Rc<RefCell<Variable>>> {
        for scope in self.inner_mut().iter_mut().rev() {
            if let Some(variable) = scope.get(&id) {
                return Some(variable);
            }
        }
        None
    }
}

impl MemoryInterface for SVTable {
    #[inline]
    fn get_owned(&self, id: usize) -> Result<Variable> {
        match self.find_variable(id) {
            Some(variable) => Ok(variable.borrow().clone()),
            None => bail!("failed to get owned value {:#?}", resolve!(id)),
        }
    }

    #[inline]
    fn get_ref(&self, id: usize) -> Result<Rc<RefCell<Variable>>> {
        match self.find_variable(id) {
            Some(variable) => Ok(variable.clone()),
            None => bail!("failed to get ref value {:#?}", resolve!(id)),
        }
    }

    #[inline]
    fn get_module(&self, id: usize) -> Result<Rc<RefCell<SVTable>>> {
        match self.modules.get(&id) {
            Some(module) => Ok(module.clone()),
            _ => {
                bail!("could not find module '{:#?}'", resolve!(id))
            }
        }
    }

    #[inline]
    fn declare(&mut self, id: usize, value: Variable, scope: usize) -> Result<()> {
        // add scopes if necessary
        while self.scopes() <= scope {
            self.add_scope();
        }

        // get variable map and insert new value. if the value already exists, bail
        let var_map = self
            .get_scope(scope)
            .context(format!("cannot declare at scope {scope}",))?;
        if var_map.insert(id, Rc::new(RefCell::new(value))).is_some() {
            bail!("variable '{}' already exists", resolve!(id));
        }
        Ok(())
    }

    #[inline]
    fn assign(&mut self, id: usize, value: Variable, scope: usize) -> Result<()> {
        // replace the value of the top-most variable if possible
        if let Some(variable) = self.find_variable_mut(id) {
            *variable.borrow_mut() = value;
            return Ok(());
        }

        // otherwise, manual insert. this is used for structures & modules
        let var_map = self
            .get_scope(scope)
            .context(format!("cannot assign at scope {scope}",))?;
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
                    id.to_path()
                        .iter()
                        .map(|id| resolve!(*id))
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
            let mut keys = scope.keys().collect::<Vec<&usize>>();
            keys.sort();
            for &key in keys {
                // obtain debug string respective to variable value
                let value = scope.get(&key).unwrap();
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
