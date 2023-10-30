use parser::{CodeRange, ExprNode, LValue};

use crate::{
    compiler::{Chunk, CompRes, Compiler, OpCode},
    value::Function,
};

impl Compiler {
    pub fn compile_function_def(
        &mut self,
        name: &str,
        params: &[LValue],
        body: &ExprNode,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        let mut func_compiler = Compiler::new(false);

        // ERROR: This should be right, as they share the global address space
        func_compiler.globals = self.globals.clone();

        // The function should be able to call itself, so add it as an argument
        func_compiler.locals.add_local(name.to_string());

        // Add all of the parameters as locals
        for param in params {
            // Parameters are just local variables in outermost scope
            func_compiler.declare_local(param, true)?;
        }
        // The function and locals take up the first `arity + 1` spots in the call frame

        let mut func_chunk = Chunk::new();
        func_compiler.compile_expression(body, &mut func_chunk)?;

        // Return implicitly in case of no other return
        func_chunk.push_opcode(OpCode::Return, range.clone());

        let func = Function::new(params.len() as u8, name.to_string(), func_chunk);

        // self.add_function(func);
        chunk.push_constant_plus(func.into(), range);

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

        // Keep special case for print a while
        // if let Expr::Var(name) = func.node.as_ref() {
        //     if name == "print" {
        //         // TODO: Fix
        //         let _ = self.compile_expression(&args[0], chunk);
        //         chunk.push_opcode(OpCode::Print, range);
        //         return Ok(());
        //     }
        // }

        // Push the bound function variable to the stack
        self.compile_expression(func, chunk)?;
        chunk.push_opcode(OpCode::FromTemp, func.range());

        // Push the args onto the stack
        for arg in args {
            self.compile_expression(arg, chunk)?;
            chunk.push_opcode(OpCode::FromTemp, func.range());
        }

        chunk.push_opcode(OpCode::Call, range);
        chunk.push_u8_offset(args.len() as u8);

        Ok(())
    }
}
