use std::{
    fmt::{Debug, Display},
    hash::Hash,
    rc::Rc,
};

use crate::error::{RunRes, RunResTrait, RuntimeError};

mod builtins;
mod closure;
mod dictionary;
mod function;
mod list;
mod string;
mod value_pointer;

pub use builtins::get_natives;
pub use closure::Closure;
pub use dictionary::Dictionary;
pub use function::Function;
pub use list::List;
pub use value_pointer::ValuePointer;

use self::{builtins::Native, string::ValueString};

// OPT: Pack as bytesting instead? Very inefficiently stored now in 128 bits
#[derive(Clone)]
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

    /// A list of values
    List(Rc<List>),

    /// A string
    String(Rc<ValueString>),

    /// A HashMap, where we use our own unstable wierd hashing
    Dictionary(Rc<Dictionary>),
}

pub enum ValueType {
    Nil,
    Bool,
    Int,
    Float,
    Function,
    Builtin,
    Closure,
    List,
    String,
    Dictionary,
}

/// Impl for delegating tasks between function types and implementing easy queries
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
            Value::List(_) => ValueType::List,
            Value::String(_) => ValueType::String,
            Value::Dictionary(_) => ValueType::Dictionary,
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
            Value::List(list) => Ok(list.truthy()),
            Value::String(string) => Ok(string.truthy()),
            Value::Dictionary(dict) => Ok(dict.truthy()),
        }
    }

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

    pub fn to_list(self) -> Option<Rc<List>> {
        if let Value::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    pub fn to_valuestring(self) -> Option<Rc<ValueString>> {
        if let Value::String(string) = self {
            Some(string)
        } else {
            None
        }
    }

    pub fn to_dict(self) -> Option<Rc<Dictionary>> {
        if let Value::Dictionary(dict) = self {
            Some(dict)
        } else {
            None
        }
    }

    /// Tries to convert a value to something iterable
    pub fn conv_to_iter(self) -> RunRes<Value> {
        let typ = self.type_of();
        match self {
            Value::List(list) => Ok(Value::List(list)),
            Value::String(string) => Ok(Value::String(string)),
            Value::Dictionary(dict) => Ok(dict.cast_list().into()),
            Value::Pointer(_) => panic!("Should not operate directly on a pointer"),
            Value::Nil
            | Value::Bool(_)
            | Value::Int(_)
            | Value::Float(_)
            | Value::Function(_)
            | Value::Closure(_)
            | Value::Native(_) => RunRes::new_err(format!("Cannot iterate over {}", typ)),
        }
    }

    /// Tries to convers the value to a list
    pub fn conv_to_list(self) -> RunRes<Rc<List>> {
        let typ = self.type_of();
        match self {
            Value::List(list) => Ok(list),
            Value::String(string) => Ok(Rc::new(string.as_ref().into())),
            Value::Dictionary(dict) => Ok(Rc::new(dict.as_ref().into())),
            Value::Pointer(_) => panic!("Should not operate directly on a pointer"),
            Value::Nil
            | Value::Bool(_)
            | Value::Int(_)
            | Value::Float(_)
            | Value::Function(_)
            | Value::Closure(_)
            | Value::Native(_) => RunRes::new_err(format!("Cannot iterate over {}", typ)),
        }
    }

    /// Tries to assign into an index of the value
    pub fn assign_at_index(&mut self, index: Value, value: Value) -> RunRes<()> {
        match self {
            Value::List(list) => list.set(index.to_int()?, value),
            Value::String(string) => string.set(index.to_int()?, value),
            Value::Dictionary(dict) => dict.set(index, value),
            otherwise => RunRes::new_err(format!("Cannot index into a {}", otherwise.type_of())),
        }
    }

    /// Tries to read at an index of the value
    pub fn read_at_index(&self, index: Value) -> RunRes<Value> {
        match self {
            Value::List(list) => list.get(index.to_int()?),
            Value::String(string) => string.get(index.to_int()?),
            Value::Dictionary(dict) => {
                Ok(dict
                    .get(index.clone())?
                    .ok_or(RuntimeError::bare_error(format!(
                        "Key {index} does not exist in the dictionary"
                    )))?)
            }
            otherwise => RunRes::new_err(format!("Cannot index into a {}", otherwise.type_of())),
        }
    }

    /// Tries to read at an index of the value
    pub fn safe_read_at_index(&self, index: Value) -> RunRes<Option<Value>> {
        match self {
            Value::List(list) => Ok(list.get(index.to_int()?).ok()),
            Value::String(string) => Ok(string.get(index.to_int()?).ok()),
            Value::Dictionary(dict) => Ok(dict.get(index.clone())?),
            otherwise => RunRes::new_err(format!("Cannot index into a {}", otherwise.type_of())),
        }
    }

    /// Tries to push a value to the end of this one
    pub fn push(&self, value: Value) -> RunRes<()> {
        match self {
            Value::List(list) => Ok(list.push(value)),
            Value::String(string) => string.push(value),

            otherwise => RunRes::new_err(format!("Cannot push to a {}", otherwise.type_of())),
        }
    }

    /// Tries to pop a value from the end of this one
    pub fn pop(&self) -> RunRes<Value> {
        match self {
            Value::List(list) => list.pop(),
            Value::String(string) => string.pop(),
            otherwise => RunRes::new_err(format!("Cannot pop from a {}", otherwise.type_of())),
        }
    }

    /// Tries to convert a value to an integer
    pub fn to_int(self) -> RunRes<i64> {
        match self {
            Value::Bool(bool) => {
                if bool {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            Value::Int(int) => Ok(int),
            Value::Float(float) => Ok(float.round() as i64),
            otherwise => RunRes::new_err(format!(
                "Cannot use a {} as an integer",
                otherwise.type_of()
            )),
        }
    }

    /// Converts the value to an int if possible. NIL is mapped to specified value.
    pub fn to_int_or_nil_none(self) -> RunRes<Option<i64>> {
        match self {
            Value::Nil => Ok(None),
            otherwise => otherwise.to_int().map(Some),
        }
    }

    /// Gets the length of a value, returning error if it does not have a length
    pub fn len(&self) -> RunRes<usize> {
        match self {
            Value::Pointer(_) => panic!("Tried to operate directly on a pointer (get len)"),
            Value::List(list) => Ok(list.len()),
            Value::String(string) => Ok(string.len()),
            Value::Dictionary(dict) => Ok(dict.len()),
            Value::Nil
            | Value::Bool(_)
            | Value::Int(_)
            | Value::Float(_)
            | Value::Function(_)
            | Value::Closure(_)
            | Value::Native(_) => {
                RunRes::new_err(format!("Cannot get the length of a {}", self.type_of()))
            }
        }
    }

    /// Tries to convert a value to a char
    fn to_char(&self) -> RunRes<u8> {
        match self {
            Value::Int(ascii_int) => {
                if *ascii_int <= 127 && *ascii_int >= 0 {
                    Ok(*ascii_int as u8)
                } else {
                    RunRes::new_err(format!("Cannot convert {} to char", ascii_int))
                }
            }
            Value::String(string) => string.to_char(),
            Value::Nil
            | Value::Bool(_)
            | Value::Float(_)
            | Value::Function(_)
            | Value::Closure(_)
            | Value::Native(_)
            | Value::Dictionary(_)
            | Value::Pointer(_)
            | Value::List(_) => {
                RunRes::new_err(format!("Cannot convert {} to char", self.type_of()))
            }
        }
    }

    /// Tries to append this and another value
    pub fn append(self, other: Value) -> RunRes<Value> {
        match (self, other) {
            (Value::List(lhs), Value::List(rhs)) => Ok(lhs.append(rhs.as_ref()).into()),
            (Value::String(lhs), Value::String(rhs)) => Ok(lhs.append(rhs.as_ref()).into()),
            (lhs, rhs) => RunRes::new_err(format!(
                "Cannot append {} to {}",
                rhs.type_of(),
                lhs.type_of()
            )),
        }
    }

    /// Deeply clones a value and all its contained references
    pub fn deepclone(&self) -> Self {
        match self {
            Value::Nil => self.clone(),
            Value::Bool(_) => self.clone(),
            Value::Int(_) => self.clone(),
            Value::Float(_) => self.clone(),
            Value::Function(_) => self.clone(),
            Value::Closure(_) => self.clone(),
            Value::Native(_) => self.clone(),
            Value::Pointer(pointer) => pointer.get_clone().deepclone(),
            Value::List(list) => list.deepclone().into(),
            Value::String(string) => string.as_ref().clone().into(),
            Value::Dictionary(dict) => dict.deepclone().into(),
        }
    }

    /// Shallowly clones a value and all its contained references
    ///
    /// For any collection type, it copies all the references (does not deeply clone them)
    pub fn shallowclone(&self) -> Self {
        match self {
            Value::Nil => self.clone(),
            Value::Bool(_) => self.clone(),
            Value::Int(_) => self.clone(),
            Value::Float(_) => self.clone(),
            Value::Function(_) => self.clone(),
            Value::Closure(_) => self.clone(),
            Value::Native(_) => self.clone(),
            Value::Pointer(pointer) => pointer.get_clone().shallowclone(),
            Value::List(list) => list.shallowclone().into(),
            Value::String(string) => string.as_ref().clone().into(),
            Value::Dictionary(dict) => dict.shallowclone().into(),
        }
    }

    fn try_hash<H: std::hash::Hasher>(&self, state: &mut H) -> RunRes<()> {
        // ERROR: Can potentially get stuck in infinite loops
        match self {
            Value::Nil => Ok(0u8.hash(state)),
            Value::Bool(b) => Ok(b.hash(state)),
            Value::Int(i) => Ok(i.hash(state)),
            Value::Float(f) => {
                let bytes = f.to_le_bytes();
                Ok([bytes[5], bytes[3], bytes[1]].hash(state))
            }
            Value::List(l) => {
                let len = l.len();
                if len == 0 {
                    Ok(0.hash(state))
                } else {
                    l.get(0).unwrap().try_hash(state)?;
                    l.get((len / 2) as i64).unwrap().try_hash(state)?;
                    l.get(len as i64 - 1).unwrap().try_hash(state)?;
                    Ok(())
                }
            }
            Value::Pointer(p) => p.borrow_value().try_hash(state),
            Value::String(s) => Ok(s.hash(state)),
            Value::Dictionary(_) | Value::Function(_) | Value::Closure(_) | Value::Native(_) => {
                panic!("This should not be part of a KeyValue")
            }
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
            (Value::String(x), Value::String(y)) => x.partial_cmp(y),
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
            (Value::Float(a), Value::Float(b)) => a.total_cmp(b).is_eq(),
            (Value::Function(ref a), Value::Function(ref b)) => {
                // Compare the pointers, to see if they are the exact same function
                Rc::ptr_eq(a, b)
            }
            (Value::Closure(ref a), Value::Closure(ref b)) => {
                // Compare the pointers, to see if they are the exact same function
                Rc::ptr_eq(a, b)
            }
            (Value::String(x), Value::String(y)) => x.eq(y),
            (Value::List(x), Value::List(y)) => x.eq(y),
            (Value::Dictionary(x), Value::Dictionary(y)) => x.eq(y),
            (Value::Pointer(pointer), other) => pointer.get_clone().eq(other),
            (other, Value::Pointer(pointer)) => other.eq(&pointer.get_clone()),
            _ => false, // All other combinations are not equal
        }
    }
}

/// We use the total order for floats, and otherwise we have no problem
impl Eq for Value {}

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
            ValueType::List => write!(f, "List"),
            ValueType::String => write!(f, "String"),
            ValueType::Dictionary => write!(f, "Dictionary"),
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

impl From<i64> for Value {
    fn from(int: i64) -> Self {
        Value::Int(int)
    }
}

impl From<f64> for Value {
    fn from(float: f64) -> Self {
        Value::Float(float)
    }
}

impl From<bool> for Value {
    fn from(bool: bool) -> Self {
        Value::Bool(bool)
    }
}

impl From<List> for Value {
    fn from(list: List) -> Self {
        Value::List(Rc::new(list))
    }
}

impl From<Rc<List>> for Value {
    fn from(list: Rc<List>) -> Self {
        Value::List(list)
    }
}

impl From<ValueString> for Value {
    fn from(string: ValueString) -> Self {
        Value::String(Rc::new(string))
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        ValueString::from(value).into()
    }
}

impl From<Dictionary> for Value {
    fn from(value: Dictionary) -> Self {
        Value::Dictionary(Rc::new(value))
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
            Value::Pointer(pointer) => Display::fmt(&pointer.get_clone(), f),
            Value::Closure(closure) => write!(f, "{}", closure.function().name()),
            Value::List(list) => {
                write!(f, "[")?;
                let len = list.len();
                for ind in 0..len {
                    if ind != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", list.get(ind as i64).unwrap())?;
                }
                write!(f, "]")?;
                Ok(())
            }
            Value::String(string) => write!(f, "{}", string),
            Value::Dictionary(dict) => write!(f, "{}", dict),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "Nil"),
            Value::Bool(value) => write!(f, "Bool({value})"),
            Value::Int(value) => write!(f, "Int({value})"),
            Value::Float(value) => write!(f, "Float({value})"),
            Value::Function(value) => write!(f, "Function({})", value.name()),
            Value::Closure(value) => write!(f, "Closure({})", value.function().name()),
            Value::Native(value) => write!(f, "Native({})", value.name()),
            Value::Pointer(value) => write!(f, "Pointer({:?})", value),
            Value::List(value) => write!(f, "List({:?})", value),
            Value::String(value) => write!(f, "String({value})"),
            Value::Dictionary(dict) => write!(f, "{:?}", dict),
        }
    }
}
