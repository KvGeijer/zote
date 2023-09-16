use std::fmt::Display;

use crate::error::RunRes;

// OPT: Pack as bytesting instead? Very inefficiently stored now in 128 bits
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
}

pub enum ValueType {
    Nil,
    Bool,
    Int,
    Float,
}

impl Value {
    pub fn type_of(&self) -> ValueType {
        match self {
            Value::Nil => ValueType::Nil,
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
        }
    }

    pub fn truthy(&self) -> RunRes<bool> {
        match self {
            Value::Nil => Ok(false),
            Value::Bool(bool) => Ok(*bool),
            Value::Int(x) => Ok(*x != 0),
            Value::Float(x) => Ok(*x != 0.0), // not very useful
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // TODO: Implement other values
        match (self, other) {
            (Value::Bool(x), Value::Bool(y)) => x.partial_cmp(y),
            (Value::Int(x), Value::Int(y)) => x.partial_cmp(y),
            (Value::Int(x), Value::Float(y)) => (*x as f64).partial_cmp(y),
            (Value::Float(x), Value::Int(y)) => x.partial_cmp(&(*y as f64)),
            (Value::Float(x), Value::Float(y)) => x.partial_cmp(y),
            _ => None,
        }
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Nil => write!(f, "Nil"),
            ValueType::Bool => write!(f, "Bool"),
            ValueType::Int => write!(f, "Int"),
            ValueType::Float => write!(f, "Float"),
        }
    }
}
