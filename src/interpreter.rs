use crate::{code_loc::CodeLoc, errors::ErrorReporter, parser::StmtNode};

use environment::Environment;

mod environment;
mod expressions;
mod statements;

type RuntimeError = (CodeLoc, CodeLoc, String);

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
            Err((start, end, reason)) => {
                return error_reporter.runtime_error(&start, &end, &reason)
            }
        }
    }
}
