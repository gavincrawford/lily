//! Implements the outward-facing functions for executing a file with a given set of configuration
//! options. This allows the end user to customize the behavior of the interpreter.

use crate::{interpreter::*, lexer::*, parser::*};
use anyhow::{Context, Result};
use std::{
    io::{Read, Write},
    rc::Rc,
};

/// Lily configuration.
pub struct LyConfig {
    /// Files to include during parsing, if applicable.
    include: Box<Option<Vec<String>>>,
    /// If true, debug lexer output.
    dbg_tokens: bool,
    /// If true, debug parser output.
    dbg_ast: bool,
}

impl LyConfig {
    /// Creates a new default config.
    pub fn new() -> Self {
        Self {
            include: None.into(),
            dbg_ast: false,
            dbg_tokens: false,
        }
    }

    /// Adds a file to be included at base scope.
    pub fn include(&mut self, buffer: String) -> &mut Self {
        if let Some(incldues) = &mut *self.include {
            incldues.push(buffer);
        } else {
            self.include = Some(vec![buffer]).into();
        }
        self
    }

    /// Toggles debug mode on parser output.
    pub fn debug_parser(&mut self, debug: bool) -> &mut Self {
        self.dbg_ast = debug;
        self
    }

    /// Toggles debug mode on lexer output.
    pub fn debug_lexer(&mut self, debug: bool) -> &mut Self {
        self.dbg_tokens = debug;
        self
    }

    /// Executes the provided file with the given context that is represented within this
    /// configuration. All config items should be set before this function is used.
    pub fn execute<Out: Write, In: Read>(
        &self,
        buffer: impl Into<String>,
        output: Out,
        input: In,
    ) -> Result<Interpreter<Out, In>> {
        // Interpret file
        let mut lexer = Lexer::new();
        let tokens = lexer.lex(buffer.into()).context("failed to lex buffer")?;

        // Debug lexer, if applicable
        if self.dbg_tokens {
            println!("[TOKENS]\n{tokens:#?}");
        }

        // Parse includes before main file
        // TODO: Ugh I don't want to clone here. Damn you borrow checker
        let includes = (&*self.include.clone().unwrap_or(vec![]))
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
            .collect::<Vec<Rc<ASTNode>>>();

        // Parse file
        let mut parser = Parser::new(tokens);
        let ast = parser
            .parse_with_imports(includes)
            .context("failed to parse buffer")?;

        // Debug parser, if applicable
        if self.dbg_ast {
            println!("[AST]\n{ast:#?}");
        }

        // Interpret AST
        let mut interpreter = Interpreter::new(input, output);
        interpreter
            .execute(ast)
            .context("failed to execute buffer")?;
        Ok(interpreter)
    }
}
