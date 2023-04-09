use std::fmt;

use crate::parser::{
    AstLoc, AstNode, BinOper, BinOperNode, Expr, ExprNode, Stmt, StmtNode, UnOper, UnOperNode,
};

use super::{environment::Environment, statements, RuntimeError};

// An interface between Zote and Rust values
#[derive(PartialEq, Debug, PartialOrd, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Uninitialized,
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Value::Bool(bool) => *bool,
            Value::Int(int) => *int != 0,
            Value::Float(float) => *float != 0.0,
            Value::String(string) => !string.is_empty(),
            Value::Uninitialized => panic!("Use of uninit value!"),
        }
    }

    pub fn stringify(&self) -> String {
        match self {
            Value::Bool(bool) => format!("{bool}"),
            Value::Int(int) => format!("{int}"),
            Value::Float(float) => format!("{float}"),
            Value::String(string) => string.to_string(),
            Value::Uninitialized => panic!("Use of uninit value!"),
        }
    }
}

pub fn eval(expr: &ExprNode, env: &Environment) -> Result<Value, RuntimeError> {
    match &expr.node {
        Expr::Call => Err((expr.loc, "Function calls not implemented".to_string())),
        Expr::Binary(left, op, right) => {
            let val = eval(right, env)?;
            eval_binary(eval(left, env)?, op, val, expr.loc)
        }
        Expr::Unary(op, right) => eval_unary(op, eval(right, env)?, expr.loc),
        Expr::Assign(lvalue, expr) => {
            let val = eval(expr, env)?;
            eval_assign(lvalue, val, env, expr.loc)
        }
        Expr::Var(id) => env
            .get(id)
            .ok_or_else(|| (expr.loc, format!("Variable '{id}' not declared"))),
        Expr::Int(int) => Ok(Value::Int(*int)),
        Expr::Float(float) => Ok(Value::Float(*float)),
        Expr::Bool(bool) => Ok(Value::Bool(*bool)),
        Expr::String(string) => Ok(Value::String(string.clone())),
        Expr::Block(stmts) => eval_block(stmts, env, expr.loc),
    }
}

fn eval_block(
    stmts: &Vec<StmtNode>,
    env: &Environment,
    _loc: AstLoc,
) -> Result<Value, RuntimeError> {
    // Not super pretty, would maybe be better with rusts use of no colon if we return
    let nested_env = env.nest();
    for stmt in stmts[0..(stmts.len() - 1)].iter() {
        statements::eval(stmt, &nested_env)?;
    }

    match stmts.last() {
        Some(AstNode {
            node: Stmt::Expr(expr),
            loc: _loc,
        }) => eval(expr, &nested_env),
        Some(stmt) => {
            statements::eval(stmt, &nested_env)?;
            Ok(Value::Uninitialized)
        }
        None => Ok(Value::Uninitialized),
    }
}

fn eval_assign(
    lvalue: &str,
    rvalue: Value,
    env: &Environment,
    start: AstLoc,
) -> Result<Value, RuntimeError> {
    if env.assign(lvalue, rvalue.clone()).is_some() {
        Ok(rvalue)
    } else {
        Err((start, format!("Variable '{lvalue}' not declared")))
    }
}

fn eval_binary(
    left: Value,
    op: &BinOperNode,
    right: Value,
    loc: AstLoc,
) -> Result<Value, RuntimeError> {
    match &op.node {
        // TODO: implicit conversion from int to float. Now they only work together on comparisions
        BinOper::Add => bin_add(left, right, loc),
        BinOper::Sub => bin_sub(left, right, loc),
        BinOper::Mult => bin_mult(left, right, loc),
        BinOper::Div => bin_div(left, right, loc),
        BinOper::And => Ok(Value::Bool(left.truthy() && right.truthy())),
        BinOper::Or => Ok(Value::Bool(left.truthy() || right.truthy())),
        BinOper::Eq => bin_eq(left, right, loc),
        BinOper::Neq => bin_neq(left, right, loc),
        BinOper::Lt => bin_lt(left, right, loc),
        BinOper::Leq => bin_leq(left, right, loc),
        BinOper::Gt => bin_gt(left, right, loc),
        BinOper::Geq => bin_geq(left, right, loc),
    }
}

fn bin_add(left: Value, right: Value, loc: AstLoc) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
        (Value::String(x), Value::String(y)) => Ok(Value::String(x + &y)),
        _other => Err((
            loc,
            "Addition operands must be two numbers or two strings".to_string(),
        )),
    }
}

fn bin_sub(left: Value, right: Value, loc: AstLoc) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
        _other => Err((loc, "Subtraction operands must be two numbers".to_string())),
    }
}

fn bin_mult(left: Value, right: Value, loc: AstLoc) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
        _other => Err((
            loc,
            "Multiplication operands must be two numbers".to_string(),
        )),
    }
}

fn bin_div(left: Value, right: Value, loc: AstLoc) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x / y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x / y)),
        _other => Err((loc, "Division operands must be two numbers".to_string())),
    }
}

fn bin_eq(left: Value, right: Value, _loc: AstLoc) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x == y as f64)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Bool(x as f64 == y)),
        (x, y) => Ok(Value::Bool(x == y)),
    }
}

fn bin_neq(left: Value, right: Value, _loc: AstLoc) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x != y as f64)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Bool(x as f64 != y)),
        (x, y) => Ok(Value::Bool(x == y)),
    }
}

fn bin_lt(left: Value, right: Value, loc: AstLoc) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x < y as f64)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Bool((x as f64) < y)),
        (x, y) if same_type(&x, &y) => Ok(Value::Bool(x < y)),
        (x, y) => Err((loc, format!("Cannot compare {} and {}", x, y))),
    }
}

fn bin_leq(left: Value, right: Value, loc: AstLoc) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x <= y as f64)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Bool((x as f64) <= y)),
        (x, y) if same_type(&x, &y) => Ok(Value::Bool(x <= y)),
        (x, y) => Err((loc, format!("Cannot compare {} and {}", x, y))),
    }
}

fn bin_gt(left: Value, right: Value, loc: AstLoc) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x > y as f64)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Bool((x as f64) > y)),
        (x, y) if same_type(&x, &y) => Ok(Value::Bool(x > y)),
        (x, y) => Err((loc, format!("Cannot compare {} and {}", x, y))),
    }
}

fn bin_geq(left: Value, right: Value, loc: AstLoc) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x >= y as f64)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Bool((x as f64) >= y)),
        (x, y) if same_type(&x, &y) => Ok(Value::Bool(x >= y)),
        (x, y) => Err((loc, format!("Cannot compare {} and {}", x, y))),
    }
}

fn eval_unary(op: &UnOperNode, right: Value, loc: AstLoc) -> Result<Value, RuntimeError> {
    match op.node {
        UnOper::Sub => match right {
            Value::Int(int) => Ok(Value::Int(-int)),
            Value::Float(float) => Ok(Value::Float(-float)),
            _other => Err((loc, "Unary subtraction only works for a number".to_string())),
        },
        UnOper::Not => Ok(Value::Bool(!right.truthy())),
    }
}

fn same_type(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Float(_), Value::Float(_))
        | (Value::Int(_), Value::Int(_))
        | (Value::String(_), Value::String(_))
        | (Value::Bool(_), Value::Bool(_)) => true,
        _ => false,
    }
}

// Simple print with the value wrapped in its type, for informative prints
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Bool(bool) => write!(f, "Bool({bool})"),
            Value::Int(int) => write!(f, "Int({int})"),
            Value::Float(float) => write!(f, "Float({float})"),
            Value::String(string) => write!(f, "String({string})"),
            Value::Uninitialized => panic!("Use of uninit value!"),
        }
    }
}
