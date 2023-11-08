use std::{fmt::Display, rc::Rc};

use crate::error::{RunRes, RuntimeError};

mod builtins;
mod closure;
mod function;
mod value_pointer;

pub use builtins::get_natives;
pub use closure::Closure;
pub use function::Function;
pub use value_pointer::ValuePointer;

use self::builtins::Native;

// OPT: Pack as bytesting instead? Very inefficiently stored now in 128 bits
#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Function(Rc<Function>),
    Closure(Rc<Closure>),
    Native(Native),

    /// A value closed over by a function must be stored on the heap
    Pointer(ValuePointer),
}

pub enum ValueType {
    Nil,
    Bool,
    Int,
    Float,
    Function,
    Builtin,
    Closure,
}

impl Value {
    pub fn type_of(&self) -> ValueType {
        match self {
            Value::Nil => ValueType::Nil,
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::Function(_) => ValueType::Function,
            Value::Native(_) => ValueType::Builtin,
            Value::Pointer(pointer) => pointer.get_clone().type_of(),
            Value::Closure(_) => ValueType::Closure,
        }
    }

    pub fn truthy(&self) -> RunRes<bool> {
        match self {
            Value::Nil => Ok(false),
            Value::Bool(bool) => Ok(*bool),
            Value::Int(x) => Ok(*x != 0),
            Value::Float(x) => Ok(*x != 0.0),
            Value::Function(f) => {
                RuntimeError::error(format!("Functions don't have a truthiness ({})", f.name()))
            }
            Value::Native(f) => RuntimeError::error(format!(
                "Builtint functions don't have a truthiness ({})",
                f.name()
            )),
            Value::Pointer(pointer) => pointer.get_clone().truthy(),
            Value::Closure(_) => {
                RuntimeError::error("A closure does not have a truthiness".to_string())
            }
        }
    }

    /// Reads the value, following pointers if necessary
    // pub fn read(&self) -> Value {
    //     if let Value::Pointer(pointer) = self {
    //         pointer.get_clone()
    //     } else {
    //         self.clone()
    //     }
    // }

    pub fn to_closure(self) -> Option<Rc<Closure>> {
        if let Value::Closure(closure) = self {
            Some(closure)
        } else {
            None
        }
    }

    pub fn to_function(self) -> Option<Rc<Function>> {
        if let Value::Function(function) = self {
            Some(function)
        } else {
            None
        }
    }

    // Updates the value to a new one
    pub fn write(&mut self) {}
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b, // Could allow eq between bool/int
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Function(ref a), Value::Function(ref b)) => {
                // Compare the pointers, to see if they are the exact same function
                Rc::ptr_eq(a, b)
            }
            (Value::Closure(ref a), Value::Closure(ref b)) => {
                // Compare the pointers, to see if they are the exact same function
                Rc::ptr_eq(a, b)
            }
            (Value::Pointer(pointer), other) => pointer.get_clone().eq(other),
            (other, Value::Pointer(pointer)) => other.eq(&pointer.get_clone()),
            _ => false, // All other combinations are not equal
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
            ValueType::Function => write!(f, "Function"),
            ValueType::Builtin => write!(f, "Function"),
            ValueType::Closure => write!(f, "Closure"),
        }
    }
}

impl From<Function> for Value {
    fn from(func: Function) -> Self {
        Value::Function(Rc::new(func))
    }
}

impl From<Closure> for Value {
    fn from(func: Closure) -> Self {
        Value::Closure(Rc::new(func))
    }
}

impl From<Native> for Value {
    fn from(func: Native) -> Self {
        Value::Native(func)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "Nil"),
            Value::Bool(bool) => write!(f, "{}", bool),
            Value::Int(int) => write!(f, "{}", int),
            Value::Float(float) => write!(f, "{}", float),
            Value::Function(func) => write!(f, "{}", func.name()),
            // Value::Function(func) => write!(f, "fn {}/{}", func.name(), func.arity()), // TODO
            Value::Native(native) => write!(f, "fn {}/{}", native.name(), native.arity()),
            Value::Pointer(pointer) => pointer.get_clone().fmt(f),
            Value::Closure(closure) => write!(f, "{}", closure.function().name()),
        }
    }
}
