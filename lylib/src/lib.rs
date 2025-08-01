pub mod interner;
pub mod interpreter;
pub mod lexer;
pub mod parser;

#[macro_use]
mod macros;

use crate::{
    interner::StringInterner,
    interpreter::Interpreter,
    lexer::Lexer,
    parser::{ASTNode, Parser},
};
pub use anyhow;
use anyhow::{Context, Result};
use std::{
    io::{Read, Write},
    sync::{Mutex, OnceLock},
};

/// Global interner. Used just about everywhere to access interned values and their respective
/// string counterparts.
static GLOBAL_INTERNER: OnceLock<Mutex<StringInterner>> = OnceLock::new();

pub fn get_global_interner() -> &'static Mutex<StringInterner> {
    GLOBAL_INTERNER.get_or_init(|| Mutex::new(StringInterner::new()))
}

/// Executes a file.
/// Internally, this lexes the provided buffer, parses them into an AST, and executes it. Returns
/// the interpreter used to execute the buffer.
pub fn execute<Out: Write, In: Read>(
    buffer: impl Into<String>,
    output: Out,
    input: In,
    include: Vec<String>,
) -> Result<Interpreter<Out, In>> {
    let mut lexer = Lexer::new();
    let tokens = lexer.lex(buffer.into()).context("failed to lex buffer")?;
    let mut parser = Parser::new(tokens);
    let ast = parser
        .parse_with_imports(
            include
                .iter()
                .map(|include_buffer| {
                    ASTNode::Module {
                        alias: None,
                        body: Parser::new(
                            Lexer::new()
                                .lex(include_buffer.to_owned().to_string())
                                .unwrap(),
                        )
                        .parse()
                        .unwrap()
                        .into(),
                    }
                    .into()
                })
                .collect(),
        )
        .context("failed to parse buffer")?;
    let mut interpreter = Interpreter::new(input, output);
    interpreter
        .execute(ast)
        .context("failed to execute buffer")?;
    Ok(interpreter)
}
