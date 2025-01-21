use lily::{interpreter::*, lexer::*, parser::*};
use std::{env, fs, path::PathBuf, process};

fn main() {
    // enable full backtraces
    env::set_var("RUST_BACKTRACE", "1");

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

    // get pwd
    let mut path = PathBuf::from(file_path);
    path.pop();

    // execute file
    let mut parser = Parser::new(Lexer::new().lex(buf));
    parser.set_pwd(path);
    let ast = parser.parse();
    let mut interp = Interpreter::new();
    interp.execute(&ast);

    // TODO for debugging
    #[cfg(debug_assertions)]
    dbg!(interp.variables);
}
