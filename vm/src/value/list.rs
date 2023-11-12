use std::cell::RefCell;

use crate::error::{RunRes, RunResTrait, RuntimeError};

use super::Value;

/// A list of values
///
/// It can be modified from non-mutable references from
/// several places at once. Not thread safe though.
#[derive(Debug)]
pub struct List {
    vec: RefCell<Vec<Value>>,
}

impl List {
    /// The length of the list
    pub fn len(&self) -> usize {
        self.vec.borrow().len()
    }

    /// If the list has values
    pub fn truthy(&self) -> bool {
        !self.empty()
    }

    /// Is the list empty?
    pub fn empty(&self) -> bool {
        self.len() == 0
    }

    /// Sets the value at the index, potentially wrapping for negative values
    pub fn set(&self, index: i64, value: Value) -> RunRes<()> {
        let vec = self.vec.borrow_mut();
        let len = vec.len();

        let uindex = index_wrap(index, len)?;
        self.vec.borrow_mut()[uindex] = value;

        Ok(())
    }

    /// Gets the value at the index, potentially wrapping for negative values
    pub fn get(&self, index: i64) -> RunRes<Value> {
        let vec = self.vec.borrow_mut();
        let len = vec.len();

        let uindex = index_wrap(index, len)?;
        Ok(vec[uindex].clone())
    }

    /// Pushes a value to the end of the list
    pub fn push(&self, value: Value) {
        self.vec.borrow_mut().push(value)
    }

    /// Pops the value at the end of the list
    pub fn pop(&self) -> RunRes<Value> {
        self.vec.borrow_mut().pop().ok_or(RuntimeError::bare_error(
            "Cannot pop an empty list".to_string(),
        ))
    }

    /// Creates a list from a pythonic slice
    ///
    /// Can iterate backwards if the step is negative.
    /// Returns an empty list if it will not converge
    pub fn from_slice(start: i64, stop: i64, step: i64) -> Self {
        if start >= stop && step >= 0 || start <= stop && step <= 0 {
            return vec![].into();
        }

        let mut vec = vec![];
        let mut pos = start;
        while (pos.cmp(&stop)) == (start.cmp(&stop)) {
            vec.push(pos.into());
            pos += step;
        }
        vec.into()
    }
}

impl From<Vec<Value>> for List {
    fn from(value: Vec<Value>) -> Self {
        Self {
            vec: RefCell::new(value),
        }
    }
}

/// Returns the wrapped index into a list, if it is not outside the list
fn index_wrap(index: i64, len: usize) -> RunRes<usize> {
    if index < 0 {
        let wrapped = len as i64 + index;
        if wrapped >= 0 {
            Ok(wrapped as usize)
        } else {
            RunRes::new_err(format!("If indexing with a negative value ({index}), it must not exceed the list length ({len})"))
        }
    } else {
        if (index as usize) < len {
            Ok(index as usize)
        } else {
            RunRes::new_err(format!(
                "Cannot index outside of a list (index: {index}, len {len})"
            ))
        }
    }
}
