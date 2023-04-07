use crate::errors::ErrorReporter;
use crate::scanner::TokenInfo;

mod ast_loc;
mod expressions;
mod generics;
mod statements;

pub use ast_loc::AstLoc;
pub use expressions::{BinOper, BinOperNode, Expr, ExprNode, UnOper, UnOperNode};
pub use statements::{Stmt, StmtNode};

// Each node in the AST is some branch/leaf wrapped in this extra info
#[derive(Debug)]
pub struct AstNode<T> {
    pub node: T,
    pub loc: AstLoc,
}

pub fn parse(tokens: &[TokenInfo], error_reporter: &mut ErrorReporter) -> Option<Vec<StmtNode>> {
    let mut parser = Parser::new(tokens, error_reporter);
    let mut stmts = Vec::new();
    while !parser.at_end() {
        stmts.push(parser.statement());
    }

    if stmts.iter().any(|stmt| stmt.node == Stmt::Invalid) {
        None
    } else {
        Some(stmts)
    }
}

// All submodules will add some functionality to this, like parsing expressions
struct Parser<'a> {
    tokens: Vec<&'a TokenInfo>,
    current: usize,
    error_reporter: &'a mut ErrorReporter,
}
