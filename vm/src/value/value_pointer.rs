use std::{cell::RefCell, rc::Rc};

use super::Value;

/// A mutable reference to a value
#[derive(Debug, Clone)]
pub struct ValuePointer {
    pointer: Rc<RefCell<Value>>,
}

impl ValuePointer {
    /// Initiates the pointer to point at NIL
    pub fn new() -> Self {
        Self {
            pointer: Rc::new(RefCell::new(Value::Nil)),
        }
    }

    pub fn set(&self, value: Value) {
        *self.pointer.borrow_mut() = value;
    }

    pub fn get_clone(&self) -> Value {
        self.pointer.borrow().clone()
    }
}
