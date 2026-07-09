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
                // avoid a full clone of the argument when this call owns the only reference to it
                let owned = Rc::try_unwrap(arg).unwrap_or_else(|arg| (*arg).clone());
                self.declare(arg_id, Variable::Owned(owned))?;
            }

            // get result and clear scoped vars
            let result = self.execute(body.clone())?;
            self.drop_scope();

            return Ok(result);
        }
        bail!("failed to execute non-function value")
    }
}
