mod cmp_ops;
mod locals;
mod logic_ops;
mod num_ops;

use crate::{
    compiler::{Chunk, OpCode},
    disassembler::disassemble_instruction,
    error::RunRes,
    value::Value,
};

const STACK_SIZE: usize = 4096;
const GLOBALS_SIZE: usize = 256;
const TEMP_STACK_SIZE: usize = 1024;

struct VM<'a> {
    chunk: &'a Chunk,
    pc: usize, // Could be &[u8] pointing somewhere into the chunk code as well. TODO: Try this instead
    /// The first emply location on the stack
    rbp: usize, // Base pointers points to the base of the current stack frame
    temp_top: usize,
    globals: [Value; GLOBALS_SIZE], // Similar to having a region for globals
    temp_stack: [Value; TEMP_STACK_SIZE], // Stores all temporary values for executing expressions
    stack: [Value; STACK_SIZE], // Stores variables, as the C stack (can't merge with temps due to stmts in exprs)
}

const NIL: Value = Value::Nil;

pub fn interpret(chunk: &Chunk, debug: bool) -> RunRes<()> {
    let mut vm = VM {
        chunk,
        pc: 0,
        temp_top: 0,
        rbp: 0,
        globals: [NIL; GLOBALS_SIZE],
        stack: [NIL; STACK_SIZE],
        temp_stack: [NIL; TEMP_STACK_SIZE],
    };
    // TODO: Where do we handle the error print?
    vm.run(debug)
}

impl<'a> VM<'a> {
    fn run(&mut self, debug: bool) -> RunRes<()> {
        while self.pc < self.chunk.len() {
            // TODO: Change to compile feature?
            if debug {
                disassemble_instruction(&self.chunk, self.pc, &mut std::io::stdout())
                    .expect("Could not disassemble an opcode");
            }

            let opcode_ip = self.pc;
            let opcode = self
                .read_byte()
                .try_into()
                .expect("Cannot read opcode at expected ip");
            match self.handle_opcode(opcode) {
                Ok(InstrResult::Ok) => (),
                Ok(InstrResult::Return) => return Ok(()),
                Err(mut error) => {
                    // TODO: Add whole stack trace
                    error.add_trace(
                        "script".to_string(),
                        self.chunk
                            .get_range(opcode_ip)
                            .expect("OpCodes should have code loc debug info stored")
                            .clone(),
                    );
                    return Err(error);
                }
            }
            if debug && self.temp_top > 0 {
                println!("Top value: {:?}", self.temp_stack[self.temp_top - 1])
            }
        }
        Ok(())
    }

    fn handle_opcode(&mut self, opcode: OpCode) -> RunRes<InstrResult> {
        match opcode {
            OpCode::Return => {
                println!("{:?}", self.pop());
                return Ok(InstrResult::Return);
            }
            OpCode::Constant => {
                // Deserialize the constant
                let constant = self.read_constant();
                self.push(constant);
            }
            OpCode::Nil => self.push(Value::Nil),
            OpCode::True => self.push(Value::Bool(true)),
            OpCode::False => self.push(Value::Bool(false)),
            OpCode::Negate => {
                let x = self.pop();
                self.push(num_ops::negate(x)?);
            }
            OpCode::Not => {
                let x = self.pop();
                self.push(logic_ops::not(x)?);
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
            OpCode::Modulo => {
                let y = self.pop();
                let x = self.pop();
                self.push(num_ops::modulo(x, y)?);
            }
            OpCode::Power => {
                let y = self.pop();
                let x = self.pop();
                self.push(num_ops::power(x, y)?);
            }
            OpCode::Equality => {
                let y = self.pop();
                let x = self.pop();
                self.push(cmp_ops::equal(x, y)?);
            }
            OpCode::NonEquality => {
                let y = self.pop();
                let x = self.pop();
                self.push(cmp_ops::not_equal(x, y)?);
            }
            OpCode::LessThan => {
                let y = self.pop();
                let x = self.pop();
                self.push(cmp_ops::less(x, y)?);
            }
            OpCode::LessEqual => {
                let y = self.pop();
                let x = self.pop();
                self.push(cmp_ops::less_eq(x, y)?);
            }
            OpCode::GreaterThan => {
                let y = self.pop();
                let x = self.pop();
                self.push(cmp_ops::greater(x, y)?);
            }
            OpCode::GreaterEqual => {
                let y = self.pop();
                let x = self.pop();
                self.push(cmp_ops::greater_eq(x, y)?);
            }
            OpCode::AssignGlobal => {
                let offset = self.read_byte();
                let x = self.pop();
                self.globals[offset as usize] = x;
            }
            OpCode::ReadGlobal => {
                let offset = self.read_byte();
                let global = self.globals[offset as usize].clone();
                self.push(global)
            }
            OpCode::AssignLocal => {
                let offset = self.read_byte();
                let x = self.pop();
                self.stack[self.rbp + offset as usize] = x;
            }
            OpCode::ReadLocal => {
                let offset = self.read_byte();
                let local = self.stack[self.rbp + offset as usize].clone();
                self.push(local)
            }
            OpCode::Print => {
                let x = self.pop();
                println!("{:?}", x);
                self.push(x);
            }
        }

        Ok(InstrResult::Ok)
    }

    fn read_byte(&mut self) -> u8 {
        let ip = self.pc;
        let byte = self.chunk[ip];
        self.pc += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        self.chunk
            .get_constant(self.read_byte())
            .expect("Could not find constant!")
            .clone()
    }

    /// Pushes the topmost temp value
    fn push(&mut self, value: Value) {
        self.temp_stack[self.temp_top] = value;
        self.temp_top += 1;
    }

    /// Pops the topmost temp value
    fn pop(&mut self) -> Value {
        self.temp_top -= 1;
        self.temp_stack[self.temp_top].clone()
    }
}

// The different results from a sucsesfully evaled opcode
enum InstrResult {
    Ok,
    Return,
}
