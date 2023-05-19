use super::{functions::Function, numerical::Numerical, RunRes, Value};

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
    pub(super) fn push(&self, value: Value) {
        self.vec.borrow_mut().push(value);
    }

    /// Pops a value from the list
    pub(super) fn pop(&self) -> Value {
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

    pub(super) fn map(&self, func: &Function) -> RunRes<Value> {
        let mut mapped = vec![];
        for value in self.vec.borrow().iter() {
            // Shoud we do something to the error info?
            mapped.push(func.call(vec![value.clone()])?);
        }
        Ok(mapped.into())
    }

    pub(super) fn split(&self, delimiter: &Value) -> RunRes<Value> {
        let mut splitted = vec![];
        let mut sublist = vec![];
        for value in self.vec.borrow().iter() {
            if value == delimiter {
                splitted.push(sublist.into());
                sublist = vec![];
            } else {
                sublist.push(value.clone());
            }
        }
        if !sublist.is_empty() {
            splitted.push(sublist.into());
        }
        Ok(splitted.into())
    }

    /// Sums a list with numericals. Errors if any nonnumerical.
    pub(super) fn sum(&self) -> Result<Value, String> {
        let mut sum: Numerical = 0.into();
        for val in self.vec.borrow().iter() {
            match val {
                Value::Numerical(num) => sum = sum.add(*num),
                val => {
                    return Err(format!(
                        "List.sum only implemented for numbers, but got {}",
                        val.type_of()
                    ));
                }
            }
        }
        Ok(sum.into())
    }
}
