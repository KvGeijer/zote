use std::rc::Rc;

use crate::compiler::Chunk;

#[derive(Debug, Clone)]
pub struct CallFrame {
    /// The code chunk of the current function (could also find on stack)
    chunk: Rc<Chunk>, // Does this have to be in an RC? Or just static lifetime?

    /// The Root Base Pointer, where on the stack the call frame begins
    pub rbp: usize,

    /// The current program counter
    pub pc: usize,
}

impl CallFrame {
    pub fn new(chunk: Rc<Chunk>) -> Self {
        Self {
            chunk,
            rbp: 0,
            pc: 0,
        }
    }

    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    /// Initiates the call frame to be used
    pub fn init(&mut self, chunk: Rc<Chunk>, rbp: usize) {
        self.chunk = chunk;
        self.rbp = rbp;
        self.pc = 0;
    }
}
