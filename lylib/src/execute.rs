//! Implements the outward-facing functions for executing a file with a given set of configuration
//! options. This allows the end user to customize the behavior of the interpreter.

use crate::{interpreter::*, lexer::*, parser::*};
use anyhow::{Context, Result};
use std::{
    io::{Read, Write},
    rc::Rc,
};

/// Lily configuration.
/// Use this struct to configure & run the interpreter.
/// # Example
/// ```
/// use lylib::LyConfig;
/// use std::io::{stdin, stdout};
/// # fn main() {
/// let cfg = LyConfig::new()
///     .debug_parser(false) // disable/enable AST debugger
///     .debug_lexer(false) // disable/enable lexer debugger
///     .include_as("math", "func add a b do return a + b end;") // add imported code
///     .execute("let result = math.add(1, 2);", stdout(), stdin()); // run buffer with stdin/out
/// # }
/// ```
/// This example adds an included file under module name "math", which you can see is available
/// when the provided buffer runs. This is useful for integrating Lily into existing applications.
pub struct LyConfig {
    /// Files to include during parsing, if applicable.
    /// Each value must be a tuple in which the values coorespond to `(module alias, module source)`.
    include: Vec<(Option<usize>, String)>,
    /// If true, debug lexer output.
    dbg_tokens: bool,
    /// If true, debug parser output.
    dbg_ast: bool,
}

impl LyConfig {
    /// Creates a new default config.
    pub fn new() -> Self {
        Self {
            include: vec![],
            dbg_ast: false,
            dbg_tokens: false,
        }
    }

    /// Adds a file to be included at base scope.
    pub fn include(&mut self, buffer: impl Into<String>) -> &mut Self {
        let buffer = buffer.into();
        self.include.push((None, buffer));
        self
    }

    /// Adds a file as a module under a provided alias.
    pub fn include_as(&mut self, alias: impl Into<String>, buffer: impl Into<String>) -> &mut Self {
        let (alias, buffer) = (alias.into(), buffer.into());
        self.include.push((Some(intern!(alias)), buffer));
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
        let includes = self
            .include
            .iter()
            .map(|(alias, source)| {
                ASTNode::Module {
                    alias: *alias,
                    body: Parser::new(Lexer::new().lex(source.clone().to_string()).unwrap())
                        .parse()
                        .unwrap(),
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
