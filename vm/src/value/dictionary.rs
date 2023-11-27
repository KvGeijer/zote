use std::{
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::Hash,
};

use crate::error::{RunRes, RunResTrait, RuntimeError};

use super::{List, Value};

/// A lookup dictionary between Values
///
/// It uses inner mutability to be mutable with read-only references.
/// Keys are deeploned before insertion to disallow mutability.
/// Don't allow function types as keys
#[derive(Debug)]
pub struct Dictionary {
    map: RefCell<HashMap<KeyValue, Value>>,
}

impl Dictionary {
    /// Creates a new empty dict
    pub fn new() -> Self {
        Self {
            map: RefCell::new(HashMap::new()),
        }
    }

    /// Gets the number of entries in the dict
    pub fn len(&self) -> usize {
        self.map.borrow().len()
    }

    /// Checks if it is empty
    pub fn is_emtpy(&self) -> bool {
        self.map.borrow().is_empty()
    }

    /// Converts it to a truthy boolean
    pub fn truthy(&self) -> bool {
        !self.is_emtpy()
    }

    /// Sets the value of a key. Potentially overriding old value
    pub fn set(&self, key: Value, value: Value) -> RunRes<()> {
        // TODO: Play around with this to see if we should just always clone it

        if !valid_key(&key, 0) {
            return RunRes::new_err(format!(
                "Cannot use a {} as a key to a dictionary",
                key.type_of()
            ));
        }

        // Just for lookups, as we have not cloned the value
        let unsafe_key = KeyValue(key);
        let mut dict = self.borrow_mut();
        if let Some(present) = dict.get_mut(&unsafe_key) {
            *present = value;
        } else {
            // Get the key again, and clone it to insert with it.
            let KeyValue(key) = unsafe_key;
            let key: KeyValue = key
                .try_into()
                .map_err(|reason| RuntimeError::bare_error(reason))?;
            dict.insert(key, value);
        }
        Ok(())
    }

    /// Inserts a key in the dict
    ///
    /// Equivalent to 'set', but immediately deepclones the key
    pub fn insert(&self, key: Value, value: Value) -> RunRes<()> {
        let key: KeyValue = key
            .try_into()
            .map_err(|reason| RuntimeError::bare_error(reason))?;
        self.borrow_mut().insert(key, value);
        Ok(())
    }

    /// Gets the potential value of a key
    pub fn get(&self, key: Value) -> RunRes<Option<Value>> {
        // TODO: It would be faster if we removed this check...
        if !valid_key(&key, 0) {
            return RunRes::new_err(format!(
                "Cannot use a {} as a key to a dictionary",
                key.type_of()
            ));
        }

        let key = KeyValue(key);
        Ok(self.borrow().get(&key).cloned())
    }

    /// Clones and casts the dict to a list
    pub fn cast_list(&self) -> List {
        self.borrow()
            .iter()
            .map(|(key, value)| List::from(vec![key.clone().to_value(), value.clone()]).into())
            .collect::<Vec<_>>()
            .into()
    }

    /// Creates a deep clone of the dict
    ///
    /// It deepclones all values, but not keys, as they are immutable
    pub fn deepclone(&self) -> Self {
        self.borrow()
            .iter()
            .map(|(key, val)| (key.clone(), val.deepclone()))
            .collect::<HashMap<_, _>>()
            .into()
    }

    /// Calculates the intersection of two dicts over keys, using values from the first one
    pub fn intersect(&self, other: &Self) -> Self {
        let other_borrow = other.borrow();
        let other_keys = other_borrow.keys().collect::<HashSet<&KeyValue>>();

        let mut intersection = HashMap::new();

        for (key, value) in self.borrow().iter() {
            if other_keys.contains(key) {
                intersection.insert(key.clone(), value.clone());
            }
        }

        intersection.into()
    }

    /// Calculates the union of two dicts over keys, prefering values from the first one
    pub fn union(&self, other: &Self) -> Self {
        let mut union = HashMap::new();

        for (key, value) in other.borrow().iter() {
            union.insert(key.clone(), value.clone());
        }
        for (key, value) in self.borrow().iter() {
            union.insert(key.clone(), value.clone());
        }

        union.into()
    }

    /// Borrows the map
    fn borrow(&self) -> Ref<HashMap<KeyValue, Value>> {
        self.map.borrow()
    }

    /// Borrows the map mutably
    fn borrow_mut(&self) -> RefMut<HashMap<KeyValue, Value>> {
        self.map.borrow_mut()
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<KeyValue, Value>> for Dictionary {
    fn from(value: HashMap<KeyValue, Value>) -> Self {
        Self {
            map: RefCell::new(value),
        }
    }
}

impl Display for Dictionary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dict{{")?;
        let dict = self.borrow();
        let mut iter = dict.iter();
        if let Some((key, value)) = iter.next() {
            write!(f, "{key}: {value}")?;
        }

        for (key, value) in iter {
            write!(f, ", {key}: {value}")?;
        }

        write!(f, "}}")?;
        Ok(())
    }
}

/// Intermediate type as Value can't be used directly as a key in a hash-map
#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct KeyValue(Value);

impl KeyValue {
    fn to_value(self) -> Value {
        let KeyValue(value) = self;
        value
    }
}

// TODO: Play around with this, improving it using benchmarks
impl Hash for KeyValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let KeyValue(value) = self;
        value
            .try_hash(state)
            .expect("Should not be able to have unhashable objects within a hash map")
    }
}

impl Eq for KeyValue {}

impl TryFrom<Value> for KeyValue {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Nil
            | Value::Bool(_)
            | Value::Int(_)
            | Value::Float(_)
            | Value::Pointer(_)
            | Value::List(_)
            | Value::String(_) => Ok(KeyValue(value.deepclone())),
            otherwise => Err(format!(
                "Cannot use {} as a key to a dictionary",
                otherwise.type_of()
            )),
        }
    }
}

impl Display for KeyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let KeyValue(value) = self;
        value.fmt(f)
    }
}

/// Checks if a value is a valid key, without actually converting it
fn valid_key(value: &Value, depth: usize) -> bool {
    if depth > 10000 {
        println!(
            "WARNING: A cyclic dependency tried to be used as a key for a {}",
            value.type_of()
        );
        return false;
    }
    match value {
        Value::Nil => true,
        Value::Bool(_) => true,
        Value::Int(_) => true,
        Value::Float(_) => true,
        Value::Function(_) => false,
        Value::Closure(_) => false,
        Value::Native(_) => false,
        Value::Pointer(p) => valid_key(&p.borrow_value(), depth + 1),
        Value::List(l) => l.borrow_slice().iter().all(|v| valid_key(v, depth + 1)),
        Value::String(_) => true,
        Value::Dictionary(_) => false,
    }
}

impl Into<List> for &Dictionary {
    fn into(self) -> List {
        self.borrow()
            .iter()
            .map(|(KeyValue(key), value)| {
                Value::from(List::from(vec![key.deepclone(), value.clone()]))
            })
            .collect::<Vec<Value>>()
            .into()
    }
}

impl PartialEq for Dictionary {
    fn eq(&self, other: &Self) -> bool {
        let this = self.borrow();
        let other = other.borrow();

        if this.len() != other.len() {
            return false;
        }

        for (key, value) in this.iter() {
            if other.get(key) != Some(value) {
                return false;
            }
        }

        true
    }
}
