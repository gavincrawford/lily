use super::*;

impl Interpreter {
    /// Makes all references inside the expression absolute.
    pub fn resolve_refs(&mut self, mut expr: ASTNode) -> Result<Rc<ASTNode>> {
        if let ASTNode::List(ref mut items) = expr {
            // get list items, always at scope zero
            let mut table = items.borrow_mut();
            let items = table.get_scope(0).unwrap();

            // resolve list items
            for (_, item) in items.iter_mut() {
                let mut handle = item.borrow_mut();
                if let Variable::Owned(value) = &*handle {
                    match value {
                        ASTNode::Index {
                            target: _,
                            index: _,
                        } => {
                            let resolved_item = self
                                .execute_expr(value.clone().into())
                                .context("could not flatten index inside list")?
                                .unwrap();
                            *handle = Variable::Owned(ASTNode::inner_to_owned(&resolved_item));
                        }
                        ASTNode::List(_) => {
                            let resolved_refs = self.resolve_refs(value.to_owned())?;
                            *handle = Variable::Owned(ASTNode::inner_to_owned(&resolved_refs));
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(expr.into())
    }
}
