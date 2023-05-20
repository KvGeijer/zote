use itertools::Itertools;
use std::{fs::read_to_string, rc::Rc};

use crate::interpreter::{RunRes, RuntimeError, Value};

// Could make Function a struct, hiding Builtin, getting rid of ugly path
pub(in super::super) trait Builtin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value>;
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
    box_builtins![Time, Print, Str, Push, Pop, Read, Int, Max, Map, Split, Sum, Sort]
}

struct Time;

impl Builtin for Time {
    fn run(&self, _args: Vec<Value>) -> RunRes<Value> {
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
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
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
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
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
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().collect_tuple().unwrap() {
            // Strange if pushing a list to itself. Print crashes :D
            (value, Value::List(list)) => {
                list.push(value);
                Ok(Value::Nil)
            }
            (_, _) => Err(RuntimeError::ErrorReason(
                "Second argument to push must be a list".to_string(),
            )),
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
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().next().unwrap() {
            Value::List(list) => Ok(list.pop()),
            _ => Err(RuntimeError::ErrorReason(
                "Argument to pop must be a list".to_string(),
            )),
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
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().next().unwrap() {
            Value::String(path) => read_to_string(&path)
                .map(|content| Value::String(content)) // Should we have constructors for these instead?
                .map_err(|_| RuntimeError::ErrorReason(format!("Could not read file at {path}"))),
            _ => Err(RuntimeError::ErrorReason(
                "Argument to pop must be a list".to_string(),
            )),
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
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().next().unwrap() {
            Value::String(string) => {
                if let Ok(int) = string.parse::<i64>() {
                    Ok(int.into())
                } else {
                    Err(RuntimeError::ErrorReason(format!(
                        "Cannot parse {string} as integer"
                    )))
                }
            }
            Value::Numerical(num) => Ok(num.to_int().into()),
            Value::Nil => Ok(0.into()),
            val => Err(RuntimeError::ErrorReason(format!(
                "Cannot convert {val} to an int"
            ))),
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
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().next().unwrap() {
            Value::List(list) => list
                .max()
                .map_err(|reason| RuntimeError::ErrorReason(reason)),
            _ => Err(RuntimeError::ErrorReason(
                "So far max is only implemented for lists".to_string(),
            )),
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "max"
    }
}

struct Split;
impl Builtin for Split {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().collect_tuple().unwrap() {
            (Value::String(string), Value::String(delimiter)) => {
                let splitted: Vec<Value> = string
                    .split(&delimiter)
                    .map(|str| str.to_string().into())
                    .collect();
                Ok(splitted.into())
            }
            (Value::List(list), value) => list.split(&value),
            (left, right) => Err(RuntimeError::ErrorReason(format!(
                "Arguments {} and {} are not valid for split",
                left.type_of(),
                right.type_of()
            ))),
        }
    }

    fn arity(&self) -> usize {
        2
    }

    fn name(&self) -> &str {
        "split"
    }
}

struct Map;
impl Builtin for Map {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().collect_tuple().unwrap() {
            (Value::List(list), Value::Callable(func)) => list.map(&func),
            (left, right) => Err(RuntimeError::ErrorReason(format!(
                "Expected a list and a function as arguments to map, but got {} and {}",
                left.type_of(),
                right.type_of()
            ))),
        }
    }

    fn arity(&self) -> usize {
        2
    }

    fn name(&self) -> &str {
        "map"
    }
}

struct Sum;
impl Builtin for Sum {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().next().unwrap() {
            Value::List(list) => list
                .sum()
                .map_err(|reason| RuntimeError::ErrorReason(reason)),
            arg => Err(RuntimeError::ErrorReason(format!(
                "Expected a list as argument to sum, but got {}",
                arg.type_of(),
            ))),
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "sum"
    }
}

struct Sort;
impl Builtin for Sort {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().next().unwrap() {
            Value::List(list) => list
                .sort()
                .map_err(|reason| RuntimeError::ErrorReason(reason)),
            arg => Err(RuntimeError::ErrorReason(format!(
                "Expected a list as argument to sort, but got {}",
                arg.type_of(),
            ))),
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "sort"
    }
}
