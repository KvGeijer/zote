use crate::parser::{ExprNode, Stmt, StmtNode};

use super::{
    environment::Environment,
    expressions::{self, Value},
    RuntimeError,
};

pub(super) fn eval(stmt: &StmtNode, env: &Environment) -> Result<(), RuntimeError> {
    match &stmt.node {
        Stmt::Decl(id, expr) => decl(id, expr, env),
        Stmt::Expr(expr) => expressions::eval(expr, env).map(|_| ()),
        Stmt::Print(expr) => {
            println!("{}", expressions::eval(expr, env)?.stringify());
            Ok(())
        }
        Stmt::Invalid => panic!("Tried to interpret an invalid statement!"),
    }
}

fn decl(id: &str, expr: &Option<ExprNode>, env: &Environment) -> Result<(), RuntimeError> {
    let value = if let Some(expr) = expr {
        expressions::eval(expr, env)?
    } else {
        Value::Uninitialized
    };
    env.define(id.to_owned(), value);
    Ok(())
}
