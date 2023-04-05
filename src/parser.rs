use crate::errors::ErrorReporter;
use crate::scanner::TokenInfo;

mod expressions;
mod generics;

pub use expressions::{BinOper, Expr, UnOper};

pub fn parse(tokens: &[TokenInfo], error_reporter: &mut ErrorReporter) -> Option<Expr> {
    let mut parser = Parser::new(tokens, error_reporter);
    parser.expression()
}

struct Parser<'a> {
    tokens: Vec<&'a TokenInfo>,
    current: usize,
    error_reporter: &'a mut ErrorReporter,
}
