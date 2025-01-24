//! ID structure that allows for many kinds of identifiers.
// TODO this file should be with the interpreter

use super::*;

#[derive(Debug, PartialEq, Clone)]
pub struct ID {
    id: IDKind,
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
}

impl From<&str> for ID {
    fn from(value: &str) -> Self {
        Self {
            id: IDKind::Literal(value.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum IDKind {
    Literal(String),
    Member {
        parent: Rc<IDKind>,
        member: Rc<IDKind>,
    },
}
