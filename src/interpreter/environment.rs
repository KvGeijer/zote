use super::value::Value;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

// Initially wanted to use mutable references for next and not have to use a RefCell. But it does not work.
// See https://stackoverflow.com/questions/60177810/lifetime-in-recursive-struct-with-mutable-reference)
pub struct Environment {
    values: RefCell<HashMap<String, Value>>,
    next: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            values: RefCell::new(HashMap::new()),
            next: None,
        })
    }

    pub fn nest(next: &Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            values: RefCell::new(HashMap::new()),
            next: Some(Rc::clone(next)),
        })
    }

    pub fn define(&self, name: String, value: Value) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        // Feels quite bad to clone so much if we use strings
        if let Some(value) = self.values.borrow().get(name) {
            Some(value.clone())
        } else if let Some(next) = &self.next {
            next.get(name)
        } else {
            None
        }
    }

    pub fn assign(&self, name: &str, value: Value) -> Option<()> {
        if let Some(current) = self.values.borrow_mut().get_mut(name) {
            *current = value;
            Some(())
        } else if let Some(next) = &self.next {
            next.assign(name, value)
        } else {
            None
        }
    }
}
