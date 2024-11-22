use std::{cell::RefCell, fmt::Display, hash::Hash};

use crate::error::{RunRes, RunResTrait, RuntimeError};

use super::{List, Value};

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
        let len = string.len();

        let uindex = index_wrap(index, len);

        match string.get(uindex) {
            Some(&entry) => Ok(ValueString::from(entry).into()),
            None => RunRes::new_err(format!(
                "Index {index} out of bound for list of length {len}."
            )),
        }
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
        let mut new = self.string.borrow().clone();
        new.extend(other.string.borrow().iter());
        Self {
            string: RefCell::new(new),
        }
    }

    /// Constructs a new string from a slice of this one
    pub fn slice(&self, start: Option<i64>, stop: Option<i64>, step: Option<i64>) -> RunRes<Self> {
        let step = step.unwrap_or(1);

        let string = self.string.borrow();
        let mut slice = vec![];
        for ind in super::list::slice_iter(start, stop, step, string.len())? {
            slice.push(string[ind])
        }

        Ok(slice.into())
    }

    /// Tries to parse the string as an int
    pub fn parse_int(&self) -> RunRes<i64> {
        self.to_string()
            .parse()
            .map_err(|reason| RuntimeError::bare_error(format!("Failed parsing an int: {reason}")))
    }

    /// Tries to parse the string as a float
    pub fn parse_float(&self) -> RunRes<f64> {
        self.to_string()
            .parse()
            .map_err(|reason| RuntimeError::bare_error(format!("Failed parsing an int: {reason}")))
    }

    /// Splits string into a list around dgiven delimiter. Cannot get empty entries
    pub fn split(&self, delimiter: Value) -> RunRes<Vec<ValueString>> {
        let Some(str_delim) = delimiter.to_valuestring() else {
            return RunRes::new_err(format!("Can only split string around another string."));
        };

        let mut splits: Vec<ValueString> = vec![];
        let mut last_start_ind = 0;

        let string = self.string.borrow();
        let other = str_delim.string.borrow();

        let mut start_ind = 0;
        while start_ind + other.len() <= string.len() {
            if string[start_ind..(start_ind + other.len())] == other[..] {
                if last_start_ind != start_ind {
                    splits.push(string[last_start_ind..start_ind].to_vec().into())
                }
                start_ind += str_delim.len();
                last_start_ind = start_ind;
            } else {
                start_ind += 1;
            }
        }
        if last_start_ind < string.len() {
            splits.push(string[last_start_ind..].to_vec().into());
        }

        Ok(splits.into())
    }

    /// Checks if the string contains a char or another string
    pub fn contains_subsequence(&self, value: Value) -> RunRes<bool> {
        if let Ok(byte) = value.to_char() {
            return Ok(self.string.borrow().contains(&(byte as u8)));
        }

        let kind = value.type_of();
        let Some(other_str) = value.to_valuestring() else {
            return RunRes::new_err(format!(
                "Can only check if a char, or a string is contained within another string. Got {}.",
                kind
            ));
        };

        Ok(self.to_string().contains(&other_str.to_string()))
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

impl From<Vec<u8>> for ValueString {
    fn from(value: Vec<u8>) -> Self {
        Self {
            string: RefCell::new(value),
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

impl Into<List> for &ValueString {
    fn into(self) -> List {
        self.to_string()
            .chars()
            .map(|char| ValueString::from(char as u8).into())
            .collect::<Vec<Value>>()
            .into()
    }
}
