use parser::CodeRange;
use std::io::Cursor;

use vm::compiler::{Chunk, OpCode};
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
    chunk.push_opcode(OpCode::Return, CodeRange::from_ints(0, 0, 0, 5, 0, 5));
    chunk.push_opcode(OpCode::Return, CodeRange::from_ints(6, 0, 6, 11, 0, 11));
    let mut out = new_out();
    disassembler::disassemble_chunk(&chunk, "simple test", &mut out).unwrap();
    assert_eq!(
        out_to_string(out),
        "== simple test ==\n0000   0:0  -  0:5   Return\n0001   0:6  -  0:11  Return\n"
    );
}
