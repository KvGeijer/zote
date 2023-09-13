use std::io::{self, Write};

use crate::compiler::{Chunk, OpCode};

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

    if let Ok(opcode) = chunk.as_bytes()[offset].try_into() {
        match opcode {
            OpCode::Return => simple_instruction("Return", out),
            OpCode::Constant => constant_instruction("Constant", chunk, offset, out),
            OpCode::Negate => simple_instruction("Negate", out),
            OpCode::Add => simple_instruction("Add", out),
            OpCode::Subtract => simple_instruction("Subtract", out),
            OpCode::Multiply => simple_instruction("Multiply", out),
            OpCode::Divide => simple_instruction("Divide", out),
            OpCode::Not => simple_instruction("Not", out),
            OpCode::Modulo => simple_instruction("Modulo", out),
            OpCode::Power => simple_instruction("Power", out),
            OpCode::Equality => simple_instruction("Equality", out),
            OpCode::NonEquality => simple_instruction("NonEquality", out),
            OpCode::LessThan => simple_instruction("LessThan", out),
            OpCode::LessEqual => simple_instruction("LessEqual", out),
            OpCode::GreaterThan => simple_instruction("GreaterThan", out),
            OpCode::GreaterEqual => simple_instruction("GreaterEqual", out),
        }
    } else {
        simple_instruction("Invalid OpCode", out)
        // Err(DisassemblerError::CustomError(format!(
        //     "Invalid opcode {}\n",
        //     bytes[offset]
        // )))
    }
}

fn simple_instruction<W: Write>(name: &str, out: &mut W) -> io::Result<usize> {
    write!(out, "{name}\n")?;
    Ok(1)
}

fn constant_instruction<W: Write>(
    name: &str,
    chunk: &Chunk,
    offset: usize,
    out: &mut W,
) -> io::Result<usize> {
    let constant = chunk.as_bytes()[offset + 1];
    let value = chunk
        .get_constant(constant)
        .expect("Could not find constant!");
    write!(out, "{:<16} {:4} {:?}\n", name, constant, value)?;
    Ok(2)
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
