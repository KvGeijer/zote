use crate::parser::{AstNode, BinOper, BinOperNode, Expr, ExprNode, UnOper, UnOperNode};

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

pub fn interpret(program: &ExprNode) {
    match eval(program) {
        Ok(val) => println!("{}", val.stringify()),
        Err(reason) => println!("Error: {reason}"),
    };
}

// TODO Err handling. Want code location and maybe original string here as well...
fn eval(expr: &ExprNode) -> Result<Value, String> {
    match &expr.node {
        Expr::Call => Err("Function calls not implemented".to_string()),
        Expr::Binary(left, op, right) => eval_binary(eval(left)?, op, eval(right)?),
        Expr::Unary(op, right) => eval_unary(op, eval(right)?),
        Expr::Int(int) => Ok(Value::Int(*int)),
        Expr::Float(float) => Ok(Value::Float(*float)),
        Expr::Bool(bool) => Ok(Value::Bool(*bool)),
        Expr::String(string) => Ok(Value::String(string.clone())),
    }
}

fn eval_binary(left: Value, op: &BinOperNode, right: Value) -> Result<Value, String> {
    match &op.node {
        BinOper::Add => bin_add(left, right),
        BinOper::Sub => bin_sub(left, right),
        BinOper::Mult => bin_mult(left, right),
        BinOper::Div => bin_div(left, right),
        BinOper::And => Ok(Value::Bool(left.truthy() && right.truthy())),
        BinOper::Or => Ok(Value::Bool(left.truthy() || right.truthy())),
        BinOper::Eq => Ok(Value::Bool(left == right)),
        BinOper::Neq => Ok(Value::Bool(left != right)),
        BinOper::Lt => Ok(Value::Bool(left < right)),
        BinOper::Leq => Ok(Value::Bool(left <= right)),
        BinOper::Gt => Ok(Value::Bool(left > right)),
        BinOper::Geq => Ok(Value::Bool(left >= right)),
    }
}

fn bin_add(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
        (Value::String(x), Value::String(y)) => Ok(Value::String(x + &y)),
        _other => Err("Addition operands must be two numbers or two strings".to_string()),
    }
}

fn bin_sub(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
        _other => Err("Subtraction operands must be two numbers".to_string()),
    }
}

fn bin_mult(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
        _other => Err("Multiplication operands must be two numbers".to_string()),
    }
}

fn bin_div(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x / y)),
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x / y)),
        _other => Err("Division operands must be two numbers".to_string()),
    }
}

fn eval_unary(op: &UnOperNode, right: Value) -> Result<Value, String> {
    match op.node {
        UnOper::Sub => match right {
            Value::Int(int) => Ok(Value::Int(-int)),
            Value::Float(float) => Ok(Value::Float(-float)),
            _other => Err("Unary subtraction only works for a number".to_string()), // TODO use the ast loc to get a good error message!
        },
        UnOper::Not => Ok(Value::Bool(!right.truthy())),
    }
}
