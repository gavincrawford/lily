use super::{mem::variable::ExFn, *};
use crate::errors::ExternalFunctionError;
use anyhow::anyhow;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Adds an arbitrary external function to this interpreter.
    pub fn inject_extern(&mut self, id: impl AsID, closure: Rc<ExFn>) -> Result<()> {
        self.declare(&id.as_id(), Variable::Extern(closure))
    }

    /// Adds the default external functions to this interpreter.
    /// This should only be called from `Interpreter::new`.
    pub(crate) fn inject_builtins(&mut self) -> Result<()> {
        /// Helper to inject external functions.
        macro_rules! external {
            ($fxn:expr) => {
                self.inject_extern(stringify!($fxn), $fxn)?
            };
        }

        /// Helper to unzip argument vectors.
        macro_rules! unpack {
            ($args:ident => $($arg:ident),*) => {
                let [$($arg),*] = $args.as_slice() else {
                    Err(ExternalFunctionError::InvalidArguments)?
                };
            };
        }

        // print
        let print: Rc<ExFn> = Rc::new(|stdout, _stdin, args| {
            unpack!(args => value);
            match &**value {
                ASTNode::Literal(token) => writeln!(stdout, "{token}"),
                other => writeln!(stdout, "{other:?}"),
            }?;
            Ok(None)
        });
        external!(print);

        // length
        let len: Rc<ExFn> = Rc::new(|_stdout, _stdin, args| {
            unpack!(args => item);
            match &**item {
                ASTNode::List(items) => Ok(Some(lit!(Token::Number(items.len() as f32)))),
                ASTNode::Literal(Token::Str(string)) => {
                    Ok(Some(lit!(Token::Number(string.len() as f32))))
                }
                _ => bail!("cannot take length of {:?}", &**item),
            }
        });
        external!(len);

        // sort
        let sort: Rc<ExFn> = Rc::new(|_stdout, _stdin, args| {
            unpack!(args => list);
            match &**list {
                ASTNode::List(items) => {
                    let mut clone = items.clone();
                    clone.sort();
                    Ok(Some(ASTNode::List(clone).into()))
                }
                _ => bail!("cannot sort {:?}", &**list),
            }
        });
        external!(sort);

        // split string by delimiter into list of strings
        let split: Rc<ExFn> = Rc::new(|_stdout, _stdin, args| {
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
                            Variable::Owned(ASTNode::Literal(Token::Str(part.to_string()))).into()
                        })
                        .collect();
                    Ok(Some(ASTNode::List(parts).into()))
                }
                _ => bail!("cannot split {:?}", &**string),
            }
        });
        external!(split);

        // chars (get characters of string as list)
        let chars: Rc<ExFn> = Rc::new(|_stdout, _stdin, args| {
            unpack!(args => string);
            match &**string {
                ASTNode::Literal(Token::Str(v)) => {
                    let values: Vec<Rc<RefCell<Variable>>> = v
                        .chars()
                        .map(|ch| Variable::Owned(ASTNode::Literal(Token::Char(ch))).into())
                        .collect();
                    Ok(Some(ASTNode::List(values).into()))
                }
                _ => bail!("cannot fetch characters of {:?}", &**string),
            }
        });
        external!(chars);

        // assert (returns err if condition != true)
        let assert: Rc<ExFn> = Rc::new(|_stdout, _stdin, args| {
            unpack!(args => condition);
            match &**condition {
                ASTNode::Literal(Token::Bool(true)) => {}
                _ => return Err(anyhow!("assertion failed")),
            }
            Ok(None)
        });
        external!(assert);

        // ================= MATH =================

        let cos: Rc<ExFn> = Rc::new(|_stdout, _stdin, args| {
            unpack!(args => n);
            match &**n {
                ASTNode::Literal(Token::Number(n)) => Ok(Some(lit!(Token::Number(n.cos())))),
                _ => Err(anyhow!("cannot call cosine on: {n:#?}")),
            }
        });
        external!(cos);

        let sin: Rc<ExFn> = Rc::new(|_stdout, _stdin, args| {
            unpack!(args => n);
            match &**n {
                ASTNode::Literal(Token::Number(n)) => Ok(Some(lit!(Token::Number(n.sin())))),
                _ => Err(anyhow!("cannot call sine on: {n:#?}")),
            }
        });
        external!(sin);

        Ok(())
    }
}
