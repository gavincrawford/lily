use super::*;
use std::{cmp::Ordering, fmt::Debug, mem::discriminant};

/// External function signature.
/// The first two arguments are the output and input handles. The third contains arguments.
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
    /// For builtin functions.
    /// The attached usize is *not* a symbol in this case, but rather an index over the list of
    /// static function closures that are built at the beginning of run-time.
    Builtin(usize),
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
            Variable::Builtin(n) => Variable::Builtin(*n),
            Variable::Type(node) => Variable::Type(node.clone()),
        }
    }
}

impl Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::Owned(node) => write!(f, "{node:#?}"),
            Variable::Function(node) | Variable::Type(node) => write!(f, "&{node:#?}"),
            Variable::Builtin(_) => write!(f, "EXTERN"),
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
        Some(self.cmp(other))
    }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                Variable::Owned(ASTNode::Literal(Token::Number(a))),
                Variable::Owned(ASTNode::Literal(Token::Number(b))),
            ) => a.total_cmp(b),
            (
                Variable::Owned(ASTNode::Literal(Token::Str(a))),
                Variable::Owned(ASTNode::Literal(Token::Str(b))),
            ) => a.cmp(b),
            _ => panic!("cannot order variables ({self:?}, {other:?})"),
        }
    }
}

impl MemoryInterface for Variable {
    fn get_owned(&self, id: usize) -> Result<Variable, MemoryError> {
        if let Variable::Owned(ASTNode::List(items)) = self {
            let item = items.get(id).ok_or(MemoryError::IndexOutOfBounds(id))?;
            let inner = item.borrow().clone();
            Ok(inner)
        } else {
            Err(MemoryError::VariableRead(format!("{self:#?}")))
        }
    }

    fn get_ref(&self, id: usize) -> Result<Rc<RefCell<Variable>>, MemoryError> {
        if let Variable::Owned(ASTNode::List(items)) = self {
            let item = items.get(id).ok_or(MemoryError::IndexOutOfBounds(id))?;
            Ok(item.clone())
        } else {
            Err(MemoryError::VariableRead(format!("{self:#?}")))
        }
    }

    fn get_module(&self, _: usize) -> Result<Rc<RefCell<SVTable>>, MemoryError> {
        Err(MemoryError::ModuleInVar)
    }

    fn declare(&mut self, id: usize, value: Variable, _: usize) -> Result<(), MemoryError> {
        if let Variable::Owned(ASTNode::List(items)) = self {
            items.insert(id, value.into());
            Ok(())
        } else {
            Err(MemoryError::VariableWrite(format!("{self:#?}")))
        }
    }

    fn assign(&mut self, id: usize, value: Variable, _: usize) -> Result<(), MemoryError> {
        if let Variable::Owned(ASTNode::List(items)) = self {
            *items.get_mut(id).ok_or(MemoryError::IndexOutOfBounds(id))? = value.into();
            Ok(())
        } else {
            Err(MemoryError::VariableWrite(format!("{self:#?}")))
        }
    }
}
