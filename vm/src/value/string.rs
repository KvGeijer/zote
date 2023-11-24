use std::{cell::RefCell, fmt::Display, hash::Hash};

use crate::error::{RunRes, RunResTrait};

use super::Value;

#[derive(Debug, Clone)]
pub struct ValueString {
    string: RefCell<Vec<u8>>,
}

impl ValueString {
    pub fn len(&self) -> usize {
        self.string.borrow().len()
    }

    pub fn truthy(&self) -> bool {
        self.len() != 0
    }

    pub fn set(&self, index: i64, value: Value) -> RunRes<()> {
        let mut string = self.string.borrow_mut();

        let uindex = index_wrap(index, string.len());

        let byte = value.to_char()?;
        string[uindex] = byte;

        Ok(())
    }

    pub fn get(&self, index: i64) -> RunRes<Value> {
        let string = self.string.borrow();

        let uindex = index_wrap(index, string.len());

        Ok(ValueString::from(string[uindex]).into())
    }

    pub fn push(&self, value: Value) -> RunRes<()> {
        let byte = value.to_char()?;
        self.string.borrow_mut().push(byte);
        Ok(())
    }

    pub fn pop(&self) -> RunRes<Value> {
        if let Some(byte) = self.string.borrow_mut().pop() {
            Ok(ValueString::from(byte).into())
        } else {
            RunRes::new_err(format!("Cannot pop from an empty string"))
        }
    }

    pub fn to_char(&self) -> RunRes<u8> {
        if self.string.borrow().len() == 1 {
            Ok(self.string.borrow()[0])
        } else {
            RunRes::new_err(format!("Cannot convert {} to char", self))
        }
    }

    /// Appends two strings, creating a new one with copied contents
    pub fn append(&self, other: &Self) -> Self {
        let mut new_vec = vec![];
        for value in self.string.borrow().iter() {
            new_vec.push(value.clone());
        }
        for value in other.string.borrow().iter() {
            new_vec.push(value.clone());
        }
        Self {
            string: RefCell::new(new_vec),
        }
    }
}

impl From<String> for ValueString {
    fn from(string: String) -> Self {
        ValueString {
            string: RefCell::new(string.into_bytes()),
        }
    }
}

impl From<&str> for ValueString {
    fn from(string: &str) -> Self {
        ValueString {
            string: RefCell::new(string.as_bytes().to_vec()),
        }
    }
}

impl From<u8> for ValueString {
    fn from(byte: u8) -> Self {
        Self {
            string: RefCell::new(vec![byte]),
        }
    }
}

impl Display for ValueString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.string.borrow()))
    }
}

impl PartialEq for ValueString {
    fn eq(&self, other: &Self) -> bool {
        let other_ref = other.string.borrow();
        let other_str: &Vec<u8> = other_ref.as_ref();
        self.string.borrow().eq(other_str)
    }
}
impl Eq for ValueString {}

impl PartialOrd for ValueString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.string.borrow().partial_cmp(&other.string.borrow())
    }
}

/// Returns the wrapped index into a string. Does not handle out of bounds
fn index_wrap(index: i64, len: usize) -> usize {
    // Kept separate from list function in case they diverge
    if index < 0 {
        let wrapped = len as i64 + index;
        if wrapped >= 0 {
            wrapped as usize
        } else {
            // Just return 0 as maximum wrapping
            0
        }
    } else {
        index as usize
    }
}

impl Hash for ValueString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let vec = self.string.borrow();
        for c in vec.iter().take(2) {
            c.hash(state);
        }

        for c in vec.iter().rev().take(2) {
            c.hash(state);
        }
    }
}
