use super::*;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Makes all references inside the expression absolute.
    pub(crate) fn resolve_refs(&mut self, mut expr: ASTNode) -> Result<Rc<ASTNode>> {
        if let ASTNode::List(ref mut items) = expr {
            // resolve list items
            for variable in items.iter_mut() {
                if let Variable::Owned(value) = &*variable {
                    match value {
                        ASTNode::Index {
                            target: _,
                            index: _,
                        } => {
                            let resolved_item = self
                                .execute_expr(value.clone().into())
                                .context("could not flatten index inside list")?
                                .unwrap();
                            *variable = Variable::Owned(ASTNode::inner_to_owned(&resolved_item));
                        }
                        ASTNode::List(_) => {
                            let resolved_refs = self.resolve_refs(value.to_owned())?;
                            *variable = Variable::Owned(ASTNode::inner_to_owned(&resolved_refs));
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(expr.into())
    }
}
