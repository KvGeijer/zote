use std::io::{self, Write};

use crate::compiler::bytecode::{Chunk, OpCode};

#[derive(Debug)]
pub enum DisassemblerError {
    IOError(io::Error),
    CustomError(String),
}

impl From<io::Error> for DisassemblerError {
    fn from(error: io::Error) -> Self {
        DisassemblerError::IOError(error)
    }
}

pub fn disassemble_chunk<W: Write>(
    chunk: &Chunk,
    name: &str,
    out: &mut W,
) -> Result<(), DisassemblerError> {
    let mut offset = 0;
    let bytes = chunk.as_bytes();

    write!(out, "== {name} ==\n")?;
    while offset < bytes.len() {
        offset += disassemble_instruction(bytes, offset, out)?;
    }
    Ok(())
}

pub fn disassemble_instruction<W: Write>(
    bytes: &[u8],
    offset: usize,
    out: &mut W,
) -> io::Result<usize> {
    write!(out, "{:04} ", offset)?;

    match bytes[offset].try_into() {
        Err(_) => {
            simple_instruction("Invalid OpCode", out)
            // Err(DisassemblerError::CustomError(format!(
            //     "Invalid opcode {}\n",
            //     bytes[offset]
            // )))
        }
        Ok(OpCode::Return) => simple_instruction("Return", out),
    }
}

pub fn simple_instruction<W: Write>(name: &str, out: &mut W) -> io::Result<usize> {
    write!(out, "{name}\n")?;
    Ok(1)
}
