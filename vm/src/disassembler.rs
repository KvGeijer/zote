use std::io::{self, Write};

use crate::{
    compiler::{Chunk, OpCode},
    value::Value,
};

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

    write!(out, "====== <{name}> ======\n")?;
    while offset < chunk.len() {
        offset += disassemble_instruction(chunk, offset, out)?;
    }
    write!(out, "===== <!{name}> =====\n\n")?;
    Ok(())
}

pub fn disassemble_instruction<W: Write>(
    chunk: &Chunk,
    offset: usize,
    out: &mut W,
) -> Result<usize, DisassemblerError> {
    write!(out, "{:04} ", offset)?;
    write_coderange(chunk, offset, out)?;

    if let Ok(opcode) = chunk[offset].try_into() {
        match opcode {
            OpCode::Return => simple_instruction("Return", out),
            OpCode::Constant => constant_instruction("Constant", chunk, offset, out),
            OpCode::Nil => simple_instruction("Nil", out),
            OpCode::True => simple_instruction("True", out),
            OpCode::False => simple_instruction("False", out),
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
            OpCode::AssignGlobal => offset_instruction("AssignGlobal", chunk, offset, out),
            OpCode::ReadGlobal => offset_instruction("ReadGlobal", chunk, offset, out),
            OpCode::AssignLocal => offset_instruction("AssignLocal", chunk, offset, out),
            OpCode::ReadLocal => offset_instruction("ReadLocal", chunk, offset, out),
            OpCode::JumpIfFalse => jump_instruction("JumpIfFalse", chunk, offset, out),
            OpCode::Jump => jump_instruction("Jump", chunk, offset, out),
            OpCode::Discard => simple_instruction("Discard", out),
            OpCode::Call => offset_instruction("Call", chunk, offset, out),
            OpCode::AssignUpValue => offset_instruction("AssignUpValue", chunk, offset, out),
            OpCode::ReadUpValue => offset_instruction("AssignUpValue", chunk, offset, out),
            OpCode::InitClosure => closure_init(chunk, offset, out),
            OpCode::AssignPointer => offset_instruction("AssignPointer", chunk, offset, out),
            OpCode::ReadPointer => offset_instruction("ReadPointer", chunk, offset, out),
            OpCode::Drop => offset_instruction("Drop", chunk, offset, out),
            OpCode::EmptyPointer => simple_instruction("EmptyPointer", out),
            OpCode::AssignAtIndex => simple_instruction("AssignAtIndex", out),
            OpCode::ReadAtIndex => simple_instruction("ReadAtIndex", out),
            OpCode::ListFromSlice => simple_instruction("ListFromSlice", out),
            OpCode::ListFromValues => offset_instruction("ListFromValues", chunk, offset, out),
            OpCode::ReadAtSlice => simple_instruction("ReadAtSlice", out),
            OpCode::Duplicate => simple_instruction("Duplicate", out),
        }
    } else {
        simple_instruction("Invalid OpCode", out)
    }
}

fn simple_instruction<W: Write>(name: &str, out: &mut W) -> Result<usize, DisassemblerError> {
    write!(out, "{name}\n")?;
    Ok(1)
}

fn constant_instruction<W: Write>(
    name: &str,
    chunk: &Chunk,
    offset: usize,
    out: &mut W,
) -> Result<usize, DisassemblerError> {
    let constant = chunk[offset + 1];
    let value = chunk
        .get_constant(constant)
        .expect("Could not find constant!");
    if let Value::Function(func) = value {
        write!(out, "{:<16} {:4} {:?}\n", name, constant, func.name())?;
        disassemble_chunk(func.chunk_ref(), func.name(), out)?;
    } else {
        write!(out, "{:<16} {:4} {:?}\n", name, constant, value)?;
    }
    Ok(2)
}

fn offset_instruction<W: Write>(
    name: &str,
    chunk: &Chunk,
    op_offset: usize,
    out: &mut W,
) -> Result<usize, DisassemblerError> {
    let offset = chunk[op_offset + 1];
    write!(out, "{:<16} {:4}\n", name, offset)?;
    Ok(2)
}

fn jump_instruction<W: Write>(
    name: &str,
    chunk: &Chunk,
    op_offset: usize,
    out: &mut W,
) -> Result<usize, DisassemblerError> {
    let offset = i16::from_be_bytes([chunk[op_offset + 1], chunk[op_offset + 2]]);
    write!(out, "{:<16} {:4}\n", name, offset)?;
    Ok(3)
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

fn closure_init<W: Write>(
    chunk: &Chunk,
    op_offset: usize,
    out: &mut W,
) -> Result<usize, DisassemblerError> {
    let offset = chunk[op_offset + 1];
    let function_value = chunk
        .get_constant(offset)
        .ok_or(DisassemblerError::CustomError(
            "Could not find function for closure".to_string(),
        ))?;
    let Value::Function(function) = function_value else {
        return Err(DisassemblerError::CustomError(
            "Trying to init closure from non-function!".to_string(),
        ));
    };

    let nbr_upvalues = chunk[op_offset + 2] as usize;
    write!(
        out,
        "{:<19} {:<3} {:<}\n",
        "InitClosure",
        nbr_upvalues,
        function.name()
    )?;
    for offset in 0..nbr_upvalues {
        let from_upvalue = chunk[op_offset + 3 + 2 * offset] != 0;
        let stack_offset = chunk[op_offset + 4 + 2 * offset];
        write!(out, "{:04} ", op_offset + 4 + 2 * offset)?;
        write!(
            out,
            "{:15} {:<19} {:<7} {}\n",
            "",
            " |",
            if from_upvalue { "upvalue" } else { "local" },
            stack_offset
        )?;
    }
    disassemble_chunk(function.chunk_ref(), function.name(), out)?;

    Ok(3 + 2 * nbr_upvalues)
}
