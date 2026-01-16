use super::*;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Evaluates all expressions inside this list to their resolved values.
    /// This resolves variable references, indices, operations, and nested lists.
    pub(crate) fn resolve_refs(&mut self, mut expr: ASTNode) -> Result<Rc<ASTNode>> {
        // PERF: this requires ownership of the expression passed to it-- meaning we clone lists
        // every single time we return them. there's gotta be another way...

        // if this value isn't a list, it doesn't need to be resolved
        let ASTNode::List(ref mut items) = expr else {
            return Ok(expr.into());
        };

        // resolve indices, sub-lists, variable references, and expressions
        for variable in items.iter_mut() {
            let mut handle = variable.borrow_mut();
            match &*handle {
                // recursively resolve nested lists
                Variable::Owned(value) if matches!(value, ASTNode::List(_)) => {
                    let resolved_refs = self.resolve_refs(value.to_owned())?;
                    *handle = Variable::Owned(ASTNode::inner_to_owned(&resolved_refs));
                }
                // skip simple literals (already resolved)
                Variable::Owned(ASTNode::Literal(Token::Number(_)))
                | Variable::Owned(ASTNode::Literal(Token::Str(_)))
                | Variable::Owned(ASTNode::Literal(Token::Char(_)))
                | Variable::Owned(ASTNode::Literal(Token::Bool(_)))
                | Variable::Owned(ASTNode::Literal(Token::Undefined)) => {}
                // evaluate any other expression (identifiers, indices, ops, calls, etc.)
                Variable::Owned(value) => {
                    let resolved_item = self
                        .execute_expr(&value.clone().into())
                        .context("failed to resolve expression in list")?
                        .unwrap();
                    *handle = Variable::Owned(ASTNode::inner_to_owned(&resolved_item));
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
            if let Variable::Owned(ASTNode::Literal(Token::Number(n))) = &*items[0].borrow() {
                assert_eq!(*n, 2.0);
            }
        }
    }

    #[test]
    fn resolve_variable_refs_in_list() {
        let mut interpreter = Interpreter::new(Cursor::new(vec![]), Cursor::new(vec![]));

        // declare a variable
        interpreter
            .declare(
                &ID::new_sym(intern!("val")),
                Variable::Owned(ASTNode::Literal(Token::Number(42.0))),
            )
            .unwrap();

        // create a list with a variable reference: [val]
        let list_with_var = node!([ident!("val")]);

        // resolve references - should convert val to 42
        let result = interpreter.resolve_refs(list_with_var).unwrap();
        if let ASTNode::List(items) = &*result {
            assert_eq!(items.len(), 1);
            if let Variable::Owned(ASTNode::Literal(Token::Number(n))) = &*items[0].borrow() {
                assert_eq!(*n, 42.0);
            } else {
                panic!("expected number, got {:?}", items[0].borrow());
            }
        } else {
            panic!("expected list");
        }
    }
}
