use itertools::Itertools;

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
            Function::Builtin(builtin) => builtin
                .run(args)
                .map_err(|msg| RuntimeError::Error(start.clone(), end.clone(), msg)),
        };

        match result {
            Err(RuntimeError::Break) => Err(RuntimeError::Error(
                start,
                end,
                "Break encountered outside loop".to_string(),
            )),
            Err(RuntimeError::Return(value)) => Ok(value),
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
        Some(self.name().cmp(other.name()))
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
    fn run(&self, args: Vec<Value>) -> Result<Value, String>;
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
    define_builtins!(env, Time, Print, Str, Push, Pop);
}

struct Time;

impl Builtin for Time {
    fn run(&self, _args: Vec<Value>) -> Result<Value, String> {
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
    fn run(&self, args: Vec<Value>) -> Result<Value, String> {
        println!("{}", args[0].stringify());
        Ok(Value::Nil)
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "print"
    }
}

struct Str;

impl Builtin for Str {
    fn run(&self, args: Vec<Value>) -> Result<Value, String> {
        Ok(Value::String(args[0].stringify()))
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "str"
    }
}

struct Push;

impl Builtin for Push {
    fn run(&self, args: Vec<Value>) -> Result<Value, String> {
        match args.into_iter().collect_tuple().unwrap() {
            // Strange if pushing a list to itself. Print crashes :D
            (Value::List(mut list), value) => {
                list.push(value);
                Ok(Value::Nil)
            }
            (_, _) => Err("First argument to push must be a list".to_string()),
        }
    }

    fn arity(&self) -> usize {
        2
    }

    fn name(&self) -> &str {
        "push"
    }
}

struct Pop;

impl Builtin for Pop {
    fn run(&self, args: Vec<Value>) -> Result<Value, String> {
        match args.into_iter().next().unwrap() {
            Value::List(mut list) => Ok(list.pop()),
            _ => Err("Argument to pop must be a list".to_string()),
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "pop"
    }
}
