use super::ASTNode;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    /// For owned variables.
    Owned(ASTNode),
    /// For referenced variables.
    Reference(Rc<ASTNode>),
    /// For non-standard types, such as structures.
    Type(Rc<ASTNode>),
}
