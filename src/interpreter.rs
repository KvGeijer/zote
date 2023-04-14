use crate::{code_loc::CodeLoc, errors::ErrorReporter, parser::StmtNode};

use environment::Environment;

mod environment;
mod expressions;
mod statements;

pub struct InterpreterState {
    env: Environment<'static>,
}

impl InterpreterState {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
        }
    }
}

pub fn interpret(
    program: &Vec<StmtNode>,
    error_reporter: &mut ErrorReporter,
    env: &mut InterpreterState,
) {
    for stmt in program {
        match statements::eval(stmt, &mut env.env) {
            Ok(_) => continue,
            Err(RuntimeError::Error(start, end, reason)) => {
                return error_reporter.runtime_error(&start, &end, &reason)
            }
            Err(RuntimeError::Break) => {
                error_reporter.runtime_panic("Break propagated to top-level scope")
            }
        }
    }
}

enum RuntimeError {
    Error(CodeLoc, CodeLoc, String),
    Break,
}
