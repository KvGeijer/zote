#![feature(let_chains)]

// TODO: Fix what should be public and what not soon
pub mod compiler;
pub mod disassembler;
mod error;
pub mod interpreter;
pub mod value;

use std::rc::Rc;

use compiler::compile;
use interpreter::interpret;
use semantic_analyzer::AttributedAst;

const DEBUG: bool = false;

pub fn interpret_once(ast: &AttributedAst) -> i32 {
    let Some(chunk) = compile(ast) else {
        return 65; // ? Which error to use? Should we send back trace?
    };

    if DEBUG {
        disassembler::disassemble_chunk(&chunk, "main", &mut std::io::stdout()).unwrap();
    }
    match interpret(Rc::new(chunk), DEBUG) {
        Ok(_) => 0,
        Err(trace) => {
            eprintln!("{}", trace);
            70
        }
    }
}
