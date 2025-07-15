use super::*;
use crate::interpreter::{IDKind, SVTable, Variable, ID};
use std::{cell::RefCell, fmt::Display};

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
        target: Rc<ASTNode>,
        index: Rc<ASTNode>,
    },
    Assign {
        target: Rc<ASTNode>,
        value: Rc<ASTNode>,
    },
    Declare {
        target: Rc<ASTNode>,
        value: Rc<ASTNode>,
    },
    Function {
        id: ID,
        arguments: Vec<String>,
        body: Rc<ASTNode>,
    },
    FunctionCall {
        target: Rc<ASTNode>,
        arguments: Vec<Rc<ASTNode>>,
    },
    Struct {
        id: ID,
        body: Rc<ASTNode>,
    },
    Instance {
        kind: Rc<Variable>,
        id: ID,
        svt: Rc<RefCell<SVTable>>,
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
    List(Vec<Variable>),
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

    /// Returns the truthiness of this node.
    /// True booleans and literals are truthy. Non-literal expressions are not.
    pub fn is_truthy(&self) -> bool {
        match self {
            ASTNode::Literal(Token::Bool(v)) => *v,
            ASTNode::Literal(ref t) if *t != Token::Undefined => true,
            _ => false,
        }
    }

    /// Create the default SVT for this struct, if applicable. Returns `None` if otherwise.
    pub fn default_svt(&self) -> Option<SVTable> {
        if let ASTNode::Struct { id: _, body } = self {
            if let ASTNode::Block(nodes) = &**body {
                let mut default_fields = vec![];
                for node in nodes {
                    if let ASTNode::Declare { target: id, value } = &**node {
                        default_fields.push((id.clone(), ASTNode::inner_to_owned(value)));
                    }
                }
                let mut svt = SVTable::new();
                svt.add_scope();
                let inner_table = svt.inner_mut();
                default_fields.iter().for_each(|(target, value)| {
                    // convert this field to an ID
                    let id = ID::node_to_id(target.to_owned()).unwrap();

                    // get the first value in the path
                    let id = id.to_path().get(0).unwrap().to_owned();

                    // add it to the table
                    inner_table
                        .first_mut()
                        .unwrap()
                        .insert(id.to_owned(), Variable::Owned(value.to_owned()).into());
                });
                return Some(svt);
            }
        }
        None
    }

    pub fn inner_to_owned(rc: &Rc<ASTNode>) -> ASTNode {
        (&*rc.clone()).clone()
    }
}

impl Display for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Literal(token) => write!(f, "{}", token),
            _ => write!(f, "{:?}", self),
        }?;
        Ok(())
    }
}
