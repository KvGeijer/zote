use std::rc::Rc;

use crate::parser::{ExprNode, Stmt, StmtNode};

use super::{
    environment::Environment,
    expressions::{self, Value},
    RunRes,
};

pub(super) fn eval(stmt: &StmtNode, env: &Rc<Environment>) -> RunRes<()> {
    match &stmt.node {
        Stmt::Decl(id, expr) => decl(id, expr, env),
        Stmt::Expr(expr) => expressions::eval(expr, env).map(|_| ()),
        Stmt::Invalid => panic!("Tried to interpret an invalid statement!"),
    }
}

fn decl(id: &str, expr: &Option<ExprNode>, env: &Rc<Environment>) -> RunRes<()> {
    let value = if let Some(expr) = expr {
        expressions::eval(expr, env)?
    } else {
        Value::Uninitialized
    };
    env.define(id.to_owned(), value);
    Ok(())
}
