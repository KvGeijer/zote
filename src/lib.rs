#![feature(box_patterns, let_chains)]

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

pub fn run_file(file: &str) {
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

pub fn run_repl() {
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
        // Should we look at error_reporter instead? Probably way better
        interpreter::interpret(&stmts, error_reporter, state);
    }
}