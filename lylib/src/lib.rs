//!  # Lily
//!  ## Running a Buffer
//!  To run a buffer, use the configuration struct:
/// ```
/// use lylib::LyConfig;
/// use std::io::{stdin, stdout};
/// # fn main() {
/// let cfg = LyConfig::new()
///     .execute("print(\"hello world!\")", stdout(), stdin());
/// # }
/// ```

// Export macros crate-wide
#[macro_use]
mod macros;

// Re-export anyhow
pub use anyhow;

// Expose config factory
mod execute;
pub use execute::LyConfig;

// Internals
pub mod interner;
pub mod interpreter;
pub mod lexer;
pub mod parser;

use crate::interner::StringInterner;
use anyhow::Result;
use std::sync::{Mutex, MutexGuard, OnceLock};

/// Global interner. Used just about everywhere to access interned values and their respective
/// string counterparts.
static GLOBAL_INTERNER: OnceLock<Mutex<StringInterner>> = OnceLock::new();

/// Fetches a lock of the global interner. If a lock cannot be acquired, this function will return
/// `Err`. Only one lock should be active at any given time.
fn get_global_interner() -> Result<MutexGuard<'static, StringInterner>> {
    if let Ok(mutex_guard) = GLOBAL_INTERNER
        .get_or_init(|| Mutex::new(StringInterner::new()))
        .lock()
    {
        Ok(mutex_guard)
    } else {
        Err(anyhow::anyhow!(
            "failed to lock interner due to conflicting usage"
        ))
    }
}
