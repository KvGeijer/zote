use std::rc::Rc;

use crate::parser::{ExprNode, Stmt, StmtNode};

use super::{
    environment::Environment,
    expressions::{self, Value},
    functions::{Closure, Function},
    RunRes,
};

pub(super) fn eval(stmt: &StmtNode, env: &Rc<Environment>) -> RunRes<()> {
    match &stmt.node {
        Stmt::Decl(id, expr) => decl(id, expr, env),
        Stmt::FuncDecl(name, param, body) => func_decl(name, param, body, env),
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

fn func_decl(id: &str, param: &[String], body: &ExprNode, env: &Rc<Environment>) -> RunRes<()> {
    let closure = Closure::new(id.to_string(), param.to_vec(), body.clone(), env);
    env.define(id.to_string(), Value::Callable(Function::Closure(closure)));
    Ok(())
}
