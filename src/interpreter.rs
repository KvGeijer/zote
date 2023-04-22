use std::rc::Rc;

use crate::{code_loc::CodeLoc, errors::ErrorReporter, parser::Stmts};

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

/// Top level interpret function
pub fn interpret(program: &Stmts, error_reporter: &mut ErrorReporter, env: &mut InterpreterState) {
    let mut output = Value::Uninitialized;
    for stmt in program.stmts.iter() {
        match statements::eval(stmt, &env.env) {
            Ok(Some(val)) => output = val,
            Ok(None) => continue,
            Err(RuntimeError::Error(start, end, reason)) => {
                return error_reporter.runtime_error(&start, &end, &reason)
            }
            Err(RuntimeError::Break) => {
                error_reporter.runtime_panic("Break propagated to top-level scope")
            }
            Err(RuntimeError::Return(v)) => error_reporter.runtime_panic(&format!(
                "Return {} propagated to top-level scope",
                v.stringify()
            )),
        }
    }

    if program.output {
        println!("{}", output.stringify());
    }
}

type RunRes<T> = Result<T, RuntimeError>;
#[derive(Debug)]
enum RuntimeError {
    Error(CodeLoc, CodeLoc, String),
    Break, // Maybe include code loc for error messages? Or just handle that with static analysis?
    Return(Value),
}
