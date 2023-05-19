use itertools::Itertools;
use std::{fs::read_to_string, rc::Rc};

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
    box_builtins![Time, Print, Str, Push, Pop, Read, Int, Max]
}

struct Time;

impl Builtin for Time {
    fn run(&self, _args: Vec<Value>) -> Result<Value, String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64();
        Ok(now.into())
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

struct Read;

impl Builtin for Read {
    fn run(&self, args: Vec<Value>) -> Result<Value, String> {
        match args.into_iter().next().unwrap() {
            Value::String(path) => read_to_string(&path)
                .map(|content| Value::String(content)) // Should we have constructors for these instead?
                .map_err(|_| format!("Could not read file at {path}")),
            _ => Err("Argument to pop must be a list".to_string()),
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "read"
    }
}

struct Int;
impl Builtin for Int {
    fn run(&self, args: Vec<Value>) -> Result<Value, String> {
        match args.into_iter().next().unwrap() {
            Value::String(string) => {
                if let Ok(int) = string.parse::<i64>() {
                    Ok(int.into())
                } else {
                    Err(format!("Cannot parse {string} as integer"))
                }
            }
            Value::Numerical(num) => Ok(num.to_int().into()),
            Value::Nil => Ok(0.into()),
            val => Err(format!("Cannot convert {val} to an int")),
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "int"
    }
}

struct Max;
impl Builtin for Max {
    fn run(&self, args: Vec<Value>) -> Result<Value, String> {
        match args.into_iter().next().unwrap() {
            Value::List(list) => list.max(),
            _ => Err("So far max is only implemented for lists".to_string()),
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "max"
    }
}
