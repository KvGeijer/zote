use std::rc::Rc;

use crate::{code_loc::CodeLoc, errors::ErrorReporter, parser::StmtNode};

use environment::Environment;

mod environment;
mod expressions;
mod functions;
mod statements;

use expressions::Value;

pub struct InterpreterState {
    env: Rc<Environment>,
}

impl InterpreterState {
    pub fn new() -> Self {
        let base = Environment::new();
        functions::define_builtins(&base);
        Self {
            env: Environment::nest(&base),
        }
    }
}

pub fn interpret(
    program: &Vec<StmtNode>,
    error_reporter: &mut ErrorReporter,
    env: &mut InterpreterState,
) {
    for stmt in program {
        match statements::eval(stmt, &env.env) {
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

type RunRes<T> = Result<T, RuntimeError>;
enum RuntimeError {
    Error(CodeLoc, CodeLoc, String),
    Break,
}
