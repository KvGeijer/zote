use itertools::Itertools;
use std::{cmp::Ordering, fs::read_to_string, rc::Rc};

use crate::interpreter::{
    collections::{Collection, Dict},
    RunError, RunRes, Value,
};

pub trait Builtin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value>;
    fn accept_arity(&self, arity: usize) -> bool;
    fn name(&self) -> &str;
    fn arity(&self) -> &str;
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
    let mut builtins: Vec<Rc<dyn Builtin>> = box_builtins![DictBuiltin, SetBuiltin, JoinBuiltin];

    builtins.new_0arg("time", || {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64();
        Ok(now.into())
    });

    builtins.new_1arg("print", |arg| {
        println!("{}", arg.stringify());
        Ok(arg)
    });

    builtins.new_1arg("str", |arg| Ok(arg.stringify().into()));

    builtins.new_1arg("pop", |arg| match arg {
        Value::Collection(Collection::List(list)) => list.pop(),
        _ => RunError::error("Argument to pop must be a list".to_string()),
    });

    builtins.new_1arg("int", |arg| match arg {
        Value::Collection(Collection::String(string)) => string
            .parse::<i64>()
            .map(|int| int.into())
            .map_err(|_| RunError::bare_error(format!("Cannot parse {string} as integer"))),
        Value::Numerical(num) => Ok(num.to_int().into()),
        Value::Nil => Ok(0.into()),
        val => RunError::error(format!("Cannot convert {} to an int", val.type_of())),
    });

    builtins.new_1arg("sum", |arg| match arg {
        Value::Collection(Collection::List(list)) => list.sum(),
        arg => RunError::error(format!(
            "Expected a list as argument to sum, but got {}",
            arg.type_of(),
        )),
    });

    builtins.new_1arg("sort", |arg| match arg {
        Value::Collection(Collection::List(list)) => list.sort(),
        arg => RunError::error(format!(
            "Expected a list as argument to sort, but got {}",
            arg.type_of(),
        )),
    });

    builtins.new_1arg("len", |arg| match arg {
        Value::Collection(coll) => Ok((coll.len() as i64).into()),
        arg => RunError::error(format!(
            "Expected a collection as argument to len, but got {}",
            arg.type_of(),
        )),
    });

    builtins.new_1arg("list", |arg| match arg {
        Value::Collection(coll) => Ok(coll.to_iter().collect::<Vec<Value>>().into()),
        arg => RunError::error(format!(
            "Expected a collection as argument to list, but got {}",
            arg.type_of(),
        )),
    });

    builtins.new_1arg("to_ascii", |arg| {
        match arg {
        Value::Collection(Collection::String(string)) => {
            if let Some(char) = string.chars().next() && char.is_ascii() {
                Ok((char as i64).into())
            } else {
                RunError::error(format!("Cannot convert {string} to a single ascii value"))
            }
        }
        _ => RunError::error("Can only convert string to ascii".to_string()),
    }
    });

    builtins.new_1arg("read", |arg| match arg {
        Value::Collection(Collection::String(path)) => read_to_string(&path)
            .map(|content| content.into())
            .map_err(|_| RunError::bare_error(format!("Could not read file at {path}"))),
        _ => RunError::error("Argument to read must be a string".to_string()),
    });

    builtins.new_1arg("rev", |arg| {
        Ok(arg.to_iter()?.rev().collect::<Vec<Value>>().into())
    });

    builtins.new_1arg("max", |arg| {
        arg.to_iter()?
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
    });

    builtins.new_2arg("push", |item, stack| match (item, stack) {
        (value, Value::Collection(Collection::List(list))) => {
            list.push(value);
            Ok(Value::Nil)
        }
        (_, _) => RunError::error("Second argument to push must be a list".to_string()),
    });

    builtins.new_2arg("split", |base, delim| match (base, delim) {
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
    });

    // Maps the function over the iterable, then converting it back into a list
    builtins.new_2arg("map", |base, func| match (base, func) {
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
    });

    builtins.new_2arg("in", |item, base| match (item, base) {
        (value, Value::Collection(Collection::List(list))) => {
            Ok(list.to_iter().contains(&value).into())
        }
        (value, Value::Collection(Collection::Dict(dict))) => Ok(dict.contains_key(&value)?.into()),
        (_, arg) => RunError::error(format!(
            "Expected a list or dict as second argument to in (string not implemented), but got {}",
            arg.type_of(),
        )),
    });

    builtins.new_2arg("zip", |left, right| {
        Ok(left
            .to_iter()?
            .zip(right.to_iter()?)
            .map(|(x, y)| vec![x, y].into())
            .collect::<Vec<Value>>()
            .into())
    });

    builtins
}

struct SetBuiltin;
impl Builtin for SetBuiltin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        let set = Dict::new();

        if let Some(arg) = args.into_iter().next() {
            for val in arg.to_iter()? {
                set.assign_into(val, Value::Nil)?;
            }
        }
        Ok(set.into())
    }

    fn accept_arity(&self, arity: usize) -> bool {
        [0, 1].contains(&arity)
    }

    fn name(&self) -> &str {
        "set"
    }

    fn arity(&self) -> &str {
        "[0, 1]"
    }
}

struct DictBuiltin;
impl Builtin for DictBuiltin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        let dict = Dict::new();

        if let Some(arg) = args.into_iter().next() {
            for entry in arg.to_iter()? {
                let list = entry.cast_list("Expect a list of key-value in dict iterator")?;
                if list.len() != 2 {
                    return RunError::error(format!(
                        "Expect a key and value, but found {} values",
                        list.len()
                    ));
                }
                let (key, value) = list.to_iter().tuples().next().unwrap();
                dict.assign_into(key, value)?;
            }
        }
        Ok(dict.into())
    }

    fn accept_arity(&self, arity: usize) -> bool {
        [0, 1].contains(&arity)
    }

    fn name(&self) -> &str {
        "dict"
    }

    fn arity(&self) -> &str {
        "[0, 1]"
    }
}

struct JoinBuiltin;
impl Builtin for JoinBuiltin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        let mut arg_iter = args.into_iter();
        let joining = arg_iter.next().unwrap();
        let delim = if let Some(joiner) = arg_iter.next() {
            joiner.cast_string("Second arg to join should be the string to be interspersed")?
        } else {
            "".to_string()
        };

        let mut joined = String::new();
        let mut first = true;
        for value in joining.to_iter()? {
            if !first {
                joined.push_str(&delim);
            }
            // This way it becomes much harder to see where the error is.
            let str = value.cast_string("Expect to only be joining strings")?;
            joined.push_str(&str);
            first = false;
        }
        Ok(joined.into())
    }

    fn accept_arity(&self, arity: usize) -> bool {
        [1, 2].contains(&arity)
    }

    fn name(&self) -> &str {
        "join"
    }

    fn arity(&self) -> &str {
        "[1, 2]"
    }
}

/// Trait for more easily adding builtins with a certain number of args
trait Builtins {
    fn new_0arg(&mut self, name: &'static str, func: impl Fn() -> RunRes<Value> + 'static);
    fn new_1arg(&mut self, name: &'static str, func: impl Fn(Value) -> RunRes<Value> + 'static);
    fn new_2arg(
        &mut self,
        name: &'static str,
        func: impl Fn(Value, Value) -> RunRes<Value> + 'static,
    );
}

impl Builtins for Vec<Rc<dyn Builtin>> {
    fn new_0arg(&mut self, name: &'static str, func: impl Fn() -> RunRes<Value> + 'static) {
        let builtin = Rc::new(ZeroArgBuiltin {
            name,
            func: Box::new(func),
        });
        self.push(builtin);
    }

    fn new_1arg(&mut self, name: &'static str, func: impl Fn(Value) -> RunRes<Value> + 'static) {
        let builtin = Rc::new(OneArgBuiltin {
            name,
            func: Box::new(func),
        });
        self.push(builtin);
    }

    fn new_2arg(
        &mut self,
        name: &'static str,
        func: impl Fn(Value, Value) -> RunRes<Value> + 'static,
    ) {
        let builtin = Rc::new(TwoArgBuiltin {
            name,
            func: Box::new(func),
        });
        self.push(builtin);
    }
}

struct ZeroArgBuiltin {
    name: &'static str,
    func: Box<dyn Fn() -> RunRes<Value>>,
}

impl Builtin for ZeroArgBuiltin {
    fn run(&self, _args: Vec<Value>) -> RunRes<Value> {
        (self.func)()
    }

    fn accept_arity(&self, arity: usize) -> bool {
        arity == 0
    }

    fn name(&self) -> &str {
        self.name
    }

    fn arity(&self) -> &str {
        "0"
    }
}

struct OneArgBuiltin {
    name: &'static str,
    func: Box<dyn Fn(Value) -> RunRes<Value>>,
}

impl Builtin for OneArgBuiltin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        let arg = args.into_iter().next().expect("Incorrect number of args");
        (self.func)(arg)
    }

    fn accept_arity(&self, arity: usize) -> bool {
        arity == 1
    }

    fn name(&self) -> &str {
        self.name
    }

    fn arity(&self) -> &str {
        "1"
    }
}

struct TwoArgBuiltin {
    name: &'static str,
    func: Box<dyn Fn(Value, Value) -> RunRes<Value>>,
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

    fn accept_arity(&self, arity: usize) -> bool {
        arity == 2
    }

    fn name(&self) -> &str {
        self.name
    }

    fn arity(&self) -> &str {
        "2"
    }
}
