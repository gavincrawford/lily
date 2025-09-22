use super::*;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Makes all references inside this list absolute.
    pub(crate) fn resolve_refs(&mut self, mut expr: ASTNode) -> Result<Rc<ASTNode>> {
        // PERF: this requires ownership of the expression passed to it-- meaning we clone lists
        // every single time we return them. there's gotta be another way...

        // if this value isn't a list, it doesn't need to be resolved
        let ASTNode::List(ref mut items) = expr else {
            return Ok(expr.into());
        };

        // resolve indices & sub-lists
        for variable in items.iter_mut() {
            let mut handle = variable.borrow_mut();
            match &*handle {
                Variable::Owned(value) if matches!(value, ASTNode::Index { .. }) => {
                    let resolved_item = self
                        .execute_expr(&value.clone().into())
                        .context("failed to resolve list value")?
                        .unwrap();
                    *handle = Variable::Owned(ASTNode::inner_to_owned(&resolved_item));
                }
                Variable::Owned(value) if matches!(value, ASTNode::List(_)) => {
                    let resolved_refs = self.resolve_refs(value.to_owned())?;
                    *handle = Variable::Owned(ASTNode::inner_to_owned(&resolved_refs));
                }
                _ => {}
            }
        }
        Ok(expr.into())
    }
}
