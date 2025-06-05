use super::*;
use crate::lit;

impl<Out: Write, In: Read> Interpreter<Out, In> {
    pub fn inject_builtins(&mut self) -> Result<()> {
        /// Adds an external function.
        macro_rules! exfn {
            ($id:tt, |$($arg:ident),*;$out:ident, $in:ident| $body:expr) => {
                self.declare(
                    &ID::from(stringify!($id)),
                    Variable::Extern(Rc::new(|stdout, stdin, args| {
                        let mut $out = stdout.borrow_mut();
                        let mut $in = stdin.borrow_mut();
                        let [$($arg),*] = args.as_slice() else { bail!("invalid arguments provided to external function"); };
                        $body
                    }))
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
                    let mut handle = items.borrow_mut();
                    let length = match handle.get_scope(0) {
                        Some(scope) => scope.len(),
                        None => 0,
                    };
                    Ok(Some(lit!(Token::Number(length as f32))))
                }
                ASTNode::Literal(Token::Str(string)) => {
                    Ok(Some(lit!(Token::Number(string.len() as f32))))
                }
                _ => bail!("cannot take length of {:?}", &**item)
            }
        });

        Ok(())
    }
}
