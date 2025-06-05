use super::*;

macro_rules! exfn {
    (|$($arg:ident),*;$in:ident, $out:ident| $body:expr) => {
        Variable::Extern(Rc::new(|$in, $out, args| {
            let [$($arg),*] = args.as_slice() else { bail!("invalid arguments provided to external function"); };
            $body
        }))
    };
}

impl<Out: Write, In: Read> Interpreter<Out, In> {
    pub fn inject_builtins(&mut self) -> Result<()> {
        self.declare(
            &ID::from("print"),
            exfn!(|value; stdout, _stdin| {
                let mut out = stdout.borrow_mut();
                match &**value {
                    ASTNode::Literal(Token::Str(s)) => write!(out, "{}\n", s),
                    ASTNode::Literal(Token::Number(n)) => write!(out, "{}\n", n),
                    ASTNode::Literal(Token::Char(c)) => write!(out, "{}\n", c),
                    ASTNode::Literal(Token::Bool(b)) => write!(out, "{}\n", b),
                    other => write!(out, "{:?}\n", other),
                }?;
                Ok(None)
            }),
        )?;
        Ok(())
    }
}
