use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc, vec};

use crate::ast_interpreter::{
    runtime_error::{RunError, RunRes},
    value::Value,
};

use super::Collection;

#[derive(Clone, Debug, PartialEq)]
pub struct Dict {
    dict: Rc<RefCell<HashMap<ValueKey, Value>>>,
}

impl PartialOrd for Dict {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl Dict {
    pub fn new() -> Self {
        Self {
            dict: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn deepclone(&self) -> Self {
        // Keys are not cloned, as they should not have mutable references
        self.dict
            .borrow()
            .iter()
            .map(|(key, value)| (key.clone(), value.deepclone()))
            .collect::<HashMap<_, _>>()
            .into()
    }

    pub fn is_empty(&self) -> bool {
        self.dict.borrow().is_empty()
    }

    pub fn stringify(&self) -> String {
        let mut string = "Dict{".to_string();
        for (key, value) in self.dict.borrow().iter() {
            string.push_str(&format!(" {}: {},", key.val.stringify(), value.stringify()));
        }
        string.push('}');
        string
    }

    pub fn assign_into(&self, key: Value, value: Value) -> RunRes<Value> {
        let key = ValueKey::new_clone(key)?;
        self.insert(key, value.clone());
        Ok(value)
    }

    fn insert(&self, key: ValueKey, value: Value) {
        self.dict.borrow_mut().insert(key, value);
    }

    pub fn get(&self, key: &Value) -> RunRes<Value> {
        self.safe_get(key)?.ok_or(RunError::bare_error(format!(
            "Key \"{}\" not in dict",
            key.stringify()
        )))
    }

    fn safe_get(&self, key: &Value) -> RunRes<Option<Value>> {
        let key = ValueKey::new(key)?;
        Ok(self.dict.borrow().get(&key).cloned())
    }

    pub fn to_iter(&self) -> vec::IntoIter<Value> {
        self.dict
            .borrow()
            .iter()
            .map(|(key, value)| vec![key.val.deepclone(), value.clone()].into())
            .collect::<Vec<Value>>()
            .into_iter()
    }

    pub fn len(&self) -> usize {
        self.dict.borrow().len()
    }

    pub fn contains_key(&self, key: &Value) -> RunRes<bool> {
        Ok(self.safe_get(key)?.is_some())
    }

    pub fn keys(&self) -> Vec<Value> {
        self.dict
            .borrow()
            .keys()
            .map(|key| key.val.deepclone())
            .collect()
    }

    pub fn values(&self) -> Vec<Value> {
        self.dict.borrow().values().cloned().collect()
    }

    /// Keeps all key-value pairs present in both
    pub fn intersect(&self, other: &Dict) -> Dict {
        let intersect = Dict::new();
        let left = self.dict.borrow();
        let right = other.dict.borrow();
        for (key, value) in left.iter() {
            match right.get(key) {
                Some(rval) if rval == value => intersect.insert(key.clone(), value.clone()),
                _other => (),
            }
        }
        intersect
    }

    /// Keeps all key value-value pairs from both (the first has precedence on values)
    pub fn union(&self, other: &Dict) -> Dict {
        let union = Dict::new();
        for (key, value) in other.dict.borrow().iter() {
            union.insert(key.clone(), value.clone())
        }
        for (key, value) in self.dict.borrow().iter() {
            union.insert(key.clone(), value.clone())
        }
        union
    }
}

impl From<HashMap<ValueKey, Value>> for Dict {
    fn from(value: HashMap<ValueKey, Value>) -> Self {
        Self {
            dict: Rc::new(RefCell::new(value)),
        }
    }
}

#[derive(Clone, Debug)]
struct ValueKey {
    val: Value,
}

// This is only true since we don't allow ValueKeys to be constructed from incomparable types
impl Eq for ValueKey {}

impl PartialEq for ValueKey {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val || self.val.is_nan() && other.val.is_nan()
    }
}

impl Hash for ValueKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        value_hash(&self.val, state)
    }
}

// Just implemented to work, but not at all be efficient
fn value_hash<H: std::hash::Hasher>(value: &Value, state: &mut H) {
    match value {
        Value::Numerical(num) => num.to_rint().hash(state),
        Value::Collection(Collection::String(string)) => string.as_ref().hash(state),
        Value::Collection(coll) => {
            for inner in coll.to_iter() {
                value_hash(&inner, state);
            }
        }
        Value::Callable(_) => panic!("Cannot hash a function"),
        Value::Nil => 2.hash(state),
        Value::Uninitialized => panic!("use of uninitialized in hash"),
    }
}

fn valid_key(value: &Value) -> bool {
    match value {
        Value::Numerical(_) => true,
        Value::Collection(Collection::String(_)) => true,
        Value::Collection(coll) => coll.to_iter().all(|inner| valid_key(&inner)),
        Value::Callable(_) => false,
        Value::Nil => true,
        Value::Uninitialized => false,
    }
}

impl ValueKey {
    fn new(value: &Value) -> RunRes<ValueKey> {
        if valid_key(value) {
            Ok(ValueKey {
                val: value.deepclone(),
            })
        } else {
            RunError::error("Tried to hash an invalid key".to_string())
        }
    }

    fn new_clone(value: Value) -> RunRes<ValueKey> {
        if valid_key(&value) {
            Ok(ValueKey {
                val: value.deepclone(),
            })
        } else {
            RunError::error("Tried to hash an invalid key".to_string())
        }
    }
}
