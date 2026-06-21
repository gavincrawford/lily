use super::*;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Converts a node to an ID, if applicable.
    pub(crate) fn node_to_id(&mut self, node: Rc<ASTNode>) -> Result<ID> {
        match &*node {
            ASTNode::Identifier(id) | ASTNode::Function { id, .. } => Ok(id.clone()),
            ASTNode::Index { target, index } => {
                let parent = self.node_to_id(target.clone())?.into();
                let index = self
                    .execute_expr(index)?
                    .context("index cannot be undefined")?
                    .as_index()?;
                Ok(ID::Member {
                    parent,
                    member: ID::Index(index).into(),
                })
            }
            ASTNode::Deref { parent, child } => {
                // recursively resolve the parent to get its ID
                let parent_id = self.node_to_id(parent.clone())?;

                // get the child identifier
                if let ASTNode::Identifier(child_id) = &**child {
                    // construct a member access ID
                    Ok(ID::Member {
                        parent: Rc::new(parent_id),
                        member: Rc::new(child_id.clone()),
                    })
                } else {
                    bail!("deref child must be an identifier, found {child:#?}");
                }
            }
            _ => {
                bail!("cannot convert '{node:#?}' to ID")
            }
        }
    }
}
