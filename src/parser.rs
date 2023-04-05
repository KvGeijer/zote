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

// Could not for the life of me get this trait bound to work for references...
impl<T> AstNode<T> {
    fn new<F>(node: T, start: F, end: F) -> AstNode<T>
    where
        F: Into<AstLoc>,
    {
        let start_astloc: AstLoc = start.into();
        let end_astloc: AstLoc = end.into();
        let loc = AstLoc::new(
            start_astloc.row_start(),
            end_astloc.row_end(),
            start_astloc.col_start(),
            end_astloc.col_end(),
        );
        AstNode { node, loc }
    }
}

// This is really nice, although it should move to another file now :D
impl<T: PartialEq> PartialEq for AstNode<T> {
    fn eq(&self, other: &Self) -> bool {
        &self.node == &other.node
    }
}
