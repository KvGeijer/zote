use crate::error::RunRes;

// OPT: Pack as bytesting instead? Very inefficiently stored now in 128 bits
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
}

impl Value {
    pub fn truthy(&self) -> RunRes<bool> {
        match self {
            Value::Nil => Ok(false),
            Value::Bool(bool) => Ok(bool),
            Value::Int(x) => Ok(x != 0),
            Value::Float(x) => Ok(x != 0.0), // not very useful
        }
    }
}
