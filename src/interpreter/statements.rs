use crate::parser::{AstLoc, Stmt, StmtNode};

use super::expressions;

pub fn eval(stmt: &StmtNode) -> Result<(), (AstLoc, String)> {
    match &stmt.node {
        Stmt::Expr(expr) => expressions::eval(expr).map(|_| ()),
        Stmt::Print(expr) => {
            println!("{}", expressions::eval(expr)?.stringify());
            Ok(())
        }
        Stmt::Invalid => panic!("Tried to interpret an invalid statement!"),
    }
}
