use std::{
    iter::{Skip, StepBy, Take},
    rc::Rc,
    vec,
};

use crate::{
    interpreter::RunError,
    parser::{ExprNode, Index, Slice},
};

use super::{environment::Environment, expressions, numerical::Numerical, value::Value, RunRes};

pub use list::List;

mod list;

#[derive(PartialEq, Debug, Clone)]
pub enum Collection {
    List(List),
    String(String), // TODO: Wrap in Rc & refcell (Should we wrap all collEnum in this?)
}

impl Collection {
    pub fn is_empty(&self) -> bool {
        match self {
            Collection::List(list) => list.is_empty(),
            Collection::String(string) => string.is_empty(),
        }
    }

    pub fn stringify(&self) -> String {
        match self {
            Collection::List(list) => list.stringify(),
            Collection::String(string) => string.to_string(),
        }
    }

    pub fn type_of(&self) -> &'static str {
        match self {
            Collection::List(_) => "List",
            Collection::String(_) => "String",
        }
    }

    pub fn new_string(string: String) -> Self {
        Self::String(string)
    }

    pub fn new_list(values: Vec<Value>) -> Self {
        Self::List(List::new(values))
    }

    pub fn assign_into(&self, rvalue: Value, index: IndexValue) -> RunRes<Value> {
        match self {
            Collection::List(list) => list.assign_into(rvalue, index),
            Collection::String(_) => {
                RunError::error("Assigning into string not implemented".to_string())
            }
        }
    }

    pub fn get(&self, index: IndexValue) -> RunRes<Value> {
        match (self, index) {
            (Collection::List(list), IndexValue::At(at)) => list.get(at),
            (Collection::List(list), IndexValue::Slice(slice)) => list.slice(slice),
            (Collection::String(string), IndexValue::At(Value::Numerical(num))) => string
                .chars()
                .nth(num.to_rint() as usize)
                .map(|char| char.to_string().into())
                .ok_or(RunError::bare_error(format!(
                    "Index {} out of bound for sting of len {}",
                    num.to_rint(),
                    string.len()
                ))),
            (Collection::String(_), IndexValue::At(other)) => {
                RunError::error(format!("Cannot index into string with {}", other.type_of()))
            }
            (Collection::String(string), IndexValue::Slice(slice)) => {
                let len = string.chars().count();
                let sliced: String = slice_iter(string.chars(), slice, len)?.collect();
                Ok(sliced.into())
            }
        }
    }

    pub fn to_iter(self) -> vec::IntoIter<Value> {
        match self {
            Collection::List(list) => list.to_iter(),
            Collection::String(string) => string
                .chars()
                .map(|char| char.to_string().into())
                .collect::<Vec<Value>>()
                .into_iter(),
        }
    }

    pub fn concat(self, other: Value) -> RunRes<Value> {
        match (self, other) {
            (Collection::List(list), other) => {
                let clone = list.deepclone();
                for item in other.to_iter()? {
                    clone.push(item);
                }
                Ok(clone.into())
            }
            (Collection::String(string), Value::Collection(Collection::String(other))) => {
                Ok((string + &other).into())
            }
            (Collection::String(string), Value::Numerical(num)) => {
                Ok((string + &num.to_rint().to_string()).into())
            }
            (left, right) => RunError::error(format!(
                "Cannot append {} to {}",
                right.type_of(),
                left.type_of()
            )),
        }
    }
}

impl PartialOrd for Collection {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Collection::List(_), Collection::List(_)) => None, // Could want to sort in some way here
            (Collection::String(x), Collection::String(y)) => x.partial_cmp(y),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IndexValue {
    At(Value),
    Slice(SliceValue),
}

#[derive(Debug, Clone)]
pub struct SliceValue {
    pub start: Option<Numerical>,
    pub stop: Option<Numerical>,
    pub step: Option<Numerical>,
}

impl SliceValue {
    fn new(start: Option<Numerical>, stop: Option<Numerical>, step: Option<Numerical>) -> Self {
        Self { start, stop, step }
    }
}

pub fn eval_index(index: &Index, env: &Rc<Environment>) -> RunRes<IndexValue> {
    match index {
        Index::At(expr) => Ok(IndexValue::At(expressions::eval(expr, env)?)),
        Index::Slice(slice) => Ok(IndexValue::Slice(eval_slice(slice, env)?)),
    }
}

pub fn eval_slice(
    Slice { start, stop, step }: &Slice,
    env: &Rc<Environment>,
) -> RunRes<SliceValue> {
    Ok(SliceValue::new(
        eval_opt_ind(start, env)?,
        eval_opt_ind(stop, env)?,
        eval_opt_ind(step, env)?,
    ))
}

fn eval_opt_ind(ind: &Option<ExprNode>, env: &Rc<Environment>) -> RunRes<Option<Numerical>> {
    match ind {
        Some(expr) => match expressions::eval(expr, env)? {
            Value::Numerical(num) => Ok(Some(num)),
            other => RunError::error(format!(
                "Expects slice index to be numerical, got {}",
                other.type_of()
            )),
        },
        None => Ok(None),
    }
}

// Helpers for all collections

fn index_wrap(index: i64, len: usize) -> usize {
    if index < 0 {
        index.rem_euclid(len as i64) as usize
    } else {
        index as usize
    }
}

/// Takes an iterator and zote numerical slice and iterates over it
pub fn slice_iter<T, I: Iterator<Item = T>>(
    iter: I,
    SliceValue { start, stop, step }: SliceValue,
    len: usize,
) -> RunRes<Take<StepBy<Skip<I>>>> {
    let start = index_wrap(start.map(|num| num.to_rint()).unwrap_or(0), len);
    let stop = index_wrap(stop.map(|num| num.to_rint()).unwrap_or(len as i64), len);
    let step = step.map(|num| num.to_rint()).unwrap_or(1);
    if step < 0 {
        RunError::error("Negatice steps in slices not implemented".to_string())
    } else {
        let steps = slice_len(start, stop, step)?;
        Ok(iter.skip(start).step_by(step as usize).take(steps))
    }
}

fn slice_len(start: usize, stop: usize, step: i64) -> RunRes<usize> {
    if step < 0 {
        RunError::error("Negatice steps in slices not implemented".to_string())
    } else {
        if stop > start {
            Ok(((stop - start) + (step - 1) as usize) / step as usize)
        } else {
            Ok(0)
        }
    }
}
