mod ast_compiler;
mod bytecode;
mod chunk;

pub use bytecode::OpCode;
pub use chunk::Chunk;
use parser::Stmts;

use ast_compiler::compile_statement;

// TODO: How do we do things in part when using REPL?
/// Compile AST to bytecode
pub fn compile(stmts: &Stmts) -> Option<Chunk> {
    let mut chunk = Chunk::new(); // TODO: Take as arg instead?

    // TODO: Implement output field (Just push Nil if no output?)
    for stmt in stmts.stmts.iter() {
        compile_statement(stmt, &mut chunk)?;
    }

    Some(chunk)
}
