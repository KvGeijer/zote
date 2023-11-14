use crate::{
    error::{RunRes, RunResTrait},
    value::Value,
};

use super::{FRAMES_SIZE, VM};

impl VM {
    pub fn call_value(&mut self, callee: Value, arg_count: usize) -> RunRes<()> {
        if self.frame_count == FRAMES_SIZE - 1 {
            return RunRes::new_err(format!(
                "STACK OVERFLOW: Exceeded max level of nesting ({FRAMES_SIZE})"
            ));
        }

        match callee {
            Value::Function(_func) => {
                panic!("Should now not allow calls of raw functions")
                // // Create the next call frame
                // let new_rbp = self.stack_top - arg_count - 1;
                // let current_rtp = self.temp_top;

                // // Change to it, and init
                // self.frame_count += 1;
                // self.frame_mut().init(func.chunk_rc(), new_rbp, current_rtp);

                // Ok(())
            }
            Value::Closure(closure) => {
                // Create the next call frame
                // The closure and args should be pushed on the stack
                let new_rbp = self.stack_top - 1 - arg_count;

                // Change to it, and init
                self.frame_count += 1;
                self.frame_mut().init(closure.chunk_rc(), new_rbp);

                // Increment the stack top to cover all eventual local variables
                self.stack_top += closure.nbr_locals() - arg_count;
                Ok(())
            }
            Value::Native(native) => {
                let args = self.stack[(self.stack_top - arg_count)..self.stack_top].to_vec();
                let ret = native.call(args)?;

                // Remove the args and function from the stack
                // ERROR: If there are Upvalues here, this will destroy shit, but that should not be the case
                self.stack_top -= arg_count + 1;

                // Then push the return value
                self.push(ret);

                Ok(())
            }
            _ => RunRes::new_err(format!("Can only call functions, not {}", callee.type_of())),
        }
    }
}
