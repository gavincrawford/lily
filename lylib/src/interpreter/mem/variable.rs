use super::*;
use std::{cmp::Ordering, fmt::Debug, mem::discriminant};

/// External function signature.
/// The first two arguments are the input and output handles. The third contains arguments.
pub type ExFn = dyn for<'a> Fn(
    &'a mut dyn Write,
    &'a mut dyn Read,
    &Vec<Rc<ASTNode>>,
) -> Result<Option<Rc<ASTNode>>>;

/// Represents stored information.
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

impl From<Variable> for Rc<RefCell<Variable>> {
    fn from(val: Variable) -> Self {
        Rc::new(RefCell::new(val))
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

impl Clone for Variable {
    fn clone(&self) -> Self {
        match self {
            // lists deeply clone their items
            Variable::Owned(ASTNode::List(items)) => {
                let cloned_items: Vec<_> = items
                    .iter()
                    .map(|item| Rc::new(RefCell::new(item.borrow().clone())))
                    .collect();
                Variable::Owned(ASTNode::List(cloned_items))
            }

            // all other variables are cloned as is
            Variable::Owned(node) => Variable::Owned(node.clone()),
            Variable::Function(node) => Variable::Function(node.clone()),
            Variable::Extern(func) => Variable::Extern(func.clone()),
            Variable::Type(node) => Variable::Type(node.clone()),
        }
    }
}

impl Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::Owned(node) => write!(f, "{node:#?}"),
            Variable::Function(node) | Variable::Type(node) => write!(f, "&{node:#?}"),
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
            _ => panic!("cannot compare external variables ({self:?}, {other:?})"),
        }
    }
}

impl Eq for Variable {}

impl PartialOrd for Variable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (
                Variable::Owned(ASTNode::Literal(Token::Number(a))),
                Variable::Owned(ASTNode::Literal(Token::Number(b))),
            ) => a.partial_cmp(b),
            (
                Variable::Owned(ASTNode::Literal(Token::Str(a))),
                Variable::Owned(ASTNode::Literal(Token::Str(b))),
            ) => a.partial_cmp(b),
            _ => panic!("cannot order variables ({self:?}, {other:?})"),
        }
    }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}

impl MemoryInterface for Variable {
    fn get_owned(&self, id: usize) -> Result<Variable> {
        if let Variable::Owned(ASTNode::List(items)) = self {
            let item = items.get(id).context("index out of bounds")?;
            let inner = item.borrow().clone();
            Ok(inner)
        } else {
            bail!("invalid access to variable '{self:#?}'");
        }
    }

    fn get_ref(&self, id: usize) -> Result<Rc<RefCell<Variable>>> {
        if let Variable::Owned(ASTNode::List(items)) = self {
            let item = items.get(id).context("index out of bounds")?;
            Ok(item.clone())
        } else {
            bail!("invalid access to variable '{self:#?}'");
        }
    }

    fn get_module(&self, _: usize) -> Result<Rc<RefCell<SVTable>>> {
        bail!("variables cannot contain modules");
    }

    fn declare(&mut self, id: usize, value: Variable, _: usize) -> Result<()> {
        if let Variable::Owned(ASTNode::List(items)) = self {
            items.insert(id, value.into());
            Ok(())
        } else {
            bail!("invalid declaration to variable '{self:#?}'");
        }
    }

    fn assign(&mut self, id: usize, value: Variable, _: usize) -> Result<()> {
        if let Variable::Owned(ASTNode::List(items)) = self {
            *items.get_mut(id).context("index out of bounds")? = value.into();
            Ok(())
        } else {
            bail!("invalid assignment to variable '{self:#?}'");
        }
    }
}
