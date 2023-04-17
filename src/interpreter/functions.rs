use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    rc::Rc,
};

use crate::{code_loc::CodeLoc, parser::ExprNode};

use super::{environment::Environment, expressions, RunRes, RuntimeError, Value};

#[derive(Clone)]
pub(super) enum Function {
    // Closure(),
    Closure(Closure),
    Builtin(Rc<dyn Builtin>),
}

impl Function {
    pub(super) fn call(&self, args: Vec<Value>, start: CodeLoc, end: CodeLoc) -> RunRes<Value> {
        let result = match self {
            Function::Closure(closure) => closure.call(args),
            Function::Builtin(builtin) => builtin.run(args),
        };

        match result {
            Err(RuntimeError::Break) => Err(RuntimeError::Error(
                start,
                end,
                "Break encountered outside loop".to_string(),
            )),
            otherwise => otherwise,
        }
    }

    pub(super) fn arity(&self) -> usize {
        match self {
            Function::Closure(closure) => closure.arity(),
            Function::Builtin(builtin) => builtin.arity(),
        }
    }

    pub(super) fn name(&self) -> &str {
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
            (Function::Builtin(a), Function::Builtin(b)) => {
                a.name() == b.name() && a.arity() == b.arity()
            }
            _ => false,
        }
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Arbitraty ordering
        println!("WARNING: Comparing function handles");
        Some(self.name().cmp(&other.name()))
    }
}

#[derive(Clone)]
pub(super) struct Closure {
    id: String, // Maybe not ideal to have
    params: Vec<String>,
    body: ExprNode, // Should we have this as a borrow instead maybe? Probably just a hassle
    env: Rc<Environment>,
}

impl Closure {
    pub(super) fn new(
        id: String,
        params: Vec<String>,
        body: ExprNode,
        env: &Rc<Environment>,
    ) -> Self {
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
            env.define(param.to_string(), arg);
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

pub(super) trait Builtin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value>;
    fn arity(&self) -> usize;
    fn name(&self) -> &str;
}

macro_rules! define_builtins {
    ($env:expr, $($builtin:expr),* $(,)?) => {
        $(
            $env.define(
                $builtin.name().to_string(),
                Value::Callable(Function::Builtin(Rc::new($builtin)))
            );
        )*
    };
}

pub(super) fn define_builtins(env: &Environment) {
    define_builtins!(env, Time, Print);
}

struct Time;

impl Builtin for Time {
    fn run(&self, _args: Vec<Value>) -> RunRes<Value> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64();
        Ok(Value::Float(now))
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "time"
    }
}

struct Print;

impl Builtin for Print {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        println!("{}", args[0].stringify());
        Ok(Value::Uninitialized)
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "print"
    }
}
