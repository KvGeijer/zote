use std::cmp::Ordering;

use crate::{
    error::{RunRes, RunResTrait},
    value::Value,
};

pub fn equal(x: Value, y: Value) -> RunRes<Value> {
    Ok(Value::Bool(x.eq(&y)))
}

pub fn not_equal(x: Value, y: Value) -> RunRes<Value> {
    Ok(Value::Bool(!x.eq(&y)))
}

// Helper function for orderings
fn cmp(x: &Value, y: &Value) -> RunRes<Ordering> {
    match x.partial_cmp(y) {
        Some(ord) => Ok(ord),
        None => RunRes::new_err(format!("Cannot order {} and {}", x.type_of(), y.type_of())),
    }
}

pub fn less(x: Value, y: Value) -> RunRes<Value> {
    Ok(match cmp(&x, &y)? {
        Ordering::Less => Value::Bool(true),
        _ => Value::Bool(false),
    })
}

pub fn greater(x: Value, y: Value) -> RunRes<Value> {
    Ok(match cmp(&x, &y)? {
        Ordering::Greater => Value::Bool(true),
        _ => Value::Bool(false),
    })
}

pub fn less_eq(x: Value, y: Value) -> RunRes<Value> {
    Ok(match cmp(&x, &y)? {
        Ordering::Greater => Value::Bool(false),
        _ => Value::Bool(true),
    })
}

pub fn greater_eq(x: Value, y: Value) -> RunRes<Value> {
    Ok(match cmp(&x, &y)? {
        Ordering::Less => Value::Bool(false),
        _ => Value::Bool(true),
    })
}
