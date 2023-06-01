use super::{
    super::{functions::Function, numerical::Numerical, RunRes, Value},
    index_wrap, slice_iter, IndexValue, SliceValue,
};

use std::{cell::RefCell, cmp::Ordering, rc::Rc};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct List {
    vec: Rc<RefCell<Vec<Value>>>,
}

impl List {
    pub fn new(values: Vec<Value>) -> Self {
        Self {
            vec: Rc::new(RefCell::new(values)),
        }
    }

    /// Pushes a value to the list
    pub fn push(&self, value: Value) {
        self.vec.borrow_mut().push(value);
    }

    /// Pops a value from the list
    pub fn pop(&self) -> Value {
        match self.vec.borrow_mut().pop() {
            Some(value) => value,
            None => Value::Nil,
        }
    }

    /// Checks if the list is empty
    pub fn is_empty(&self) -> bool {
        // Should this be added as a zote function? What name?
        self.vec.borrow().is_empty()
    }

    // /// Converts list to bool, which just checks if empty
    // pub fn to_bool(&self) -> bool {
    //     self.is_empty() == false.into()
    // }

    pub fn stringify(&self) -> String {
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

    pub fn get(&self, at: Value) -> Result<Value, String> {
        let index = if let Value::Numerical(Numerical::Int(index)) = at {
            index
        } else {
            return Err(format!(
                "Can only index into a list with an integer, but got {}",
                at.type_of()
            ));
        };

        let vec = self.vec.borrow();

        match vec.get(index_wrap(index, vec.len())).cloned() {
            Some(value) => Ok(value),
            None => Err(format!(
                "Index {index} not valid for length {} list",
                vec.len()
            )),
        }
    }

    /// Returns the max of the list, or Nil if empty
    pub fn max(&self) -> Result<Value, String> {
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

    pub fn map(&self, func: &Function) -> RunRes<Value> {
        let mut mapped = vec![];
        for value in self.vec.borrow().iter() {
            // Shoud we do something to the error info?
            mapped.push(func.call(vec![value.clone()])?);
        }
        Ok(mapped.into())
    }

    pub fn split(&self, delimiter: &Value) -> RunRes<Value> {
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
    pub fn sum(&self) -> Result<Value, String> {
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

    /// Sorts a list in descending order, using natural ordering of Value. Errors if two items not comparable
    pub fn sort(&self) -> Result<Value, String> {
        let mut success = Ok(());
        let mut vec = self.vec.borrow().clone();

        vec.sort_by(|a, b| match a.partial_cmp(b) {
            Some(order) => order.reverse(),
            None => {
                println!("Error!");
                success = Err(format!(
                    "Cannot sort a vector containing both {} and {}",
                    a.type_of(),
                    b.type_of()
                ));
                Ordering::Equal
            }
        });
        success.map(|_| vec.into())
    }

    pub fn slice(&self, slice: SliceValue) -> Result<Value, String> {
        let vec = self.vec.borrow();
        Ok(slice_iter(vec.iter().cloned(), slice, vec.len())?
            .collect::<Vec<Value>>()
            .into())
    }

    pub fn assign_into(&self, value: Value, index: IndexValue) -> Result<Value, String> {
        // A bit annoying to handle slicing...
        match index {
            IndexValue::At(Value::Numerical(ind)) => {
                let mut vec = self.vec.borrow_mut();
                let len = vec.len();
                let uind = index_wrap(ind.to_rint(), len);
                *vec.get_mut(uind).ok_or_else(|| {
                    format!("Index {} out of bounds for list of len {}", uind, len)
                })? = value.clone();
                Ok(value)
            }
            IndexValue::At(val) => Err(format!("Cannot index into list with a {}", val.type_of())),
            IndexValue::Slice(_slice) => {
                todo!("Not yet implemented slice assignment for lists")
                // let vec = self.vec.borrow_mut();
                // let vec_len = vec.len();
                // if slice_len(start, stop, step) != value.iter_len() {}

                // for (lvalue, rvalue) in
                //     slice_iter(vec.iter_mut(), start, stop, step, vec_len).zip(value.iter())
                // {
                //     *lvalue = rvalue;
                // }
                // self.vec.borrow_mut().get_mut(ind) = value;
            }
        }
    }
}
