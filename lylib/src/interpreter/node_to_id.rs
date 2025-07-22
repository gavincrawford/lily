use super::*;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Converts a node to an ID, if applicable.
    pub(crate) fn node_to_id(&mut self, node: Rc<ASTNode>) -> Result<ID> {
        match &*node {
            ASTNode::Literal(Token::Identifier(id)) => Ok(ID::new(id)),
            ASTNode::Index { target, index } => {
                let parent = self.node_to_id(target.clone())?.get_kind().into();
                if let ASTNode::Literal(Token::Number(index)) =
                    &*self.execute_expr(index.clone())?.unwrap()
                {
                    let member = IDKind::Literal(index.to_string()).into();
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
