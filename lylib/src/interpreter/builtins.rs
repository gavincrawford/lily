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
        /// Injects an external function.
        macro_rules! inject {
            ($fxn:expr) => {
                self.inject_extern(stringify!($fxn), $fxn)?
            };
        }

        // print
        let print: Rc<ExFn> = Rc::new(|stdout, _stdin, args| {
            let [value] = args.as_slice() else {
                Err(ExternalFunctionError::InvalidArguments)?
            };
            match &**value {
                ASTNode::Literal(token) => writeln!(stdout, "{token}"),
                other => writeln!(stdout, "{other:?}"),
            }?;
            Ok(None)
        });
        inject!(print);

        // length
        let len: Rc<ExFn> = Rc::new(|_stdout, _stdin, args| {
            let [item] = args.as_slice() else {
                Err(ExternalFunctionError::InvalidArguments)?
            };
            match &**item {
                ASTNode::List(items) => Ok(Some(lit!(Token::Number(items.len() as f32)))),
                ASTNode::Literal(Token::Str(string)) => {
                    Ok(Some(lit!(Token::Number(string.len() as f32))))
                }
                _ => bail!("cannot take length of {:?}", &**item),
            }
        });
        inject!(len);

        // sort
        let sort: Rc<ExFn> = Rc::new(|_stdout, _stdin, args| {
            let [list] = args.as_slice() else {
                Err(ExternalFunctionError::InvalidArguments)?
            };
            match &**list {
                ASTNode::List(items) => {
                    let mut clone = items.clone();
                    clone.sort();
                    Ok(Some(ASTNode::List(clone).into()))
                }
                _ => bail!("cannot sort {:?}", &**list),
            }
        });
        inject!(sort);

        // split string by delimiter into list of strings
        let split: Rc<ExFn> = Rc::new(|_stdout, _stdin, args| {
            let [string, delimiter] = args.as_slice() else {
                Err(ExternalFunctionError::InvalidArguments)?
            };
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
        inject!(split);

        // chars (get characters of string as list)
        let chars: Rc<ExFn> = Rc::new(|_stdout, _stdin, args| {
            let [string] = args.as_slice() else {
                Err(ExternalFunctionError::InvalidArguments)?
            };
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
        inject!(chars);

        // assert (returns err if condition != true)
        let assert: Rc<ExFn> = Rc::new(|_stdout, _stdin, args| {
            let [condition] = args.as_slice() else {
                Err(ExternalFunctionError::InvalidArguments)?
            };
            match &**condition {
                ASTNode::Literal(Token::Bool(true)) => {}
                _ => return Err(anyhow!("assertion failed")),
            }
            Ok(None)
        });
        inject!(assert);

        Ok(())
    }
}
