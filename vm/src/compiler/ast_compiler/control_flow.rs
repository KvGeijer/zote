use parser::{CodeRange, ExprNode};

use crate::{
    compiler::{Chunk, Compiler, OpCode},
    value::Value,
};

use super::CompRes;

impl Compiler {
    pub fn compile_if(
        &mut self,
        pred: &ExprNode,
        then: &ExprNode,
        otherwise: Option<&ExprNode>,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes<()> {
        self.compile_expression(pred, chunk)?;
        chunk.push_opcode(OpCode::JumpIfFalse, range.clone());
        let reserved_else = chunk.reserve_jump();
        self.compile_expression(then, chunk)?;

        chunk.push_opcode(OpCode::Jump, range);
        let reserved_end = chunk.reserve_jump();

        chunk.set_reserved_jump(reserved_else, chunk.len()); // Jump to the beginning of else clause
        self.compile_opt_expression(otherwise, chunk)?;
        chunk.set_reserved_jump(reserved_end, chunk.len()); // Jump to end of if statement

        Ok(())
    }

    pub fn compile_and(
        &mut self,
        lhs: &ExprNode,
        rhs: &ExprNode,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes<()> {
        self.compile_expression(lhs, chunk)?;

        // If false, abort
        chunk.push_opcode(OpCode::JumpIfFalse, range.clone());
        let reserved_false = chunk.reserve_jump();

        self.compile_expression(rhs, chunk)?;

        // Use what is on the stack
        chunk.push_opcode(OpCode::Jump, range.clone());
        let reserved_keep = chunk.reserve_jump();

        // Aborted, use false
        chunk.set_reserved_jump(reserved_false, chunk.len()); // Push extra false
        chunk.push_constant_plus(Value::Bool(false), range);

        chunk.set_reserved_jump(reserved_keep, chunk.len()); // finish the and

        Ok(())
    }

    pub fn compile_or(
        &mut self,
        lhs: &ExprNode,
        rhs: &ExprNode,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes<()> {
        self.compile_expression(lhs, chunk)?;

        // If false, look at rhs as well
        chunk.push_opcode(OpCode::JumpIfFalse, range.clone());
        let reserved_false = chunk.reserve_jump();

        // If true, just jump to end, and push true
        chunk.push_opcode(OpCode::Jump, range.clone());
        let reserved_true = chunk.reserve_jump();

        // Look at rhs, then jump to the exit
        chunk.set_reserved_jump(reserved_false, chunk.len());
        self.compile_expression(rhs, chunk)?;
        chunk.push_opcode(OpCode::Jump, range.clone());
        let reserved_exit = chunk.reserve_jump();

        // Short-circuit, push true
        chunk.set_reserved_jump(reserved_true, chunk.len());
        chunk.push_constant_plus(Value::Bool(true), range);
        chunk.set_reserved_jump(reserved_exit, chunk.len()); // Finished

        Ok(())
    }

    pub fn compile_while(
        &mut self,
        pred: &ExprNode,
        body: &ExprNode,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes<()> {
        let start_label = chunk.len();
        self.compile_expression(pred, chunk)?;
        chunk.push_opcode(OpCode::JumpIfFalse, range.clone());
        let reserved_exit = chunk.reserve_jump();

        self.compile_expression(body, chunk)?;
        chunk.push_opcode(OpCode::Jump, range);
        chunk.push_jump_offset(start_label);
        chunk.set_reserved_jump(reserved_exit, chunk.len());

        Ok(())
    }
}
