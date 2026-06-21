//! ID structure that allows for many kinds of identifiers.
//! These types include symbols, literals, and member access.

/// Debug implementation for `ID`.
mod debug;

use crate::interner::Symbol;
use std::rc::Rc;

/// Lily's internal identifier.
/// This is used at run-time as a stored token to track where things live.
///
/// Symbols are standard variable names. Indices name list elements, which are stored as a table
/// keyed by symbol ("1", "2", and so on), so they behave identically to symbols. Members represent
/// indirect access, as in a dereferencing expression.
#[derive(PartialEq, Eq, Hash, Clone)]
pub enum ID {
    Symbol(Symbol),
    Index(Symbol),
    Member { parent: Rc<ID>, member: Rc<ID> },
}

/// This trait provides an easy way to convert strings to symbolic IDs.
pub trait AsID {
    /// Converts into an `ID` type.
    fn as_id(&self) -> ID;
}

impl AsID for String {
    fn as_id(&self) -> ID {
        ID::new_sym(intern!(self))
    }
}

impl AsID for &'static str {
    fn as_id(&self) -> ID {
        ID::new_sym(intern!(*self))
    }
}

impl AsID for usize {
    fn as_id(&self) -> ID {
        ID::new_sym(*self)
    }
}

impl ID {
    /// Creates a new symbolic ID.
    pub(crate) fn new_sym(sym: Symbol) -> ID {
        ID::Symbol(sym)
    }

    /// Flattens an `ID`, resolving all children to form a flat path from the first to last
    /// identifier in the chain.
    pub fn to_path(&self) -> Vec<ID> {
        let mut path = Vec::new();
        collect_path_ids(self, &mut path);
        path
    }

    /// Flattens an `ID`, resolving all children to form a flat path from the first to last
    /// symbolic identifier in the chain.
    ///
    /// Note: This method discards contextual information each ID, in favor of interned symbols
    /// *only*.
    pub fn to_path_symbolic(&self) -> Vec<Symbol> {
        let mut path = Vec::new();
        collect_path_symbolic(self, &mut path);
        path
    }
}

// -------- Helpers --------

/// Helper function to recursively collect path identifiers.
fn collect_path_ids(id: &ID, path: &mut Vec<ID>) {
    match id {
        ID::Member { parent, member } => {
            collect_path_ids(parent, path);
            collect_path_ids(member, path);
        }
        leaf => path.push(leaf.clone()),
    }
}

/// Helper function to recursively collect symbolic path components.
fn collect_path_symbolic(id: &ID, path: &mut Vec<Symbol>) {
    match id {
        ID::Symbol(sym) | ID::Index(sym) => path.push(*sym),
        ID::Member { parent, member } => {
            collect_path_symbolic(parent, path);
            collect_path_symbolic(member, path);
        }
    }
}
