use parser::CodeRange;

use std::collections::HashMap;

use super::OpCode;

/// A region of bytecode, with associated information
pub struct Chunk {
    code: Vec<u8>,
    opcode_ranges: HashMap<usize, CodeRange>, // Inefficient way to store it. Do we need a range? Is is better to just use an array?
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            opcode_ranges: HashMap::new(),
        }
    }

    pub fn push_opcode(&mut self, opcode: OpCode, range: CodeRange) {
        self.opcode_ranges.insert(self.code.len(), range);
        self.code.push(opcode as u8);
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.code
    }

    pub fn get_range(&self, offset: usize) -> Option<&CodeRange> {
        self.opcode_ranges.get(&offset)
    }
}
