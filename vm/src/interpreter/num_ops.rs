// Module for handling numerical operations on values

use super::NIL;
use crate::{
    error::{RunRes, RunResTrait},
    value::Value,
};

fn promote(x: Value, y: Value) -> RunRes<(Value, Value)> {
    if x == NIL || y == NIL {
        return RunRes::new_err(format!("Numerical operations cannot operate on Nil values"));
    }

    // TODO: More types
    let promoted = match (x, y) {
        (Value::Bool(x), Value::Bool(y)) => (Value::Int(x as i64), Value::Int(y as i64)),
        (Value::Bool(x), Value::Float(y)) => (Value::Float(x as i64 as f64), Value::Float(y)),
        (Value::Bool(x), Value::Int(y)) => (Value::Int(x as i64), Value::Int(y)),
        (Value::Float(x), Value::Bool(y)) => (Value::Float(x), Value::Float(y as i64 as f64)),
        (Value::Float(x), Value::Float(y)) => (Value::Float(x), Value::Float(y as f64)),
        (Value::Float(x), Value::Int(y)) => (Value::Float(x), Value::Float(y as f64)),
        (Value::Int(x), Value::Bool(y)) => (Value::Int(x), Value::Int(y as i64)),
        (Value::Int(x), Value::Float(y)) => (Value::Float(x as f64), Value::Float(y)),
        (Value::Int(x), Value::Int(y)) => (Value::Int(x), Value::Int(y)),
        (x, y) => unimplemented!(
            "Numerical promotion not implemented for {:?} and {:?}",
            x,
            y
        ),
    };

    Ok(promoted)
}

pub fn add(x: Value, y: Value) -> RunRes<Value> {
    match promote(x, y)? {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
        (_, _) => panic!("Internal error with promote arms"),
    }
}

pub fn sub(x: Value, y: Value) -> RunRes<Value> {
    match promote(x, y)? {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
        (_, _) => panic!("Internal error with promote arms"),
    }
}

pub fn mult(x: Value, y: Value) -> RunRes<Value> {
    match promote(x, y)? {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
        (_, _) => panic!("Internal error with promote arms"),
    }
}

pub fn div(x: Value, y: Value) -> RunRes<Value> {
    match promote(x, y)? {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x / y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x / y)),
        (_, _) => panic!("Internal error with promote arms"),
    }
}

pub fn modulo(x: Value, y: Value) -> RunRes<Value> {
    match promote(x, y)? {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x.rem_euclid(y))),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x.rem_euclid(y))),
        (_, _) => panic!("Internal error with promote arms"),
    }
}

// ERROR: There might be a problem with overflow here?
pub fn power(x: Value, y: Value) -> RunRes<Value> {
    match promote(x, y)? {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x.powf(y))),
        (Value::Int(x), Value::Int(y)) if y >= 0 => {
            let safe_x: u64 = x.unsigned_abs(); // TODO Handle overflows as zote errors
            let pow = safe_x.pow(y.unsigned_abs() as u32) as i64;
            if x >= 0 || y & 1 == 0 {
                Ok(Value::Int(pow))
            } else {
                Ok(Value::Int(-pow))
            }
        }
        (Value::Int(x), Value::Int(y)) => Ok(Value::Float((x as f64).powf(y as f64))),
        (_, _) => panic!("Internal error with promote arms"),
    }
}

pub fn negate(x: Value) -> RunRes<Value> {
    match x {
        Value::Nil => return RunRes::new_err("Cannot negate Nil".to_string()),
        Value::Bool(x) => Ok(Value::Int(-(x as i64))),
        Value::Float(x) => Ok(Value::Float(-x)),
        Value::Int(x) => Ok(Value::Int(-x)),
    }
}
