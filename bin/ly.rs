use lily::{interpreter::*, lexer::*, parser::*};
use std::{env, fs, path::PathBuf, process, rc::Rc};

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

    // get pwd
    let mut path = PathBuf::from(file_path);
    path.pop();

    // lex buffer into tokens
    let tokens = match Lexer::new().lex(buf) {
        Ok(toks) => toks,
        Err(e) => {
            eprintln!("LEX ERR: {:?}\n", e);
            panic!();
        }
    };

    // parse tokens into ast
    let mut parser = Parser::new(tokens);
    parser.set_pwd(path);
    let ast = match parser.parse_with_imports(stdlib()) {
        Ok(tree) => tree,
        Err(e) => {
            eprintln!("PARSE ERR: {:?}\n", e);
            panic!();
        }
    };

    // execute interpreter
    let mut interp = Interpreter::new();
    match interp.execute(ast) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("EXEC ERR: {:?}\n", e);
            panic!();
        }
    }

    // for debugging
    #[cfg(debug_assertions)]
    dbg!(interp.memory.borrow().inner());
}

/// Creates STD module import.
fn stdlib() -> Vec<Rc<ASTNode>> {
    // TODO more informative error messages here, no unwraps
    let mut lexer = Lexer::new();
    vec![ASTNode::Module {
        alias: Some("math".into()),
        body: Parser::new(lexer.lex(include_str!("./std/math.ly").into()).unwrap())
            .parse()
            .unwrap(),
    }
    .into()]
}
