use std::rc::Rc;

use parser::{ExprNode, LValue, Stmt, StmtNode, Stmts};

use super::{
    environment::Environment,
    expressions::{self, assign, declare},
    runtime_error::RunResTrait,
    value::Value,
    RunRes,
};

pub fn eval_statements(statements: &Stmts, env: &Rc<Environment>) -> RunRes<Option<Value>> {
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
    let StmtNode {
        start_loc,
        end_loc,
        box node,
    } = stmt;
    match node {
        Stmt::Decl(id, expr) => decl(id, expr, env).map(|_| None),
        Stmt::Expr(expr) => expressions::eval(expr, env).map(Some),
        Stmt::Invalid => panic!("Tried to interpret an invalid statement!"),
    }
    .add_loc(*start_loc, *end_loc) // OPT: How slow are these polymorphic wrappers?
}

fn decl(lvalue: &LValue, expr: &Option<ExprNode>, env: &Rc<Environment>) -> RunRes<()> {
    if let Some(expr) = expr {
        let rvalue = expressions::eval(expr, env)?;
        declare(lvalue, env)?;
        assign(lvalue, rvalue, env)?;
    } else {
        // TODO: This should not be possible anymore with the new declarations
        declare(lvalue, env)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::define_builtins;
    use crate::runtime_error::RunError;
    use parser::parse;

    /// Helper to interpret an expression from a string
    fn interpret_string(program: &str) -> Option<RunRes<Option<Value>>> {
        let ast = parse("unit_test", program)?;
        let env = Environment::new();
        define_builtins(&env);
        Some(eval_statements(&ast, &env))
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
            6 >> fib  :>> result;            \
            result                          \
            ";

        assert_eq!(
            interpret_string(program).unwrap().unwrap().unwrap(),
            13.into()
        );
    }

    #[test]
    fn implicit_nil_returns() {
        let program = "fn nil_ret() -> { return }; nil_ret()";
        assert!(matches!(
            interpret_string(program).unwrap().unwrap().unwrap(),
            Value::Nil
        ));

        let program = concat!(
            "fn maybe_nil_ret(x) -> {\n",
            "    if x == Nil         \n",
            "        return          \n",
            "    else                \n",
            "        return true     \n",
            "};                      \n",
            "                        \n",
            "maybe_nil_ret(Nil)      \n",
        );
        assert!(matches!(
            interpret_string(program).unwrap().unwrap().unwrap(),
            Value::Nil
        ));

        let program = concat!(
            "maybe_nil_ret := \\x -> { \n",
            "    if x == Nil           \n",
            "        return            \n",
            "    else                  \n",
            "        return true       \n",
            "};                        \n",
            "                          \n",
            "maybe_nil_ret('Nil')      \n",
        );
        assert_eq!(
            interpret_string(program).unwrap().unwrap().unwrap(),
            true.into()
        );
    }

    // TODO Move tests into its own file under interpreter...
    #[test]
    fn list_operations() {
        let program = "pop([])";
        let result = interpret_string(program).unwrap();
        assert!(matches!(result, Err(RunError::Error(_))));

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

        assert_eq!(
            interpret_string(program).unwrap().unwrap().unwrap(),
            11.into()
        );

        let program = concat!(
            "fn replace_list(list, x) -> { ",
            "    ret := pop(list);         ",
            "    push(x, list);            ",
            "    ret                       ",
            "};                            ",
            "                              ",
            "list := [1,2,3,4];            ",
            "list[3] = 44;                 ",
            "[                             ",
            "    replace_list(list, 42),   ",
            "    replace_list(list, 42),   ",
            "    pop(list),                ",
            "    pop(list),                ",
            "    pop(list),                ",
            "    pop(list)                 ",
            "]                             ",
        );

        // Why does it comlain about unused?
        let _expected: Value = vec![
            44.into(),
            42.into(),
            42.into(),
            3.into(),
            2.into(),
            1.into(),
        ]
        .into();

        assert!(matches!(
            interpret_string(program).unwrap().unwrap().unwrap(),
            _expected
        ));
    }
}
