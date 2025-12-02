use std::{
    cell::{Ref, RefCell, RefMut},
    collections::BinaryHeap,
    fmt::Display,
};

use super::Value;
use crate::error::{RunRes, RunResTrait};

#[derive(Debug)]
pub struct PriorityQueue {
    queue: RefCell<BinaryHeap<HeapItem>>,
}

impl PriorityQueue {
    pub fn new() -> Self {
        Self {
            queue: RefCell::new(BinaryHeap::new()),
        }
    }

    /// Inserts an item with priority into the priority-queue
    pub fn push(&self, value: Value, prio: Value) {
        let item = HeapItem::new(prio, value);
        self.borrow_mut().push(item);
    }

    /// Pops the element with the highest priority
    ///
    /// The first return value is the priority, the second is the associated value
    pub fn pop_max(&self) -> RunRes<(Value, Value)> {
        if let Some(item) = self.borrow_mut().pop() {
            if !item.has_error() {
                Ok((item.prio, item.value))
            } else {
                RunRes::new_err(format!(
                    "PriorityQueue comparison error: {}",
                    item.get_error().unwrap()
                ))
            }
        } else {
            RunRes::new_err(format!("Cannot pop from an empty PriorityQueue"))
        }
    }

    /// Checks if the priority queue is empty
    pub fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }

    /// Truthiness of the queue (is it non-empty?)
    pub fn truthy(&self) -> bool {
        !self.is_empty()
    }

    /// Checks the len of the priority queue
    pub fn len(&self) -> usize {
        self.borrow().len()
    }

    /// Clones all items and priorities deeply
    pub fn deepclone(&self) -> Self {
        self.borrow()
            .iter()
            .map(|item| HeapItem::new(item.prio.deepclone(), item.value.deepclone()))
            .collect::<BinaryHeap<HeapItem>>()
            .into()
    }

    /// Borrows the heap
    fn borrow(&self) -> Ref<'_, BinaryHeap<HeapItem>> {
        self.queue.borrow()
    }

    /// Borrows the heap mutably
    fn borrow_mut(&self) -> RefMut<'_, BinaryHeap<HeapItem>> {
        self.queue.borrow_mut()
    }
}

impl Clone for PriorityQueue {
    fn clone(&self) -> Self {
        self.borrow()
            .iter()
            .map(|item| HeapItem::new(item.prio.clone(), item.value.clone()))
            .collect::<BinaryHeap<HeapItem>>()
            .into()
    }
}

impl Display for PriorityQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "priority-queue{{")?;
        let heap = self.borrow();
        let mut iter = heap.iter();
        if let Some(item) = iter.next() {
            write!(f, "{}: {}", item.value, item.prio)?;
        }

        for item in iter {
            write!(f, ", {}: {}", item.value, item.prio)?;
        }

        write!(f, "}}")?;
        Ok(())
    }
}

impl From<BinaryHeap<HeapItem>> for PriorityQueue {
    fn from(value: BinaryHeap<HeapItem>) -> Self {
        Self {
            queue: RefCell::new(value),
        }
    }
}

/// Item for combining Prio and Value for heap insertions
#[derive(Debug, PartialOrd, PartialEq, Eq)]
struct HeapItem {
    prio: Value,
    value: Value,

    /// Some error if it failed a comparison at any point
    /// OPT: Can we do this in a better way?
    error: RefCell<Option<String>>,
}

impl HeapItem {
    fn new(prio: Value, value: Value) -> Self {
        Self {
            prio,
            value,
            error: RefCell::new(None),
        }
    }

    fn set_error(&self, reason: String) {
        *self.error.borrow_mut() = Some(reason)
    }

    fn has_error(&self) -> bool {
        self.error.borrow().is_some()
    }

    fn get_error(&self) -> Option<String> {
        self.error.borrow().as_ref().map(|str| str.to_owned())
    }
}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if let Some(ord) = self.prio.partial_cmp(&other.prio) {
            ord
        } else {
            let reason = format!(
                "Cannot compare {} and {}. In priority queue.",
                self.prio.type_of(),
                other.prio.type_of()
            );
            self.set_error(reason.clone());
            other.set_error(reason);
            std::cmp::Ordering::Equal
        }
    }
}
