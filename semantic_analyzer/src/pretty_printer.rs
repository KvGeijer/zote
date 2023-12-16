use parser::Stmts;

use crate::visitor::AstVisitor;

pub fn format(ast: &Stmts) -> String {
    let mut formatter = Prettifier {
        indent: 0,
        builder: String::new(),
    };

    formatter.visit_stmts(ast);

    formatter.builder
}

struct Prettifier {
    builder: String,
    indent: usize,
}

impl Prettifier {
    fn app<S: AsRef<str>>(&mut self, end: S) {
        self.builder.push_str(end.as_ref())
    }

    fn inc(&mut self) {
        self.indent += 1;
    }

    fn dec(&mut self) {
        self.indent -= 1;
    }
}

impl AstVisitor for Prettifier {
    fn visit_stmt(&mut self, stmt: &parser::StmtNode) {
        match stmt.node.as_ref() {
            parser::Stmt::Decl(lvalue, init) => self.visit_decl(lvalue, init.as_ref()),
            parser::Stmt::Expr(expr) => self.visit_expr(expr),
            parser::Stmt::Invalid => println!("WARNING: Visiting invalid AST node"),
        }
        self.app(";\n");

        if let parser::Stmt::Decl(_, Some(expr)) = stmt.node.as_ref() {
            if matches!(
                expr.node.as_ref(),
                parser::Expr::FunctionDefinition(_, _, _)
            ) {
                self.app("\n")
            }
        }
    }

    fn visit_decl(&mut self, lvalue: &parser::LValue, init: Option<&parser::ExprNode>) {
        self.visit_lvalue(lvalue, true);

        self.app(" := ");

        if let Some(expr) = init {
            self.visit_expr(expr)
        }
    }

    fn visit_expr_delegation(&mut self, expr: &parser::ExprNode) {
        match expr.node.as_ref() {
            parser::Expr::Call(callee, args) => self.visit_call(callee, args),
            parser::Expr::IndexInto(indexee, at) => self.visit_index_into(indexee, at),
            parser::Expr::Binary(x, op, y) => self.visit_binary(x, op, y),
            parser::Expr::Unary(op, x) => self.visit_unary(op, x),
            parser::Expr::Logical(x, op, y) => self.visit_logical(x, op, y),
            parser::Expr::Assign(lvalue, value) => self.visit_assign(lvalue, value),
            parser::Expr::Var(name) => self.visit_var(name, false),
            parser::Expr::Int(int) => self.visit_int(*int),
            parser::Expr::Float(float) => self.visit_float(*float),
            parser::Expr::Bool(bool) => self.visit_bool(*bool),
            parser::Expr::String(string) => self.visit_string(string),
            parser::Expr::Block(stmts) => self.visit_block(stmts),
            parser::Expr::If(cond, then, otherwise) => {
                self.visit_if(cond, then, otherwise.as_ref())
            }
            parser::Expr::While(cond, body) => self.visit_while(cond, body),
            parser::Expr::For(lvalue, collection, body) => self.visit_for(lvalue, collection, body),
            parser::Expr::Break => self.visit_break(),
            parser::Expr::Continue => self.visit_continue(),
            parser::Expr::Return(ret) => self.visit_return(ret.as_ref()),
            parser::Expr::Nil => self.visit_nil(),
            parser::Expr::List(content) => self.visit_list(content),
            parser::Expr::Tuple(exprs) => self.visit_tuple(exprs),
            parser::Expr::FunctionDefinition(name, params, body) => {
                self.visit_function_definition(name, params, body)
            }
            parser::Expr::Match(matched, options) => self.visit_match(matched, options),
        }
    }

    fn visit_expr(&mut self, expr: &parser::ExprNode) {
        self.visit_expr_delegation(expr)
    }

    fn visit_call(&mut self, callee: &parser::ExprNode, args: &[parser::ExprNode]) {
        self.visit_expr(callee);

        self.app("(");

        let mut first = true;
        for arg in args {
            if !first {
                self.app(", ");
            } else {
                first = false;
            }
            self.visit_expr(arg)
        }

        self.app(")");
    }

    fn visit_index_into(&mut self, indexee: &parser::ExprNode, at: &parser::Index) {
        self.visit_expr(indexee);
        self.app("[");
        self.visit_index(at);
        self.app("]");
    }

    fn visit_binary(&mut self, x: &parser::ExprNode, op: &parser::BinOper, y: &parser::ExprNode) {
        self.app("(");
        self.visit_expr(x);
        self.visit_binary_oper(op);
        self.visit_expr(y);
        self.app(")");
    }

    fn visit_unary(&mut self, op: &parser::UnOper, x: &parser::ExprNode) {
        self.app("(");
        self.visit_unary_oper(op);
        self.visit_expr(x);
        self.app(")");
    }

    fn visit_logical(
        &mut self,
        x: &parser::ExprNode,
        op: &parser::LogicalOper,
        y: &parser::ExprNode,
    ) {
        self.visit_expr(x);
        self.visit_logical_oper(op);
        self.visit_expr(y);
    }

    fn visit_assign(&mut self, lvalue: &parser::LValue, value: &parser::ExprNode) {
        self.visit_lvalue(lvalue, false);

        self.app(" = ");

        self.visit_expr(value);
    }

    fn visit_var(&mut self, name: &String, _declaration: bool) {
        self.app(name)
    }

    fn visit_int(&mut self, int: i64) {
        self.app(int.to_string())
    }

    fn visit_float(&mut self, float: f64) {
        self.app(float.to_string())
    }

    fn visit_bool(&mut self, bool: bool) {
        self.app(bool.to_string())
    }

    fn visit_string(&mut self, string: &std::rc::Rc<String>) {
        self.app(format!("{:?}", string.as_ref()));
    }

    fn visit_block(&mut self, stmts: &Stmts) {
        self.app("{\n");
        self.inc();
        self.visit_stmts(stmts);
        self.dec();
        for _ in 0..self.indent {
            self.app("    ");
        }
        self.app("}");
    }

    fn visit_if(
        &mut self,
        cond: &parser::ExprNode,
        then: &parser::ExprNode,
        otherwise: Option<&parser::ExprNode>,
    ) {
        self.app("if ");
        self.visit_expr(cond);
        self.app(" ");
        self.visit_expr(then);

        if let Some(expr) = otherwise {
            self.app(" else ");
            self.visit_expr(expr);
        }
    }

    fn visit_while(&mut self, cond: &parser::ExprNode, body: &parser::ExprNode) {
        self.app("while ");
        self.visit_expr(cond);
        self.app(" ");
        self.visit_expr(body);
    }

    fn visit_for(
        &mut self,
        lvalue: &parser::LValue,
        collection: &parser::ExprNode,
        body: &parser::ExprNode,
    ) {
        self.app("for ");
        self.visit_lvalue(lvalue, true);
        self.app(" in ");
        self.visit_expr(collection);
        self.app(" ");
        self.visit_expr(body);
    }

    fn visit_break(&mut self) {
        self.app("break");
    }

    fn visit_continue(&mut self) {
        self.app("continue");
    }

    fn visit_return(&mut self, ret: Option<&parser::ExprNode>) {
        self.app("return ");
        if let Some(expr) = ret {
            self.visit_expr(expr);
        }
    }

    fn visit_nil(&mut self) {
        self.app("nil");
    }

    fn visit_list(&mut self, content: &parser::ListContent) {
        self.app("[");
        match content {
            parser::ListContent::Exprs(exprs) => {
                for (i, expr) in exprs.iter().enumerate() {
                    if i != 0 {
                        self.app(", ");
                    }
                    self.visit_expr(expr)
                }
            }
            parser::ListContent::Range(slice) => self.visit_slice(slice),
        }
        self.app("]");
    }

    fn visit_tuple(&mut self, exprs: &[parser::ExprNode]) {
        self.app("(");
        for (i, expr) in exprs.iter().enumerate() {
            if i != 0 {
                self.app(", ");
            }
            self.visit_expr(expr)
        }
        self.app(")");
    }

    fn visit_function_definition(
        &mut self,
        _name: &str,
        params: &[parser::LValue],
        body: &parser::ExprNode,
    ) {
        self.app("\\");

        for (i, param) in params.iter().enumerate() {
            if i != 0 {
                self.app(", ")
            }
            self.visit_lvalue(param, true);
        }
        self.app(" -> ");
        self.visit_expr(body);
    }

    fn visit_match(
        &mut self,
        matched: &parser::ExprNode,
        options: &[(parser::LValue, parser::ExprNode)],
    ) {
        self.app("match ");
        self.visit_expr(matched);
        self.app("{\n");
        self.inc();
        for (lvalue, then) in options {
            for _ in 0..self.indent {
                self.app("    ");
            }
            self.visit_lvalue(lvalue, true);
            self.visit_expr(then);

            self.app(",\n")
        }
        self.dec();
        for _ in 0..self.indent {
            self.app("    ");
        }
    }

    fn visit_index(&mut self, at: &parser::Index) {
        match at {
            parser::Index::At(expr) => self.visit_expr(expr),
            parser::Index::Slice(slice) => self.visit_slice(slice),
        }
    }

    fn visit_binary_oper(&mut self, op: &parser::BinOper) {
        match op {
            parser::BinOper::Add => self.app(" + "),
            parser::BinOper::Sub => self.app(" - "),
            parser::BinOper::Div => self.app(" / "),
            parser::BinOper::Mult => self.app(" * "),
            parser::BinOper::Mod => self.app(" % "),
            parser::BinOper::Pow => self.app(" ^ "),
            parser::BinOper::Eq => self.app(" == "),
            parser::BinOper::Neq => self.app(" != "),
            parser::BinOper::Lt => self.app(" < "),
            parser::BinOper::Leq => self.app(" <= "),
            parser::BinOper::Gt => self.app(" > "),
            parser::BinOper::Geq => self.app(" >= "),
            parser::BinOper::Append => self.app(" ++ "),
        }
    }

    fn visit_unary_oper(&mut self, op: &parser::UnOper) {
        match op {
            parser::UnOper::Not => self.app("!"),
            parser::UnOper::Sub => self.app("-"),
        }
    }

    fn visit_logical_oper(&mut self, op: &parser::LogicalOper) {
        match op {
            parser::LogicalOper::And => self.app(" and "),
            parser::LogicalOper::Or => self.app(" or "),
        }
    }

    fn visit_lvalue(&mut self, lvalue: &parser::LValue, declaration: bool) {
        match lvalue {
            parser::LValue::Index(indexee, at) => self.visit_index_into(indexee, at),
            parser::LValue::Var(name) => self.visit_var(name, declaration),
            parser::LValue::Tuple(lvalues) => {
                self.app("(");
                for (i, lvalue) in lvalues.iter().enumerate() {
                    if i != 0 {
                        self.app(", ");
                    }
                    self.visit_lvalue(lvalue, declaration)
                }
                self.app(")");
            }
            parser::LValue::Constant(expr) => self.visit_expr(expr),
        }
    }

    fn visit_slice(&mut self, slice: &parser::Slice) {
        if let Some(start) = &slice.start {
            self.visit_expr(start)
        };
        self.app(":");
        if let Some(stop) = &slice.stop {
            self.visit_expr(stop)
        };
        if let Some(step) = &slice.step {
            self.app(":");
            self.visit_expr(step)
        };
    }

    fn visit_stmts(&mut self, stmts: &Stmts) {
        for stmt in stmts.stmts.iter() {
            for _ in 0..self.indent {
                self.app("    ");
            }
            self.visit_stmt(stmt);
        }
        if stmts.output {
            self.builder.pop();
            self.builder.pop();
            self.app("\n");
        }
    }
}
