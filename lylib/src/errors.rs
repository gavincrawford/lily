//! Shared error types used throughout the library.

use crate::lexer::Token;
use thiserror::Error;

/// Encapsulates errors related to calling external functions.
#[derive(Error, Debug)]
pub(crate) enum ExternalFunctionError {
    #[error("invalid arguments provided to external function")]
    InvalidArguments,
}

/// Encapsulates memory failures.
#[derive(Error, Debug)]
pub(crate) enum MemoryError {
    #[error("no scope exists at this level ({0})")]
    NoScope(usize),
    #[error("no module exists: '{0}'")]
    NoModule(String),
    #[error("failed to access variable: '{0}'")]
    VariableRead(String),
    #[error("failed to assign to variable: '{0}'")]
    VariableWrite(String),
    #[error("index {0} out of bounds")]
    IndexOutOfBounds(usize),
    #[error("variables cannot contain modules")]
    ModuleInVar,
}

/// Encapsulates parser failures.
///
/// Each kind-specific variant wraps the underlying `anyhow::Error` as a source so the original
/// cause chain is preserved. The `Other` catchall provides a transparent passthrough for
/// failures that have no statement-level decoration to add, and powers the
/// `From<anyhow::Error>` impl so `?` can be used freely inside parser functions.
#[derive(Error, Debug)]
pub(crate) enum ParserError {
    // Generic error wrappers
    #[error("failed to parse import")]
    Import(#[source] anyhow::Error),
    #[error("failed to parse declaration")]
    Declaration(#[source] anyhow::Error),
    #[error("failed to parse conditional")]
    Conditional(#[source] anyhow::Error),
    #[error("failed to parse function declaration")]
    FunctionDecl(#[source] anyhow::Error),
    #[error("failed to parse structure declaration")]
    StructDecl(#[source] anyhow::Error),
    #[error("failed to parse while loop")]
    While(#[source] anyhow::Error),
    #[error("failed to parse return statement")]
    Return(#[source] anyhow::Error),
    #[error(transparent)]
    Other(anyhow::Error),

    // Expect-type errors
    /// Thrown when EOF is encountered in the middle of an ongoing expression.
    #[error("found unexpected EOF")]
    UnexpectedEOF,
    /// Errors resembling: "expected {0}, found {1}"
    /// The first field describes what was expected (e.g. a concrete token's `Debug` rendering,
    /// or a category like "identifier"); the second is the token that was actually found.
    #[error("expected {0}, found {1:?}")]
    Expected(String, Token),
}

impl From<anyhow::Error> for ParserError {
    fn from(err: anyhow::Error) -> Self {
        ParserError::Other(err)
    }
}
