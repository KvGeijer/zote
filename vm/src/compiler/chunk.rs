use parser::CodeRange;

use std::collections::HashMap;

use crate::value::Value;

use super::OpCode;

/// A region of bytecode, with associated information
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    opcode_ranges: HashMap<usize, CodeRange>, // Inefficient way to store it. Do we need a range? Is is better to just use an array?
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            opcode_ranges: HashMap::new(),
        }
    }

    pub fn push_opcode(&mut self, opcode: OpCode, range: CodeRange) {
        self.opcode_ranges.insert(self.code.len(), range);
        self.code.push(opcode as u8);
    }

    pub fn push_constant(&mut self, value: Value) {
        if self.constants.len() > u8::MAX as usize {
            panic!("Cannot have more than 255 constants, as we store index in u8");
        }
        self.code.push(self.constants.len() as u8);
        self.constants.push(value);
    }

    pub fn push_constant_plus(&mut self, value: Value, range: CodeRange) {
        self.push_opcode(OpCode::Constant, range);
        self.push_constant(value);
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.code
    }

    pub fn get_constant(&self, index: u8) -> Option<&Value> {
        self.constants.get(index as usize)
    }

    pub fn get_range(&self, offset: usize) -> Option<&CodeRange> {
        self.opcode_ranges.get(&offset)
    }
}
