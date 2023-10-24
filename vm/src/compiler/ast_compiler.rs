use parser::{BinOper, CodeRange, Expr, ExprNode, LValue, Stmt, StmtNode, Stmts, UnOper};

use super::{Chunk, Compiler, OpCode};
use crate::value::Value;

mod control_flow;

type CompRes<T> = Result<T, String>;

impl Compiler {
    pub fn compile_statement(&mut self, statement: &StmtNode, chunk: &mut Chunk, output: bool) {
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
            Stmt::Expr(expr) => {
                let res = self.compile_expression(expr, chunk);
                if !output {
                    // Keep the value if the statement should keep output
                    chunk.push_opcode(OpCode::Discard, range.clone());
                }
                res
            }
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
    ) -> CompRes<()> {
        // TODO: Pattern matching more
        match lvalue {
            LValue::Index(_, _) => todo!(),
            LValue::Var(name) => {
                // Important to compile RHS before declaring the variable
                self.compile_opt_expression(expr.as_ref(), chunk)?;
                if !self.locals.is_global() {
                    self.locals.add_local(name.to_string())
                }
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
            Expr::Call(func, args) => self.compile_call(func, args, range, chunk),
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
            Expr::Block(stmts) => self.compile_block(stmts, range, chunk),
            Expr::If(pred, then, otherwise) => {
                self.compile_if(pred, then, otherwise.as_ref(), range, chunk)?
            }
            Expr::While(_, _) => todo!(),
            Expr::For(_, _, _) => todo!(),
            Expr::Break => todo!(),
            Expr::Continue => todo!(),
            Expr::Return(opt_expr) => {
                self.compile_opt_expression(opt_expr.as_ref(), chunk)?;
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

    /// Assigns the topmost temp value to the named variable
    fn compile_assign(
        &mut self,
        name: &str,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> Result<(), String> {
        // First checks if it is local
        if let Some(offset) = self.locals.get(name) {
            chunk.push_opcode(OpCode::AssignLocal, range);
            chunk.push_u8_offset(offset);
            Ok(())
        } else if let Some(&offset) = self.globals.get(name) {
            chunk.push_opcode(OpCode::AssignGlobal, range); // Maybe bad range choice
            chunk.push_u8_offset(offset as u8);
            Ok(())
        } else {
            Err(format!("Global var '{name}' is not declared"))
        }
    }

    /// Compiles the expression, or just push Nil (without location) if no expression
    fn compile_opt_expression(
        &mut self,
        expr: Option<&ExprNode>,
        chunk: &mut Chunk,
    ) -> Result<(), String> {
        // Maybe we should be smarter and never push such a Nil value
        match expr {
            Some(expr) => self.compile_expression(expr, chunk)?,
            None => chunk.push_constant_plus(Value::Nil, CodeRange::from_ints(0, 0, 0, 0, 0, 0)),
        };
        Ok(())
    }

    /// Compiles the read of a var.
    fn compile_var(
        &mut self,
        name: &str,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> Result<(), String> {
        if let Some(offset) = self.locals.get(name) {
            chunk.push_opcode(OpCode::ReadLocal, range);
            chunk.push_u8_offset(offset);
            Ok(())
        } else if let Some(offset) = self.globals.get(name) {
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

    /// Compiles the block of statements. Does not throw, as errors are printed and escaped within.
    fn compile_block(&mut self, stmts: &Stmts, range: CodeRange, chunk: &mut Chunk) {
        self.locals.enter();
        self.compile_stmts(stmts, chunk);
        if !stmts.output || stmts.stmts.is_empty() {
            chunk.push_opcode(OpCode::Nil, range);
        }
        self.locals.exit();
    }

    /// So far can only handle hard-coded print calls
    fn compile_call(
        &mut self,
        func: &ExprNode,
        args: &[ExprNode],
        range: CodeRange,
        chunk: &mut Chunk,
    ) {
        if let Expr::Var(name) = func.node.as_ref() {
            if name == "print" {
                // TODO: Fix
                let _ = self.compile_expression(&args[0], chunk);
                chunk.push_opcode(OpCode::Print, range);
                return;
            }
        }
        todo!("Only print calls implemented yet")
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
