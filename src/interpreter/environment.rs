use super::expressions::Value;
use std::collections::HashMap;

pub struct Environment<'a> {
    values: HashMap<String, Value>,
    next: Option<&'a mut Environment<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            next: None,
        }
    }

    pub fn nest(&'a mut self) -> Self {
        Self {
            values: HashMap::new(),
            next: Some(self),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&mut self, name: &str) -> Option<Value> {
        // Feels quite bad to clone so much if we use strings
        self.values.get(name).cloned()
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Option<()> {
        if let Some(current) = self.values.get_mut(name) {
            *current = value;
            Some(())
        } else if let Some(next) = self.next.as_mut() {
            next.assign(name, value)
        } else {
            None
        }
    }
}
