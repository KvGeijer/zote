use std::cell::{Ref, RefCell};

use itertools::Itertools;

use crate::error::{RunRes, RunResTrait, RuntimeError};

use super::{Closure, Dictionary, Value};

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
        !self.is_empty()
    }

    /// Is the list empty?
    pub fn is_empty(&self) -> bool {
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
        // println!("Getting {index} in list {:?}", self);
        let vec = self.vec.borrow();
        let len = vec.len();

        let uindex = index_wrap(index, len);
        match vec.get(uindex) {
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

    /// Appends two lists, creating a new one with copied contents
    pub fn append(&self, other: &Self) -> Self {
        let mut new_vec = vec![];
        for value in self.vec.borrow().iter() {
            new_vec.push(value.clone());
        }
        for value in other.vec.borrow().iter() {
            new_vec.push(value.clone());
        }
        new_vec.into()
    }

    /// Deeply clones all contained values
    pub fn deepclone(&self) -> List {
        self.vec
            .borrow()
            .iter()
            .map(|val| val.deepclone())
            .collect::<Vec<Value>>()
            .into()
    }

    /// Borrows a reference to the slice
    /// This is a very dangerous function, as we can hand out several references
    pub fn borrow_slice(&self) -> Ref<Vec<Value>> {
        self.vec.borrow()
    }

    /// Sorts the list in ascending order
    pub fn sort(&self) -> RunRes<Self> {
        let mut vec = self.vec.borrow().clone();
        let mut errors: Vec<String> = vec![];

        vec.sort_by(|a, b| {
            a.partial_cmp(&b).unwrap_or_else(|| {
                errors.push(format!(
                    "ERROR: Trying to sort with both {} and {}",
                    a.type_of(),
                    b.type_of()
                ));
                std::cmp::Ordering::Equal
            })
        });

        if errors.is_empty() {
            Ok(vec.into())
        } else {
            RunRes::new_err(errors.into_iter().next().unwrap())
        }
    }

    /// Sorts the list in ascending order, by the comparator supplied
    pub fn sort_by(&self, _cmp: &Closure) -> RunRes<Self> {
        // TODO: We need access to the vm for this, as we need to call custom functions within this one
        RunRes::new_err("Sorting by a custom function not yet implemented in vm".to_string())
    }

    pub fn split(&self, delim: Value) -> List {
        let mut splits: Vec<Value> = vec![];
        let mut last_start_ind = 0;
        let vec = self.vec.borrow();
        for (ind, value) in vec.iter().enumerate() {
            if value == &delim {
                // split, unless emtpy
                if ind != last_start_ind {
                    splits.push(
                        List::from(vec[last_start_ind..ind].iter().cloned().collect_vec()).into(),
                    )
                }
                last_start_ind = ind + 1;
            }
        }
        // Possible last split
        if last_start_ind != vec.len() {
            splits.push(List::from(vec[last_start_ind..].iter().cloned().collect_vec()).into())
        }

        splits.into()
    }

    /// Tries to convert into dict (cannot use TryInto, as we want both set and dict which are the same type)
    pub fn try_into_dict(&self) -> RunRes<Dictionary> {
        // TODO: make this better
        let dict = Dictionary::new();
        for value in self.vec.borrow().iter() {
            let Value::List(list) = value else {
                return RuntimeError::error(format!("When initializing a dict from a list, the list must be a list of key-value pair lists."));
            };

            if let Some((key, val)) = list.vec.borrow().iter().cloned().collect_tuple() {
                dict.insert(key, val)?;
            } else {
                return RuntimeError::error(format!("When initializing a dict from a list, the list must be a list of key-value pair lists."));
            }
        }

        Ok(dict)
    }

    /// Tries to convert into set (cannot use TryInto, as we want both set and dict which are the same type)
    pub fn try_into_set(&self) -> RunRes<Dictionary> {
        // TODO: make this better
        let dict = Dictionary::new();
        for value in self.vec.borrow().iter() {
            dict.insert(value.clone(), Value::Nil)?;
        }

        Ok(dict)
    }
}

impl From<Vec<Value>> for List {
    fn from(value: Vec<Value>) -> Self {
        Self {
            vec: RefCell::new(value),
        }
    }
}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        let other_ref = other.vec.borrow();
        let other_vec: &Vec<Value> = other_ref.as_ref();
        self.vec.borrow().eq(other_vec)
    }
}
impl Eq for List {}

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

// A shallow clone, that only clones the values in the top list
impl Clone for List {
    fn clone(&self) -> Self {
        Self {
            vec: RefCell::new(self.vec.borrow().iter().cloned().collect()),
        }
    }
}
