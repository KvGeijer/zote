use itertools::Itertools;
use std::rc::Rc;

use crate::{error::RunRes, value::Value};

use super::Builtin;

/// Trait for more easily adding builtins with a certain number of args
pub trait BuiltinTemplate {
    fn new_0arg(&mut self, name: &'static str, func: impl Fn() -> RunRes<Value> + 'static);
    fn new_1arg(&mut self, name: &'static str, func: impl Fn(Value) -> RunRes<Value> + 'static);
    fn new_2arg(
        &mut self,
        name: &'static str,
        func: impl Fn(Value, Value) -> RunRes<Value> + 'static,
    );
    fn new_3arg(
        &mut self,
        name: &'static str,
        func: impl Fn(Value, Value, Value) -> RunRes<Value> + 'static,
    );
    fn new_any_arg(
        &mut self,
        name: &'static str,
        func: impl Fn(Vec<Value>) -> RunRes<Value> + 'static,
    );
}

impl BuiltinTemplate for Vec<Rc<dyn Builtin>> {
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

    fn new_3arg(
        &mut self,
        name: &'static str,
        func: impl Fn(Value, Value, Value) -> RunRes<Value> + 'static,
    ) {
        let builtin = Rc::new(ThreeArgBuiltin {
            name,
            func: Box::new(func),
        });
        self.push(builtin);
    }

    fn new_any_arg(
        &mut self,
        name: &'static str,
        func: impl Fn(Vec<Value>) -> RunRes<Value> + 'static,
    ) {
        let builtin = Rc::new(AnyArgBuiltin {
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

struct ThreeArgBuiltin {
    name: &'static str,
    func: Box<dyn Fn(Value, Value, Value) -> RunRes<Value>>,
}

impl Builtin for ThreeArgBuiltin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        let (x, y, z) = args
            .into_iter()
            .tuples()
            .next()
            .expect("Incorrect number of args");
        (self.func)(x, y, z)
    }

    fn accept_arity(&self, arity: usize) -> bool {
        arity == 3
    }

    fn name(&self) -> &str {
        self.name
    }

    fn arity(&self) -> &str {
        "3"
    }
}

struct AnyArgBuiltin {
    name: &'static str,
    func: Box<dyn Fn(Vec<Value>) -> RunRes<Value>>,
}

impl Builtin for AnyArgBuiltin {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        (self.func)(args)
    }

    fn accept_arity(&self, _arity: usize) -> bool {
        true
    }

    fn name(&self) -> &str {
        self.name
    }

    fn arity(&self) -> &str {
        "any"
    }
}
