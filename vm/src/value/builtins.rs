use std::{
    fmt::{self, Debug, Formatter},
    rc::Rc,
};

use crate::error::{RunRes, RunResTrait};

use self::natives::get_builtins;

use super::Value;

mod natives;
mod templates;

#[derive(Clone)]
pub struct Native {
    builtin: Rc<dyn Builtin>,
}

impl Native {
    pub fn call(&self, args: Vec<Value>) -> RunRes<Value> {
        if self.builtin.accept_arity(args.len()) {
            self.builtin.run(args)
        } else {
            RunRes::new_err(format!(
                "Incorrect arg count: {} expected {} args, but got {}",
                self.name(),
                self.arity(),
                args.len()
            ))
        }
    }

    pub fn name(&self) -> &str {
        self.builtin.name()
    }

    pub fn arity(&self) -> &str {
        self.builtin.arity()
    }
}

impl Debug for Native {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&format!("Builtin({}/{})", self.name(), self.arity()))
    }
}

pub trait Builtin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value>;
    fn accept_arity(&self, arity: usize) -> bool;
    fn name(&self) -> &str;
    fn arity(&self) -> &str;
}

pub fn get_natives() -> Vec<Native> {
    get_builtins()
        .into_iter()
        .map(|rc| Native { builtin: rc })
        .collect()
}
