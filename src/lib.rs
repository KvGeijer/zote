#![feature(box_patterns, iterator_try_reduce, let_chains)]

use std::fs;
use std::io::{stdin, stdout, Write};

pub use interpreter::{run_file, run_repl, run_str};

#[cfg(not(feature = "ast-interpreter"))]
pub mod interpreter {
    use super::*;

    /// Interprets a text file as a Zote script, returning the exit code.
    pub fn run_file(file: &str) -> i32 {
        // TODO: Real error
        let script = fs::read_to_string(file).expect("Could not open file.");
        run_str(&script)
    }

    /// Interprets the string as if from a file.
    pub fn run_str(code: &str) -> i32 {
        if let Some(stmts) = parser::parse(code) {
            vm::interpret_once(&stmts)
        } else {
            65
        }
    }

    /// Starts the Zote repl.
    ///
    /// Each line must be a statement or an expression (in which case its value is printed),
    /// and the state of the program is preserved between lines.
    pub fn run_repl() {
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
}

#[cfg(feature = "ast-interpreter")]
pub mod interpreter {
    use super::*;
    use ast_interpreter::InterpreterState;

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
        let mut state = InterpreterState::new();

        while {
            print!("> ");
            stdout().flush().unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap_or(0) > 0
        } {
            // Does not preserve program state between calls
            run(&line, &mut state);
            state.reset_errors();
        }
    }

    /// Interprets the string as if from a file.
    pub fn run_str(code: &str) -> i32 {
        let mut state = InterpreterState::new();

        run(code, &mut state)
    }

    fn run(code: &str, state: &mut InterpreterState) -> i32 {
        if let Some(stmts) = parser::parse(code) {
            ast_interpreter::interpret(&stmts, state);
            if state.had_error() {
                70
            } else {
                0
            }
        } else {
            65
        }
    }
}
