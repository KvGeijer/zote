use crate::errors::ErrorReporter;
use crate::scanner::TokenInfo;

mod ast_loc;
mod expressions;
mod generics;

pub use ast_loc::AstLoc;
pub use expressions::{BinOper, BinOperNode, Expr, ExprNode, UnOper, UnOperNode};

pub fn parse(tokens: &[TokenInfo], error_reporter: &mut ErrorReporter) -> Option<AstNode<Expr>> {
    let mut parser = Parser::new(tokens, error_reporter);
    parser.expression()
}

// All submodules will add some functionality to this, like parsing expressions
struct Parser<'a> {
    tokens: Vec<&'a TokenInfo>,
    current: usize,
    error_reporter: &'a mut ErrorReporter,
}

// All nodes on the AST should also have some extra info, like the location in code
#[derive(Debug)]
pub struct AstNode<T> {
    pub node: T,
    pub loc: AstLoc,
}
