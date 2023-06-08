use std::{cmp::Ordering, rc::Rc};

use crate::{
    code_loc::CodeLoc,
    parser::{BinOper, Expr, ExprNode, Index, LValue, ListContent, LogicalOper, Stmts, UnOper},
};

use super::{
    collections::{eval_index, eval_slice, SliceValue},
    environment::Environment,
    functions::{Closure, Function},
    numerical::Numerical,
    runtime_error::{RunError, RunRes, RunResTrait},
    statements,
    value::Value,
};

pub fn eval(expr: &ExprNode, env: &Rc<Environment>) -> RunRes<Value> {
    match expr.node.as_ref() {
        Expr::Binary(left, op, right) => eval_binary(eval(left, env)?, op, eval(right, env)?),
        Expr::Logical(left, op, right) => eval_logical(eval(left, env)?, op, right, env),
        Expr::Unary(op, right) => eval_unary(op, eval(right, env)?),
        Expr::Assign(lvalue, expr) => eval_assign(lvalue, eval(expr, env)?, env),
        Expr::Var(id) => env.get(id),
        Expr::Int(int) => Ok(Value::Numerical(Numerical::Int(*int))),
        Expr::Float(float) => Ok(Value::Numerical(Numerical::Float(*float))),
        Expr::Bool(bool) => Ok(Value::Numerical(Numerical::Bool(*bool))),
        Expr::String(string) => Ok(string.clone().into()),
        Expr::Block(stmts) => eval_block(stmts, env),
        Expr::If(cond, then, other) => eval_if(eval(cond, env)?, then, other.as_ref(), env),
        Expr::While(cond, repeat) => eval_while(cond, repeat, env),
        Expr::For(lvalue, iterable, body) => eval_for(lvalue, eval(iterable, env)?, body, env),
        Expr::Break => Err(RunError::Break),
        Expr::Call(callee, args) => eval_call(
            eval(callee, env)?,
            args.iter()
                .map(|arg| eval(arg, env))
                .collect::<Result<Vec<_>, _>>()?,
            expr.start_loc,
            expr.end_loc,
        ),
        Expr::Return(Some(expr)) => Err(RunError::Return(eval(expr, env)?)),
        Expr::Return(None) => Err(RunError::Return(Value::Nil)),
        Expr::Nil => Ok(Value::Nil),
        Expr::List(content) => eval_list(content, env),
        Expr::Tuple(_exprs) => {
            RunError::error("Tuples are not part of the language (yet)".to_string())
        }
        Expr::FunctionDefinition(name, param, body) => eval_func_definition(name, param, body, env),
        Expr::IndexInto(base, index) => eval_index_expr(base, index, env),
    }
    .add_loc(expr.start_loc, expr.end_loc)
}

fn eval_index_expr(base: &ExprNode, index_expr: &Index, env: &Rc<Environment>) -> RunRes<Value> {
    let index = eval_index(index_expr, env)?;
    let into_value = eval(base, env)?;
    match into_value {
        Value::Collection(collection) => collection.get(index),
        other => RunError::error(format!("Cannot index into a {}", other.type_of())),
    }
}

fn eval_func_definition(
    id: &str,
    param: &[LValue],
    body: &ExprNode,
    env: &Rc<Environment>,
) -> RunRes<Value> {
    let closure = Closure::new(id.to_string(), param.to_vec(), body.clone(), env);
    Ok(Value::Callable(Function::Closure(closure)))
}

fn eval_list(content: &ListContent, env: &Rc<Environment>) -> RunRes<Value> {
    match content {
        ListContent::Exprs(exprs) => Ok(exprs
            .iter()
            .map(|expr| eval(expr, env))
            .collect::<Result<Vec<_>, _>>()?
            .into()),
        ListContent::Range(slice) => {
            if let SliceValue {
                start: Some(start),
                stop: Some(stop),
                step,
            } = eval_slice(slice, env)?
            {
                let step = step.map(|num| num.to_rint()).unwrap_or(1);
                Ok((start.to_rint()..stop.to_rint())
                    .step_by(step as usize)
                    .map(|int| int.into())
                    .collect::<Vec<Value>>()
                    .into())
            } else {
                error(
                    "Building an array from a slice requires populated start and stops".to_string(),
                )
            }
        }
    }
}

fn eval_call(callee: Value, args: Vec<Value>, start: CodeLoc, end: CodeLoc) -> RunRes<Value> {
    if let Value::Callable(callable) = callee {
        if args.len() == callable.arity() {
            match callable.call(args) {
                Err(RunError::Break) => error("Break encountered outside loop".to_string()),
                Err(RunError::Return(value)) => Ok(value),
                otherwise => otherwise,
            }
            .add_trace(callable.name().to_string(), start, end)
        } else {
            error(format!(
                "Expected {} arguments but got {}.",
                callable.arity(),
                args.len()
            ))
        }
    } else {
        error(format!(
            "Can only call functions, but got {}",
            callee.type_of()
        ))
    }
}

fn error<T>(message: String) -> RunRes<T> {
    RunError::error(message)
}

fn def_block_return() -> Value {
    Value::Nil
}

fn eval_while(cond: &ExprNode, repeat: &ExprNode, env: &Rc<Environment>) -> RunRes<Value> {
    while eval(cond, env)?.truthy() {
        match eval(repeat, env) {
            Err(RunError::Break) => break,
            otherwise => otherwise?,
        };
    }

    Ok(def_block_return())
}

fn eval_for(
    lvalue: &LValue,
    iter: Value,
    body: &ExprNode,
    outer_env: &Rc<Environment>,
) -> RunRes<Value> {
    let env = Environment::nest(outer_env);
    for value in iter.to_iter()? {
        lvalue.declare(&env)?;
        lvalue.assign(value, &env)?;
        match eval(body, &env) {
            Err(RunError::Break) => break,
            other => other,
        }?;
    }

    Ok(def_block_return())
}

fn eval_if(
    cond: Value,
    then: &ExprNode,
    otherwise: Option<&ExprNode>,
    env: &Rc<Environment>,
) -> RunRes<Value> {
    if cond.truthy() {
        eval(then, env)
    } else if let Some(expr) = otherwise {
        eval(expr, env)
    } else {
        Ok(def_block_return())
    }
}

fn eval_block(stmts: &Stmts, env: &Rc<Environment>) -> RunRes<Value> {
    let nested_env = Environment::nest(env);
    match statements::eval_statements(stmts, &nested_env)? {
        Some(val) => Ok(val),
        None => Ok(def_block_return()),
    }
}

fn eval_assign(lvalue: &LValue, rvalue: Value, env: &Rc<Environment>) -> RunRes<Value> {
    lvalue.assign(rvalue, env)
}

fn eval_binary(left: Value, op: &BinOper, right: Value) -> RunRes<Value> {
    match op {
        BinOper::Append => bin_append(left, right),
        BinOper::Add => bin_add(left, right),
        BinOper::Sub => bin_sub(left, right),
        BinOper::Mult => bin_mult(left, right),
        BinOper::Div => bin_div(left, right),
        BinOper::Mod => bin_mod(left, right),
        BinOper::Pow => bin_pow(left, right),
        BinOper::Eq => bin_eq(left, right),
        BinOper::Neq => bin_neq(left, right),
        BinOper::Lt => bin_lt(left, right),
        BinOper::Leq => bin_leq(left, right),
        BinOper::Gt => bin_gt(left, right),
        BinOper::Geq => bin_geq(left, right),
    }
}
fn bin_append(left: Value, right: Value) -> RunRes<Value> {
    match left {
        Value::Collection(x) => x.concat(right),
        left => error(format!(
            "Cannot append {} to {}",
            right.type_of(),
            left.type_of()
        )),
    }
}

fn bin_add(left: Value, right: Value) -> RunRes<Value> {
    match (left, right) {
        (Value::Numerical(x), Value::Numerical(y)) => Ok(Value::Numerical(x.add(y))),
        (left, right) => error(format!(
            "Cannot add {} and {}",
            left.type_of(),
            right.type_of()
        )),
    }
}

fn bin_sub(left: Value, right: Value) -> RunRes<Value> {
    match (left, right) {
        (Value::Numerical(x), Value::Numerical(y)) => Ok(Value::Numerical(x.sub(y))),
        (left, right) => error(format!(
            "Cannot subtract {} from {}",
            right.type_of(),
            left.type_of()
        )),
    }
}

fn bin_mult(left: Value, right: Value) -> RunRes<Value> {
    match (left, right) {
        (Value::Numerical(x), Value::Numerical(y)) => Ok(Value::Numerical(x.mult(y))),
        (left, right) => error(format!(
            "Cannot multiply {} and {}",
            left.type_of(),
            right.type_of()
        )),
    }
}

fn bin_div(left: Value, right: Value) -> RunRes<Value> {
    match (left, right) {
        (Value::Numerical(x), Value::Numerical(y)) => Ok(x.div(y)?.into()),
        (left, right) => error(format!(
            "Cannot divide {} by {}",
            left.type_of(),
            right.type_of()
        )),
    }
}

fn bin_mod(left: Value, right: Value) -> RunRes<Value> {
    match (left, right) {
        (Value::Numerical(x), Value::Numerical(y)) => Ok(Value::Numerical(x.modulo(y))),
        _other => error("Modulo only works for numbers".to_string()),
    }
}

fn bin_pow(left: Value, right: Value) -> RunRes<Value> {
    match (left, right) {
        (Value::Numerical(x), Value::Numerical(y)) => Ok(Value::Numerical(x.pow(y))),
        _other => error("Can only take powers of numbers".to_string()),
    }
}

fn bin_eq(left: Value, right: Value) -> RunRes<Value> {
    Ok(Value::Numerical(Numerical::Bool(left == right)))
}

fn bin_neq(left: Value, right: Value) -> RunRes<Value> {
    Ok(Value::Numerical(Numerical::Bool(left != right)))
}

fn bin_lt(left: Value, right: Value) -> RunRes<Value> {
    match left.partial_cmp(&right) {
        Some(order) => Ok(Value::Numerical(Numerical::Bool(order == Ordering::Less))),
        None => error(format!(
            "Cannot compare {} with {}",
            left.type_of(),
            right.type_of()
        )),
    }
}

fn bin_leq(left: Value, right: Value) -> RunRes<Value> {
    match left.partial_cmp(&right) {
        Some(order) => Ok(Value::Numerical(Numerical::Bool(
            order != Ordering::Greater,
        ))),
        None => error(format!(
            "Cannot compare {} with {}",
            left.type_of(),
            right.type_of()
        )),
    }
}

fn bin_gt(left: Value, right: Value) -> RunRes<Value> {
    match left.partial_cmp(&right) {
        Some(order) => Ok(Value::Numerical(Numerical::Bool(
            order == Ordering::Greater,
        ))),
        None => error(format!(
            "Cannot compare {} with {}",
            left.type_of(),
            right.type_of()
        )),
    }
}

fn bin_geq(left: Value, right: Value) -> RunRes<Value> {
    match left.partial_cmp(&right) {
        Some(order) => Ok(Value::Numerical(Numerical::Bool(order != Ordering::Less))),
        None => error(format!(
            "Cannot compare {} with {}",
            left.type_of(),
            right.type_of()
        )),
    }
}

fn eval_unary(op: &UnOper, right: Value) -> RunRes<Value> {
    match op {
        UnOper::Sub => match right {
            Value::Numerical(num) => Ok(num.un_sub()?.into()),
            _other => error("Unary subtraction only works for a number".to_string()),
        },
        UnOper::Not => Ok(Value::Numerical(Numerical::Bool(!right.truthy()))),
    }
}

impl LValue {
    pub fn declare(&self, env: &Rc<Environment>) -> RunRes<()> {
        match self {
            LValue::Var(id) => {
                env.define(id.to_string(), Value::Uninitialized);
                Ok(())
            }
            LValue::Index(_expr, _index) => {
                RunError::error("Cannot include an indexing in a declaration".to_string())
            }
            LValue::Tuple(lvalues) => {
                for lvalue in lvalues {
                    lvalue.declare(env)?;
                }
                Ok(())
            }
        }
    }

    pub fn assign(&self, rvalue: Value, env: &Rc<Environment>) -> RunRes<Value> {
        match self {
            LValue::Var(id) => env.assign(id, rvalue),
            LValue::Index(callee_expr, index_expr) => {
                let index = eval_index(index_expr, env)?;
                let base = eval(callee_expr, env)?;
                match base {
                    Value::Collection(collection) => collection.assign_into(rvalue, index),
                    other => error(format!(
                        "Cannot index into {} for assignment",
                        other.type_of()
                    )),
                }
            }
            LValue::Tuple(lvalues) => {
                let mut lvalues_iter = lvalues.into_iter();
                let mut rvalues_iter = rvalue.clone().to_iter()?;
                loop {
                    match (lvalues_iter.next(), rvalues_iter.next()) {
                        (Some(lvalue), Some(rvalue)) => {
                            lvalue.assign(rvalue, env)?;
                        }
                        (None, None) => break Ok(rvalue),
                        (None, _) => {
                            break RunError::error(format!(
                                "{} rvalues remain after all lvalues",
                                rvalues_iter.count() + 1
                            ))
                        }
                        (_, None) => {
                            break RunError::error(format!(
                                "{} lvalues remain after all rvalues",
                                lvalues_iter.count() + 1
                            ))
                        }
                    }
                }
            }
        }
    }
}

fn eval_logical(
    left: Value,
    op: &LogicalOper,
    right: &ExprNode,
    env: &Rc<Environment>,
) -> RunRes<Value> {
    let res = match op {
        LogicalOper::And => left.truthy() && eval(right, env)?.truthy(),
        LogicalOper::Or => left.truthy() || eval(right, env)?.truthy(),
    };
    Ok(Value::Numerical(Numerical::Bool(res)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorReporter;
    use crate::interpreter::functions::define_builtins;
    use crate::parser::parse;
    use crate::scanner::tokenize;

    /// Helper to interpret an expression from a string
    fn interpret_expression_string(program: &str) -> RunRes<Value> {
        let mut error_reporter = ErrorReporter::new();
        let tokens = tokenize(program, &mut error_reporter);
        let ast = parse(&tokens, &mut error_reporter).unwrap();
        let env = Environment::new();
        define_builtins(&env);
        statements::eval_statements(&ast, &env)
            .map(|opt_val| opt_val.expect("Expects an uncaptured expressions"))
    }

    #[test]
    fn basic_int_math() {
        let program = "1 + 6 / 4 + 20 * -2 / 1";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Numerical(Numerical::Int(-38))));
    }

    #[test]
    fn more_int_math() {
        let program = "5^7 == 78125 and -5^7 == -78125 and 7 % 4 == 3 and -7 % 4 == 1";
        let val = interpret_expression_string(program).unwrap();
        assert_eq!(val, true.into());
    }

    #[test]
    fn float_comparisons() {
        // Not the prettiest, but easy to find if one fails, rather than having one big string.
        let program = "2.5*3125.0 > 2.499*3125.0";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Numerical(Numerical::Bool(true))));

        let program = "0.0 == 0.0";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Numerical(Numerical::Bool(true))));

        let program = "2.2/5.1 - 3.5*5.0 < -17.0";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Numerical(Numerical::Bool(true))));

        let program = "!(1.1>=1.100001)";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Numerical(Numerical::Bool(true))));

        let program = "!(2.2 != 2.2)";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Numerical(Numerical::Bool(true))));

        let program = "1.1 <= 1.01*1.11";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Numerical(Numerical::Bool(true))));

        let program = "2.000000001 % 0.1 < 0.00001";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Numerical(Numerical::Bool(true))));

        let program = "2.2^-2.2 >= 0.176";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Numerical(Numerical::Bool(true))));
    }

    #[test]
    fn short_circuits() {
        let program = "true or time('invalid argument count') and \
                       !(false and time('again the same...'))";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Numerical(Numerical::Bool(true))));
    }

    #[test]
    fn nil_returns() {
        let program = "{ 2; }";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Nil));

        // No longer a nil return!
        let program = "print(3)";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Numerical(Numerical::Int(3))));

        let program = "if false 1";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Nil));
    }
}
