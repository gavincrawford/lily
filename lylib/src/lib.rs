pub mod interner;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub use anyhow;

// global interner for temporary compatibility during transition
// TODO: remove this once interner is properly threaded through the system
use crate::interner::StringInterner;
use std::sync::{Mutex, OnceLock};
static GLOBAL_INTERNER: OnceLock<Mutex<StringInterner>> = OnceLock::new();

pub fn get_global_interner() -> &'static Mutex<StringInterner> {
    GLOBAL_INTERNER.get_or_init(|| Mutex::new(StringInterner::new()))
}

#[macro_use]
pub(crate) mod macros;
