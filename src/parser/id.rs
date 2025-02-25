//! ID structure that allows for many kinds of identifiers.
// TODO this file should be with the interpreter

use super::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ID {
    id: IDKind,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum IDKind {
    Literal(String),
    Member {
        parent: Rc<IDKind>,
        member: Rc<IDKind>,
    },
}

impl ID {
    /// Creates a new ID from a literal identifier.
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        if id.contains('.') {
            // if the id has an access pattern, process it
            let mut parts = id
                .split('.')
                .map(|s| Rc::new(IDKind::Literal(s.to_string())));
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
                id: IDKind::Literal(id),
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

    /// Converts an `ID` into a vector of strings representing the path.
    pub fn to_path(&self) -> Vec<String> {
        let mut path = Vec::new();
        self.collect_path(&self.id, &mut path);
        path
    }

    /// Helper function to recursively collect path components.
    fn collect_path(&self, kind: &IDKind, path: &mut Vec<String>) {
        match kind {
            IDKind::Literal(name) => path.push(name.clone()),
            IDKind::Member { parent, member } => {
                self.collect_path(parent, path);
                self.collect_path(member, path);
            }
        }
    }
}

impl From<&str> for ID {
    fn from(value: &str) -> Self {
        if value.contains('.') {
            let mut parts = value
                .split('.')
                .map(|s| Rc::new(IDKind::Literal(s.to_string())));
            let mut parent = parts.next().expect("expected identifier");

            for member in parts {
                parent = Rc::new(IDKind::Member {
                    parent,
                    member: member.clone(),
                });
            }

            Self {
                id: (*parent).clone(),
            }
        } else {
            Self {
                id: IDKind::Literal(value.into()),
            }
        }
    }
}
