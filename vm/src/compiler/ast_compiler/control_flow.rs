use parser::{CodeRange, ExprNode};

use crate::compiler::{Chunk, Compiler, OpCode};

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
}
