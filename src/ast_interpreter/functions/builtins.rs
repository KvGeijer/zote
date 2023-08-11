use itertools::Itertools;
use std::{cmp::Ordering, fs::read_to_string, rc::Rc};

use crate::ast_interpreter::{
    collections::{slice_iter, Collection, Dict, SliceValue},
    environment::Environment,
    statements, RunError, RunRes, Value,
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
    let mut builtins: Vec<Rc<dyn Builtin>> = box_builtins![
        DictBuiltin,
        SetBuiltin,
        MaxBuiltin,
        MinBuiltin,
        JoinBuiltin,
        SortBuiltin
    ];

    builtins.new_0arg("time", || {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64();
        Ok(now.into())
    });

    builtins.new_1arg("enumerate", |arg| {
        Ok(arg
            .to_iter()?
            .enumerate()
            .map(|(i, val)| vec![(i as i64).into(), val].into())
            .collect::<Vec<Value>>()
            .into())
    });

    builtins.new_1arg("values", |arg| {
        Ok(arg.cast_dict("values expects a dict")?.values().into())
    });

    builtins.new_1arg("keys", |arg| {
        Ok(arg.cast_dict("keys expects a dict")?.keys().into())
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
            .map_err(|_| RunError::bare_error(format!("Cannot parse {:?} as integer", string))),
        Value::Numerical(num) => Ok(num.to_int().into()),
        Value::Nil => Ok(0.into()),
        val => RunError::error(format!("Cannot convert {} to an int", val.type_of())),
    });

    builtins.new_1arg("float", |arg| match arg {
        Value::Collection(Collection::String(string)) => string
            .parse::<f64>()
            .map(|float| float.into())
            .map_err(|_| RunError::bare_error(format!("Cannot parse {:?} as float", string))),
        Value::Numerical(num) => Ok(num.to_float().into()),
        Value::Nil => Ok(0.0.into()),
        val => RunError::error(format!("Cannot convert {} to an float", val.type_of())),
    });

    builtins.new_1arg("bool", |arg| Ok(arg.truthy().into()));

    // TODO: Should work for any iterator
    builtins.new_1arg("sum", |arg| match arg {
        Value::Collection(Collection::List(list)) => list.sum(),
        arg => RunError::error(format!(
            "Expected a list as argument to sum, but got {}",
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

    builtins.new_1arg("id", Ok);

    builtins.new_1arg("rev", |arg| {
        Ok(arg.to_iter()?.rev().collect::<Vec<Value>>().into())
    });

    builtins.new_1arg("id", Ok);

    builtins.new_1arg("head", |arg| {
        arg.to_iter()?.next().ok_or(RunError::bare_error(
            "Cannot take head of empty iterator".to_string(),
        ))
    });

    builtins.new_1arg("abs", |arg| {
        Ok(arg
            .cast_numerical("Abs can only be callable on a numerical")?
            .abs()
            .into())
    });

    builtins.new_1arg("typeof", |arg| Ok(arg.type_of().to_string().into()));

    builtins.new_1arg("eval", |arg| {
        // Very powerful... And probably wrong...
        let mut error_reporter = crate::errors::ErrorReporter::new();
        let tokens = crate::scanner::tokenize(
            &arg.cast_string("Can only eval strings")?,
            &mut error_reporter,
        );
        if !error_reporter.had_compilation_error && let Some(stmts) = crate::parser::parse(&tokens, &mut error_reporter) {
            // Should we look at error_reporter instead? Probably way better
            let env = Environment::new();
            match statements::eval_statements(&stmts, &env) {
                Ok(Some(val)) => Ok(val),
                Ok(None) => Ok(Value::Nil),
                Err(err) => Err(err),
            }
        } else {
            RunError::error("Failed to parse string as zote".to_string())
        }
    });

    builtins.new_2arg("const", |_, val| Ok(val));

    builtins.new_2arg("push", |item, stack| match (item, stack) {
        (value, Value::Collection(Collection::List(list))) => {
            list.push(value.clone());
            Ok(value)
        }
        (_, _) => RunError::error("Second argument to push must be a list".to_string()),
    });

    builtins.new_2arg("insert", |item, stack| match (item, stack) {
        (value, Value::Collection(Collection::Dict(dict))) => {
            dict.assign_into(value, Value::Nil)?;
            Ok(Value::Nil)
        }
        (_, _) => RunError::error("Second argument to insert must be a dict".to_string()),
    });

    builtins.new_2arg("split", |base, delim| match (base, delim) {
        (
            Value::Collection(Collection::String(string)),
            Value::Collection(Collection::String(delimiter)),
        ) => {
            let mut splitted: Vec<Value> = string
                .split(&delimiter)
                .map(|str| str.to_string().into())
                .collect();

            if splitted.last() == Some(&"".to_string().into()) {
                splitted.remove(splitted.len() - 1);
            }
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
            "Expected a collection and a function as arguments to map, but got {} and {}",
            left.type_of(),
            right.type_of()
        )),
    });

    // Very bad way to do this. Is there a better functional way?
    builtins.new_2arg("filter", |base, func| match (base, func) {
        (Value::Collection(coll), Value::Callable(func)) => {
            let mut filtered = vec![];
            for val in coll.to_iter() {
                if func.call(vec![val.clone()])?.truthy() {
                    filtered.push(val);
                }
            }
            Ok(filtered.into())
        }
        (left, right) => RunError::error(format!(
            "Expected a collection and a function as arguments to filter, but got {} and {}",
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

    builtins.new_2arg("intersect", |left, right| {
        Ok(left
            .cast_dict("In first arg to intersect")?
            .intersect(&right.cast_dict("In first arg to intersect")?)
            .into())
    });

    builtins.new_2arg("union", |left, right| {
        Ok(left
            .cast_dict("In first arg to union")?
            .union(&right.cast_dict("In first arg to union")?)
            .into())
    });

    builtins.new_2arg("take", |data, nbr| {
        let slice = slice_iter(
            data.to_iter()?,
            SliceValue {
                start: None,
                stop: Some(
                    nbr.cast_numerical("Expect a number of items to take.")?
                        .to_int(),
                ),
                step: None,
            },
            data.to_iter()?.len(), // TODO: Slow (Really needed?)
        )?
        .collect_vec()
        .into();
        Ok(slice)
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

/// If a single value, iterates over it to find max, and if several values, the max of them
struct MaxBuiltin;
impl Builtin for MaxBuiltin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        let mut value_iter = if args.len() == 1 {
            args[0].to_iter()?
        } else {
            args.into_iter()
        };
        value_iter
            .try_reduce(|x, y| match x.partial_cmp(&y) {
                None => RunError::error(format!(
                    "Cannot compare {} with {}. For finding max among values.",
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

    fn accept_arity(&self, arity: usize) -> bool {
        arity > 0
    }

    fn name(&self) -> &str {
        "max"
    }

    fn arity(&self) -> &str {
        "[>0]"
    }
}

/// If a single value, iterates over it to find max, and if several values, the max of them
struct MinBuiltin;
impl Builtin for MinBuiltin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        let mut value_iter = if args.len() == 1 {
            args[0].to_iter()?
        } else {
            args.into_iter()
        };
        value_iter
            .try_reduce(|x, y| match x.partial_cmp(&y) {
                None => RunError::error(format!(
                    "Cannot compare {} with {}. For finding min among values.",
                    x.type_of(),
                    y.type_of(),
                )),
                Some(Ordering::Less) => Ok(x),
                Some(_) => Ok(y),
            })?
            .ok_or(RunError::bare_error(
                "Canot get min from empty iterator".to_string(),
            ))
    }

    fn accept_arity(&self, arity: usize) -> bool {
        arity > 0
    }

    fn name(&self) -> &str {
        "min"
    }

    fn arity(&self) -> &str {
        "[>0]"
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

struct SortBuiltin;
impl Builtin for SortBuiltin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        let mut arg_iter = args.into_iter();
        let sorting = arg_iter.next().unwrap();
        let list = sorting.cast_list("Expect a list as first argument to sort.")?;

        let sorted = if let Some(comparator) = arg_iter.next() {
            let cmp_func =
                comparator.cast_func("Second argument to sort reserved for sorting function.")?;
            list.sort_by(cmp_func)?
        } else {
            list.sort()?
        };

        Ok(sorted)
    }

    fn accept_arity(&self, arity: usize) -> bool {
        [1, 2].contains(&arity)
    }

    fn name(&self) -> &str {
        "sort"
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
