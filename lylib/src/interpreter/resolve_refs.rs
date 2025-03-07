use super::*;

impl Interpreter {
    /// Makes all references inside the expression absolute.
    pub fn resolve_refs(&mut self, mut expr: ASTNode) -> Result<Rc<ASTNode>> {
        if let ASTNode::List(ref mut items) = expr {
            for item in items.iter_mut() {
                match **item {
                    ASTNode::Index { id: _, index: _ } => {
                        *item = self
                            .execute_expr(item.clone())
                            .context("could not flatten index inside list")?
                            .unwrap();
                    }
                    ASTNode::List(_) => {
                        *item = self.resolve_refs(ASTNode::inner_to_owned(item))?;
                    }
                    _ => {}
                }
            }
        }
        Ok(expr.into())
    }
}
