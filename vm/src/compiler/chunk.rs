use parser::CodeRange;

use std::{collections::HashMap, ops::Index};

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

    pub fn push_u8_offset(&mut self, u8: u8) {
        self.code.push(u8);
    }

    /// Reserves two bytes at the index for a future jump
    pub fn reserve_jump(&mut self) -> usize {
        self.code.push(255);
        self.code.push(255);
        self.code.len()
    }

    /// Set the jump offset at the given reserved index.
    ///
    /// Panics if the index is not already reserved
    pub fn set_reserved_jump(&mut self, reserved: usize, target: usize) {
        let offset = (target as i64 - reserved as i64) as i16;
        if self.code[reserved - 2] == 255 && self.code[reserved - 1] == 255 {
            let bytes = offset.to_be_bytes();
            self.code[reserved - 2] = bytes[0];
            self.code[reserved - 1] = bytes[1];
        } else {
            panic!("Tried to set values which were not reserved")
        }
    }

    pub fn push_jump_offset(&mut self, target: usize) {
        let current = self.reserve_jump();
        self.set_reserved_jump(current, target);
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

    pub fn get_constant(&self, index: u8) -> Option<&Value> {
        self.constants.get(index as usize)
    }

    pub fn get_range(&self, offset: usize) -> Option<&CodeRange> {
        self.opcode_ranges.get(&offset)
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }
}

// To be able to directly index into the code
impl Index<usize> for Chunk {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.code[index]
    }
}
