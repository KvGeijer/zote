use parser::CodeRange;

use vm::compiler::bytecode::OpCode;
use vm::compiler::chunk::Chunk;
use vm::disassembler;

pub fn main() {
    let mut chunk = Chunk::new();
    chunk.push_opcode(OpCode::Return, CodeRange::from_ints(0, 0, 0, 5, 0, 5));
    disassembler::disassemble_chunk(&chunk, "main test", &mut std::io::stdout()).unwrap();
}
