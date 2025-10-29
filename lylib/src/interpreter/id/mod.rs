//! ID structure that allows for many kinds of identifiers.

/// Debug implementations for `ID` & `IDKind`.
mod debug;

use std::rc::Rc;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct ID {
    pub(crate) id: IDKind,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum IDKind {
    Symbol(usize),
    Literal(usize),
    Member {
        parent: Rc<IDKind>,
        member: Rc<IDKind>,
    },
}

// NOTE: this is left over from when this trait was used. it still has one use, for which it
// actually does a good job at reducing verbosity, but it doesn't quite make sense to create an
// entire trait for it

pub(crate) trait AsID {
    /// Converts into an `ID` type.
    fn as_id(self) -> ID;
}

impl AsID for String {
    fn as_id(self) -> ID {
        ID {
            id: IDKind::Symbol(intern!(self)),
        }
    }
}

impl AsID for &'static str {
    fn as_id(self) -> ID {
        ID {
            id: IDKind::Symbol(intern!(self)),
        }
    }
}

impl ID {
    /// Gets the inner `IDKind` of this identifier.
    pub fn get_kind(&self) -> IDKind {
        self.id.to_owned()
    }

    /// Gets the inner `IDKind` of this identifier as a reference.
    pub fn get_kind_ref(&self) -> &IDKind {
        &self.id
    }

    /// Converts an `ID` into a vector of `IDKind` components, preserving type information.
    pub fn to_path_kinds(&self) -> Vec<IDKind> {
        let mut path = Vec::new();
        self.collect_path_kinds(&self.id, &mut path);
        path
    }

    /// Converts an `ID` into a vector of interned identifiers (usize).
    ///
    /// Note: This method loses type information about whether components are symbols or literals.
    /// Use `to_path_kinds()` when you need to distinguish between them.
    pub fn to_path(&self) -> Vec<usize> {
        let mut path = Vec::new();
        self.collect_path_interned(&self.id, &mut path);
        path
    }

    /// Helper function to recursively collect path components with type information.
    fn collect_path_kinds(&self, kind: &IDKind, path: &mut Vec<IDKind>) {
        match kind {
            IDKind::Symbol(sym) => path.push(IDKind::Symbol(*sym)),
            IDKind::Literal(val) => path.push(IDKind::Literal(*val)),
            IDKind::Member { parent, member } => {
                self.collect_path_kinds(parent, path);
                self.collect_path_kinds(member, path);
            }
        }
    }

    /// Helper function to recursively collect interned path components.
    fn collect_path_interned(&self, kind: &IDKind, path: &mut Vec<usize>) {
        match kind {
            IDKind::Symbol(sym) => path.push(*sym),
            IDKind::Literal(val) => path.push(*val),
            IDKind::Member { parent, member } => {
                self.collect_path_interned(parent, path);
                self.collect_path_interned(member, path);
            }
        }
    }
}
