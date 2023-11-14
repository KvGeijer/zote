use std::rc::Rc;

use crate::compiler::Chunk;

#[derive(Debug)]
pub struct Function {
    arity: u8,        // TODO: Check
    chunk: Rc<Chunk>, // It is really strange having a value own the code. Probably better to index into some array
    name: String,     // TODO: Promote this to a zote string? For easier printing

    /// The largest number of locals to use at any time (including parameters)
    nbr_locals: usize,
}

impl Function {
    pub fn new(arity: u8, locals: usize, name: String, chunk: Chunk) -> Self {
        Self {
            arity,
            chunk: Rc::new(chunk),
            name,
            nbr_locals: locals,
        }
    }

    pub fn arity(&self) -> u8 {
        self.arity
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn nbr_locals(&self) -> usize {
        self.nbr_locals
    }

    pub fn chunk_rc(&self) -> Rc<Chunk> {
        self.chunk.clone()
    }

    pub fn chunk_ref(&self) -> &Chunk {
        self.chunk.as_ref()
    }
}
