use std::rc::Rc;

use crate::parser::{ExprNode, Stmt, StmtNode, Stmts};

use super::{
    environment::Environment,
    expressions::{self, Value},
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
    match stmt.node.as_ref() {
        Stmt::Decl(id, expr) => decl(id, expr, env).map(|_| None),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorReporter;
    use crate::interpreter::functions::define_builtins;
    use crate::interpreter::list::List;
    use crate::parser::parse;
    use crate::scanner::tokenize;

    /// Helper to interpret an expression from a string
    fn interpret_string(program: &str) -> RunRes<Option<Value>> {
        let mut error_reporter = ErrorReporter::new();
        let tokens = tokenize(program, &mut error_reporter);
        let ast = parse(&tokens, &mut error_reporter).unwrap();
        let env = Environment::new();
        define_builtins(&env);
        eval_statements(&ast, &env)
    }

    #[test]
    fn fibonachi() {
        let program = "                     \
            fn fib(n) -> {                  \
                if n < 0 return 0;          \
                                            \
                if n <= 1                   \
                    1                       \
                else {                      \
                    fib(n-1) + fib(n-2)     \
                }                           \
            };                              \
                                            \
            6 >> fib >>: result;            \
            result                          \
            ";

        assert!(matches!(
            interpret_string(program).unwrap().unwrap(),
            Value::Int(13)
        ));
    }

    #[test]
    fn implicit_nil_returns() {
        let program = "fn nil_ret() -> { return }; nil_ret()";
        assert!(matches!(
            interpret_string(program).unwrap().unwrap(),
            Value::Nil
        ));

        let program = concat!(
            "fn maybe_nil_ret(x) -> {",
            "    if x == Nil         ",
            "        return          ",
            "    else                ",
            "        return true     ",
            "};                      ",
            "                        ",
            "maybe_nil_ret(Nil)      ",
        );
        assert!(matches!(
            interpret_string(program).unwrap().unwrap(),
            Value::Nil
        ));

        let program = concat!(
            "var maybe_nil_ret = x -> {",
            "    if x == Nil           ",
            "        return            ",
            "    else                  ",
            "        return true       ",
            "};                        ",
            "                          ",
            "maybe_nil_ret('Nil')      ",
        );
        assert!(matches!(
            interpret_string(program).unwrap().unwrap(),
            Value::Bool(true)
        ));
    }

    // TODO Move tests into its own file under interpreter...
    #[test]
    fn list_operations() {
        let program = "pop([])";
        assert!(matches!(
            interpret_string(program).unwrap().unwrap(),
            Value::Nil
        ));

        let program = concat!(
            "fn pop_twice(list) -> { ",
            "    if list             ",
            "        pop(list);      ",
            "    if list             ",
            "        pop(list)       ",
            "    else                ",
            "        Nil             ",
            "};                      ",
            "                        ",
            "pop_twice([1,121/11, 3])",
        );

        assert!(matches!(
            interpret_string(program).unwrap().unwrap(),
            Value::Int(11)
        ));

        let program = concat!(
            "fn replace_list(list, x) -> { ",
            "    var ret = pop(list);      ",
            "    push(x, list);            ",
            "    ret                       ",
            "};                            ",
            "                              ",
            "var list = [1,2,3,4];         ",
            "[                             ",
            "    replace_list(list, 42),   ",
            "    replace_list(list, 42),   ",
            "    pop(list),                ",
            "    pop(list),                ",
            "    pop(list),                ",
            "    pop(list),                ",
            "    pop(list),                ",
            "    pop(list),                ",
            "    pop(list)                 ",
            "]                             ",
        );

        // Why does it comlain about unused?
        let _expected = Value::List(List::new(
            vec![
                Value::Int(4),
                Value::Int(42),
                Value::Int(42),
                Value::Int(3),
                Value::Int(2),
                Value::Int(1),
                Value::Nil,
                Value::Nil,
                Value::Nil,
            ]
            .into_iter(),
        ));

        assert!(matches!(
            interpret_string(program).unwrap().unwrap(),
            _expected
        ));
    }
}
