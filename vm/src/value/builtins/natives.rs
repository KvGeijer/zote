use super::templates::BuiltinTemplate;
use super::Builtin;
use std::rc::Rc;

pub fn get_builtins() -> Vec<Rc<dyn Builtin>> {
    let mut builtins: Vec<Rc<dyn Builtin>> = vec![];

    builtins.new_1arg("print", |arg| {
        println!("{}", arg);
        Ok(arg)
    });

    builtins
}
