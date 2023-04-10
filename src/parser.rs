use crate::scanner::TokenInfo;
use crate::{code_loc::CodeLoc, errors::ErrorReporter};

mod expressions;
mod generics;
mod statements;

pub use expressions::{BinOper, BinOperNode, Expr, ExprNode, UnOper, UnOperNode};
pub use statements::{Stmt, StmtNode};

// Each node in the AST is some branch/leaf wrapped in this extra info
#[derive(Debug)]
pub struct AstNode<T> {
    pub node: T,
    pub start_loc: CodeLoc,
    pub end_loc: CodeLoc, // Not including last char. Should we change?
}

pub fn parse(tokens: &[TokenInfo], error_reporter: &mut ErrorReporter) -> Option<Vec<StmtNode>> {
    let mut parser = Parser::new(tokens, error_reporter);
    parser.statements().ok()
}

// All submodules will add some functionality to this, like parsing expressions
struct Parser<'a> {
    tokens: Vec<&'a TokenInfo>,
    current: usize,
    error_reporter: &'a mut ErrorReporter,
}
