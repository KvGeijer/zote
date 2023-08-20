use parser::CodeRange;

use vm::compiler::Chunk;
use vm::compiler::OpCode;
use vm::disassembler;
use vm::interpreter::interpret;
use vm::value::Value;

pub fn main() {
    let mut chunk = Chunk::new();
    chunk.push_opcode(OpCode::Constant, CodeRange::from_ints(0, 0, 0, 1, 2, 3));
    chunk.push_constant(Value::Float(13.37));
    chunk.push_opcode(OpCode::Return, CodeRange::from_ints(0, 0, 0, 5, 0, 5));
    disassembler::disassemble_chunk(&chunk, "main chunk", &mut std::io::stdout()).unwrap();
    println!("\n== Running chunk ==");
    interpret(&chunk, true).unwrap();
}
