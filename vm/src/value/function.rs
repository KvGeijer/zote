use std::rc::Rc;

use crate::compiler::Chunk;

#[derive(Debug)]
pub struct Function {
    arity: u8,        // TODO: Check
    chunk: Rc<Chunk>, // It is really strange having a value own the code. Probably better to index into some array
    name: String,     // TODO: Promote this to a zote string? For easier printing
}

impl Function {
    pub fn new(arity: u8, name: String, chunk: Chunk) -> Self {
        Self {
            arity,
            chunk: Rc::new(chunk),
            name,
        }
    }

    pub fn arity(&self) -> u8 {
        self.arity
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn chunk_rc(&self) -> Rc<Chunk> {
        self.chunk.clone()
    }

    pub fn chunk_ref(&self) -> &Chunk {
        self.chunk.as_ref()
    }
}
