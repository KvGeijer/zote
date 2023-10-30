use enum_macros::TryFromByte;

/// A byte opcode describes what the coming bytes on in a stack are
#[derive(TryFromByte, Debug)]
pub enum OpCode {
    Return,
    Constant,
    Nil,
    True,
    False,
    Negate,
    Not,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Equality,
    NonEquality,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    AssignGlobal,
    ReadGlobal,
    AssignLocal,
    ReadLocal,
    JumpIfFalse,
    Jump,
    Discard,
    Call,
    FromTemp, // Moves a value from the temporary stack to the ordinary one
}