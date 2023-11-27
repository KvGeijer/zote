use parser::{CodeRange, ExprNode, LValue};

use crate::{
    compiler::{Chunk, CompRes, CompRetRes, Compiler, OpCode},
    value::Function,
};

impl Compiler<'_> {
    /// Compiles a function definition
    ///
    /// Args:
    ///    * locals: the max nubmer of locals (param + locals) declared at any given time
    pub fn compile_function_def(
        &mut self,
        name: &str,
        rec_name: Option<&String>,
        params: &[LValue],
        body: &ExprNode,
        upvalues: &[String],
        nbr_locals: usize,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        // Enter a new scope for the function
        self.locals.nest();

        // Declare the upvalues in the function scope
        for upvalue_name in upvalues {
            self.locals.add_upvalue(upvalue_name.to_string());
        }

        if let Some(binding) = rec_name {
            // The function should be able to call itself, so add it as an argument (it is also pushed before args on stack)
            self.locals.add_local(binding.to_owned(), false);
        } else {
            // Just add a dummy-value so that it cannot be refered to
            // ERROR: If we can create "" variable, or create conflicting dummy
            self.locals.add_local("".to_string(), false);
        }

        // The new chunk to use for the function
        let mut func_chunk = Chunk::new();

        // Add all of the parameters as reachable locals
        // The function and locals take up the first `arity + 1` spots in the call frame
        // Then they also take up one extra spot for each parameter which is a pattern match
        let extra_locals = self.declare_parameters(params, range.clone(), &mut func_chunk)?;
        self.compile_parameter_expansion(params, range.clone(), &mut func_chunk)?;

        // Compile the actual body into the func chunk
        self.compile_expression(body, &mut func_chunk)?;

        // Return implicitly in case of no other return
        func_chunk.push_opcode(OpCode::Return, range.clone());

        // Exit the function scope
        self.locals.de_nest();

        let func = Function::new(
            params.len() as u8,
            nbr_locals + extra_locals,
            name.to_string(),
            func_chunk,
        );

        // self.add_function(func);
        // chunk.push_constant_plus(func.into(), range);

        // Push which function to use for initializing the closure
        chunk.push_opcode(OpCode::InitClosure, range.clone());
        chunk.push_constant(func.into());

        // Push how many upvalues to capture
        chunk.push_u8_offset(upvalues.len() as u8);

        // Push the upvalues
        for upvalue in upvalues {
            if let Some((stack_offset, pointer)) = self.locals.get_local(upvalue) {
                // Captured from stack
                assert!(pointer);
                chunk.push_bool(false);
                chunk.push_u8_offset(stack_offset);
            } else if let Some(upvalue_index) = self.locals.get_upvalue(upvalue) {
                // Nested upvalue
                chunk.push_bool(true);
                chunk.push_u8_offset(upvalue_index);
            } else {
                // Not declared! However, this should then not be flagged as an upvalue!
                panic!("Upvalue from semantic analysis not found during compile phase!")
            }
        }

        Ok(())
    }

    /// Declares all the parameters as locals, handling pattern matching expanding
    ///
    /// Returns the extra number of locals needed to store arguments to expand into pattern matches
    fn declare_parameters(
        &mut self,
        params: &[LValue],
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRetRes<usize> {
        // This is the nubmer of params which pattern match.
        // Every such argument will be placed in its topmots form as an argument,
        // and then it will be unpacked into local variables (the inner variables).
        let mut extra_param_slots = params.len();

        for param in params {
            // Parameters are just local variables in outermost scope
            if let LValue::Var(name) = param {
                // Set already-inplace to signify we have already pushed the value at this point
                self.declare_local_var(name, true, range.clone(), chunk);

                // This uses a normal spot, so we will not need an extra slot for it
                extra_param_slots -= 1;
            } else {
                // Declare a dummy local as we will not be able to directly access the value placed in this argument location
                self.declare_local_var(&"".to_string(), true, range.clone(), chunk);
            }
        }

        Ok(extra_param_slots)
    }

    /// Compiles expansion of all arguments into pattern matches of parameters
    fn compile_parameter_expansion(
        &mut self,
        params: &[LValue],
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        for (param_ind, param) in params.iter().enumerate() {
            if !matches!(param, LValue::Var(_)) {
                // Pattern matching lvalue. The arg is stored at the arg_ind offset from rbp

                // 1: Declare the variables as locals
                self.declare_local(param, range.clone(), chunk)?;

                // 2: Read the matching arg
                chunk.push_opcode(OpCode::ReadLocal, range.clone());
                chunk.push_u8_offset((param_ind + 1) as u8);

                // 3: assign the read value into the pattern
                self.compile_assign(param, range.clone(), chunk)?;
            }
        }

        Ok(())
    }

    /// So far can only handle hard-coded print calls
    pub fn compile_call(
        &mut self,
        func: &ExprNode,
        args: &[ExprNode],
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        if args.len() > 255 {
            return Err("Cannot have more than 255 arguments".to_string());
        }

        // Push the bound function variable to the stack
        self.compile_expression(func, chunk)?;

        // Push the arguments on the stack.
        // HOWEVER: These must be re-assigned in a pattern matching way when calling
        for arg in args {
            self.compile_expression(arg, chunk)?;
        }

        chunk.push_opcode(OpCode::Call, range);
        chunk.push_u8_offset(args.len() as u8);

        Ok(())
    }
}
