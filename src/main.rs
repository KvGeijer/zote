use clap::Parser;
use errors::ErrorReporter;
use interpreter::InterpreterState;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::process::exit;

mod code_loc;
mod errors;
mod interpreter;
mod parser;
mod scanner;

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

// Maybe move these out to separate file for running and keeping state
fn run_file(file: &str) {
    let script = fs::read_to_string(file).expect("Could not open file.");
    let mut error_reporter = errors::ErrorReporter::new();
    let mut state = InterpreterState::new();

    run(&script, &mut error_reporter, &mut state);

    if error_reporter.had_compilation_error {
        exit(65);
    } else if error_reporter.had_runtime_error {
        exit(70);
    }
}

fn run_repl() {
    let reader = stdin();
    let mut line = String::new();
    let mut error_reporter = errors::ErrorReporter::new();
    let mut state = InterpreterState::new();

    while {
        print!("> ");
        stdout().flush().unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap_or(0) > 0
    } {
        // Does not preserve program state between calls
        run(&line, &mut error_reporter, &mut state);
        error_reporter.reset();
    }
}

fn run(code: &str, error_reporter: &mut ErrorReporter, state: &mut InterpreterState) {
    let tokens = scanner::tokenize(code, error_reporter);

    if let Some(stmts) = parser::parse(&tokens, error_reporter) {
        interpreter::interpret(&stmts, error_reporter, state);
    }
}
