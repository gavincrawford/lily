use crate::{interpreter::*, lexer::*, parser::*};
use anyhow::{Context, Result};
use std::io::{Read, Write};

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
