use clap::Parser;

use zote::{run_file, run_repl};

#[derive(Parser)]
struct Args {
    /// The script to run
    file: Option<String>,
}

fn main() {
    let args = Args::parse();

    if let Some(ref file) = args.file {
        run_file(file);
    } else {
        run_repl();
    }
}
