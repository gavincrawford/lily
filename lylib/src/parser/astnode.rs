use super::*;
use crate::interpreter::{AsID, IDKind, MemoryInterface, SVTable, Variable, ID};
use std::{cell::RefCell, fmt::Display};

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    /// Represents a block of statements, grouped in a scope.
    Block(Vec<Rc<ASTNode>>),
    /// Holds a block, but represents a separate module.
    Module {
        alias: Option<usize>,
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
    Deref {
        parent: Rc<ASTNode>,
        child: Rc<ASTNode>,
    },
    Function {
        id: ID,
        arguments: Vec<usize>,
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
    UnaryOp {
        target: Rc<ASTNode>,
        op: Token,
    },
    Return(Rc<ASTNode>),
    Literal(Token),
    List(Vec<Rc<RefCell<Variable>>>),
}

impl ASTNode {
    /// Returns a reference to the constructor of the structure represented by this node. If this
    /// node is not a structure, or no constructor was found, returns `None`.
    pub(crate) fn constructor(&self) -> Option<Rc<ASTNode>> {
        if let ASTNode::Struct { id, body } = self {
            if let ASTNode::Block(nodes) = &**body {
                // get the struct name for comparison
                if let IDKind::Literal(struct_name) = id.get_kind() {
                    for node in nodes {
                        if let ASTNode::Function {
                            id,
                            arguments: _,
                            body: _,
                        } = &**node
                        {
                            if let IDKind::Literal(name) = id.get_kind() {
                                // functions with an identical name to the structure are
                                // constructors, and should be treated as such
                                if name == struct_name {
                                    return Some(node.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Returns the truthiness of this node.
    /// True boolians, literals, lists, and structure instances are truthy.
    pub(crate) fn is_truthy(&self) -> bool {
        match self {
            ASTNode::Literal(Token::Bool(v)) => *v,
            ASTNode::Literal(ref t) if *t != Token::Undefined => true,
            ASTNode::List(_) => true,
            ASTNode::Instance { .. } => true,
            _ => false,
        }
    }

    /// Returns `Err` if this index is out of bounds. Otherwise, returns `Ok(idx)`.
    /// Technically applies to all literal numbers, but should be used to check if an index value
    /// is within `0 < n < usize::MAX`.
    pub(crate) fn as_index(&self) -> Result<usize> {
        if let ASTNode::Literal(Token::Number(n)) = self {
            if *n < 0. {
                bail!("index values must be non-negative");
            } else if *n > usize::MAX as f32 {
                bail!("index value larger than {}", usize::MAX);
            }
            return Ok(*n as usize);
        }
        unreachable!("attempted to convert non-numeric type into index");
    }

    /// Create the default SVT for this struct if applicable.
    pub(crate) fn create_struct_template(&self) -> Result<SVTable> {
        if let ASTNode::Struct { id: _, body } = self {
            if let ASTNode::Block(nodes) = &**body {
                let mut default_fields = vec![];
                for node in nodes {
                    match &**node {
                        // if the member is a structure variable, add an owned value
                        ASTNode::Declare { target, value } => {
                            // if this field is literal, add it, bail otherwise
                            if let ASTNode::Literal(Token::Identifier(variable)) = &**target {
                                default_fields.push((
                                    variable.as_id(),
                                    Variable::Owned(ASTNode::inner_to_owned(value)),
                                ));
                            } else {
                                bail!("invalid default field '{:?}'", target);
                            }
                        }

                        // if the member is a function, add a reference to it
                        ASTNode::Function {
                            id,
                            arguments: _,
                            body: _,
                        } => default_fields.push((id.clone(), Variable::Function(node.clone()))),

                        other => {
                            bail!("unexpected structure field: {other:?}")
                        }
                    }
                }
                let mut svt = SVTable::new();
                for (target, value) in default_fields {
                    // get the first value in the interned path
                    let id = *target.to_path().first().unwrap();

                    // add it to the table
                    svt.declare(id, value, 0)?;
                }
                return Ok(svt);
            }
        }
        bail!("cannot create template of non-structure value: {:?}", self);
    }

    pub(crate) fn inner_to_owned(rc: &Rc<ASTNode>) -> ASTNode {
        (**rc).clone()
    }
}

impl Display for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Literal(token) => write!(f, "{token}"),
            _ => write!(f, "{self:?}"),
        }?;
        Ok(())
    }
}
