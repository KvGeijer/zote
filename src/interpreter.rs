use crate::{errors::ErrorReporter, parser::StmtNode};

mod expressions;
mod statements;

pub fn interpret(program: &Vec<StmtNode>, error_reporter: &mut ErrorReporter) {
    for stmt in program {
        match statements::eval(stmt) {
            Ok(_) => continue,
            Err((loc, reason)) => return error_reporter.runtime_error(&loc, &reason),
        }
    }
}
