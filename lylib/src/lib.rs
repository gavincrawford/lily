pub mod interner;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub use anyhow;

use crate::interner::StringInterner;
use std::sync::{Mutex, OnceLock};
/// Global interner. Used just about everywhere to access interned values and their respective
/// string counterparts.
static GLOBAL_INTERNER: OnceLock<Mutex<StringInterner>> = OnceLock::new();

pub fn get_global_interner() -> &'static Mutex<StringInterner> {
    GLOBAL_INTERNER.get_or_init(|| Mutex::new(StringInterner::new()))
}

#[macro_use]
pub(crate) mod macros;
