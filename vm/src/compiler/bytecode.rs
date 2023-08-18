use enum_macros::TryFromByte;

/// A byte opcode describes what the coming bytes on in a stack are
#[derive(TryFromByte)]
pub enum OpCode {
    Return,
}

/// A region of bytecode
pub struct Chunk {
    code: Vec<u8>,
}

impl Chunk {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn push(&mut self, byte: u8) {
        self.code.push(byte);
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.code
    }
}
