use crate::{
    errors::ErrorReporter,
    parser::{AstLoc, StmtNode},
};

mod environment;
mod expressions;
mod statements;

type RuntimeError = (AstLoc, String);

pub fn interpret(program: &Vec<StmtNode>, error_reporter: &mut ErrorReporter) {
    let mut env = environment::Environment::new();
    for stmt in program {
        match statements::eval(stmt, &mut env) {
            Ok(_) => continue,
            Err((loc, reason)) => return error_reporter.runtime_error(&loc, &reason),
        }
    }
}
