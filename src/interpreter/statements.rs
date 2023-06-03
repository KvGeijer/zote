use std::rc::Rc;

use crate::{
    code_loc::CodeLoc,
    parser::{ExprNode, LValue, Stmt, StmtNode, Stmts},
};

use super::{environment::Environment, expressions, value::Value, RunRes, RuntimeError};

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
        Stmt::Decl(id, expr) => decl(id, expr, env, *start_loc, *end_loc).map(|_| None),
        Stmt::Expr(expr) => expressions::eval(expr, env).map(Some),
        Stmt::Invalid => panic!("Tried to interpret an invalid statement!"),
    }
}

fn decl(
    lvalue: &LValue,
    expr: &Option<ExprNode>,
    env: &Rc<Environment>,
    start: CodeLoc,
    end: CodeLoc,
) -> RunRes<()> {
    lvalue
        .declare(env)
        .map_err(|reason| RuntimeError::Error(start, end, reason))?;
    if let Some(expr) = expr {
        let rvalue = expressions::eval(expr, env)?;
        lvalue.assign(rvalue, env)?;
        // .map_err(|reason| RuntimeError::Error(start, end, reason))?;
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorReporter;
    use crate::interpreter::functions::define_builtins;
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

        assert_eq!(interpret_string(program).unwrap().unwrap(), 13.into());
    }

    #[test]
    fn implicit_nil_returns() {
        let program = "fn nil_ret() -> { return }; nil_ret()";
        assert!(matches!(
            interpret_string(program).unwrap().unwrap(),
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
            interpret_string(program).unwrap().unwrap(),
            Value::Nil
        ));
        println!("RATARTART");

        let program = concat!(
            "var maybe_nil_ret = x -> {\n",
            "    if x == Nil           \n",
            "        return            \n",
            "    else                  \n",
            "        return true       \n",
            "};                        \n",
            "                          \n",
            "maybe_nil_ret('Nil')      \n",
        );
        assert_eq!(interpret_string(program).unwrap().unwrap(), true.into());
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

        assert_eq!(interpret_string(program).unwrap().unwrap(), 11.into());

        let program = concat!(
            "fn replace_list(list, x) -> { ",
            "    var ret = pop(list);      ",
            "    push(x, list);            ",
            "    ret                       ",
            "};                            ",
            "                              ",
            "var list = [1,2,3,4];         ",
            "list[3] = 44;                 ",
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
        let _expected: Value = vec![
            44.into(),
            42.into(),
            42.into(),
            3.into(),
            2.into(),
            1.into(),
            Value::Nil,
            Value::Nil,
            Value::Nil,
        ]
        .into();

        assert!(matches!(
            interpret_string(program).unwrap().unwrap(),
            _expected
        ));
    }
}
