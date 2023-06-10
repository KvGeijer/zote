use either::Either;

use super::{expressions::MAX_ARGS, AstNode, Expr, ExprNode, LValue, Parser};
use crate::scanner::Token;

pub type StmtNode = AstNode<Stmt>;

#[derive(Debug, PartialEq, Clone)]
pub struct Stmts {
    pub stmts: Vec<StmtNode>,
    pub output: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Decl(LValue, Option<ExprNode>),
    Expr(ExprNode),
    Invalid,
}

impl<'a> Parser<'a> {
    // Parses a number of statements, terminated by a certain token
    pub fn statements(&mut self, terminator: Token) -> Result<Stmts, Stmts> {
        let mut stmts = Stmts {
            stmts: Vec::new(),
            output: false,
        };

        while self.peek() != &terminator && !self.at_end() {
            match self.statement(true) {
                Either::Left(stmt) => stmts.stmts.push(stmt),
                Either::Right(expr) => {
                    let start = expr.start_loc;
                    let end = expr.end_loc;
                    stmts
                        .stmts
                        .push(StmtNode::new(Stmt::Expr(expr), start, end));
                    stmts.output = true;
                    if self.peek() != &terminator {
                        self.error(&format!("Expect {:?} to terminate statements", terminator));
                        return Err(stmts);
                    }
                }
            }
        }

        if stmts
            .stmts
            .iter()
            .any(|stmt| stmt.node.as_ref() == &Stmt::Invalid)
        {
            Err(stmts) // In case we still want to be able so see the ast
        } else {
            Ok(stmts)
        }
    }

    // If allow_expr is on, it will match an expression instead of causing error if there is no closing ;
    fn statement(&mut self, allow_expr: bool) -> Either<StmtNode, ExprNode> {
        if let Some(node) = self.top_statement(allow_expr) {
            node
        } else {
            // Should we propagate a result to here instead?
            self.synchronize_error();
            Either::Left(StmtNode::new(
                Stmt::Invalid,
                *self.peek_start_loc(),
                *self.peek_start_loc(),
            ))
        }
    }

    fn top_statement(&mut self, allow_expr: bool) -> Option<Either<StmtNode, ExprNode>> {
        // Statements and decl statements
        match self.peek() {
            Token::Var => Some(Either::Left(self.decl_stmt()?)),
            Token::Fn => Some(Either::Left(self.fn_decl_stmt()?)),
            _ => self.expr_stmt(allow_expr),
        }
    }

    fn fn_decl_stmt(&mut self) -> Option<StmtNode> {
        // "fn" var "(" parameters? ")" "->" expression ;
        let start = *self.peek_start_loc();
        self.accept(Token::Fn, "Internal fn_decl_stmt error")?;
        if let Token::Identifier(name) = self.peek() {
            let name = name.to_string();
            self.take();
            self.accept(Token::LPar, "Expect '(' before function parameters")?;
            let params = self.parameter_list()?;
            self.accept(Token::RPar, "Expect ')' after function parameters")?;
            self.accept(Token::RArrow, "Expect '->' before function body")?;
            let body = self.expression()?;
            let end = body.end_loc;

            // TODO Do we want to change this?
            self.accept(
                Token::Semicolon,
                "Function decl statement must end with ';'",
            )?;
            let id = format!("fn {name}/{}", params.len());
            let func = ExprNode::new(
                super::Expr::FunctionDefinition(id, params, body),
                start,
                end,
            );

            Some(StmtNode::new(
                Stmt::Decl(LValue::Var(name), Some(func)),
                start,
                end,
            ))
        } else {
            self.error("Expect function name after fn");
            None
        }
    }

    fn parameter_list(&mut self) -> Option<Vec<LValue>> {
        if self.peek() != &Token::RPar {
            let mut params = vec![];
            let mut first = true;
            while first || self.match_token(Token::Comma) {
                first = false;

                let param = self.lvalue(true)?;
                params.push(param);
            }
            if params.len() >= MAX_ARGS {
                self.error("Cannot have more than {MAX_ARGS} parameters");
            }
            Some(params)
        } else {
            Some(vec![])
        }
    }

    fn decl_stmt(&mut self) -> Option<StmtNode> {
        // TODO Add declaring multiple in a row and/or tuple based init
        // varDecl        → "var" expression ";" ;
        let start = *self.peek_start_loc();
        self.accept(Token::Var, "Expect 'var' at start of declaration");
        let expr = self.expression()?; // We can't separate lvalues and assignmen here :/
        let end = *self.peek_end_loc();
        if let Expr::Assign(lvalue, rvalue) = *expr.node {
            self.accept(Token::Semicolon, "Decl statement must end with ';'")?;
            Some(StmtNode::new(Stmt::Decl(lvalue, Some(rvalue)), start, end))
        } else {
            let lvalue = self.expr_to_lvalue(expr, true)?;
            self.accept(Token::Semicolon, "Decl statement must end with ';'")?;
            Some(StmtNode::new(Stmt::Decl(lvalue, None), start, end))
        }
    }

    fn expr_stmt(&mut self, allow_expr: bool) -> Option<Either<StmtNode, ExprNode>> {
        // expr_stmt      → expression ">>:" IDENTIFIER | epression | expression ";"

        let expr = self.expression()?;
        let start = expr.start_loc;

        // This first case is to desugar >>: to a declaration statement, could be combined in some way
        if self.match_token(Token::PipeColon) {
            let lvalue = self.lvalue(true)?;
            let end = *self.peek_end_loc();
            self.accept(Token::Semicolon, "Expect ';' after expression statement")?;
            let decl_stmt = StmtNode::new(Stmt::Decl(lvalue, Some(expr)), start, end);
            Some(Either::Left(decl_stmt))
        } else if !allow_expr || self.peek() == &Token::Semicolon {
            let end = *self.peek_end_loc();
            self.accept(Token::Semicolon, "Expect ';' after expression statement")?;
            Some(Either::Left(StmtNode::new(Stmt::Expr(expr), start, end)))
        } else {
            Some(Either::Right(expr))
        }
    }
}
