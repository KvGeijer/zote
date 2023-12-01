use crate::{error::RunRes, value::Value};

pub fn not(value: Value) -> RunRes<Value> {
    Ok(Value::Bool(!value.truthy()?))
}
