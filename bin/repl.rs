use lily::{interpreter::*, lexer::*, parser::*};
use std::io::{stdin, BufRead};

fn main() {
    let mut interpreter = Interpreter::new();
    loop {
        interpret_line(&mut interpreter);
        dbg!(&interpreter.variables);
    }
}

fn interpret_line(interpreter: &mut Interpreter) {
    let mut buf = String::new();
    stdin().lock().read_line(&mut buf).unwrap();
    let ast = Parser::new(Lexer::new().lex(buf)).parse();
    interpreter.execute(ast);
}
