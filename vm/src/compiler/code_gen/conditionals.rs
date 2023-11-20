use parser::{CodeRange, ExprNode, LValue};

use crate::{
    compiler::{Chunk, Compiler, OpCode},
    value::Value,
};

use super::CompRes;

impl Compiler<'_> {
    pub fn compile_if(
        &mut self,
        pred: &ExprNode,
        then: &ExprNode,
        otherwise: Option<&ExprNode>,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        self.compile_expression(pred, chunk)?;
        chunk.push_opcode(OpCode::JumpIfFalse, range.clone());
        let reserved_else = chunk.reserve_jump();
        self.compile_expression(then, chunk)?;

        chunk.push_opcode(OpCode::Jump, range);
        let reserved_end = chunk.reserve_jump();

        chunk.patch_reserved_jump(reserved_else); // Jump to the beginning of else clause
        self.compile_opt_expression(otherwise, chunk)?;
        chunk.patch_reserved_jump(reserved_end); // Jump to end of if statement

        Ok(())
    }

    pub fn compile_and(
        &mut self,
        lhs: &ExprNode,
        rhs: &ExprNode,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        self.compile_expression(lhs, chunk)?;

        // If false, abort
        chunk.push_opcode(OpCode::JumpIfFalse, range.clone());
        let reserved_false = chunk.reserve_jump();

        self.compile_expression(rhs, chunk)?;

        // Use what is on the stack
        chunk.push_opcode(OpCode::Jump, range.clone());
        let reserved_keep = chunk.reserve_jump();

        // Aborted, use false
        chunk.patch_reserved_jump(reserved_false); // Push extra false
        chunk.push_constant_plus(Value::Bool(false), range);

        chunk.patch_reserved_jump(reserved_keep); // finish the and

        Ok(())
    }

    pub fn compile_or(
        &mut self,
        lhs: &ExprNode,
        rhs: &ExprNode,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        self.compile_expression(lhs, chunk)?;

        // If false, look at rhs as well
        chunk.push_opcode(OpCode::JumpIfFalse, range.clone());
        let reserved_false = chunk.reserve_jump();

        // If true, just jump to end, and push true
        chunk.push_opcode(OpCode::Jump, range.clone());
        let reserved_true = chunk.reserve_jump();

        // Look at rhs, then jump to the exit
        chunk.patch_reserved_jump(reserved_false);
        self.compile_expression(rhs, chunk)?;
        chunk.push_opcode(OpCode::Jump, range.clone());
        let reserved_exit = chunk.reserve_jump();

        // Short-circuit, push true
        chunk.patch_reserved_jump(reserved_true);
        chunk.push_constant_plus(Value::Bool(true), range);
        chunk.patch_reserved_jump(reserved_exit); // Finished

        Ok(())
    }

    pub fn compile_while(
        &mut self,
        pred: &ExprNode,
        body: &ExprNode,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        // Start of every loop iteration
        let start_label = chunk.len();
        self.flow_points.push_loop_entry(start_label);

        // Evaluate predicate, potentially exiting
        self.compile_expression(pred, chunk)?;
        chunk.push_opcode(OpCode::JumpIfFalse, range.clone());
        self.flow_points.push_loop_exit(chunk.reserve_jump());

        // Evaluate body, potentially containing wierd control flow
        self.compile_expression(body, chunk)?;

        // Jump back to the start
        chunk.push_opcode(OpCode::Jump, range);
        chunk.push_jump(start_label);

        // Close the loop
        self.flow_points.close_loop(chunk)
    }

    /// Compiles a for loop
    ///
    /// Pushes the collection as some inxedable collection (with enumerable indexes) to the stack
    pub fn compile_for(
        &mut self,
        lvalue: &LValue,
        collection: &ExprNode,
        body: &ExprNode,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        // Push the iterable
        self.compile_expression(collection, chunk)?;
        chunk.push_opcode(OpCode::TopToIter, range.clone());

        // Push the index of the iterable
        chunk.push_constant_plus(Value::Int(0), range.clone());

        // Start of every loop iteration
        let start_label = chunk.len();
        self.flow_points.push_loop_entry(start_label);

        // Get next item from iterable, potentially abandoning loop
        chunk.push_opcode(OpCode::NextOrJump, range.clone());
        self.flow_points.push_loop_exit(chunk.reserve_jump());

        // Enter a new scope for the loop
        self.locals.enter();

        // Assign the indexed value into the lvalue. Crash if non-compatible
        self.declare_local(lvalue, range.clone(), chunk, true)?;
        self.compile_assign(lvalue, range.clone(), chunk)?;

        // Evaluate body, potentially containing wierd control flow
        self.compile_expression(body, chunk)?;

        // Discard the value from the body
        // TODO: Maybe keep this and store it in a list in the future?
        chunk.push_opcode(OpCode::Discard, range.clone());

        self.locals.exit();

        // Jump back to the start
        chunk.push_opcode(OpCode::Jump, range.clone());
        chunk.push_jump(start_label);

        // Close the loop
        self.flow_points.close_loop(chunk)?;

        // Remove the looping values from the stack
        chunk.push_opcode(OpCode::Discard, range.clone());
        chunk.push_opcode(OpCode::Discard, range.clone());

        Ok(())
    }

    pub fn compile_break(&mut self, range: CodeRange, chunk: &mut Chunk) -> CompRes {
        // TODO: ERROR: Does not exit scopes, so does not de-allocate pointers
        chunk.push_opcode(OpCode::Jump, range);
        self.flow_points.push_break_exit(chunk.reserve_jump())
    }

    pub fn compile_continue(&mut self, range: CodeRange, chunk: &mut Chunk) -> CompRes {
        // TODO: ERROR: Does not exit scopes, so does not de-allocate pointers
        chunk.push_opcode(OpCode::Jump, range);
        let loop_entry = self.flow_points.get_loop_entry()?;
        chunk.push_jump(loop_entry);
        Ok(())
    }

    pub fn compile_return(
        &mut self,
        opt_expr: Option<&ExprNode>,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        if self.is_global() {
            return Err("Cannot return from top-level scope".to_string());
        }
        self.compile_opt_expression(opt_expr, chunk)?;

        // Drop all pointers before returning
        let pointer_offsets = self.locals.local_pointers();
        self.drop_pointers(&pointer_offsets, range.clone(), chunk);

        chunk.push_opcode(OpCode::Return, range);

        Ok(())
    }
}
