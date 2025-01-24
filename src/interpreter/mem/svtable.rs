//! Implements the SVTable, or the scoped-variable table.

use super::Variable;
use std::{collections::HashMap, slice::Iter};

#[derive(Debug)]
pub struct SVTable<'a> {
    /// Holds all the scope frames, each of which hold their respective variables.
    table: Vec<HashMap<String, Variable<'a>>>,
}

impl<'a> SVTable<'a> {
    /// Creates a new scoped-variable table
    pub fn new() -> Self {
        Self { table: vec![] }
    }

    /// Returns the iterator to the internal list of frames.
    pub fn iter(&self) -> Iter<'_, HashMap<String, Variable<'a>>> {
        self.table.iter()
    }

    /// Returns the inner list of frames.
    pub fn inner(&self) -> &Vec<HashMap<String, Variable<'a>>> {
        &self.table
    }

    /// Returns the inner list of frames, mutable.
    pub fn inner_mut(&mut self) -> &mut Vec<HashMap<String, Variable<'a>>> {
        &mut self.table
    }

    /// Adds a new scope.
    pub fn add_scope(&mut self) {
        self.table.push(HashMap::new());
    }

    /// Gets a scope map. Mutable by default.
    pub fn get_scope(&mut self, index: usize) -> Option<&mut HashMap<String, Variable<'a>>> {
        self.table.get_mut(index)
    }

    /// Returns the number of scopes in this table.
    pub fn scopes(&self) -> usize {
        self.table.len()
    }
}
