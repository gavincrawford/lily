use super::*;
use std::fmt::Debug;

impl Debug for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID({:#?})", self.get_kind())
    }
}

impl Debug for IDKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IDKind::Symbol(sym) => write!(f, "{}", resolve!(*sym)),
            IDKind::Literal(val) => write!(f, "{val}"),
            IDKind::Member { parent, member } => write!(f, "{parent:#?} -> {member:#?}"),
        }
    }
}
