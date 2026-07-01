use std::fmt::Debug;

use super::{mem::variable::ExFn, *};
use crate::{errors::ExternalFunctionError, interner::Symbol};
use anyhow::anyhow;

/// Helper to unzip argument vectors.
macro_rules! unpack {
    ($args:ident => $($arg:ident),*) => {
        let [$($arg),*] = $args.as_slice() else {
            Err(ExternalFunctionError::InvalidArguments)?
        };
    };
}

/// Owns all builtin closures, keyed by their interned name symbol; `Variable::Builtin(index)`
/// References a closure by its index in the `closures` vector.
pub(crate) struct Builtins {
    pub(crate) closures: Vec<(Symbol, Box<ExFn>)>,
}

impl Debug for Builtins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (sym, _) in &self.closures {
            writeln!(f, "{} [BUILTIN]", resolve!(*sym))?;
        }
        Ok(())
    }
}

impl Builtins {
    pub(crate) fn new() -> Self {
        Self {
            closures: vec![
                (
                    intern!("print"),
                    Box::new(|stdout, _stdin, args| {
                        unpack!(args => value);
                        match &**value {
                            ASTNode::Literal(token) => writeln!(stdout, "{token}"),
                            other => writeln!(stdout, "{other:?}"),
                        }?;
                        Ok(None)
                    }),
                ),
                (
                    intern!("len"),
                    Box::new(|_stdout, _stdin, args| {
                        unpack!(args => item);
                        match &**item {
                            ASTNode::List(items) => {
                                Ok(Some(lit!(Token::Number(items.len() as f32))))
                            }
                            ASTNode::Literal(Token::Str(string)) => {
                                Ok(Some(lit!(Token::Number(string.len() as f32))))
                            }
                            _ => bail!("cannot take length of {:?}", &**item),
                        }
                    }),
                ),
                (
                    intern!("split"),
                    Box::new(|_stdout, _stdin, args| {
                        unpack!(args => string, delimiter);
                        let delimiter = match &**delimiter {
                            ASTNode::Literal(Token::Str(s)) => s.clone(),
                            ASTNode::Literal(Token::Char(c)) => c.to_string(),
                            _ => bail!(
                                "split delimiter must be a string or char, got {:?}",
                                &**delimiter
                            ),
                        };
                        match &**string {
                            ASTNode::Literal(Token::Str(string)) => {
                                let parts: Vec<Rc<RefCell<Variable>>> = string
                                    .split(delimiter.as_str())
                                    .map(|part| {
                                        Variable::Owned(ASTNode::Literal(Token::Str(
                                            part.to_string(),
                                        )))
                                        .into()
                                    })
                                    .collect();
                                Ok(Some(ASTNode::List(parts).into()))
                            }
                            _ => bail!("cannot split {:?}", &**string),
                        }
                    }),
                ),
                (
                    intern!("sort"),
                    Box::new(|_stdout, _stdin, args| {
                        unpack!(args => list);
                        match &**list {
                            ASTNode::List(items) => {
                                let mut clone = items.clone();
                                clone.sort();
                                Ok(Some(ASTNode::List(clone).into()))
                            }
                            _ => bail!("cannot sort {:?}", &**list),
                        }
                    }),
                ),
                (
                    intern!("chars"),
                    Box::new(|_stdout, _stdin, args| {
                        unpack!(args => string);
                        match &**string {
                            ASTNode::Literal(Token::Str(v)) => {
                                let values: Vec<Rc<RefCell<Variable>>> = v
                                    .chars()
                                    .map(|ch| {
                                        Variable::Owned(ASTNode::Literal(Token::Char(ch))).into()
                                    })
                                    .collect();
                                Ok(Some(ASTNode::List(values).into()))
                            }
                            _ => bail!("cannot fetch characters of {:?}", &**string),
                        }
                    }),
                ),
                (
                    intern!("assert"),
                    Box::new(|_stdout, _stdin, args| {
                        unpack!(args => condition);
                        match &**condition {
                            ASTNode::Literal(Token::Bool(true)) => {}
                            _ => return Err(anyhow!("assertion failed")),
                        }
                        Ok(None)
                    }),
                ),
                // ========================================
                // ================= MATH =================
                // ========================================
                (
                    intern!("cos"),
                    Box::new(|_stdout, _stdin, args| {
                        unpack!(args => n);
                        match &**n {
                            ASTNode::Literal(Token::Number(n)) => {
                                Ok(Some(lit!(Token::Number(n.cos()))))
                            }
                            _ => Err(anyhow!("cannot call cosine on: {n:#?}")),
                        }
                    }),
                ),
                (
                    intern!("sin"),
                    Box::new(|_stdout, _stdin, args| {
                        unpack!(args => n);
                        match &**n {
                            ASTNode::Literal(Token::Number(n)) => {
                                Ok(Some(lit!(Token::Number(n.sin()))))
                            }
                            _ => Err(anyhow!("cannot call sine on: {n:#?}")),
                        }
                    }),
                ),
            ],
        }
    }
}

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Adds an arbitrary external function to this interpreter.
    pub fn inject_extern(&mut self, id: impl AsID, closure: Rc<ExFn>) -> Result<()> {
        self.declare(&id.as_id(), Variable::Extern(closure))
    }

    /// Declares all builtin closures into the base scope. Should only be called from `Interpreter::new`.
    pub(super) fn register_builtins(&mut self) -> Result<()> {
        for n in 0..self.builtins.closures.len() {
            let sym = self.builtins.closures[n].0;
            self.declare(&sym.as_id(), Variable::Builtin(n))?;
        }
        Ok(())
    }
}
