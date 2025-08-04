use clap::ArgMatches;
use lylib::{
    anyhow::{Context, Result},
    LyConfig,
};
use std::{
    fs,
    io::{stdin, stdout},
};

/// Executes a file.
pub fn execute(args: ArgMatches) -> Result<()> {
    //read file to buffer
    let file_path: &String = args.get_one("file").unwrap();
    let buf = fs::read_to_string(file_path).context("failed to open file")?;

    // create lily config & execute file
    let mut cfg = LyConfig::new();
    if !*args.get_one("nostd").unwrap_or(&false) {
        // NOTE: All imports of this style are added to base scope, and don't require importing to
        // be used. I want to figure out a solution where imports are only used when needed, and
        // aren't automatically globbed into memory when they aren't needed.
        cfg.include(include_str!("./std/math.ly").to_string());
    }
    let interp = cfg
        .debug_parser(*args.get_one("debugparser").unwrap_or(&false))
        .debug_lexer(*args.get_one("debuglexer").unwrap_or(&false))
        .execute(buf, stdout(), stdin())?;

    // for debugging
    #[cfg(debug_assertions)]
    println!("{}", interp.memory.borrow());
    Ok(())
}
