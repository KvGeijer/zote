use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    rc::Rc,
};

use crate::parser::{ExprNode, LValue};

use super::{environment::Environment, expressions, runtime_error::RunError, RunRes, Value};

use builtins::Builtin;

mod builtins;

#[derive(Clone)]
pub enum Function {
    Closure(Closure),
    Builtin(Rc<dyn Builtin>),
}

impl Function {
    pub fn call(&self, args: Vec<Value>) -> RunRes<Value> {
        if args.len() == self.arity() {
            match self.delegate_call(args) {
                Err(RunError::Break) => {
                    RunError::error("Break encountered outside loop".to_string())
                }
                Err(RunError::Return(value)) => Ok(value),
                otherwise => otherwise,
            }
        } else {
            RunError::error(format!(
                "Expected {} arguments but got {}.",
                self.arity(),
                args.len()
            ))
        }
    }

    fn delegate_call(&self, args: Vec<Value>) -> RunRes<Value> {
        match self {
            Function::Closure(closure) => closure.call(args),
            Function::Builtin(builtin) => builtin.run(args),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Function::Closure(closure) => closure.arity(),
            Function::Builtin(builtin) => builtin.arity(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Function::Closure(closure) => closure.name(),
            Function::Builtin(builtin) => builtin.name(),
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Function::Closure(closure) => {
                f.write_str(&format!("Closure({}/{})", closure.name(), closure.arity()))
            }
            Function::Builtin(builtin) => {
                f.write_str(&format!("Builtin({}/{})", builtin.name(), builtin.arity()))
            }
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // (Function::Closure(_, _, _), Function::Closure(_, _, _)) => true, // Uncomment this line when you add Closure variant back
            (Function::Builtin(a), Function::Builtin(b)) => a.name() == b.name(),
            _ => false,
        }
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Arbitraty ordering
        println!("WARNING: Comparing function handles");
        Some(self.name().cmp(other.name()))
    }
}

#[derive(Clone)]
pub struct Closure {
    id: String, // Maybe not ideal to have
    params: Vec<LValue>,
    body: ExprNode, // Should we have this as a borrow instead maybe? Probably just a hassle
    env: Rc<Environment>,
}

impl Closure {
    pub fn new(id: String, params: Vec<LValue>, body: ExprNode, env: &Rc<Environment>) -> Self {
        Self {
            id,
            params,
            body,
            env: env.clone(),
        }
    }

    fn call(&self, args: Vec<Value>) -> RunRes<Value> {
        let env = Environment::nest(&self.env);
        for (param, arg) in self.params.iter().zip(args.into_iter()) {
            param.declare(&env)?;
            param.assign(arg, &env)?;
        }
        expressions::eval(&self.body, &env)
    }

    fn arity(&self) -> usize {
        self.params.len()
    }

    fn name(&self) -> &str {
        &self.id
    }
}

pub fn define_builtins(env: &Environment) {
    for builtin in builtins::get_builtins() {
        env.define(
            builtin.name().to_string(),
            Value::Callable(Function::Builtin(builtin)),
        );
    }
}
