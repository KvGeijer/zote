#![feature(box_patterns, iterator_try_reduce, let_chains)]

pub use code_loc::CodeLoc;
use errors::ErrorReporter;
use scanner::TokenInfo;

mod code_loc;
mod errors;
mod expressions;
mod generics;
mod scanner;
mod statements;

pub use expressions::{
    BinOper, Expr, ExprNode, Index, LValue, ListContent, LogicalOper, Slice, UnOper,
};
pub use statements::{Stmt, StmtNode, Stmts};

// Each node in the AST is some branch/leaf wrapped in this extra info
#[derive(Debug, Clone)]
pub struct AstNode<T> {
    pub node: Box<T>,
    pub start_loc: CodeLoc,
    pub end_loc: CodeLoc, // Not including last char. Should we change?
}

pub fn parse(code: &str) -> Option<Stmts> {
    let mut error_reporter = errors::ErrorReporter::new();
    let tokens = scanner::tokenize(code, &mut error_reporter);
    if error_reporter.had_error {
        return None;
    }

    let mut parser = Parser::new(&tokens, &mut error_reporter);
    match parser.statements(crate::scanner::Token::Eof).ok() {
        Some(ast) if !error_reporter.had_error => Some(ast),
        _otherwise => None,
    }
}

// All submodules will add some functionality to this, like parsing expressions
struct Parser<'a> {
    tokens: Vec<&'a TokenInfo>,
    current: usize,
    error_reporter: &'a mut ErrorReporter,
}
