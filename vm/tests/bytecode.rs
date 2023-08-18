use std::io::Cursor;

use vm::compiler::bytecode::{Chunk, OpCode};
use vm::disassembler;

fn new_out() -> Cursor<Vec<u8>> {
    Cursor::new(vec![])
}

fn out_to_string(out: Cursor<Vec<u8>>) -> String {
    String::from_utf8(out.into_inner()).unwrap()
}

#[test]
fn disassemble_hello_world() {
    let mut chunk = Chunk::new();
    chunk.push(OpCode::Return as u8);
    chunk.push(OpCode::Return as u8);
    let mut out = new_out();
    disassembler::disassemble_chunk(&chunk, "simple test", &mut out).unwrap();
    assert_eq!(
        out_to_string(out),
        "== simple test ==\n0000 Return\n0001 Return\n"
    );
}
