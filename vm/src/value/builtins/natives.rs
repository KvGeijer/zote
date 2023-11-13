use crate::error::RuntimeError;

use super::templates::BuiltinTemplate;
use super::Builtin;
use std::rc::Rc;

pub fn get_builtins() -> Vec<Rc<dyn Builtin>> {
    let mut builtins: Vec<Rc<dyn Builtin>> = vec![];

    builtins.new_1arg("print", |arg| {
        println!("{}", arg);
        Ok(arg)
    });

    builtins.new_2arg("push", |collection, value| {
        let typ = collection.type_of();
        let list = collection
            .to_list()
            .ok_or(RuntimeError::bare_error(format!(
                "Can only call 'push' on a List, not a {typ}"
            )))?;
        list.push(value);
        Ok(list.into())
    });

    builtins
}
