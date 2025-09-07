mod execute;
use clap::{arg, command, Parser};
use execute::execute;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// Target file.
    buffer: String,

    /// Run without adding standard library modules.
    #[arg(long)]
    no_std: bool,

    /// Debug parser output during execution.
    #[arg(long)]
    debug_parser: bool,

    /// Debug lexer output during execution.
    #[arg(long)]
    debug_lexer: bool,
}

fn main() {
    // parse arguments
    let cmd = Args::parse();

    // execute file
    if let Err(e) = execute(cmd) {
        eprintln!("{e:?}");
    }
}
