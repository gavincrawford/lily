use super::*;

macro_rules! exfn {
    (|$($arg:ident),*| $body:expr) => {
        Variable::Extern(Rc::new(|args| {
            let [$($arg),*] = args.as_slice() else { bail!("invalid arguments provided to external function"); };
            $body
        }))
    };
}
impl Interpreter {
    pub fn inject_builtins(&mut self) -> Result<()> {
        self.declare(
            &ID::from("print"),
            exfn!(|value| {
                match &**value {
                    ASTNode::Literal(Token::Str(s)) => println!("{}", s),
                    ASTNode::Literal(Token::Number(n)) => println!("{}", n),
                    ASTNode::Literal(Token::Char(c)) => println!("{}", c),
                    ASTNode::Literal(Token::Bool(b)) => println!("{}", b),
                    other => println!("{:?}", other),
                }
                Ok(None)
            }),
        )?;
        Ok(())
    }
}
