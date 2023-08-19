use std::io::{self, Write};

use crate::compiler::bytecode::OpCode;
use crate::compiler::chunk::Chunk;

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

    write!(out, "== {name} ==\n")?;
    while offset < chunk.as_bytes().len() {
        offset += disassemble_instruction(chunk, offset, out)?;
    }
    Ok(())
}

pub fn disassemble_instruction<W: Write>(
    chunk: &Chunk,
    offset: usize,
    out: &mut W,
) -> io::Result<usize> {
    write!(out, "{:04} ", offset)?;
    write_coderange(chunk, offset, out)?;

    match chunk.as_bytes()[offset].try_into() {
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

fn simple_instruction<W: Write>(name: &str, out: &mut W) -> io::Result<usize> {
    write!(out, "{name}\n")?;
    Ok(1)
}

pub fn write_coderange<W: Write>(chunk: &Chunk, offset: usize, out: &mut W) -> io::Result<()> {
    if let Some(range) = chunk.get_range(offset) {
        write!(
            out,
            "{:3}:{:<3}-{:3}:{:<3} ",
            range.sl(),
            range.sc(),
            range.el(),
            range.ec()
        )
    } else {
        write!(out, "{:15} ", "?No location?")
    }
}
