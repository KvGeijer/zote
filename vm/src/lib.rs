#![feature(let_chains)]

// TODO: Fix what should be public and what not soon
pub mod compiler;
pub mod disassembler;
mod error;
pub mod interpreter;
pub mod value;

pub use error::Trace;

#[derive(Debug)]
pub enum VMError {
    CompileError,
    RuntimeError(Trace),
}
