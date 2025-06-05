use super::*;
use std::{fmt::Debug, mem::discriminant};

/// External function signature.
/// The first two arguments are the input and output handles. The third contains arguments.
type ExFn = dyn for<'a> Fn(
    Rc<RefCell<dyn Write + 'a>>,
    Rc<RefCell<dyn Read + 'a>>,
    &Vec<Rc<ASTNode>>,
) -> Result<Option<Rc<ASTNode>>>;

/// Represents stored information.
#[derive(Clone)]
pub enum Variable {
    /// For owned variables.
    Owned(ASTNode),
    /// For functions.
    Function(Rc<ASTNode>),
    /// For external functions.
    Extern(Rc<ExFn>),
    /// For non-standard types, such as structures.
    Type(Rc<ASTNode>),
}

impl Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::Owned(node) => write!(f, "{:?}", node),
            Variable::Function(node) | Variable::Type(node) => write!(f, "&{:?}", node),
            Variable::Extern(_) => write!(f, "EXTERN"),
        }
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        // if variables are not the same variant, false
        if !(discriminant(self) == discriminant(other)) {
            return false;
        }

        // otherwise, all variants follow regular comparison rules except externals
        match (self, other) {
            (Variable::Owned(a), Variable::Owned(b)) => a == b,
            (Variable::Function(a), Variable::Function(b))
            | (Variable::Type(a), Variable::Type(b)) => a == b,
            _ => panic!(
                "cannot comapre external variables ({:?}, {:?})",
                self, other
            ),
        }
    }
}
