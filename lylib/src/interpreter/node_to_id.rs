use super::*;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Converts a node to an ID, if applicable.
    pub(crate) fn node_to_id(&mut self, node: Rc<ASTNode>) -> Result<ID> {
        match &*node {
            ASTNode::Literal(Token::Identifier(id)) => Ok(ID::from_interned(*id)),
            ASTNode::Index { target, index } => {
                let parent = self.node_to_id(target.clone())?.get_kind().into();
                if let ASTNode::Literal(Token::Number(index)) =
                    &*self.execute_expr(index.clone())?.unwrap()
                {
                    // TODO: This creates an interned string from a number index, which may not be correct
                    // We need access to interner here to properly handle this case
                    let member = IDKind::Literal(*index as usize).into();
                    return Ok(ID {
                        id: IDKind::Member { parent, member },
                    });
                }
                panic!()
            }
            _ => {
                bail!("cannot convert '{:?}' to ID", node)
            }
        }
    }
}
