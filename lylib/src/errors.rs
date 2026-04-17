//! Shared error types used throughout the library.

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
