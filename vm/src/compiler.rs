mod bytecode;
mod chunk;
mod code_gen;
mod control_flow;
mod locals;

use std::collections::HashMap;

pub use bytecode::OpCode;
pub use chunk::Chunk;
use parser::{CodeRange, Stmts};
use semantic_analyzer::AttributedAst;

use crate::value::get_natives;

use self::{control_flow::FlowPoints, locals::LocalState};

/// Error type when compiling
type CompRetRes<T> = Result<T, String>;
type CompRes = CompRetRes<()>;

/// Struct to store metadata during and between compilations
pub struct Compiler<'a> {
    attributes: &'a AttributedAst<'a>,
    globals: HashMap<String, usize>,
    locals: LocalState,
    flow_points: FlowPoints,
    had_error: bool,
}

/// Compile AST to bytecode (top-level)
pub fn compile<'a>(ast: &'a AttributedAst<'a>) -> Option<Chunk> {
    let mut compiler = Compiler::new(ast);
    compiler.compile()
}

impl<'a> Compiler<'a> {
    pub fn new(attributes: &'a AttributedAst) -> Self {
        Self {
            attributes,
            globals: HashMap::with_capacity(32),
            locals: LocalState::new(),
            flow_points: FlowPoints::new(),
            had_error: false,
        }
    }

    // TODO: How do we do things in part when using REPL?
    /// Compile AST to bytecode (top-level)
    fn compile(&mut self) -> Option<Chunk> {
        let mut chunk = Chunk::new(); // TODO: Take as arg instead? How do we then handle errors?

        self.declare_natives(&mut chunk);
        self.declare_globals(self.attributes.stmts());

        self.compile_stmts(self.attributes.stmts(), &mut chunk);

        match self.had_error {
            false => Some(chunk),
            true => None,
        }
    }

    fn compile_stmts(&mut self, stmts: &Stmts, chunk: &mut Chunk) {
        for (ind, stmt) in stmts.stmts.iter().enumerate() {
            self.compile_statement(
                stmt,
                chunk,
                stmts.output && (ind == (stmts.stmts.len() - 1)),
            );
        }
    }

    fn is_global(&self) -> bool {
        self.locals.is_global()
    }

    pub fn declare_natives(&mut self, chunk: &mut Chunk) {
        let range = CodeRange::from_ints(0, 0, 0, 0, 0, 0);
        for native in get_natives() {
            // Declare it as a global
            let offset = self.declare_global(native.name());

            // Push the native as a constant
            chunk.push_constant_plus(native.into(), range.clone());

            // Assign it to a global
            chunk.push_opcode(OpCode::AssignGlobal, range.clone());
            chunk.push_u8_offset(offset as u8);
            chunk.push_opcode(OpCode::Discard, range.clone());
        }
    }
}
