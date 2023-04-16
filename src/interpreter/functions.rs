use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    rc::Rc,
};

use super::{environment::Environment, RunRes, Value};

#[derive(Clone)]
pub(super) enum Function {
    // Closure(Box<ExprNode>, Vec<String>, Rc<Environment>),
    // Closure(closure),
    Builtin(Rc<dyn Builtin>),
}

impl Function {
    pub(super) fn call(&self, args: Vec<Value>) -> RunRes<Value> {
        match self {
            // Function::Closure(closure) => params.len(),
            Function::Builtin(builtin) => builtin.run(args),
        }
    }

    pub(super) fn arity(&self) -> usize {
        match self {
            // Function::Closure(closure) => params.len(),
            Function::Builtin(builtin) => builtin.arity(),
        }
    }

    pub(super) fn name(&self) -> &str {
        match self {
            // Function::Closure(_, params, _) => params.len(),
            Function::Builtin(builtin) => builtin.name(),
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            // Function::Closure(_, _, _) => f.write_str("Closure"),
            Function::Builtin(builtin) => f.write_str(builtin.name()),
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
