use super::{IDKind, Rc, Token, ID};
use crate::interpreter::Variable;

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    /// Represents a block of statements, grouped in a scope.
    Block(Vec<Rc<ASTNode>>),
    /// Holds a block, but represents a separate module.
    Module {
        alias: Option<String>,
        body: Rc<ASTNode>,
    },

    Index {
        id: ID,
        index: Rc<ASTNode>,
    },
    Assign {
        id: ID,
        value: Rc<ASTNode>,
    },
    Declare {
        id: ID,
        value: Rc<ASTNode>,
    },
    Function {
        id: ID,
        arguments: Vec<String>,
        body: Rc<ASTNode>,
    },
    FunctionCall {
        id: ID,
        arguments: Vec<Rc<ASTNode>>,
    },
    Struct {
        id: ID,
        body: Rc<ASTNode>,
    },
    Instance {
        kind: Rc<Variable>,
        id: ID,
        fields: Vec<Rc<ASTNode>>,
    },
    Conditional {
        condition: Rc<ASTNode>,
        if_body: Rc<ASTNode>,
        else_body: Rc<ASTNode>,
    },
    Loop {
        condition: Rc<ASTNode>,
        body: Rc<ASTNode>,
    },
    Op {
        lhs: Rc<ASTNode>,
        op: Token,
        rhs: Rc<ASTNode>,
    },
    Return(Rc<ASTNode>),
    Literal(Token),
    List(Vec<Rc<ASTNode>>),
}

impl ASTNode {
    /// Returns a reference to the constructor of the structure represented by this node. If this
    /// node is not a structure, or no constructor was found, returns `None`.
    pub fn constructor(&self) -> Option<Rc<ASTNode>> {
        if let ASTNode::Struct { id: _, body } = self {
            if let ASTNode::Block(nodes) = &**body {
                for node in nodes {
                    if let ASTNode::Function {
                        id,
                        arguments: _,
                        body: _,
                    } = &**node
                    {
                        if let IDKind::Literal(name) = id.get_kind() {
                            if name == "constructor" {
                                return Some(node.clone());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Gets the default fields of this struct, if applicable. Returns `None` if otherwise.
    pub fn default_fields(&self) -> Option<Vec<Rc<ASTNode>>> {
        if let ASTNode::Struct { id: _, body } = self {
            if let ASTNode::Block(nodes) = &**body {
                let mut default_fields = vec![];
                for node in nodes {
                    if let ASTNode::Declare { id: _, value: _ } = &**node {
                        default_fields.push(ASTNode::inner_to_owned(node).into());
                    }
                }
                if !default_fields.is_empty() {
                    return Some(default_fields);
                }
            }
        }
        None
    }

    pub fn inner_to_owned(rc: &Rc<ASTNode>) -> ASTNode {
        (&*rc.clone()).clone()
    }
}
