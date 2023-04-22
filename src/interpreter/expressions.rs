use std::{fmt, rc::Rc};

use crate::{
    code_loc::CodeLoc,
    parser::{
        BinOper, BinOperNode, Expr, ExprNode, LogicalOper, LogicalOperNode, Stmts, UnOper,
        UnOperNode,
    },
};

use super::{environment::Environment, functions::Function, statements, RunRes, RuntimeError};

// An interface between Zote and Rust values
#[derive(PartialEq, Debug, PartialOrd, Clone)]
pub(super) enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Callable(Function),
    Uninitialized,
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Value::Bool(bool) => *bool,
            Value::Int(int) => *int != 0,
            Value::Float(float) => *float != 0.0,
            Value::String(string) => !string.is_empty(),
            Value::Callable(_) => panic!("Can't convert function to bool"), // TODO: real error, or just warning
            Value::Uninitialized => panic!("Use of uninit value!"),
        }
    }

    pub fn stringify(&self) -> String {
        match self {
            Value::Bool(bool) => format!("{bool}"),
            Value::Int(int) => format!("{int}"),
            Value::Float(float) => format!("{float}"),
            Value::String(string) => string.to_string(),
            Value::Callable(callable) => callable.name().to_string(),
            Value::Uninitialized => panic!("Use of uninit value!"),
        }
    }
}

pub(super) fn eval(expr: &ExprNode, env: &Rc<Environment>) -> RunRes<Value> {
    let start = expr.start_loc.clone();
    let end = expr.end_loc.clone();
    match &expr.node {
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
        Expr::Int(int) => Ok(Value::Int(*int)),
        Expr::Float(float) => Ok(Value::Float(*float)),
        Expr::Bool(bool) => Ok(Value::Bool(*bool)),
        Expr::String(string) => Ok(Value::String(string.clone())),
        Expr::Block(stmts) => eval_block(stmts, env, start, end),
        Expr::If(cond, then, other) => eval_if(eval(cond, env)?, then, other.as_deref(), env),
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
        Expr::Return(expr) => Err(RuntimeError::Return(eval(expr, env)?)),
    }
}

fn eval_call(callee: Value, args: Vec<Value>, start: CodeLoc, end: CodeLoc) -> RunRes<Value> {
    if let Value::Callable(callable) = callee {
        if args.len() == callable.arity() {
            callable.call(args, start, end)
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

fn error(start: CodeLoc, end: CodeLoc, message: String) -> RunRes<Value> {
    Err(RuntimeError::Error(start, end, message))
}

fn def_block_return() -> Value {
    Value::Uninitialized
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
    match &op.node {
        // TODO: implicit conversion from int to float. Now they only work together on comparisions
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
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
        (Value::String(x), Value::String(y)) => Ok(Value::String(x + &y)),
        _other => error(
            start_loc,
            end_loc,
            "Addition operands must be two numbers or two strings".to_string(),
        ),
    }
}

fn bin_sub(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
        _other => error(
            start_loc,
            end_loc,
            "Subtraction operands must be two numbers".to_string(),
        ),
    }
}

fn bin_mult(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
        _other => error(
            start_loc,
            end_loc,
            "Multiplication operands must be two numbers".to_string(),
        ),
    }
}

fn bin_div(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x / y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x / y)),
        _other => error(
            start_loc,
            end_loc,
            "Division operands must be two numbers".to_string(),
        ),
    }
}

fn bin_mod(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x.rem_euclid(y))),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x.rem_euclid(y))),
        _other => error(
            start_loc,
            end_loc,
            "Modulo operands must be two numbers".to_string(),
        ),
    }
}

fn bin_pow(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x.powf(y))),
        (Value::Int(x), Value::Int(y)) if y >= 0 => Ok(Value::Int({
            let safe_x: u32 = x.abs().try_into().expect("Overflow in pow base");
            let safe_y: u32 = y.try_into().expect("Overflow in pow exponent");
            let pow = safe_x.pow(safe_y) as i64;
            if x >= 0 {
                pow
            } else {
                -pow
            }
        })),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Float((x as f64).powf(y as f64))),
        _other => error(
            start_loc,
            end_loc,
            "Division operands must be two numbers".to_string(),
        ),
    }
}

fn bin_eq(left: Value, right: Value, _start_loc: CodeLoc, _end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x == y as f64)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Bool(x as f64 == y)),
        (x, y) => Ok(Value::Bool(x == y)),
    }
}

fn bin_neq(left: Value, right: Value, _start_loc: CodeLoc, _end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x != y as f64)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Bool(x as f64 != y)),
        (x, y) => Ok(Value::Bool(x != y)),
    }
}

fn bin_lt(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x < y as f64)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Bool((x as f64) < y)),
        (x, y) if same_type(&x, &y) => Ok(Value::Bool(x < y)),
        (x, y) => error(
            start_loc,
            end_loc,
            format!("Cannot compare {} and {}", x, y),
        ),
    }
}

fn bin_leq(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x <= y as f64)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Bool((x as f64) <= y)),
        (x, y) if same_type(&x, &y) => Ok(Value::Bool(x <= y)),
        (x, y) => error(
            start_loc,
            end_loc,
            format!("Cannot compare {} and {}", x, y),
        ),
    }
}

fn bin_gt(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x > y as f64)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Bool((x as f64) > y)),
        (x, y) if same_type(&x, &y) => Ok(Value::Bool(x > y)),
        (x, y) => error(
            start_loc,
            end_loc,
            format!("Cannot compare {} and {}", x, y),
        ),
    }
}

fn bin_geq(left: Value, right: Value, start_loc: CodeLoc, end_loc: CodeLoc) -> RunRes<Value> {
    match (left, right) {
        (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x >= y as f64)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Bool((x as f64) >= y)),
        (x, y) if same_type(&x, &y) => Ok(Value::Bool(x >= y)),
        (x, y) => error(
            start_loc,
            end_loc,
            format!("Cannot compare {} and {}", x, y),
        ),
    }
}

fn eval_unary(
    op: &UnOperNode,
    right: Value,
    start_loc: CodeLoc,
    end_loc: CodeLoc,
) -> RunRes<Value> {
    match op.node {
        UnOper::Sub => match right {
            Value::Int(int) => Ok(Value::Int(-int)),
            Value::Float(float) => Ok(Value::Float(-float)),
            _other => error(
                start_loc,
                end_loc,
                "Unary subtraction only works for a number".to_string(),
            ),
        },
        UnOper::Not => Ok(Value::Bool(!right.truthy())),
    }
}

fn same_type(left: &Value, right: &Value) -> bool {
    matches!(
        (left, right),
        (Value::Float(_), Value::Float(_))
            | (Value::Int(_), Value::Int(_))
            | (Value::String(_), Value::String(_))
            | (Value::Bool(_), Value::Bool(_))
    )
}

fn eval_logical(
    left: Value,
    op: &LogicalOperNode,
    right: &ExprNode,
    env: &Rc<Environment>,
    _start_loc: CodeLoc,
    _end_loc: CodeLoc,
) -> RunRes<Value> {
    let res = match op.node {
        LogicalOper::And => left.truthy() && eval(right, env)?.truthy(),
        LogicalOper::Or => left.truthy() || eval(right, env)?.truthy(),
    };
    Ok(Value::Bool(res))
}

// Simple print with the value wrapped in its type, for informative prints
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Bool(bool) => write!(f, "Bool({bool})"),
            Value::Int(int) => write!(f, "Int({int})"),
            Value::Float(float) => write!(f, "Float({float})"),
            Value::String(string) => write!(f, "String({string})"),
            Value::Callable(callable) => write!(f, "fn {}/{}", callable.name(), callable.arity()),
            Value::Uninitialized => panic!("Use of uninit value!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorReporter;
    use crate::parser::parse;
    use crate::scanner::tokenize;

    /// Helper to interpret an expression from a string
    fn interpret_expression_string(program: &str) -> RunRes<Value> {
        let mut error_reporter = ErrorReporter::new();
        let tokens = tokenize(program, &mut error_reporter);
        let ast = parse(&tokens, &mut error_reporter).unwrap();
        statements::eval_statements(&ast, &Environment::new())
            .map(|opt_val| opt_val.expect("Expects an uncaptured expressions"))
    }

    #[test]
    fn basic_int_math() {
        let program = "1 + 6 / 4 + 20 * -2 / 1";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Int(-38)));
    }

    #[test]
    fn more_int_math() {
        let program = "5^6 == 15625 and -5^6 == -15625 and 7 % 4 == 3 and -7 % 4 == 1";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Bool(true)));
    }

    #[test]
    fn float_comparisons() {
        // Not the prettiest, but easy to find if one fails, rather than having one big string.
        let program = "2.5*3125.0 > 2.499*3125.0";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Bool(true)));

        let program = "0.0 == 0.0";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Bool(true)));

        let program = "2.2/5.1 - 3.5*5.0 < -17.0";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Bool(true)));

        let program = "!(1.1>=1.100001)";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Bool(true)));

        let program = "!(2.2 != 2.2)";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Bool(true)));

        let program = "1.1 <= 1.01*1.11";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Bool(true)));

        let program = "2.000000001 % 0.1 < 0.00001";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Bool(true)));

        let program = "2.2^-2.2 >= 0.176";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Bool(true)));
    }

    #[test]
    fn short_circuits() {
        let program = "true or time('invalid argument count') and \
                       !(false and time('again the same...'))";
        let val = interpret_expression_string(program).unwrap();
        assert!(matches!(val, Value::Bool(true)));
    }
}
