use crate::Args;
use lylib::{
    anyhow::{Context, Result},
    LyConfig,
};
use std::{
    fs,
    io::{stdin, stdout},
};

/// Executes a file.
pub fn execute(args: Args) -> Result<()> {
    //read file to buffer
    let buf = fs::read_to_string(args.buffer).context("failed to open file")?;

    // create lily config & execute file
    let mut cfg = LyConfig::new();
    if !args.no_std {
        cfg.include_as("math", include_str!("./std/math.ly").to_string());
        cfg.include_as("complex", include_str!("./std/complex.ly").to_string());
    }
    let interp = cfg
        .debug_parser(args.debug_parser)
        .debug_lexer(args.debug_lexer)
        .execute(buf, stdout(), stdin())?;

    // for debugging
    #[cfg(debug_assertions)]
    println!("{}", interp.memory.borrow());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math() {
        let res = execute(Args {
            buffer: "./src/std/test/math.test.ly".into(),
            no_std: false,
            debug_parser: false,
            debug_lexer: false,
        });
        assert!(res.is_ok());
    }

    #[test]
    fn complex() {
        let res = execute(Args {
            buffer: "./src/std/test/complex.test.ly".into(),
            no_std: false,
            debug_parser: false,
            debug_lexer: false,
        });
        assert!(res.is_ok());
    }
}
