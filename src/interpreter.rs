use crate::{code_loc::CodeLoc, errors::ErrorReporter, parser::StmtNode};

mod environment;
mod expressions;
mod statements;

type RuntimeError = (CodeLoc, CodeLoc, String);

pub fn interpret(program: &Vec<StmtNode>, error_reporter: &mut ErrorReporter) {
    let mut env = environment::Environment::new();
    for stmt in program {
        match statements::eval(stmt, &mut env) {
            Ok(_) => continue,
            Err((start, end, reason)) => {
                return error_reporter.runtime_error(&start, &end, &reason)
            }
        }
    }
}
