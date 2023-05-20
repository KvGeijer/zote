use std::{cmp::Ordering, fmt, iter::Take, rc::Rc};

use crate::{
    code_loc::CodeLoc,
    interpreter::list::slice_iter,
    parser::{
        BinOper, BinOperNode, Expr, ExprNode, Index, LogicalOper, LogicalOperNode, Stmts, UnOper,
        UnOperNode,
    },
};

use super::{
    environment::Environment,
    functions::{Closure, Function},
    list::List,
    numerical::Numerical,
    statements, RunRes, RuntimeError,
};

// An interface between Zote and Rust values
#[derive(PartialEq, Debug, Clone)]
pub(super) enum Value {
    Numerical(Numerical),
    String(String),
    Callable(Function),
    List(List),
    Nil,
    Uninitialized,
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Value::Numerical(num) => num.truthy(),
            Value::String(string) => !string.is_empty(),
            Value::Callable(_) => panic!("Can't convert function to bool"), // TODO: real error, or just warning
            Value::Nil => false,
            Value::Uninitialized => false,
            Value::List(list) => list.to_bool(),
        }
    }

    pub fn stringify(&self) -> String {
        // OPT Could we just return &str here?
        match self {
            Value::Numerical(num) => num.stringify(),
            Value::String(string) => string.to_string(),
            Value::Callable(callable) => callable.name().to_string(),
            Value::Nil => "Nil".to_string(),
            Value::Uninitialized => panic!("Use of uninit value!"),
            Value::List(list) => list.stringify(),
        }
    }

    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Numerical(num) => num.type_of(),
            Value::String(_) => "String",
            Value::Callable(_) => "Function",
            Value::Nil => "Nil",
            Value::Uninitialized => "Uninitialized",
            Value::List(_) => "List",
        }
    }
}

pub(super) fn eval(expr: &ExprNode, env: &Rc<Environment>) -> RunRes<Value> {
    let start = expr.start_loc.clone();
    let end = expr.end_loc.clone();
    match expr.node.as_ref() {
        Expr::Binary(left, op, right) => {
            eval_binary(eval(left, env)?, op, eval(right, env)?, start, end)
        }
        Expr::Logical(left, op, right) => {
            eval_logical(eval(left, env)?, op, right, env, start, end)
        }

        Expr::Unary(op, right) => eval_unary(op, eval(right, env)?, start, end),
        Expr::Assign(lvalue, expr) => {
            let val = eval(expr, env)?;
            eval_assign(lvalue, val, env, start, end)
        }
        Expr::Var(id) => env.get(id).ok_or_else(|| {
            RuntimeError::Error(start, end, format!("Variable '{id}' not declared"))
        }),
        Expr::Int(int) => Ok(Value::Numerical(Numerical::Int(*int))),
        Expr::Float(float) => Ok(Value::Numerical(Numerical::Float(*float))),
        Expr::Bool(bool) => Ok(Value::Numerical(Numerical::Bool(*bool))),
        Expr::String(string) => Ok(Value::String(string.clone())),
        Expr::Block(stmts) => eval_block(stmts, env, start, end),
        Expr::If(cond, then, other) => eval_if(eval(cond, env)?, then, other.as_ref(), env),
        Expr::While(cond, repeat) => eval_while(cond, repeat, env),
        Expr::Break => Err(RuntimeError::Break),
        Expr::Call(callee, args) => eval_call(
            eval(callee, env)?,
            args.iter()
                .map(|arg| eval(arg, env))
                .collect::<Result<Vec<_>, _>>()?,
            start,
            end,
        ),
        Expr::Return(Some(expr)) => Err(RuntimeError::Return(eval(expr, env)?)),
        Expr::Return(None) => Err(RuntimeError::Return(Value::Nil)),
        Expr::Nil => Ok(Value::Nil),
        Expr::List(exprs) => eval_list(exprs, env),
        Expr::Tuple(_exprs) => error(
            start,
            end,
            "Tuples are not part of the language (yet)".to_string(),
        ),
        Expr::FunctionDefinition(name, param, body) => eval_func_definition(name, param, body, env),
        Expr::Index(base, index) => eval_index(base, index, end, env),
    }
}

fn up_err<T>(result: Result<T, String>, start: CodeLoc, end: CodeLoc) -> RunRes<T> {
    match result {
        Err(reason) => error(start, end, reason),
        Ok(val) => Ok(val),
    }
}

// Is this the most beautiful function ever?!?
fn eval_index(
    base: &ExprNode,
    index: &Index,
    end: CodeLoc,
    env: &Rc<Environment>,
) -> RunRes<Value> {
    let start_loc = base.start_loc.clone();
    match index {
        Index::At(at) => match eval(base, env)? {
            Value::List(list) => up_err(list.get(eval(at, env)?), start_loc, end),
            Value::String(string) => {
                if let Value::Numerical(num) = eval(at, env)? {
                    let maybe_indexed = string
                        .chars()
                        .nth(num.to_rint() as usize)
                        .map(|char| char.to_string().into());
                    up_err(
                        maybe_indexed.ok_or(format!(
                            "Index {} out of bound for string of length {}",
                            num.to_rint(),
                            string.len()
                        )),
                        start_loc,
                        end,
                    )
                } else {
                    error(
                        start_loc,
                        end,
                        "Can only index into a string with a numerical".to_string(),
                    )
                }
            }
            other => error(
                start_loc,
                end,
                format!("Cannot index into a {}", other.type_of()),
            ),
        },
        Index::Slice { start, stop, step } => {
            let start = match start.as_ref().map(|expr| eval(expr, env)) {
                Some(Ok(Value::Numerical(num))) => Some(num.to_rint()),
                None => None,
                Some(Err(err)) => return Err(err),
                Some(_other) => {
                    return error(
                        start_loc,
                        end,
                        "Start of slice must be a numerical".to_string(),
                    )
                }
            };

            let stop = match stop.as_ref().map(|expr| eval(expr, env)) {
                Some(Ok(Value::Numerical(num))) => Some(num.to_rint()),
                None => None,
                Some(Err(err)) => return Err(err),
                Some(_other) => {
                    return error(
                        start_loc,
                        end,
                        "Stop of slice must be a numerical".to_string(),
                    )
                }
            };

            let step = match step.as_ref().map(|expr| eval(expr, env)) {
                Some(Ok(Value::Numerical(num))) => Some(num.to_rint()),
                None => None,
                Some(Err(err)) => return Err(err),
                Some(_other) => {
                    return error(
                        start_loc,
                        end,
                        "Step of slice must be a numerical".to_string(),
                    )
                }
            };

            match eval(base, env)? {
                Value::List(list) => up_err(list.slice(start, stop, step), start_loc, end),
                Value::String(string) => {
                    let len = string.chars().count();
                    let sliced: String = up_err::<Take<_>>(
                        slice_iter(string.chars().into_iter(), start, stop, step, len),
                        start_loc,
                        end,
                    )?
                    .collect();
                    Ok(sliced.into())
                }
                other => error(
                    start_loc,
                    end,
                    format!("Cannot slice a {}", other.type_of()),
                ),
            }
        }
    }
}

fn eval_func_definition(
    id: &str,
    param: &[String],
    body: &ExprNode,
    env: &Rc<Environment>,
) -> RunRes<Value> {
    let closure = Closure::new(id.to_string(), param.to_vec(), body.clone(), env);
    Ok(Value::Callable(Function::Closure(closure)))
}

fn eval_list(exprs: &[ExprNode], env: &Rc<Environment>) -> RunRes<Value> {
    Ok(Value::List(List::new(
        exprs
            .iter()
            .map(|expr| eval(expr, env))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter(), // Not beautiful collecting and then making back to iterator
    )))
}

fn eval_call(callee: Value, args: Vec<Value>, start: CodeLoc, end: CodeLoc) -> RunRes<Value> {
    if let Value::Callable(callable) = callee {
        if args.len() == callable.arity() {
            match callable.call(args) {
                Err(RuntimeError::Break) => Err(RuntimeError::Error(
                    start,
                    end,
                    "Break encountered outside loop".to_string(),
                )),
                Err(RuntimeError::Return(value)) => Ok(value),
                Err(RuntimeError::ErrorReason(reason)) => error(start, end, reason),
                // TODO Error traces...
                otherwise => otherwise,
            }
        } else {
            error(
                start,
                end,
                format!(
                    "Expected {} arguments but got {}.",
                    callable.arity(),
                    args.len()
                ),
            )
        }
    } else {
        error(start, end, "Can only call functions".to_string())
    }
}

fn error<T>(start: CodeLoc, end: CodeLoc, message: String) -> RunRes<T> {
    Err(RuntimeError::Error(start, end, message))
}

fn def_block_return() -> Value {
    Value::Nil
}

fn eval_while(cond: &ExprNode, repeat: &ExprNode, env: &Rc<Environment>) -> RunRes<Value> {
    while eval(cond, env)?.truthy() {
        match eval(repeat, env) {
            Err(RuntimeError::Break) => break,
            otherwise => otherwise?,
        };
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

fn eval_block(
    stmts: &Stmts,
    env: &Rc<Environment>,
    _start_loc: CodeLoc,
    _end_loc: CodeLoc,
) -> RunRes<Value> {
    // Not super pretty, would maybe be better with rusts use of no colon if we return
    let nested_env = Environment::nest(env);
    match statements::eval_statements(stmts, &nested_env)? {
        Some(val) => Ok(val),
        None => Ok(def_block_return()),
    }
}

fn eval_assign(
    lvalue: &str,
    rvalue: Value,
    env: &Rc<Environment>,
    start: CodeLoc,
    end: CodeLoc,
) -> RunRes<Value> {
    if env.assign(lvalue, rvalue.clone()).is_some() {
        Ok(rvalue)
    } else {
        error(start, end, format!("Variable '{lvalue}' not declared"))
    }
}

fn eval_binary(
    left: Value,
    op: &BinOperNode,
    right: Value,
    start_loc: CodeLoc,
    end_loc: CodeLoc,
) -> RunRes<Value> {
    match op.node.as_ref() {
        BinOper::Add => bin_add(left, right, start_loc, end_loc),
        BinOper::Sub => bin_sub(left, right, start_loc, end_loc),
        BinOper::Mult => bin_mult(left, right, start_loc, end_loc),
        BinOper::Div => bin_div(left, right, start_loc, end_loc),
        BinOper::Mod => bin_mod(left, right, start_loc, end_loc),
        BinOper::Pow => bin_pow(left, right, start_loc, end_loc),
        BinOper::Eq => bin_eq(left, right, start_loc, end_loc),
        BinOper::Neq => bin_neq(left, right, start_loc, end_loc),
        BinOper::Lt => bin_lt(left, right, start_loc, end_loc),
        BinOper::Leq => bin_leq(left, right, start_loc, end_loc),
        BinOper::Gt => bin_gt(left, right, start_loc, end_loc),
        BinOper::Geq => bin_geq(left, right, start_loc, end_loc),
    }
}

fn bin_add(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Numerical(x), Value::Numerical(y)) => Ok(Value::Numerical(x.add(y))),
        (Value::String(x), Value::String(y)) => Ok(Value::String(x + &y)),
        (left, right) => error(
            start_loc,
            end_loc,
            format!("Cannot add {} and {}", left.stringify(), right.stringify()),
        ),
    }
}

fn bin_sub(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Numerical(x), Value::Numerical(y)) => Ok(Value::Numerical(x.sub(y))),
        (left, right) => error(
            start_loc,
            end_loc,
            format!(
                "Cannot subtract {} from {}",
                right.stringify(),
                left.stringify()
            ),
        ),
    }
}

fn bin_mult(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Numerical(x), Value::Numerical(y)) => Ok(Value::Numerical(x.mult(y))),
        (left, right) => error(
            start_loc,
            end_loc,
            format!(
                "Cannot multiply {} and {}",
                left.stringify(),
                right.stringify()
            ),
        ),
    }
}

fn bin_div(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Numerical(x), Value::Numerical(y)) => match x.div(y) {
            Ok(num) => Ok(Value::Numerical(num)),
            Err(reason) => error(start_loc, end_loc, reason),
        },
        (left, right) => error(
            start_loc,
            end_loc,
            format!(
                "Cannot divide {} by {}",
                left.stringify(),
                right.stringify()
            ),
        ),
    }
}

fn bin_mod(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Numerical(x), Value::Numerical(y)) => Ok(Value::Numerical(x.modulo(y))),
        _other => error(start_loc, end_loc, format!("Modulo only works for numbers")),
    }
}

fn bin_pow(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Numerical(x), Value::Numerical(y)) => Ok(Value::Numerical(x.pow(y))),
        _other => error(
            start_loc,
            end_loc,
            format!("Can only take powers of numbers"),
        ),
    }
}

fn bin_eq(left: Value, right: Value, _start: CodeLoc, _end: CodeLoc) -> RunRes<Value> {
    Ok(Value::Numerical(Numerical::Bool(left == right)))
}

fn bin_neq(left: Value, right: Value, _start: CodeLoc, _end: CodeLoc) -> RunRes<Value> {
    Ok(Value::Numerical(Numerical::Bool(left != right)))
}

fn bin_lt(left: Value, right: Value, start: CodeLoc, end: CodeLoc) -> RunRes<Value> {
    match left.partial_cmp(&right) {
        Some(order) => Ok(Value::Numerical(Numerical::Bool(order == Ordering::Less))),
        None => error(
            start,
            end,
            format!("Cannot compare {} with {}", left.type_of(), right.type_of()),
        ),
    }
}

fn bin_leq(left: Value, right: Value, start: CodeLoc, end: CodeLoc) -> RunRes<Value> {
    match left.partial_cmp(&right) {
        Some(order) => Ok(Value::Numerical(Numerical::Bool(
            order != Ordering::Greater,
        ))),
        None => error(
            start,
            end,
            format!("Cannot compare {} with {}", left.type_of(), right.type_of()),
        ),
    }
}

fn bin_gt(left: Value, right: Value, start: CodeLoc, end: CodeLoc) -> RunRes<Value> {
    match left.partial_cmp(&right) {
        Some(order) => Ok(Value::Numerical(Numerical::Bool(
            order == Ordering::Greater,
        ))),
        None => error(
            start,
            end,
            format!("Cannot compare {} with {}", left.type_of(), right.type_of()),
        ),
    }
}

fn bin_geq(left: Value, right: Value, start: CodeLoc, end: CodeLoc) -> RunRes<Value> {
    match left.partial_cmp(&right) {
        Some(order) => Ok(Value::Numerical(Numerical::Bool(order != Ordering::Less))),
        None => error(
            start,
            end,
            format!("Cannot compare {} with {}", left.type_of(), right.type_of()),
        ),
    }
}

fn eval_unary(op: &UnOperNode, right: Value, start: CodeLoc, end: CodeLoc) -> RunRes<Value> {
    match op.node.as_ref() {
        UnOper::Sub => match right {
            Value::Numerical(num) => match num.un_sub() {
                Ok(num) => Ok(Value::Numerical(num)),
                Err(reason) => error(start, end, reason),
            },
            _other => error(
                start,
                end,
                "Unary subtraction only works for a number".to_string(),
            ),
        },
        UnOper::Not => Ok(Value::Numerical(Numerical::Bool(!right.truthy()))),
    }
}

fn eval_logical(
    left: Value,
    op: &LogicalOperNode,
    right: &ExprNode,
    env: &Rc<Environment>,
    _start_loc: CodeLoc,
    _end_loc: CodeLoc,
) -> RunRes<Value> {
    let res = match op.node.as_ref() {
        LogicalOper::And => left.truthy() && eval(right, env)?.truthy(),
        LogicalOper::Or => left.truthy() || eval(right, env)?.truthy(),
    };
    Ok(Value::Numerical(Numerical::Bool(res)))
}

// Simple print with the value wrapped in its type, for informative prints
// Why do we need this and stringify?
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Callable(callable) => write!(f, "fn {}/{}", callable.name(), callable.arity()),
            Value::Nil => write!(f, "Nil"),
            Value::Uninitialized => panic!("Use of uninit value!"),
            value => write!(f, "{}({})", value.type_of(), value.stringify()),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Numerical(x), Value::Numerical(y)) => x.partial_cmp(y),
            (Value::String(x), Value::String(y)) => x.partial_cmp(y),
            _ => None,
        }
    }
}

impl From<Numerical> for Value {
    fn from(item: Numerical) -> Self {
        Value::Numerical(item)
    }
}

impl From<i64> for Value {
    fn from(item: i64) -> Self {
        let num: Numerical = item.into();
        num.into()
    }
}

impl From<f64> for Value {
    fn from(item: f64) -> Self {
        let num: Numerical = item.into();
        num.into()
    }
}

impl From<bool> for Value {
    fn from(item: bool) -> Self {
        let num: Numerical = item.into();
        num.into()
    }
}

impl From<String> for Value {
    fn from(item: String) -> Self {
        Value::String(item)
    }
}

impl From<Vec<Value>> for Value {
    fn from(item: Vec<Value>) -> Self {
        Value::List(List::new(item.into_iter()))
    }
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

        let program = "print(3)";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Nil));

        let program = "if false 1";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Nil));
    }
}
