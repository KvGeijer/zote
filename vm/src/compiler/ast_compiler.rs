use parser::{BinOper, CodeRange, Expr, ExprNode, LValue, Stmt, StmtNode, Stmts, UnOper};

use super::{Chunk, Compiler, OpCode};
use crate::value::Value;

impl Compiler {
    pub fn compile_statement(&mut self, statement: &StmtNode, chunk: &mut Chunk) {
        let StmtNode {
            node,
            start_loc,
            end_loc,
        } = statement;
        let range = CodeRange::from_locs(*start_loc, *end_loc);

        let res = match node.as_ref() {
            Stmt::Decl(lvalue, expr) => {
                self.compile_declaration(lvalue, expr, range.clone(), chunk)
            }
            Stmt::Expr(expr) => self.compile_expression(expr, chunk),
            Stmt::Invalid => todo!(),
        };

        if let Err(reason) = res {
            // TODO: Push some garbage opcode?
            eprintln!("COMPILER ERROR: [{range}] {reason}");
        }
    }

    fn compile_declaration(
        &mut self,
        lvalue: &LValue,
        expr: &Option<ExprNode>,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> Result<(), String> {
        // For now just declares it as a global variable
        match lvalue {
            LValue::Index(_, _) => todo!(),
            LValue::Var(name) => {
                self.compile_opt_expression(expr, chunk)?;
                // TODO: declare as local if not top-level
                self.compile_assign(name, range, chunk)?;
            }
            LValue::Tuple(_) => todo!(),
            LValue::Constant(_) => todo!(),
        };
        Ok(())
    }

    fn declare_global(&mut self, name: &str) -> usize {
        let len = self.globals.len();
        *self.globals.entry(name.to_string()).or_insert(len)
    }

    // TODO: Variable resolution
    fn compile_expression(&mut self, expr: &ExprNode, chunk: &mut Chunk) -> Result<(), String> {
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
                self.compile_expression(x, chunk)?;
                self.compile_expression(y, chunk)?;
                let opcode = binop_opcode_conv(binop);
                chunk.push_opcode(opcode, range);
            }
            Expr::Unary(unop, x) => {
                self.compile_expression(x, chunk)?;
                let opcode = unop_opcode_conv(unop);
                chunk.push_opcode(opcode, range);
            }
            Expr::Logical(_, _, _) => todo!(),
            Expr::Assign(lvalue, expr) => {
                self.compile_lvalue_assignment(lvalue, expr, range, chunk)?
            }
            Expr::Var(name) => self.compile_var(name, range, chunk)?,
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
                self.compile_opt_expression(opt_expr, chunk)?;
                chunk.push_opcode(OpCode::Return, range);
            }
            Expr::Nil => chunk.push_constant_plus(Value::Nil, range),
            Expr::List(_) => todo!(),
            Expr::Tuple(_) => todo!(),
            Expr::FunctionDefinition(_, _, _) => todo!(),
            Expr::Match(_, _) => todo!(),
        };

        Ok(())
    }

    fn compile_lvalue_assignment(
        &mut self,
        lvalue: &LValue,
        expr: &ExprNode,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> Result<(), String> {
        self.compile_expression(expr, chunk)?;
        match lvalue {
            LValue::Index(_, _) => todo!(),
            LValue::Var(name) => self.compile_assign(name, range, chunk),
            LValue::Tuple(_) => todo!(),
            LValue::Constant(_) => todo!(),
        }
    }

    fn compile_assign(
        &mut self,
        name: &str,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> Result<(), String> {
        // TODO: Check for local variable
        if let Some(&offset) = self.globals.get(name) {
            chunk.push_opcode(OpCode::AssignGlobal, range); // Maybe bad range choice
            chunk.push_u8_offset(offset as u8);
            Ok(())
        } else {
            Err(format!("Var '{name}' is not declared"))
        }
    }

    /// Compiles the expression, or just push Nil (without location) if no expression
    fn compile_opt_expression(
        &mut self,
        expr: &Option<ExprNode>,
        chunk: &mut Chunk,
    ) -> Result<(), String> {
        // Maybe we should be smarter and never push such a Nil value
        match expr {
            Some(expr) => self.compile_expression(expr, chunk)?,
            None => chunk.push_constant_plus(Value::Nil, CodeRange::from_ints(0, 0, 0, 0, 0, 0)),
        };
        Ok(())
    }

    /// Compiles the read of a var. Now only global
    fn compile_var(
        &mut self,
        name: &str,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> Result<(), String> {
        // TODO: Check if a local one exists first

        if let Some(offset) = self.globals.get(name) {
            chunk.push_opcode(OpCode::ReadGlobal, range);
            chunk.push_u8_offset(*offset as u8);
            Ok(())
        } else {
            // ERROR: Compile error!
            Err(format!("Var '{name}' is not declared"))
        }
    }

    /// Declare all top-level declarations, so as to use late-binding
    pub fn declare_globals(&mut self, stmts: &Stmts) {
        for stmt in stmts.stmts.iter() {
            if let Stmt::Decl(lvalue, _) = stmt.node.as_ref() {
                self.declare_global_lvalue(lvalue);
            }
        }
    }

    fn declare_global_lvalue(&mut self, lvalue: &LValue) {
        match lvalue {
            LValue::Index(_, _) => (),
            LValue::Var(name) => {
                self.declare_global(name);
            }
            LValue::Tuple(lvalues) => {
                for lvalue in lvalues.iter() {
                    self.declare_global_lvalue(lvalue);
                }
            }
            LValue::Constant(_) => (),
        }
    }
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
