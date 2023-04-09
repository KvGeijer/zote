use super::expressions::Value;
use std::{cell::RefCell, collections::HashMap};

// Initially wanted to use mutable references for next and not have to use a RefCell. But it does not work.
// See https://stackoverflow.com/questions/60177810/lifetime-in-recursive-struct-with-mutable-reference)
pub struct Environment<'a> {
    values: RefCell<HashMap<String, Value>>,
    next: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self {
            values: RefCell::new(HashMap::new()),
            next: None,
        }
    }

    pub fn nest(&'a self) -> Self {
        Self {
            values: RefCell::new(HashMap::new()),
            next: Some(self),
        }
    }

    pub fn define(&self, name: String, value: Value) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        // Feels quite bad to clone so much if we use strings
        if let Some(value) = self.values.borrow().get(name) {
            Some(value.clone())
        } else if let Some(next) = self.next {
            next.get(name)
        } else {
            None
        }
    }

    pub fn assign(&self, name: &str, value: Value) -> Option<()> {
        if let Some(current) = self.values.borrow_mut().get_mut(name) {
            *current = value;
            Some(())
        } else if let Some(next) = self.next {
            next.assign(name, value)
        } else {
            None
        }
    }
}
