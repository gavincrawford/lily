use super::ASTNode;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Variable<'a> {
    Owned(ASTNode),
    Reference(&'a Rc<ASTNode>),
}
