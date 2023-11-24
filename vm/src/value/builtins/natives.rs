use crate::error::{RunRes, RuntimeError};
use crate::value::{Dictionary, List, Value};

use super::templates::BuiltinTemplate;
use super::Builtin;
use std::rc::Rc;

pub fn get_builtins() -> Vec<Rc<dyn Builtin>> {
    let mut builtins: Vec<Rc<dyn Builtin>> = vec![Rc::new(DictNative)];

    builtins.new_1arg("pop", |collection| collection.pop());

    builtins.new_1arg("len", |value| {
        value.len().map(|usize| Value::Int(usize as i64))
    });

    builtins.new_2arg("push", |value, collection| {
        collection.push(value)?;
        Ok(collection)
    });

    builtins.new_any_arg("print", |args| {
        for arg in args.iter() {
            print!("{}", arg);
        }
        println!("");
        Ok(args.get(0).cloned().unwrap_or(Value::Nil))
    });

    builtins
}

struct DictNative;
impl Builtin for DictNative {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        if let Some(value) = args.into_iter().next() {
            let kind = value.type_of();
            let list: Rc<List> = value.to_list().ok_or(RuntimeError::bare_error(format!("The function 'dict' takes a single list as argument with all its pairs, or no list, but got {kind}")))?;
            let dict: Dictionary = list.as_ref().try_into()?;
            Ok(dict.into())
        } else {
            Ok(Dictionary::new().into())
        }
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
