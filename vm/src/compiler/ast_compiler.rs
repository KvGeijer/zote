use parser::{BinOper, CodeRange, Expr, ExprNode, Stmt, StmtNode, UnOper};

use super::{Chunk, OpCode};
use crate::value::Value;

pub fn compile_statement(statement: &StmtNode, chunk: &mut Chunk) -> Option<()> {
    let StmtNode {
        node,
        start_loc: _,
        end_loc: _,
    } = statement;
    // let range = CodeRange::from_locs(*start_loc, *end_loc);

    match node.as_ref() {
        Stmt::Decl(_, _) => todo!(),
        Stmt::Expr(expr) => compile_expression(expr, chunk)?,
        Stmt::Invalid => todo!(),
    }

    Some(())
}

// TODO: Variable resolution
fn compile_expression(expr: &ExprNode, chunk: &mut Chunk) -> Option<()> {
    let ExprNode {
        node,
        start_loc,
        end_loc,
    } = expr;
    let range = CodeRange::from_locs(*start_loc, *end_loc);

    match node.as_ref() {
        Expr::Call(_, _) => todo!(),
        Expr::IndexInto(_, _) => todo!(),
        Expr::Binary(x, binop, y) => {
            compile_expression(x, chunk)?;
            compile_expression(y, chunk)?;
            let opcode = binop_opcode_conv(binop);
            chunk.push_opcode(opcode, range);
        }
        Expr::Unary(unop, x) => {
            compile_expression(x, chunk)?;
            let opcode = unop_opcode_conv(unop);
            chunk.push_opcode(opcode, range);
        }
        Expr::Logical(_, _, _) => todo!(),
        Expr::Assign(_, _) => todo!(),
        Expr::Var(_) => todo!(),
        Expr::Int(x) => chunk.push_constant_plus(Value::Int(*x), range),
        Expr::Float(x) => chunk.push_constant_plus(Value::Float(*x), range),
        Expr::Bool(x) => chunk.push_constant_plus(Value::Bool(*x), range),
        Expr::String(_) => todo!(),
        Expr::Block(_) => todo!(),
        Expr::If(_, _, _) => todo!(),
        Expr::While(_, _) => todo!(),
        Expr::For(_, _, _) => todo!(),
        Expr::Break => todo!(),
        Expr::Continue => todo!(),
        Expr::Return(opt_expr) => {
            if let Some(expr) = opt_expr {
                compile_expression(expr, chunk)?;
            } else {
                // TODO: How do we handle this? Two different return types? Or just use registers...
                chunk.push_constant_plus(Value::Nil, range.clone());
            }
            chunk.push_opcode(OpCode::Return, range);
        }
        Expr::Nil => chunk.push_constant_plus(Value::Nil, range),
        Expr::List(_) => todo!(),
        Expr::Tuple(_) => todo!(),
        Expr::FunctionDefinition(_, _, _) => todo!(),
        Expr::Match(_, _) => todo!(),
    };

    Some(())
}

fn binop_opcode_conv(binop: &BinOper) -> OpCode {
    match binop {
        BinOper::Add => OpCode::Add,
        BinOper::Sub => OpCode::Subtract,
        BinOper::Div => OpCode::Divide,
        BinOper::Mult => OpCode::Multiply,
        BinOper::Mod => OpCode::Modulo,
        BinOper::Pow => OpCode::Power,
        BinOper::Eq => OpCode::Equality,
        BinOper::Neq => OpCode::NonEquality,
        BinOper::Lt => OpCode::LessThan,
        BinOper::Leq => OpCode::LessEqual,
        BinOper::Gt => OpCode::GreaterThan,
        BinOper::Geq => OpCode::GreaterEqual,
        BinOper::Append => todo!(),
    }
}

fn unop_opcode_conv(unop: &UnOper) -> OpCode {
    match unop {
        UnOper::Not => OpCode::Not,
        UnOper::Sub => OpCode::Negate,
    }
}
