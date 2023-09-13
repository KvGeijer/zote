use parser::CodeRange;

use vm::compiler::Chunk;
use vm::compiler::OpCode;
use vm::disassembler;
use vm::interpreter::interpret;
use vm::value::Value;

fn push_opcode(chunk: &mut Chunk, opcode: OpCode) {
    chunk.push_opcode(opcode, CodeRange::from_ints(0, 0, 0, 0, 0, 0));
}

fn push_const(chunk: &mut Chunk, constant: Value) {
    push_opcode(chunk, OpCode::Constant);
    chunk.push_constant(constant);
}

pub fn main() {
    test_num_ops();
}

fn test_num_ops() {
    println!("=== TESTING NUMERICAL OPS ===");
    let mut chunk = Chunk::new();
    push_const(&mut chunk, Value::Int(13));
    push_const(&mut chunk, Value::Int(37));
    push_opcode(&mut chunk, OpCode::Subtract);
    push_opcode(&mut chunk, OpCode::Negate);
    push_const(&mut chunk, Value::Int(42));
    push_const(&mut chunk, Value::Int(37));
    push_const(&mut chunk, Value::Int(3));
    push_opcode(&mut chunk, OpCode::Add);
    push_opcode(&mut chunk, OpCode::Subtract);
    push_opcode(&mut chunk, OpCode::Multiply);
    push_const(&mut chunk, Value::Int(13));
    push_const(&mut chunk, Value::Int(-7));
    push_opcode(&mut chunk, OpCode::Divide);
    disassembler::disassemble_chunk(&chunk, "main chunk", &mut std::io::stdout()).unwrap();
    println!("\n== Running chunk ==");
    interpret(&chunk, true).unwrap();
}
