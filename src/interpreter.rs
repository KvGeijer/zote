use std::rc::Rc;

use crate::{errors::ErrorReporter, parser::Stmts};

use environment::Environment;

mod collections;
mod environment;
mod expressions;
mod functions;
mod numerical;
mod runtime_error;
mod statements;

use expressions::Value;
use runtime_error::{RunRes, RuntimeError};

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

/// Top level interpret function
pub fn interpret(program: &Stmts, error_reporter: &mut ErrorReporter, env: &mut InterpreterState) {
    match statements::eval_statements(program, &env.env) {
        Ok(Some(Value::Nil)) => (), // Might want to print this sometimes, but mostly I assume it is not intended
        Ok(Some(value)) => println!("{}", value.stringify()),
        Ok(None) => (),
        Err(RuntimeError::Error(start, end, reason)) => {
            error_reporter.runtime_error(&start, &end, &reason)
        }
        Err(RuntimeError::ErrorReason(reason)) => {
            error_reporter.runtime_panic(&format!("Error {reason} propagated to top"))
        }
        Err(RuntimeError::Break) => {
            error_reporter.runtime_panic("Break propagated to top-level scope")
        }
        // Should we allow this and just print nicely?
        Err(RuntimeError::Return(v)) => error_reporter.runtime_panic(&format!(
            "Return {} propagated to top-level scope",
            v.stringify()
        )),
    }
}
