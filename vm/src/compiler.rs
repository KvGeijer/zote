mod ast_compiler;
mod bytecode;
mod chunk;
mod locals;

use std::collections::HashMap;

pub use bytecode::OpCode;
pub use chunk::Chunk;
use parser::Stmts;

use self::locals::LocalState;

/// Struct to store metadata during and between compilations
pub struct Compiler {
    globals: HashMap<String, usize>,
    locals: LocalState,
    had_error: bool,
}

pub fn compile(stmts: &Stmts) -> Option<Chunk> {
    let mut compiler = Compiler::new();
    compiler.compile(stmts)
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            globals: HashMap::with_capacity(32),
            locals: LocalState::new(),
            had_error: false,
        }
    }

    // TODO: How do we do things in part when using REPL?
    /// Compile AST to bytecode (top-level)
    pub fn compile(&mut self, stmts: &Stmts) -> Option<Chunk> {
        let mut chunk = Chunk::new(); // TODO: Take as arg instead? How do we then handle errors?

        self.declare_globals(stmts);
        self.compile_stmts(stmts, &mut chunk);

        match self.had_error {
            false => Some(chunk),
            true => None,
        }
    }

    pub fn compile_stmts(&mut self, stmts: &Stmts, chunk: &mut Chunk) {
        for (ind, stmt) in stmts.stmts.iter().enumerate() {
            self.compile_statement(
                stmt,
                chunk,
                stmts.output && (ind == (stmts.stmts.len() - 1)),
            );
        }
    }
}
