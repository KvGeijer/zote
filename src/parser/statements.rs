use either::Either;

use super::{expressions::MAX_ARGS, AstNode, ExprNode, LValue, Parser};
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
                    let start = expr.start_loc.clone();
                    let end = expr.end_loc.clone();
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
                self.peek_start_loc().clone(),
                self.peek_start_loc().clone(),
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
        let start = self.peek_start_loc().clone();
        self.accept(Token::Fn, "Internal fn_decl_stmt error")?;
        if let Token::Identifier(name) = self.peek() {
            let name = name.to_string();
            self.take();
            self.accept(Token::LPar, "Expect '(' before function parameters")?;
            let params = self.parameter_list()?;
            self.accept(Token::RPar, "Expect ')' after function parameters")?;
            self.accept(Token::RArrow, "Expect '->' before function body")?;
            let body = self.expression()?;
            let end = body.end_loc.clone();

            // TODO Do we want to change this?
            self.accept(
                Token::Semicolon,
                "Function decl statement must end with ';'",
            )?;
            let id = format!("fn {name}/{}", params.len());
            let func = ExprNode::new(
                super::Expr::FunctionDefinition(id, params, body),
                start.clone(),
                end.clone(),
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

    fn parameter_list(&mut self) -> Option<Vec<String>> {
        if self.peek() != &Token::RPar {
            let mut params = vec![];
            let mut first = true;
            while first || self.peek() == &Token::Comma {
                if !first {
                    self.accept(Token::Comma, "Internal error in paramater_list");
                } else {
                    first = false
                }
                if let Token::Identifier(param) = self.peek() {
                    let param = param.to_string();
                    self.take();
                    params.push(param); // OPT Should take string and not copy.
                } else {
                    self.error("Expected parameter after ',' in parameter list");
                    return None;
                }
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
        let start = self.peek_start_loc().clone();
        self.accept(Token::Var, "Expect 'var' at start of declaration");
        let box node = self.expression()?.node; // We can't separate lvalues and assignmen here :/
        let end = self.peek_end_loc().clone();
        match node {
            super::Expr::Assign(lvalue, rvalue) => {
                self.accept(Token::Semicolon, "Decl statement must end with ';'")?;
                Some(StmtNode::new(Stmt::Decl(lvalue, Some(rvalue)), start, end))
            }
            other => match other.to_lvalue() {
                Ok(lvalue) => {
                    self.accept(Token::Semicolon, "Decl statement must end with ';'")?;
                    Some(StmtNode::new(Stmt::Decl(lvalue, None), start, end))
                }
                Err(reason) => {
                    self.error(&reason);
                    None
                }
            },
        }
    }

    fn expr_stmt(&mut self, allow_expr: bool) -> Option<Either<StmtNode, ExprNode>> {
        // expr_stmt      → expression ">>:" IDENTIFIER | epression | expression ";"

        let expr = self.expression()?;
        let start = expr.start_loc.clone();

        // This first case is to desugar >>: to a declaration statement, could be combined in some way
        if self.match_token(Token::PipeColon) {
            match self.expression()?.node.to_lvalue() {
                Ok(lvalue) => {
                    let end = self.peek_end_loc().clone();
                    self.accept(Token::Semicolon, "Expect ';' after expression statement")?;
                    let decl_stmt = StmtNode::new(Stmt::Decl(lvalue, Some(expr)), start, end);
                    Some(Either::Left(decl_stmt))
                }
                Err(reason) => {
                    self.error(&reason);
                    None
                }
            }
        } else if !allow_expr || self.peek() == &Token::Semicolon {
            let end = self.peek_end_loc().clone();
            self.accept(Token::Semicolon, "Expect ';' after expression statement")?;
            Some(Either::Left(StmtNode::new(Stmt::Expr(expr), start, end)))
        } else {
            Some(Either::Right(expr))
        }
    }
}
