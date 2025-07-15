use super::*;
use std::{fmt::Debug, mem::discriminant};

/// External function signature.
/// The first two arguments are the input and output handles. The third contains arguments.
pub type ExFn = dyn for<'a> Fn(
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

impl Into<Rc<RefCell<Variable>>> for Variable {
    fn into(self) -> Rc<RefCell<Variable>> {
        Rc::new(RefCell::new(self))
    }
}

impl From<ASTNode> for Variable {
    fn from(value: ASTNode) -> Self {
        Self::Owned(value)
    }
}

impl From<Rc<ASTNode>> for Variable {
    fn from(value: Rc<ASTNode>) -> Self {
        Self::Owned(ASTNode::inner_to_owned(&value))
    }
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

impl MemoryInterface for Variable {
    fn get_owned(&self, id: String) -> Result<Variable> {
        if let Variable::Owned(ASTNode::List(items)) = self {
            // saftey: checked in interpreter
            let idx = id.parse::<usize>().unwrap();
            let item = items.get(idx).context("index out of bounds")?;
            let inner = (&*item.clone()).clone().into_inner();
            Ok(inner)
        } else {
            bail!("invalid access to variable '{:?}'", self);
        }
    }

    fn get_ref(&self, id: String) -> Result<Rc<RefCell<Variable>>> {
        if let Variable::Owned(ASTNode::List(items)) = self {
            // saftey: checked in interpreter
            let idx = id.parse::<usize>().unwrap();
            let item = items.get(idx).context("index out of bounds")?;
            Ok(item.clone())
        } else {
            bail!("invalid access to variable '{:?}'", self);
        }
    }

    fn get_module(&self, _: String) -> Result<Rc<RefCell<SVTable>>> {
        bail!("variables cannot contain modules");
    }

    fn declare(&mut self, id: String, value: Variable, _: usize) -> Result<()> {
        if let Variable::Owned(ASTNode::List(items)) = self {
            // saftey: checked in interpreter
            let idx = id.parse::<usize>().unwrap();
            items.insert(idx, value.into());
            Ok(())
        } else {
            bail!("invalid declaration to variable '{:?}'", self);
        }
    }

    fn assign(&mut self, id: String, value: Variable, _: usize) -> Result<()> {
        if let Variable::Owned(ASTNode::List(items)) = self {
            // saftey: checked in interpreter
            let idx = id.parse::<usize>().unwrap();
            *items.get_mut(idx).context("index out of bounds")? = value.into();
            Ok(())
        } else {
            bail!("invalid assignment to variable '{:?}'", self);
        }
    }
}
