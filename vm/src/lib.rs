#![feature(let_chains)]

// TODO: Fix what should be public and what not soon
pub mod compiler;
pub mod disassembler;
mod error;
pub mod interpreter;
pub mod value;

use compiler::compile;
pub use error::Trace;
use interpreter::interpret;
use parser::Stmts;

#[derive(Debug)]
pub enum VMError {
    CompileError,
    RuntimeError(Trace),
}

pub fn interpret_once(ast: &Stmts) -> i32 {
    let Some(chunk) = compile(ast) else {
        return 65; // ? Which error to use? Should we send back trace?
    };

    disassembler::disassemble_chunk(&chunk, "main", &mut std::io::stdout()).unwrap();
    match interpret(&chunk, false) {
        Ok(_) => 0,
        Err(_) => 70,
    }
}
