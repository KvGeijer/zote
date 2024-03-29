use parser::CodeRange;

use std::{collections::HashMap, ops::Index};

use crate::value::Value;

use super::OpCode;

/// A region of bytecode, with associated information
#[derive(Debug)]
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

    pub fn push_bool(&mut self, bool: bool) {
        self.code.push(bool as u8);
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
    pub fn patch_reserved_jump(&mut self, reserved: usize) {
        self.set_reserved_jump(reserved, self.len())
    }

    fn set_reserved_jump(&mut self, reserved: usize, target: usize) {
        let offset = (target as i64 - reserved as i64) as i16;
        if self.code[reserved - 2] == 255 && self.code[reserved - 1] == 255 {
            let bytes = offset.to_be_bytes();
            self.code[reserved - 2] = bytes[0];
            self.code[reserved - 1] = bytes[1];
        } else {
            panic!("Tried to set values which were not reserved")
        }
    }

    pub fn push_jump(&mut self, target: usize) {
        let current = self.reserve_jump();
        self.set_reserved_jump(current, target);
    }

    pub fn push_constant(&mut self, value: Value) {
        if let Some(index) = self
            .constants
            .iter()
            .enumerate()
            .find_map(|(ind, const_value)| value.eq(const_value).then_some(ind))
        {
            self.code.push(index as u8);
        } else {
            self.code.push(self.constants.len() as u8);
            self.constants.push(value);
        }

        if self.constants.len() > u8::MAX as usize {
            panic!("Cannot have more than 255 constants, as we store index in u8");
        }
    }

    /// Pushes a constant and its opcode to the bytecode
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

    /// Gets the range preceeding offset
    pub fn get_prev_range(&self, mut offset: usize) -> Option<&CodeRange> {
        loop {
            if let Some(range) = self.opcode_ranges.get(&offset) {
                return Some(range);
            }

            if offset == 0 {
                return None;
            } else {
                offset -= 1;
            }
        }
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
