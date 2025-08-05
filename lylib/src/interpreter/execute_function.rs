use super::*;
use crate::lit;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Executes a given function with the given arguments.
    pub(crate) fn execute_function(
        &mut self,
        call_args: &Vec<Rc<ASTNode>>,
        function: Rc<ASTNode>,
    ) -> Result<Option<Rc<ASTNode>>> {
        if let ASTNode::Function {
            id: _id,
            arguments: fn_args,
            body,
        } = &*function
        {
            // push arguments
            assert_eq!(call_args.len(), fn_args.len());
            self.scope_id += 1;
            for (idx, arg) in fn_args.iter().enumerate() {
                let arg_expr = call_args.get(idx).unwrap(); // safety: assertion
                self.declare(
                    &ID::from_interned(*arg),
                    Variable::Owned(ASTNode::inner_to_owned(arg_expr)),
                )?;
            }

            // get result and clear scoped vars
            let result = self.execute(body.clone())?;
            self.drop_scope();

            // return result if present. otherwise, undefined
            return Ok(Some(result.unwrap_or(lit!(Token::Undefined))));
        }
        bail!("failed to execute non-function value")
    }
}
