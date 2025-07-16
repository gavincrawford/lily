use clap::ArgMatches;
use lylib::{
    anyhow::{Context, Result},
    interpreter::*,
    lexer::*,
    parser::*,
};
use std::{
    fs,
    io::{stdin, stdout},
    rc::Rc,
};

/// Executes a file.
pub fn execute(args: ArgMatches) -> Result<()> {
    //read file to buffer
    let file_path: &String = args.get_one("file").unwrap();
    let buf = fs::read_to_string(file_path).context("failed to open file")?;

    // lex buffer into tokens
    let tokens = Lexer::new().lex(buf).context("failed to lex file")?;

    // parse tokens into ast
    let mut parser = Parser::new(tokens);
    let ast = parser
        .parse_with_imports(match args.get_one("nostd").unwrap() {
            true => vec![],
            false => stdlib().context("failed to resolve standard library")?,
        })
        .context("failed to parse file")?;
    if *args.get_one("debugast").unwrap() {
        println!("[BUILT AST]\n{:#?}\n", ast);
    }

    // execute interpreter
    let mut interp = Interpreter::new(stdin(), stdout());
    interp.execute(ast).context("failed to execute file")?;

    // for debugging
    #[cfg(debug_assertions)]
    println!("{}", interp.memory.borrow());
    Ok(())
}

/// Creates STD module import.
fn stdlib() -> Result<Vec<Rc<ASTNode>>> {
    let mut lexer = Lexer::new();
    Ok(vec![ASTNode::Module {
        alias: Some("math".into()),
        body: Parser::new(lexer.lex(include_str!("./std/math.ly").into()).unwrap()).parse()?,
    }
    .into()])
}
