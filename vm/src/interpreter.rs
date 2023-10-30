mod call_frame;
mod caller;
mod cmp_ops;
mod logic_ops;
mod num_ops;

use std::rc::Rc;

use crate::{
    compiler::{Chunk, OpCode},
    disassembler::disassemble_instruction,
    error::RunRes,
    value::Value,
};

use self::call_frame::CallFrame;

const STACK_SIZE: usize = 4096;
const GLOBALS_SIZE: usize = 256;
const TEMP_STACK_SIZE: usize = 1024;
const FRAMES_SIZE: usize = 128;

struct VM {
    // TODO: Try to have some of these in vecs instead
    /// Keeps the stack of call frames
    call_frames: [CallFrame; FRAMES_SIZE],
    frame_count: usize,

    /// Static storage region of all global values
    globals: [Value; GLOBALS_SIZE],

    /// Stores all temporary values for executing expressions
    temp_stack: [Value; TEMP_STACK_SIZE],
    temp_top: usize,

    /// Stores variables, similar to hardware stack
    ///
    /// Would be nice to merge with temp stack, but it is not easy as statements in expressions -> interleavings of variables and temporaries
    /// Would also like to merge with call_frames, but that at least needs this to store raw bytes
    stack: [Value; STACK_SIZE],
    stack_top: usize,
}

const NIL: Value = Value::Nil;

pub fn interpret(chunk: Rc<Chunk>, debug: bool) -> RunRes<()> {
    let mut vm = VM {
        frame_count: 1,
        // TODO: Unsafe or array_init, or custom macro like `vec!`?
        call_frames: vec![CallFrame::new(chunk); FRAMES_SIZE].try_into().unwrap(),
        temp_top: 0,
        globals: [NIL; GLOBALS_SIZE],
        stack: [NIL; STACK_SIZE],
        stack_top: 0,
        temp_stack: [NIL; TEMP_STACK_SIZE],
    };
    vm.run(debug)
}

impl VM {
    fn run(&mut self, debug: bool) -> RunRes<()> {
        while self.pc() < self.chunk().len() {
            // TODO: Change to compile feature?
            if debug {
                print!(
                    "Frame {}, pc {}, stack top {}: ",
                    self.frame_count - 1,
                    self.pc(),
                    self.stack_top
                );
                disassemble_instruction(&self.chunk(), self.pc(), &mut std::io::stdout())
                    .expect("Could not disassemble an opcode");
            }

            let opcode_pc = self.pc();
            let opcode = self
                .read_byte()
                .try_into()
                .expect("Cannot read opcode at expected ip");
            match self.handle_opcode(opcode) {
                Ok(InstrResult::Ok) => (),
                Ok(InstrResult::Return) => return Ok(()),
                Err(mut error) => {
                    // TODO: Add whole stack trace. Change so that an opcode just returns Res<..., String>
                    eprintln!("ERROR!!! TODO: FIX TRACE");
                    error.add_trace(
                        "script".to_string(),
                        self.chunk()
                            .get_range(opcode_pc)
                            .expect("OpCodes should have code loc debug info stored")
                            .clone(),
                    );
                    return Err(error);
                }
            }
            if debug && self.temp_top > 0 {
                println!("Top value: {}", self.temp_stack[self.temp_top - 1])
            }
        }
        Ok(())
    }

    fn handle_opcode(&mut self, opcode: OpCode) -> RunRes<InstrResult> {
        match opcode {
            OpCode::Return => {
                let ret_val = self.pop();
                if self.frame_count == 1 {
                    // self.pop(); // TODO: Book pushes a script func here, which we should push in case we also do
                    return Ok(InstrResult::Return);
                }
                self.stack_top = self.frame().rbp;
                self.temp_top = self.frame().root_temp_pointer;
                self.push(ret_val);

                // Lower the frame
                self.frame_count -= 1;
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
                let x = self.peek();
                self.globals[offset as usize] = x;
            }
            OpCode::ReadGlobal => {
                let offset = self.read_byte();
                let global = self.globals[offset as usize].clone();
                self.push(global)
            }
            OpCode::AssignLocal => {
                let offset = self.read_byte();
                let x = self.peek();
                self.stack[self.rbp() + offset as usize] = x;
            }
            OpCode::ReadLocal => {
                let offset = self.read_byte();
                let local = self.stack[self.rbp() + offset as usize].clone();
                self.push(local)
            }
            OpCode::JumpIfFalse => {
                let pred = self.pop();
                let jump = i16::from_be_bytes(self.read_2bytes());
                if !pred.truthy()? {
                    self.jump(jump);
                }
            }
            OpCode::Jump => {
                let jump = i16::from_be_bytes(self.read_2bytes());
                self.jump(jump)
            }
            OpCode::Discard => {
                self.pop();
            }
            OpCode::Call => {
                let arg_count = self.read_byte() as usize;
                let callee = self.stack[self.stack_top - arg_count - 1].clone();
                self.call_value(callee, arg_count)?;
            }
            OpCode::FromTemp => {
                self.stack[self.stack_top] = self.temp_stack[self.temp_top - 1].clone();
                self.temp_top -= 1;
                self.stack_top += 1;
            }
        }

        Ok(InstrResult::Ok)
    }

    fn read_byte(&mut self) -> u8 {
        let pc = self.pc();
        let byte = self.chunk()[pc];
        self.jump(1); // TODO: Try just incrementing without i16
        byte
    }

    fn read_2bytes(&mut self) -> [u8; 2] {
        let pc = self.pc();
        let bytes = [self.chunk()[pc], self.chunk()[pc + 1]];
        self.jump(2);
        bytes
    }

    fn read_constant(&mut self) -> Value {
        let offset = self.read_byte();
        self.chunk()
            .get_constant(offset)
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

    /// Peeks the topmost temp value
    fn peek(&mut self) -> Value {
        self.peek_many(1)
    }

    /// Peeks a value further back on the stack
    fn peek_many(&mut self, offset: usize) -> Value {
        self.temp_stack[self.temp_top - offset].clone()
    }

    /// Gets the current call frame
    fn frame(&self) -> &CallFrame {
        &self.call_frames[self.frame_count - 1]
    }

    /// Gets a mutable reference to the current call frame
    fn frame_mut(&mut self) -> &mut CallFrame {
        &mut self.call_frames[self.frame_count - 1]
    }

    /// Gets the pc in the current call frame
    fn pc(&self) -> usize {
        self.frame().pc
    }

    /// Gets the rbp in the current call frame
    fn rbp(&self) -> usize {
        self.frame().rbp
    }

    /// Gets the chunk in the current call frame
    fn chunk(&self) -> &Chunk {
        self.frame().chunk()
    }

    /// Performs a jump in the current chunk, from the pc with size offset
    fn jump(&mut self, offset: i16) {
        let new_pc = add_i16_to_usize(self.pc(), offset);
        self.frame_mut().pc = new_pc;
    }
}

// The different results from a sucsesfully evaled opcode
enum InstrResult {
    Ok,
    Return,
}

fn add_i16_to_usize(value: usize, diff: i16) -> usize {
    if diff.is_positive() {
        value + diff as usize
    } else {
        value - diff.abs() as usize
    }
}
