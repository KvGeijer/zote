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
mod value;

use runtime_error::{RunError, RunRes};

use value::Value;

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
        Err(RunError::Error(trace)) => error_reporter.runtime_error(&format!("{trace}")),
        Err(RunError::Break) => error_reporter.runtime_panic("Break propagated to top-level scope"),
        // Should we allow this and just print nicely?
        Err(RunError::Return(v)) => error_reporter.runtime_panic(&format!(
            "Return {} propagated to top-level scope",
            v.stringify()
        )),
    }
}
