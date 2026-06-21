use super::*;
use std::fmt::Debug;

impl Debug for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID(")?;
        match self {
            ID::Symbol(sym) => write!(f, "{}", resolve!(*sym)),
            ID::Index(val) => write!(f, "{val}"),
            ID::Member { parent, member } => write!(f, "{parent:#?} -> {member:#?}"),
        }?;
        write!(f, ")")
    }
}
