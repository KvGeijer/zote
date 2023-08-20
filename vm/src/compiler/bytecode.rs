use enum_macros::TryFromByte;

/// A byte opcode describes what the coming bytes on in a stack are
#[derive(TryFromByte, Debug)]
pub enum OpCode {
    Return,
    Constant,
}
