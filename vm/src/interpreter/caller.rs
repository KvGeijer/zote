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

        if let Value::Function(func) = callee {
            // Create the next call frame
            let new_rbp = self.stack_top - arg_count - 1;
            let current_rtp = self.temp_top;

            // Change to it, and init
            self.frame_count += 1;
            self.frame_mut().init(func.chunk_rc(), new_rbp, current_rtp);

            Ok(())
        } else if let Value::Native(native) = callee {
            let args = self.stack[(self.stack_top - arg_count)..self.stack_top].to_vec();
            let ret = native.call(args)?;
            self.push(ret);

            // Remove the args and function from the stack
            self.stack_top -= arg_count + 1;

            Ok(())
        } else {
            RunRes::new_err(format!("Can only call functions, not {}", callee.type_of()))
        }
    }
}
