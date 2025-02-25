//! Implements the SVTable, or the scoped-variable table.

use super::Variable;
use anyhow::{bail, Result};
use std::{cell::RefCell, collections::HashMap, rc::Rc, slice::Iter};

#[derive(Debug)]
pub struct SVTable<'a> {
    /// Holds all the scope frames, each of which hold their respective variables.
    table: Vec<HashMap<String, Rc<RefCell<Variable>>>>,
    /// Holds all the modules defined at this SVTable's scope.
    modules: HashMap<String, Rc<RefCell<SVTable<'a>>>>,
}

impl<'a> SVTable<'a> {
    /// Creates a new scoped-variable table
    pub fn new() -> Self {
        Self {
            table: vec![],
            modules: HashMap::new(),
        }
    }

    /// Returns the iterator to the internal list of frames.
    pub fn iter(&self) -> Iter<'_, HashMap<String, Rc<RefCell<Variable>>>> {
        self.table.iter()
    }

    /// Returns the inner list of frames.
    pub fn inner(&self) -> &Vec<HashMap<String, Rc<RefCell<Variable>>>> {
        &self.table
    }

    /// Returns the inner list of frames, mutable.
    pub fn inner_mut(&mut self) -> &mut Vec<HashMap<String, Rc<RefCell<Variable>>>> {
        &mut self.table
    }

    /// Adds a new module. Returns a reference to the newly created module.
    pub fn add_module(&mut self, name: impl Into<String>) -> Rc<RefCell<SVTable<'a>>> {
        let name = name.into();
        self.modules
            .insert(name.to_owned(), RefCell::new(SVTable::new()).into());
        self.modules.get(&name.to_owned()).unwrap().to_owned()
    }

    /// Gets a module by name. Returns an immutable reference to the module if found.
    pub fn get_module(&self, name: impl Into<String>) -> Result<Rc<RefCell<SVTable<'a>>>> {
        let name = name.into();
        if let Some(module) = self.modules.get(&name) {
            return Ok(module.clone());
        } else {
            bail!("failed to find module '{}'", name);
        }
    }

    /// Adds a new scope.
    pub fn add_scope(&mut self) {
        self.table.push(HashMap::new());
    }

    /// Gets a scope map. Mutable by default.
    pub fn get_scope(
        &mut self,
        index: usize,
    ) -> Option<&mut HashMap<String, Rc<RefCell<Variable>>>> {
        self.table.get_mut(index)
    }

    /// Returns the number of scopes in this table.
    pub fn scopes(&self) -> usize {
        self.table.len()
    }
}
