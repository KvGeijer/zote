#![feature(box_patterns, iterator_try_reduce)]

use crate::errors::ErrorReporter;
use environment::Environment;
use parser::Stmts;
use runtime_error::{RunError, RunRes};
use value::Value;

use std::rc::Rc;

mod collections;
mod environment;
mod errors;
mod expressions;
mod functions;
mod numerical;
mod runtime_error;
mod statements;
mod value;

pub struct InterpreterState {
    env: Rc<Environment>,
    error_reporter: ErrorReporter,
}

impl InterpreterState {
    pub fn new() -> Self {
        let base = Environment::new();
        functions::define_builtins(&base);
        Self {
            env: Environment::nest(&base),
            error_reporter: ErrorReporter::new(),
        }
    }

    pub fn reset_errors(&mut self) {
        self.error_reporter.reset()
    }

    pub fn had_error(&self) -> bool {
        self.error_reporter.had_error
    }
}

/// Top level interpret function
pub fn interpret(program: &Stmts, env: &mut InterpreterState) {
    let InterpreterState {
        env,
        error_reporter,
    } = env;

    match statements::eval_statements(program, &env) {
        Ok(Some(Value::Nil)) => (), // Might want to print this sometimes, but mostly I assume it is not intended
        Ok(Some(value)) => println!("{}", value.stringify()),
        Ok(None) => (),
        Err(RunError::Error(trace)) => error_reporter.runtime_error(&format!("{trace}")),
        Err(RunError::Break) => error_reporter.runtime_panic("Break propagated to top-level scope"),
        Err(RunError::Continue) => {
            error_reporter.runtime_panic("Continue propagated to top-level scope")
        }
        // Just prints and terminates
        Err(RunError::Return(v)) => match v {
            Value::Nil => (),
            v => println!("{}", v.stringify()),
        },
    }
}
