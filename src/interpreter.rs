use crate::{errors::ErrorReporter, parser::ExprNode};

mod expressions;

pub fn interpret(program: &ExprNode, error_reporter: &mut ErrorReporter) {
    match expressions::eval(program) {
        Ok(value) => println!("{}", value.stringify()),
        Err((loc, reason)) => error_reporter.runtime_error(&loc, &reason),
    }
}
