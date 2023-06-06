use crate::scanner::TokenInfo;
use crate::{code_loc::CodeLoc, errors::ErrorReporter};

mod expressions;
mod generics;
mod statements;

pub use expressions::{BinOper, Expr, ExprNode, Index, LValue, LogicalOper, UnOper};
pub use statements::{Stmt, StmtNode, Stmts};

// Each node in the AST is some branch/leaf wrapped in this extra info
#[derive(Debug, Clone)]
pub struct AstNode<T> {
    pub node: Box<T>,
    pub start_loc: CodeLoc,
    pub end_loc: CodeLoc, // Not including last char. Should we change?
}

pub fn parse(tokens: &[TokenInfo], error_reporter: &mut ErrorReporter) -> Option<Stmts> {
    let mut parser = Parser::new(tokens, error_reporter);
    parser.statements(crate::scanner::Token::Eof).ok()
}

// All submodules will add some functionality to this, like parsing expressions
struct Parser<'a> {
    tokens: Vec<&'a TokenInfo>,
    current: usize,
    error_reporter: &'a mut ErrorReporter,
}
