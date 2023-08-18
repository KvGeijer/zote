use vm::compiler::bytecode::{Chunk, OpCode};
use vm::disassembler;

pub fn main() {
    let mut chunk = Chunk::new();
    chunk.push(OpCode::Return as u8);
    disassembler::disassemble_chunk(&chunk, "main test", &mut std::io::stdout()).unwrap();
}
