#![feature(box_patterns, iterator_try_reduce, let_chains)]

use ast_interpreter::InterpreterState;
use errors::ErrorReporter;
use std::fs;
use std::io::{stdin, stdout, Write};

mod ast_interpreter;
mod code_loc;
mod errors;
mod parser;
mod scanner;

/// Interprets a text file as a Zote script, returning the exit code.
pub fn run_file(file: &str) -> i32 {
    let script = fs::read_to_string(file).expect("Could not open file.");
    run_str(&script)
}

/// Starts the Zote repl.
///
/// Each line must be a statement or an expression (in which case its value is printed),
/// and the state of the program is preserved between lines.
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

/// Interprets the string as if from a file.
pub fn run_str(code: &str) -> i32 {
    let mut error_reporter = errors::ErrorReporter::new();
    let mut state = InterpreterState::new();

    run(code, &mut error_reporter, &mut state);

    if error_reporter.had_compilation_error {
        65
    } else if error_reporter.had_runtime_error {
        70
    } else {
        0
    }
}

fn run(code: &str, error_reporter: &mut ErrorReporter, state: &mut InterpreterState) {
    let tokens = scanner::tokenize(code, error_reporter);

    if !error_reporter.had_compilation_error && let Some(stmts) = parser::parse(&tokens, error_reporter) {
        // Should we look at error_reporter instead? Probably way better
        ast_interpreter::interpret(&stmts, error_reporter, state);
    }
}
