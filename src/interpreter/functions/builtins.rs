use itertools::Itertools;
use std::rc::Rc;

use crate::interpreter::Value;

// Could make Function a struct, hiding Builtin, getting rid of ugly path
pub(in super::super) trait Builtin {
    fn run(&self, args: Vec<Value>) -> Result<Value, String>;
    fn arity(&self) -> usize;
    fn name(&self) -> &str;
}

macro_rules! box_builtins {
    ($($builtin:expr),* $(,)?) => {
        vec![
            $(
                Rc::new($builtin),
            )*
        ]
    };
}

pub(super) fn get_builtins() -> Vec<Rc<dyn Builtin>> {
    box_builtins![Time, Print, Str, Push, Pop]
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

// push(item, list), so we can do item >> push(list)
struct Push;

impl Builtin for Push {
    fn run(&self, args: Vec<Value>) -> Result<Value, String> {
        match args.into_iter().collect_tuple().unwrap() {
            // Strange if pushing a list to itself. Print crashes :D
            (value, Value::List(mut list)) => {
                list.push(value);
                Ok(Value::Nil)
            }
            (_, _) => Err("Second argument to push must be a list".to_string()),
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

