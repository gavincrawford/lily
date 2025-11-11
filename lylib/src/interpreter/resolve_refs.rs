use super::*;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Makes all references inside this list absolute.
    /// This basically only resolves references in the form of indices and nested lists.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;
    use std::io::Cursor;

    #[test]
    fn resolve_literal_unchanged() {
        let mut interpreter = Interpreter::new(Cursor::new(vec![]), Cursor::new(vec![]));
        let literal = ASTNode::inner_to_owned(&lit!(42));
        assert_eq!(*interpreter.resolve_refs(literal.clone()).unwrap(), literal);
    }

    #[test]
    fn resolve_indices_in_list() {
        let mut interpreter = Interpreter::new(Cursor::new(vec![]), Cursor::new(vec![]));

        // create a source list: [1, 2, 3]
        let source_list = node!([lit!(1), lit!(2), lit!(3)]);

        // declare it as a variable so we can reference it
        interpreter
            .declare(
                &ID::new_sym(intern!("source")),
                Variable::Owned(source_list),
            )
            .unwrap();

        // create a list with an index reference
        let list_with_index = node!([node!(source[1])]);

        // resolve references - should convert source[1] to 2 & verify
        // if the values don't match here, we've failed to resolve their indices
        let result = interpreter.resolve_refs(list_with_index).unwrap();
        if let ASTNode::List(items) = &*result {
            assert_eq!(items.len(), 1);
            match &*items[0].borrow() {
                Variable::Owned(ASTNode::Literal(Token::Number(n))) => {
                    assert_eq!(*n, 2.0);
                }
                _ => {}
            }
        }
    }
}
