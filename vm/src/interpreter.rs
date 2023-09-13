mod num_ops;

use crate::{
    compiler::{Chunk, OpCode},
    disassembler::disassemble_instruction,
    error::RunRes,
    value::Value,
};

const STACK_SIZE: usize = 4096;

struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize, // Could be &[u8] pointing somewhere into the chunk code as well. TODO: Try this instead
    /// The first emply location on the stack
    stack_top: usize,
    stack: [Value; STACK_SIZE],
}

const NIL: Value = Value::Nil;

pub fn interpret(chunk: &Chunk, debug: bool) -> RunRes<()> {
    let mut vm = VM {
        chunk,
        ip: 0,
        stack_top: 0,
        stack: [NIL; STACK_SIZE],
    };
    vm.run(debug)
}

impl<'a> VM<'a> {
    fn run(&mut self, debug: bool) -> RunRes<()> {
        while self.ip < self.chunk.as_bytes().len() {
            if debug {
                disassemble_instruction(&self.chunk, self.ip, &mut std::io::stdout())
                    .expect("Could not disassemble an opcode");
            }

            let opcode = self.chunk.as_bytes()[self.ip]
                .try_into()
                .expect("Cannot read opcode at expected ip");
            self.ip += 1;
            match opcode {
                OpCode::Return => {
                    println!("{:?}", self.pop());
                    return Ok(());
                }
                OpCode::Constant => {
                    // Deserialize the constant
                    let constant = self.read_constant();
                    self.push(constant);
                }
                OpCode::Negate => {
                    let x = self.pop();
                    self.push(num_ops::negate(x)?);
                }
                OpCode::Add => {
                    let y = self.pop();
                    let x = self.pop();
                    self.push(num_ops::add(x, y)?);
                }
                OpCode::Subtract => {
                    let y = self.pop();
                    let x = self.pop();
                    self.push(num_ops::sub(x, y)?);
                }
                OpCode::Multiply => {
                    let y = self.pop();
                    let x = self.pop();
                    self.push(num_ops::mult(x, y)?);
                }
                OpCode::Divide => {
                    let y = self.pop();
                    let x = self.pop();
                    self.push(num_ops::div(x, y)?);
                }
            }
            if debug && self.stack_top > 0 {
                println!("Top value: {:?}", self.stack[self.stack_top - 1])
            }
        }
        Ok(())
    }

    fn read_byte(&mut self) -> usize {
        let ip = self.ip;
        self.ip += 1;
        ip
    }

    fn read_constant(&mut self) -> Value {
        self.chunk
            .get_constant(self.chunk.as_bytes()[self.read_byte()])
            .expect("Could not find constant!")
            .clone()
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top].clone()
    }
}
