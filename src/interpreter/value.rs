use std::{fmt, vec};

use super::{
    collections::{Collection, List},
    functions::Function,
    numerical::Numerical,
    runtime_error::{RunError, RunRes},
};

// An interface between Zote and Rust values
#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Numerical(Numerical),
    Collection(Collection),
    Callable(Function),
    Nil,
    Uninitialized,
}

impl Value {
    pub fn truthy(&self) -> bool {
        match self {
            Value::Numerical(num) => num.truthy(),
            Value::Collection(collection) => !collection.is_empty(),
            Value::Callable(_) => panic!("Can't convert function to bool"), // TODO: real error, or just warning
            Value::Nil => false,
            Value::Uninitialized => false,
        }
    }

    pub fn stringify(&self) -> String {
        // OPT Could we just return &str here?
        match self {
            Value::Numerical(num) => num.stringify(),
            Value::Collection(collection) => collection.stringify(),
            Value::Callable(callable) => callable.name().to_string(),
            Value::Nil => "Nil".to_string(),
            Value::Uninitialized => panic!("Use of uninit value!"),
        }
    }

    // TODO: If this should be used in the code, it should be an enum
    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Numerical(num) => num.type_of(),
            Value::Collection(collection) => collection.type_of(),
            Value::Callable(_) => "Function",
            Value::Nil => "Nil",
            Value::Uninitialized => "Uninitialized",
        }
    }

    pub fn to_iter(self) -> RunRes<vec::IntoIter<Value>> {
        match self {
            Value::Collection(collection) => Ok(collection.to_iter()),
            other => RunError::error(format!("Cannot convert {} to an iterator", other.type_of())),
        }
    }
}

// Simple print with the value wrapped in its type, for informative prints
// Why do we need this and stringify?
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Callable(callable) => write!(f, "fn {}/{}", callable.name(), callable.arity()),
            Value::Nil => write!(f, "Nil"),
            Value::Uninitialized => panic!("Use of uninit value!"),
            value => write!(f, "{}({})", value.type_of(), value.stringify()),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Numerical(x), Value::Numerical(y)) => x.partial_cmp(y),
            (Value::Collection(x), Value::Collection(y)) => x.partial_cmp(y),
            _ => None,
        }
    }
}

impl From<Numerical> for Value {
    fn from(item: Numerical) -> Self {
        Value::Numerical(item)
    }
}

impl From<i64> for Value {
    fn from(item: i64) -> Self {
        let num: Numerical = item.into();
        num.into()
    }
}

impl From<f64> for Value {
    fn from(item: f64) -> Self {
        let num: Numerical = item.into();
        num.into()
    }
}

impl From<bool> for Value {
    fn from(item: bool) -> Self {
        let num: Numerical = item.into();
        num.into()
    }
}

impl From<String> for Value {
    fn from(item: String) -> Self {
        Value::Collection(Collection::new_string(item))
    }
}

impl From<Vec<Value>> for Value {
    fn from(values: Vec<Value>) -> Self {
        Value::Collection(Collection::new_list(values))
    }
}

impl From<Collection> for Value {
    fn from(coll: Collection) -> Self {
        Value::Collection(coll)
    }
}
impl From<List> for Value {
    fn from(list: List) -> Self {
        Value::Collection(Collection::List(list))
    }
}
