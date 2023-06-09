use itertools::Itertools;
use std::{cmp::Ordering, fs::read_to_string, rc::Rc};

use crate::interpreter::{
    collections::{Collection, Dict},
    RunError, RunRes, Value,
};

pub trait Builtin {
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

pub fn get_builtins() -> Vec<Rc<dyn Builtin>> {
    let mut builtins: Vec<Rc<dyn Builtin>> = box_builtins![
        Time, Print, Str, Pop, Read, Int, Max, Sum, Sort, NewDict, List, Len, ToAscii, Rev
    ];

    builtins.push(TwoArgBuiltin::new("push", |item, stack| {
        match (item, stack) {
            (value, Value::Collection(Collection::List(list))) => {
                list.push(value);
                Ok(Value::Nil)
            }
            (_, _) => RunError::error("Second argument to push must be a list".to_string()),
        }
    }));

    builtins.push(TwoArgBuiltin::new("split", |base, delim| {
        match (base, delim) {
            (
                Value::Collection(Collection::String(string)),
                Value::Collection(Collection::String(delimiter)),
            ) => {
                let splitted: Vec<Value> = string
                    .split(&delimiter)
                    .map(|str| str.to_string().into())
                    .collect();
                Ok(splitted.into())
            }
            (Value::Collection(Collection::List(list)), value) => list.split(&value),
            (left, right) => RunError::error(format!(
                "Arguments {} and {} are not valid for split",
                left.type_of(),
                right.type_of()
            )),
        }
    }));

    // Maps the function over the iterable, then converting it back into a list
    builtins.push(TwoArgBuiltin::new("map", |base, func| match (base, func) {
        (Value::Collection(coll), Value::Callable(func)) => Ok(coll
            .to_iter()
            .map(|val| func.call(vec![val]))
            .collect::<Result<Vec<Value>, _>>()?
            .into()),
        (left, right) => RunError::error(format!(
            "Expected a list and a function as arguments to map, but got {} and {}",
            left.type_of(),
            right.type_of()
        )),
    }));

    builtins.push(TwoArgBuiltin::new("in", |item, base| match (item, base) {
        (value, Value::Collection(Collection::List(list))) => {
            Ok(list.to_iter().contains(&value).into())
        }
        (value, Value::Collection(Collection::Dict(dict))) => Ok(dict.contains_key(&value)?.into()),
        (_, arg) => RunError::error(format!(
            "Expected a list or dict as second argument to in (string not implemented), but got {}",
            arg.type_of(),
        )),
    }));

    builtins
}

struct TwoArgBuiltin {
    name: &'static str,
    func: Box<dyn Fn(Value, Value) -> RunRes<Value>>,
}

impl TwoArgBuiltin {
    fn new(name: &'static str, func: impl Fn(Value, Value) -> RunRes<Value> + 'static) -> Rc<Self> {
        Rc::new(Self {
            name,
            func: Box::new(func),
        })
    }
}

impl Builtin for TwoArgBuiltin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        let (x, y) = args
            .into_iter()
            .tuples()
            .next()
            .expect("Incorrect number of args");
        (self.func)(x, y)
    }

    fn arity(&self) -> usize {
        2
    }

    fn name(&self) -> &str {
        self.name
    }
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
        let val = args.into_iter().next().unwrap();
        println!("{}", val.stringify());
        Ok(val)
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
        Ok(args[0].stringify().into())
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "str"
    }
}

struct Pop;

impl Builtin for Pop {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().next().unwrap() {
            Value::Collection(Collection::List(list)) => list.pop(),
            _ => RunError::error("Argument to pop must be a list".to_string()),
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "pop"
    }
}

/// Reads the file at the given path into a string
struct Read;

impl Builtin for Read {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().next().unwrap() {
            Value::Collection(Collection::String(path)) => read_to_string(&path)
                .map(|content| content.into())
                .map_err(|_| RunError::bare_error(format!("Could not read file at {path}"))),
            _ => RunError::error("Argument to read must be a string".to_string()),
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
            Value::Collection(Collection::String(string)) => {
                if let Ok(int) = string.parse::<i64>() {
                    Ok(int.into())
                } else {
                    RunError::error(format!("Cannot parse {string} as integer"))
                }
            }
            Value::Numerical(num) => Ok(num.to_int().into()),
            Value::Nil => Ok(0.into()),
            val => RunError::error(format!("Cannot convert {val} to an int")),
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
        args.into_iter()
            .next()
            .unwrap()
            .to_iter()?
            .try_reduce(|x, y| match x.partial_cmp(&y) {
                None => RunError::error(format!(
                    "Cannot compare {} with {}. For finding max in a list.",
                    x.type_of(),
                    y.type_of(),
                )),
                Some(Ordering::Less) => Ok(y),
                Some(_) => Ok(x),
            })?
            .ok_or(RunError::bare_error(
                "Canot get max from empty iterator".to_string(),
            ))
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "max"
    }
}

struct Sum;
impl Builtin for Sum {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().next().unwrap() {
            Value::Collection(Collection::List(list)) => list.sum(),
            arg => RunError::error(format!(
                "Expected a list as argument to sum, but got {}",
                arg.type_of(),
            )),
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
            Value::Collection(Collection::List(list)) => list.sort(),
            arg => RunError::error(format!(
                "Expected a list as argument to sort, but got {}",
                arg.type_of(),
            )),
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "sort"
    }
}

struct NewDict;
impl Builtin for NewDict {
    fn run(&self, _args: Vec<Value>) -> RunRes<Value> {
        Ok(Dict::new().into())
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "dict"
    }
}

struct Len;
impl Builtin for Len {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().next().unwrap() {
            Value::Collection(coll) => Ok((coll.len() as i64).into()),
            arg => RunError::error(format!(
                "Expected a collection as argument to len, but got {}",
                arg.type_of(),
            )),
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "len"
    }
}

struct List;
impl Builtin for List {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().next().unwrap() {
            Value::Collection(coll) => Ok(coll.to_iter().collect::<Vec<Value>>().into()),
            arg => RunError::error(format!(
                "Expected a collection as argument to list, but got {}",
                arg.type_of(),
            )),
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "list"
    }
}

struct ToAscii;
impl Builtin for ToAscii {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        match args.into_iter().next().unwrap() {
            Value::Collection(Collection::String(string)) => {
                if string.len() == 1 {
                    Ok((string.into_bytes()[0] as i64).into())
                } else {
                    RunError::error(format!("Cannot convert {string} to a single ascii value"))
                }
            }
            _ => RunError::error("Can only convert string to ascii".to_string()),
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "to_ascii"
    }
}

struct Rev;
impl Builtin for Rev {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        Ok(args
            .into_iter()
            .next()
            .unwrap()
            .to_iter()?
            .rev()
            .collect::<Vec<Value>>()
            .into())
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "rev"
    }
}
