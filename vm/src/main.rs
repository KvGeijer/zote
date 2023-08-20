use parser::CodeRange;

use vm::compiler::Chunk;
use vm::compiler::OpCode;
use vm::disassembler;
use vm::interpreter::interpret;

pub fn main() {
    let mut chunk = Chunk::new();
    chunk.push_opcode(OpCode::Return, CodeRange::from_ints(0, 0, 0, 5, 0, 5));
    disassembler::disassemble_chunk(&chunk, "main chunk", &mut std::io::stdout()).unwrap();
    println!("\n== Running chunk ==");
    interpret(&chunk, true).unwrap();
}
