use lily::{interpreter::*, lexer::*, parser::*};
use std::{env, fs, process};

fn main() {
    // read file to buffer
    let args: Vec<String> = env::args().collect();
    let file_path = &args.get(1).expect("no file provided.");
    let buf = match fs::read_to_string(file_path) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("Error reading file {}: {}", file_path, err);
            process::exit(1);
        }
    };

    // execute file
    let toks = Lexer::new().lex(buf);
    let ast = Parser::new(toks).parse();
    let mut interp = Interpreter::new();
    interp.execute(&ast);

    // TODO for debugging
    #[cfg(debug_assertions)]
    dbg!(interp.variables);
}
