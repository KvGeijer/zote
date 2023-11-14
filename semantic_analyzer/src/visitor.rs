use std::rc::Rc;

use parser::{
    BinOper, Expr, ExprNode, Index, LValue, ListContent, LogicalOper, Slice, StmtNode, Stmts,
    UnOper,
};

pub trait AstVisitor {
    fn visit_stmts(&mut self, stmts: &Stmts) {
        for stmt in stmts.stmts.iter() {
            self.visit_stmt(stmt)
        }
    }

    fn visit_stmt(&mut self, stmt: &StmtNode) {
        match stmt.node.as_ref() {
            parser::Stmt::Decl(lvalue, init) => self.visit_decl(lvalue, init.as_ref()),
            parser::Stmt::Expr(expr) => self.visit_expr(expr),
            parser::Stmt::Invalid => println!("WARNING: Visiting invalid AST node"),
        }
    }

    fn visit_decl(&mut self, lvalue: &LValue, init: Option<&ExprNode>) {
        self.visit_lvalue(lvalue, true);
        if let Some(expr) = init {
            self.visit_expr(expr)
        }
    }

    fn visit_expr_delegation(&mut self, expr: &ExprNode) {
        match expr.node.as_ref() {
            Expr::Call(callee, args) => self.visit_call(callee, args),
            Expr::IndexInto(indexee, at) => self.visit_index_into(indexee, at),
            Expr::Binary(x, op, y) => self.visit_binary(x, op, y),
            Expr::Unary(op, x) => self.visit_unary(op, x),
            Expr::Logical(x, op, y) => self.visit_logical(x, op, y),
            Expr::Assign(lvalue, value) => self.visit_assign(lvalue, value),
            Expr::Var(name) => self.visit_var(name, false),
            Expr::Int(int) => self.visit_int(*int),
            Expr::Float(float) => self.visit_float(*float),
            Expr::Bool(bool) => self.visit_bool(*bool),
            Expr::String(string) => self.visit_string(string),
            Expr::Block(stmts) => self.visit_block(stmts),
            Expr::If(cond, then, otherwise) => self.visit_if(cond, then, otherwise.as_ref()),
            Expr::While(cond, body) => self.visit_while(cond, body),
            Expr::For(lvalue, collection, body) => self.visit_for(lvalue, collection, body),
            Expr::Break => self.visit_break(),
            Expr::Continue => self.visit_continue(),
            Expr::Return(ret) => self.visit_return(ret.as_ref()),
            Expr::Nil => self.visit_nil(),
            Expr::List(content) => self.visit_list(content),
            Expr::Tuple(exprs) => self.visit_tuple(exprs),
            Expr::FunctionDefinition(name, params, body) => {
                self.visit_function_definition(name, params, body)
            }
            Expr::Match(matched, options) => self.visit_match(matched, options),
        }
    }

    fn visit_expr(&mut self, expr: &ExprNode) {
        self.visit_expr_delegation(expr)
    }

    fn visit_call(&mut self, callee: &ExprNode, args: &[ExprNode]) {
        self.visit_expr(callee);

        for arg in args {
            self.visit_expr(arg)
        }
    }

    fn visit_index_into(&mut self, indexee: &ExprNode, at: &Index) {
        self.visit_expr(indexee);
        self.visit_index(at);
    }

    fn visit_binary(&mut self, x: &ExprNode, op: &BinOper, y: &ExprNode) {
        self.visit_expr(x);
        self.visit_binary_oper(op);
        self.visit_expr(y);
    }

    fn visit_unary(&mut self, op: &UnOper, x: &ExprNode) {
        self.visit_unary_oper(op);
        self.visit_expr(x);
    }

    fn visit_logical(&mut self, x: &ExprNode, op: &LogicalOper, y: &ExprNode) {
        self.visit_expr(x);
        self.visit_logical_oper(op);
        self.visit_expr(y);
    }

    fn visit_assign(&mut self, lvalue: &LValue, value: &ExprNode) {
        self.visit_lvalue(lvalue, false);
        self.visit_expr(value);
    }

    fn visit_var(&mut self, _name: &String, _declaration: bool) {} // String instead of str to be more sure of pointer magic
    fn visit_int(&mut self, _int: i64) {}
    fn visit_float(&mut self, _float: f64) {}
    fn visit_bool(&mut self, _bool: bool) {}
    fn visit_string(&mut self, _string: &Rc<String>) {}

    fn visit_block(&mut self, stmts: &Stmts) {
        self.visit_stmts(stmts)
    }

    fn visit_if(&mut self, cond: &ExprNode, then: &ExprNode, otherwise: Option<&ExprNode>) {
        self.visit_expr(cond);
        self.visit_expr(then);

        if let Some(expr) = otherwise {
            self.visit_expr(expr);
        }
    }

    fn visit_while(&mut self, cond: &ExprNode, body: &ExprNode) {
        self.visit_expr(cond);
        self.visit_expr(body);
    }

    fn visit_for(&mut self, lvalue: &LValue, collection: &ExprNode, body: &ExprNode) {
        self.visit_lvalue(lvalue, true);
        self.visit_expr(collection);
        self.visit_expr(body);
    }

    fn visit_break(&mut self) {}
    fn visit_continue(&mut self) {}

    fn visit_return(&mut self, ret: Option<&ExprNode>) {
        if let Some(expr) = ret {
            self.visit_expr(expr);
        }
    }

    fn visit_nil(&mut self) {}

    fn visit_list(&mut self, content: &ListContent) {
        match content {
            ListContent::Exprs(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr)
                }
            }
            ListContent::Range(slice) => self.visit_slice(slice),
        }
    }

    fn visit_tuple(&mut self, exprs: &[ExprNode]) {
        for expr in exprs {
            self.visit_expr(expr)
        }
    }

    fn visit_function_definition(&mut self, _name: &str, params: &[LValue], body: &ExprNode) {
        for param in params {
            self.visit_lvalue(param, true);
        }
        self.visit_expr(body);
    }

    fn visit_match(&mut self, matched: &ExprNode, options: &[(LValue, ExprNode)]) {
        self.visit_expr(matched);
        for (lvalue, then) in options {
            self.visit_lvalue(lvalue, true);
            self.visit_expr(then);
        }
    }

    fn visit_index(&mut self, at: &Index) {
        match at {
            Index::At(expr) => self.visit_expr(expr),
            Index::Slice(slice) => self.visit_slice(slice),
        }
    }

    fn visit_binary_oper(&mut self, _op: &BinOper) {}
    fn visit_unary_oper(&mut self, _op: &UnOper) {}
    fn visit_logical_oper(&mut self, _op: &LogicalOper) {}

    fn visit_lvalue(&mut self, lvalue: &LValue, declaration: bool) {
        match lvalue {
            LValue::Index(indexee, at) => self.visit_index_into(indexee, at),
            LValue::Var(name) => self.visit_var(name, declaration),
            LValue::Tuple(lvalues) => {
                for lvalue in lvalues {
                    self.visit_lvalue(lvalue, declaration)
                }
            }
            LValue::Constant(expr) => self.visit_expr(expr),
        }
    }

    fn visit_slice(&mut self, _slice: &Slice) {}
}
