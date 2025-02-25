use super::ASTNode;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Owned(ASTNode),
    Reference(Rc<ASTNode>),
}
