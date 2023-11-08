use clap::Parser;
use std::{
    fs,
    io::{stdin, stdout, Write},
    process::exit,
};

#[derive(Parser)]
struct Args {
    /// The script to run
    file: Option<String>,
}

fn main() {
    let args = Args::parse();

    if let Some(ref file) = args.file {
        exit(run_file(file));
    } else {
        run_repl();
    }
}

/// Interprets a text file as a Zote script, returning the exit code.
fn run_file(file: &str) -> i32 {
    // TODO: Real error
    let script = fs::read_to_string(file).expect("Could not open file.");
    run_str(&script)
}

/// Interprets the string as if from a file.
fn run_str(code: &str) -> i32 {
    if let Some(stmts) = parser::parse(code) {
        let ast = semantic_analyzer::analyze_ast(&stmts);
        vm::interpret_once(&ast)
    } else {
        65
    }
}

/// Starts the Zote repl.
///
/// Each line must be a statement or an expression (in which case its value is printed),
/// and the state of the program is preserved between lines.
fn run_repl() {
    let reader = stdin();
    let mut line = String::new();
    // TODO: Save state between repl calls

    while {
        print!("> ");
        stdout().flush().unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap_or(0) > 0
    } {
        // Does not preserve program state between calls
        run_str(&line);
    }
}
