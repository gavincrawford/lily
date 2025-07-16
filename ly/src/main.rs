mod execute;
use clap::{arg, Command};
use execute::execute;

fn main() {
    // specify and parse arguments
    let cmd = Command::new("ly")
        .arg(arg!(<file>))
        .arg(arg!(--nostd "run without standard library"))
        .arg(arg!(--debugast "(debug) prints the AST during execution"))
        .get_matches();

    //execute file
    match execute(cmd) {
        Err(e) => {
            eprintln!("{:?}", e);
        }
        _ => {}
    }
}
