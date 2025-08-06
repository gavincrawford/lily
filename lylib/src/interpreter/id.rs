//! ID structure that allows for many kinds of identifiers.

use crate::{get_global_interner, intern};
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

impl From<&usize> for ID {
    fn from(value: &usize) -> ID {
        ID {
            id: IDKind::Literal(*value),
        }
    }
}

impl From<usize> for ID {
    fn from(value: usize) -> ID {
        ID {
            id: IDKind::Literal(value),
        }
    }
}

impl ID {
    /// Creates a new ID from a string, interning it in the process.
    pub fn from_str(string: impl Into<String>) -> Self {
        let string = string.into();
        if string.contains('.') {
            // if the id has an access pattern, process it
            let mut parts = string.split('.').map(|s| {
                let interned_id = intern!(s.to_string());
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
            Self {
                id: IDKind::Literal(intern!(string)),
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
