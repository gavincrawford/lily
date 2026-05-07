//! A generic flat map backed by a `Vec`, indexed by interned identifiers.

use crate::interner::Symbol;
use std::{cell::RefCell, rc::Rc, vec::IntoIter};

#[derive(Debug, PartialEq)]
pub(crate) struct FlatRcMap<T: PartialEq> {
    map: Vec<Option<Rc<RefCell<T>>>>,
}

impl<T: PartialEq> Default for FlatRcMap<T> {
    fn default() -> Self {
        Self { map: Vec::new() }
    }
}

impl<T: PartialEq> IntoIterator for FlatRcMap<T> {
    type Item = Option<Rc<RefCell<T>>>;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<T: PartialEq> FlatRcMap<T> {
    pub(crate) fn new(map: Vec<Option<Rc<RefCell<T>>>>) -> Self {
        Self { map }
    }

    /// Shallow clone this map, sharing `Rc` references with the original.
    /// Writes to the clone go through `SVTable::assign`, which replaces the `Rc` slot
    /// rather than mutating through it (copy-on-write).
    pub(crate) fn shallow_clone(&self) -> Self {
        Self::new(self.map.clone())
    }

    /// Returns whether an item exists at this identifier.
    pub(crate) fn contains(&self, id: Symbol) -> bool {
        self.get(id).is_some()
    }

    /// Gets a value by its identifier, cloning the inner `Rc`.
    pub(crate) fn get(&self, id: Symbol) -> Option<Rc<RefCell<T>>> {
        self.map.get(id).and_then(|value| value.clone())
    }

    /// Inserts a value into the map, returning the previous value if it existed.
    pub(crate) fn insert(&mut self, id: Symbol, value: Rc<RefCell<T>>) -> Option<Rc<RefCell<T>>> {
        if id < self.map.len() {
            // replace the slot (which may be None) and return the old value if present
            std::mem::replace(&mut self.map[id], Some(value))
        } else {
            // fill gaps with None
            while id > self.map.len() {
                self.map.push(None);
            }
            // push new value at the end
            self.map.push(Some(value));
            None
        }
    }

    /// Removes all values from this map.
    pub(crate) fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns a list of the values within this map by ID.
    pub(crate) fn keys(&self) -> Vec<Symbol> {
        let mut keys = vec![];
        for (idx, value) in self.map.iter().enumerate() {
            if value.is_some() {
                keys.push(idx);
            }
        }
        keys
    }
}
