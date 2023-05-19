use super::Value;

use std::{cell::RefCell, cmp::Ordering, rc::Rc};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub(super) struct List {
    vec: Rc<RefCell<Vec<Value>>>,
}

impl List {
    pub(super) fn new<T: Iterator<Item = Value>>(values: T) -> Self {
        Self {
            vec: Rc::new(RefCell::new(Vec::from_iter(values))),
        }
    }

    /// Pushes a value to the list
    pub(super) fn push(&mut self, value: Value) {
        self.vec.borrow_mut().push(value);
    }

    /// Pops a value from the list
    pub(super) fn pop(&mut self) -> Value {
        match self.vec.borrow_mut().pop() {
            Some(value) => value,
            None => Value::Nil,
        }
    }

    /// Checks if the list is empty
    pub(super) fn is_empty(&self) -> Value {
        // Should this be added as a zote function? What name?
        self.vec.borrow().is_empty().into()
    }

    /// Converts list to bool, which just checks if empty
    pub(super) fn to_bool(&self) -> bool {
        self.is_empty() == false.into()
    }

    pub(super) fn stringify(&self) -> String {
        let mut string = String::from("[");
        let mut first = true;
        for value in self.vec.borrow().iter() {
            if !first {
                string.push_str(", ");
            } else {
                first = false;
            }
            string.push_str(&value.stringify());
        }
        string.push(']');
        string
    }

    pub(super) fn get(&self, index: i64) -> Result<Value, String> {
        let vec = self.vec.borrow();

        let uindex = if index < 0 {
            index.rem_euclid(vec.len() as i64) as usize
        } else {
            index as usize
        };

        match vec.get(uindex).cloned() {
            Some(value) => Ok(value),
            None => Err(format!(
                "Index {index} not valid for length {} list",
                vec.len()
            )),
        }
    }

    /// Returns the max of the list, or Nil if empty
    pub(super) fn max(&self) -> Result<Value, String> {
        let vec = self.vec.borrow();
        let mut iter = vec.iter();
        let mut max = iter.next().cloned().unwrap_or(Value::Nil);
        for val in iter {
            match max.partial_cmp(val) {
                Some(Ordering::Less) => max = val.clone(),
                None => {
                    return Err("Cannot compare {} with {}. For finding max in a list.".to_string())
                }
                _ => (),
            }
        }
        Ok(max)
    }
}
