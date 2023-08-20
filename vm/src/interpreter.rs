use crate::{
    compiler::{Chunk, OpCode},
    disassembler::disassemble_instruction,
};

struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize, // Could be &[u8] pointing somewhere into the chunk code as well. TODO: Try this instead
}

#[derive(Debug, Clone, Copy)]
pub enum InterpreterError {
    CompileError,
    RuntimeError,
}

pub fn interpret(chunk: &Chunk, debug: bool) -> Result<(), InterpreterError> {
    let mut vm = VM { chunk, ip: 0 };
    vm.run(debug)
}

impl<'a> VM<'a> {
    fn run(&mut self, debug: bool) -> Result<(), InterpreterError> {
        loop {
            if debug {
                disassemble_instruction(&self.chunk, self.ip, &mut std::io::stdout())
                    .map_err(|_| InterpreterError::RuntimeError)?;
            }

            let opcode = self.chunk.as_bytes()[self.ip]
                .try_into()
                .map_err(|_| InterpreterError::RuntimeError)?;
            self.ip += 1;
            match opcode {
                OpCode::Return => return Ok(()),
            }
        }
    }
}
