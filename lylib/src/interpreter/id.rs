//! ID structure that allows for many kinds of identifiers.

use crate::{get_global_interner, interner::StringInterner};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ID {
    pub(crate) id: IDKind,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum IDKind {
    Literal(usize),
    Member {
        parent: Rc<IDKind>,
        member: Rc<IDKind>,
    },
}

impl ID {
    /// Creates a new ID from a literal identifier using the provided interner.
    pub fn new(id: impl Into<String>, interner: &mut StringInterner) -> Self {
        let id = id.into();
        if id.contains('.') {
            // if the id has an access pattern, process it
            let mut parts = id
                .split('.')
                .map(|s| Rc::new(IDKind::Literal(interner.intern(s.to_string()))));
            let mut parent = parts.next().expect("expected identifier");

            // build the nested member structure
            for member in parts {
                parent = Rc::new(IDKind::Member {
                    parent,
                    member: member.clone(),
                });
            }

            // return the constructed member access
            Self {
                id: (*parent).clone(),
            }
        } else {
            // otherwise, this id is literal
            Self {
                id: IDKind::Literal(interner.intern(id)),
            }
        }
    }

    /// Creates a new ID from an already-interned identifier.
    pub fn from_interned(id: usize) -> Self {
        Self {
            id: IDKind::Literal(id),
        }
    }

    // TODO: remove after stabilizing interning
    /// Creates a new ID from a string using the global interner.
    /// This is a compatibility method during the transition to explicit interner passing.
    pub fn new_compat(id: impl Into<String>) -> Self {
        let id = id.into();
        if id.contains('.') {
            // if the id has an access pattern, process it
            let mut parts = id.split('.').map(|s| {
                let interned_id = get_global_interner().lock().unwrap().intern(s.to_string());
                Rc::new(IDKind::Literal(interned_id))
            });
            let mut parent = parts.next().expect("expected identifier");

            // build the nested member structure
            for member in parts {
                parent = Rc::new(IDKind::Member {
                    parent,
                    member: member.clone(),
                });
            }

            // return the constructed member access
            Self {
                id: (*parent).clone(),
            }
        } else {
            // otherwise, this id is literal
            let interned_id = get_global_interner().lock().unwrap().intern(id);
            Self {
                id: IDKind::Literal(interned_id),
            }
        }
    }

    /// Gets the inner `IDKind` of this identifier.
    pub fn get_kind(&self) -> IDKind {
        self.id.to_owned()
    }

    /// Gets the inner `IDKind` of this identifier as a reference.
    pub fn get_kind_ref(&self) -> &IDKind {
        &self.id
    }

    /// Converts an `ID` into a vector of interned identifiers (usize).
    pub fn to_path(&self) -> Vec<usize> {
        let mut path = Vec::new();
        self.collect_path_interned(&self.id, &mut path);
        path
    }

    /// Helper function to recursively collect interned path components.
    fn collect_path_interned(&self, kind: &IDKind, path: &mut Vec<usize>) {
        match kind {
            IDKind::Literal(id) => path.push(*id),
            IDKind::Member { parent, member } => {
                self.collect_path_interned(parent, path);
                self.collect_path_interned(member, path);
            }
        }
    }
}

impl From<&str> for ID {
    fn from(value: &str) -> Self {
        ID::new_compat(value)
    }
}
