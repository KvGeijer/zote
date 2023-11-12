use std::cell::RefCell;

use itertools::Itertools;

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
        let mut vec = self.vec.borrow_mut();
        let len = vec.len();

        let uindex = index_wrap(index, len);
        match vec.get_mut(uindex) {
            Some(entry) => {
                *entry = value;
                Ok(())
            }
            None => RunRes::new_err(format!(
                "Index {index} out of bound for list of length {len}."
            )),
        }
    }

    /// Gets the value at the index, potentially wrapping for negative values
    pub fn get(&self, index: i64) -> RunRes<Value> {
        let mut vec = self.vec.borrow_mut();
        let len = vec.len();

        let uindex = index_wrap(index, len);
        match vec.get_mut(uindex) {
            Some(entry) => Ok(entry.clone()),
            None => RunRes::new_err(format!(
                "Index {index} out of bound for list of length {len}."
            )),
        }
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
    pub fn from_slice(start: i64, stop: i64, step: i64) -> RunRes<Self> {
        if empty_solo_slice(start, stop, step) {
            return Ok(vec![].into());
        } else if step == 0 {
            return RunRes::new_err("Cannot have stepsize 0 in slice".to_owned());
        }

        let mut vec = vec![];
        let mut pos = start;
        while (pos.cmp(&stop)) == (start.cmp(&stop)) {
            vec.push(pos.into());
            pos += step;
        }
        Ok(vec.into())
    }

    /// Constructs a new list, from a slice of this one
    pub fn slice(&self, start: Option<i64>, stop: Option<i64>, step: Option<i64>) -> RunRes<Self> {
        let step = step.unwrap_or(1);

        let vec = self.vec.borrow();
        let mut slice = vec![];
        for ind in slice_iter(start, stop, step, vec.len())? {
            slice.push(vec[ind].clone())
        }

        Ok(slice.into())
    }
}

impl From<Vec<Value>> for List {
    fn from(value: Vec<Value>) -> Self {
        Self {
            vec: RefCell::new(value),
        }
    }
}

/// Returns the wrapped index into a list. Does not handle out of bounds
fn index_wrap(index: i64, len: usize) -> usize {
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

/// Calculates the indeces to use for iterating over a slice
pub fn slice_iter(
    start: Option<i64>,
    stop: Option<i64>,
    step: i64,
    len: usize,
) -> RunRes<impl Iterator<Item = usize>> {
    if step > 0 {
        let start = start.map(|ind| index_wrap(ind, len)).unwrap_or(0);
        let stop = stop.map(|ind| index_wrap(ind, len)).unwrap_or(len);

        Ok((start..stop)
            .step_by(step as usize)
            .filter(|&ind| ind < len)
            .collect_vec()
            .into_iter())
    } else if step < 0 {
        let start = start
            .map(|ind| index_wrap(ind, len))
            .unwrap_or(if len != 0 { len - 1 } else { 0 });
        let stop = stop.map(|ind| index_wrap(ind, len) + 1).unwrap_or(0);

        Ok((stop..=start)
            .rev()
            .step_by((-step) as usize)
            .filter(|&ind| ind < len)
            .collect_vec()
            .into_iter())
    } else {
        return RunRes::new_err("Cannot have stepsize 0 in slice".to_owned());
    }
}

fn empty_solo_slice(start: i64, stop: i64, step: i64) -> bool {
    start >= stop && step > 0 || start <= stop && step < 0
}
