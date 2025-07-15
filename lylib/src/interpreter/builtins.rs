use super::{mem::variable::ExFn, *};
use crate::lit;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    /// Adds an arbitrary external function to this interpreter.
    pub fn inject_extern(&mut self, id: impl Into<String>, closure: Rc<ExFn>) -> Result<()> {
        let id = id.into();
        self.declare(&ID::from(id.as_str()), Variable::Extern(closure))
    }

    // `Interpreter::new`, and we don't want anyone using it twice.
    /// Adds the default external functions to this interpreter.
    pub(crate) fn inject_builtins(&mut self) -> Result<()> {
        /// Adds an external function.
        macro_rules! exfn {
            ($id:tt, |$($arg:ident),*;$out:ident, $in:ident| $body:expr) => {
                self.inject_extern(
                    stringify!($id),
                    Rc::new(|stdout, stdin, args| {
                        let mut $out = stdout.borrow_mut();
                        let mut $in = stdin.borrow_mut();
                        let [$($arg),*] = args.as_slice() else { bail!("invalid arguments provided to external function"); };
                        $body
                    })
                )?
            };
        }

        // print
        exfn!(print, |value; stdout, _stdin| {
            match &**value {
                ASTNode::Literal(Token::Str(s)) => write!(stdout, "{}\n", s),
                ASTNode::Literal(Token::Number(n)) => write!(stdout, "{}\n", n),
                ASTNode::Literal(Token::Char(c)) => write!(stdout, "{}\n", c),
                ASTNode::Literal(Token::Bool(b)) => write!(stdout, "{}\n", b),
                other => write!(stdout, "{:?}\n", other),
            }?;
            Ok(None)
        });

        // length
        exfn!(len, |item; _stdout, _stdin| {
            match &**item {
                ASTNode::List(items) => {
                    Ok(Some(lit!(Token::Number(items.len() as f32))))
                }
                ASTNode::Literal(Token::Str(string)) => {
                    Ok(Some(lit!(Token::Number(string.len() as f32))))
                }
                _ => bail!("cannot take length of {:?}", &**item)
            }
        });

        // chars (get characters of string as list)
        exfn!(chars, |string; _stdout, _stdin| {
            match &**string {
                ASTNode::Literal(Token::Str(v)) =>
                {
                    // collect chars into a vector of nodes
                    let values: Vec<Rc<RefCell<Variable>>> = v.chars().map(|ch| {
                        Variable::Owned(ASTNode::Literal(Token::Char(ch.clone()))).into()
                    }).collect();

                    // return new list
                    Ok(Some(ASTNode::List(values).into()))
                },
                _ => bail!("cannot fetch characters of {:?}", &**string)
            }
        });

        Ok(())
    }
}
