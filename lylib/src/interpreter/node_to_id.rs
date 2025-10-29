use super::*;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Converts a node to an ID, if applicable.
    pub(crate) fn node_to_id(&mut self, node: Rc<ASTNode>) -> Result<ID> {
        match &*node {
            ASTNode::Literal(Token::Identifier(id)) => Ok(ID {
                id: IDKind::Symbol(*id),
            }),
            ASTNode::Function { id, .. } => Ok(id.clone()),
            ASTNode::Index { target, index } => {
                let parent = self.node_to_id(target.clone())?.get_kind().into();
                let index = self
                    .execute_expr(index)?
                    .context("index cannot be undefined")?
                    .as_index()?;
                Ok(ID {
                    id: IDKind::Member {
                        parent,
                        member: IDKind::Symbol(index).into(),
                    },
                })
            }
            ASTNode::Deref { parent, child } => {
                // recursively resolve the parent to get its ID
                let parent_id = self.node_to_id(parent.clone())?;

                // get the child identifier
                if let ASTNode::Literal(Token::Identifier(child_id)) = &**child {
                    // construct a member access ID
                    let parent_kind = Rc::new(parent_id.get_kind());
                    let child_kind = Rc::new(IDKind::Symbol(*child_id));

                    Ok(ID {
                        id: IDKind::Member {
                            parent: parent_kind,
                            member: child_kind,
                        },
                    })
                } else {
                    bail!("deref child must be an identifier, found {:#?}", child);
                }
            }
            _ => {
                bail!("cannot convert '{:#?}' to ID", node)
            }
        }
    }
}
