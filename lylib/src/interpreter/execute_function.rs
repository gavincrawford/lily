use super::*;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Executes a given function with the given arguments.
    pub(crate) fn execute_function(
        &mut self,
        call_args: Vec<Rc<ASTNode>>,
        function: Rc<ASTNode>,
    ) -> Result<Option<Rc<ASTNode>>> {
        if let ASTNode::Function {
            id,
            arguments,
            body,
            ..
        } = &*function
        {
            // make sure all builtins are available in execution scope
            if let Some(ctx) = self.context.clone() {
                self.register_builtins_into(&ctx)
                    .context("failed to register builtins into call context")?;
            }

            // push arguments
            if call_args.len() != arguments.len() {
                let name = match id {
                    ID::Symbol(sym) => resolve!(*sym),
                    other => format!("{other:?}"),
                };
                bail!(
                    "function '{name}' expects {} argument(s), got {}",
                    arguments.len(),
                    call_args.len()
                );
            }
            self.scope_id += 1;
            for (arg_id, arg) in arguments.iter().zip(call_args) {
                self.declare(arg_id, Variable::Owned(ASTNode::try_inner(arg)))?;
            }

            // get result and clear scoped vars
            let result = self.execute(body.clone())?;
            self.drop_scope();

            return Ok(result);
        }
        bail!("failed to execute non-function value")
    }
}
