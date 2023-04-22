use std::rc::Rc;

use crate::parser::{ExprNode, Stmt, StmtNode, Stmts};

use super::{
    environment::Environment,
    expressions::{self, Value},
    functions::{Closure, Function},
    RunRes,
};

pub(super) fn eval_statements(statements: &Stmts, env: &Rc<Environment>) -> RunRes<Option<Value>> {
    let mut output = None;
    for stmt in statements.stmts.iter() {
        match eval(stmt, env)? {
            None => continue,
            val => output = val,
        }
    }

    if statements.output {
        Ok(Some(
            output.expect("Internal error: Expexted value from statements"),
        ))
    } else {
        Ok(None)
    }
}

fn eval(stmt: &StmtNode, env: &Rc<Environment>) -> RunRes<Option<Value>> {
    match &stmt.node {
        Stmt::Decl(id, expr) => decl(id, expr, env).map(|_| None),
        Stmt::FuncDecl(name, param, body) => func_decl(name, param, body, env).map(|_| None),
        Stmt::Expr(expr) => expressions::eval(expr, env).map(Some),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorReporter;
    use crate::parser::parse;
    use crate::scanner::tokenize;

    /// Helper to interpret an expression from a string
    fn interpret_string(program: &str) -> RunRes<Option<Value>> {
        let mut error_reporter = ErrorReporter::new();
        let tokens = tokenize(program, &mut error_reporter);
        let ast = parse(&tokens, &mut error_reporter).unwrap();
        eval_statements(&ast, &Environment::new())
    }

    #[test]
    fn fibonachi() {
        let program = "                     \
            fn fib(n) {                     \
                if n < 0 return 0;          \
                                            \
                if n <= 1                   \
                    1                       \
                else {                      \
                    fib(n-1) + fib(n-2)     \
                }                           \
            };                              \
                                            \
            fib(6)                          \
            ";

        assert!(matches!(
            interpret_string(program).unwrap().unwrap(),
            Value::Int(13)
        ));
    }
}
