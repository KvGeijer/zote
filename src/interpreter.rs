use crate::{
    errors::ErrorReporter,
    parser::{AstLoc, BinOper, BinOperNode, Expr, ExprNode, UnOper, UnOperNode},
};

pub fn interpret(program: &ExprNode, error_reporter: &mut ErrorReporter) {
    match eval(program) {
        Ok(value) => println!("{}", value.stringify()),
        Err((loc, reason)) => error_reporter.runtime_error(&loc, &reason),
    }
}

// An interface between Zote and Rust values
#[derive(PartialEq, Debug, PartialOrd)]
enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Value::Bool(bool) => *bool,
            Value::Int(int) => *int != 0,
            Value::Float(float) => *float != 0.0,
            Value::String(string) => !string.is_empty(),
        }
    }

    fn stringify(&self) -> String {
        match self {
            Value::Bool(bool) => format!("{bool}"),
            Value::Int(int) => format!("{int}"),
            Value::Float(float) => format!("{float}"),
            Value::String(string) => string.to_string(),
        }
    }
}

fn eval(expr: &ExprNode) -> Result<Value, (AstLoc, String)> {
    match &expr.node {
        Expr::Call => Err((expr.loc, "Function calls not implemented".to_string())),
        Expr::Binary(left, op, right) => eval_binary(eval(left)?, op, eval(right)?, expr.loc),
        Expr::Unary(op, right) => eval_unary(op, eval(right)?, expr.loc),
        Expr::Int(int) => Ok(Value::Int(*int)),
        Expr::Float(float) => Ok(Value::Float(*float)),
        Expr::Bool(bool) => Ok(Value::Bool(*bool)),
        Expr::String(string) => Ok(Value::String(string.clone())),
    }
}

fn eval_binary(
    left: Value,
    op: &BinOperNode,
    right: Value,
    loc: AstLoc,
) -> Result<Value, (AstLoc, String)> {
    match &op.node {
        // TODO: implicit conversion from int to float
        BinOper::Add => bin_add(left, right, loc),
        BinOper::Sub => bin_sub(left, right, loc),
        BinOper::Mult => bin_mult(left, right, loc),
        BinOper::Div => bin_div(left, right, loc),
        BinOper::And => Ok(Value::Bool(left.truthy() && right.truthy())),
        BinOper::Or => Ok(Value::Bool(left.truthy() || right.truthy())),
        BinOper::Eq => Ok(Value::Bool(left == right)),
        BinOper::Neq => Ok(Value::Bool(left != right)),
        BinOper::Lt => Ok(Value::Bool(left < right)), // TODO Maybe shouldnt allow compraisions between types?
        BinOper::Leq => Ok(Value::Bool(left <= right)), // TODO Maybe shouldnt allow compraisions between types?
        BinOper::Gt => Ok(Value::Bool(left > right)), // TODO Maybe shouldnt allow compraisions between types?
        BinOper::Geq => Ok(Value::Bool(left >= right)), // TODO Maybe shouldnt allow compraisions between types?
    }
}

fn bin_add(left: Value, right: Value, loc: AstLoc) -> Result<Value, (AstLoc, String)> {
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

fn bin_sub(left: Value, right: Value, loc: AstLoc) -> Result<Value, (AstLoc, String)> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
        _other => Err((loc, "Subtraction operands must be two numbers".to_string())),
    }
}

fn bin_mult(left: Value, right: Value, loc: AstLoc) -> Result<Value, (AstLoc, String)> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
        _other => Err((
            loc,
            "Multiplication operands must be two numbers".to_string(),
        )),
    }
}

fn bin_div(left: Value, right: Value, loc: AstLoc) -> Result<Value, (AstLoc, String)> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x / y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x / y)),
        _other => Err((loc, "Division operands must be two numbers".to_string())),
    }
}

fn eval_unary(op: &UnOperNode, right: Value, loc: AstLoc) -> Result<Value, (AstLoc, String)> {
    match op.node {
        UnOper::Sub => match right {
            Value::Int(int) => Ok(Value::Int(-int)),
            Value::Float(float) => Ok(Value::Float(-float)),
            _other => Err((loc, "Unary subtraction only works for a number".to_string())),
        },
        UnOper::Not => Ok(Value::Bool(!right.truthy())),
    }
}
