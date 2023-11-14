use std::rc::Rc;

use crate::compiler::Chunk;

use super::{value_pointer::ValuePointer, Function, Value};

#[derive(Debug)]
pub struct Closure {
    // name: String,
    function: Rc<Function>,
    upvalues: Vec<ValuePointer>,
}

impl Closure {
    pub fn new(function: Rc<Function>, upvalues: Vec<ValuePointer>) -> Self {
        Self { function, upvalues }
    }

    pub fn function(&self) -> &Function {
        &self.function
    }

    pub fn nbr_upvalues(&self) -> usize {
        self.upvalues.len()
    }

    pub fn get_upvalue(&self, index: u8) -> Option<&ValuePointer> {
        self.upvalues.get(index as usize)
    }

    pub fn set_upvalue(&self, index: u8, value: Value) -> Option<()> {
        if let Some(pointer) = self.upvalues.get(index as usize) {
            pointer.set(value);
            Some(())
        } else {
            None
        }
    }

    pub fn chunk_rc(&self) -> Rc<Chunk> {
        self.function.chunk_rc()
    }

    pub fn nbr_locals(&self) -> usize {
        self.function.nbr_locals()
    }
}
