//! # Lily
//! To start using Lily, execute a buffer using the `LyConfig` helper!
//! The config helper allows you to configure language behavior.
//! ```
//! use lylib::LyConfig;
//! use std::io::{stdin, stdout};
//! # fn main() {
//! let cfg = LyConfig::default()
//!     .execute("print(\"hello world!\")", stdout(), stdin());
//! # }
//! ```

// Export macros crate-wide
#[macro_use]
mod macros;

#[cfg(not(target_env = "msvc"))]
use mimalloc::MiMalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Re-export anyhow
pub use anyhow;

// Expose config factory
mod execute;
pub use execute::LyConfig;

// Internals
mod errors;
pub mod interner;
pub mod interpreter;
pub mod lexer;
pub mod parser;

use crate::interner::StringInterner;
use std::cell::RefCell;

thread_local! {
    /// Global interner. Used just about everywhere to access interned values and their respective
    /// string counterparts. Thread-local since the lexer, parser, and interpreter for a given
    /// program all run on the same thread — this avoids the per-access atomic of a `Mutex` on the
    /// single-threaded hot path.
    static GLOBAL_INTERNER: RefCell<StringInterner> = RefCell::new(StringInterner::new());
}

/// Runs a closure with mutable access to the global string interner.
///
/// The global interner is used throughout the library to deduplicate strings
/// and provide fast identifier lookups using integer indices.
fn with_global_interner<R>(f: impl FnOnce(&mut StringInterner) -> R) -> R {
    GLOBAL_INTERNER.with(|cell| f(&mut cell.borrow_mut()))
}
