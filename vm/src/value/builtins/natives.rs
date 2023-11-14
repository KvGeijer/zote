use crate::value::Value;

use super::templates::BuiltinTemplate;
use super::Builtin;
use std::rc::Rc;

pub fn get_builtins() -> Vec<Rc<dyn Builtin>> {
    let mut builtins: Vec<Rc<dyn Builtin>> = vec![];

    builtins.new_any_arg("print", |args| {
        for arg in args.iter() {
            print!("{}", arg);
        }
        println!("");
        Ok(args.get(0).cloned().unwrap_or(Value::Nil))
    });

    builtins.new_2arg("push", |value, collection| {
        collection.push(value)?;
        Ok(collection)
    });

    builtins.new_1arg("pop", |collection| collection.pop());

    builtins
}
