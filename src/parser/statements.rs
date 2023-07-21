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
                Either::Left(stmt) => {
                    stmts.stmts.push(stmt);
                    stmts.output = false;
                }
                Either::Right(expr) => {
                    let start = expr.start_loc;
                    let end = expr.end_loc;
                    stmts
                        .stmts
                        .push(StmtNode::new(Stmt::Expr(expr), start, end));
                    stmts.output = true;
                    // if self.peek() == &terminator {
                    //     // self.error(&format!("Expect {:?} to terminate statements", terminator));
                    //     // return Err(stmts);
                    // }
                }
            }
        }

        // TODO: Remove?
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
        if let Some(node) = self.fn_statement(allow_expr) {
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

    fn fn_statement(&mut self, allow_expr: bool) -> Option<Either<StmtNode, ExprNode>> {
        // decl_stmt | "fn" var "(" parameters? ")" "->" expression ;
        let start = *self.peek_start_loc();
        if !self.match_token(Token::Fn) {
            self.rest_stmt(allow_expr)
        } else if let Token::Identifier(name) = self.peek() {
            let name = name.to_string();
            self.take();
            self.accept(Token::LPar, "Expect '(' before function parameters")?;
            let params = self.parameter_list()?;
            self.accept(Token::RPar, "Expect ')' after function parameters")?;
            self.accept(Token::RArrow, "Expect '->' before function body")?;
            let body = self.expression()?;
            let end = body.end_loc;

            if !self.match_token(Token::Semicolon) && !semicolon_elision(&body) {
                // A ; was expected, but not found
                self.error("Function decl statement with singleton expression must end with ';'");
                return None;
            }
            let id = format!("fn {name}/{}", params.len());
            let func = ExprNode::new(
                super::Expr::FunctionDefinition(id, params, body),
                start,
                end,
            );

            Some(Either::Left(StmtNode::new(
                Stmt::Decl(LValue::Var(name), Some(func)),
                start,
                end,
            )))
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

    fn rest_stmt(&mut self, allow_expr: bool) -> Option<Either<StmtNode, ExprNode>> {
        // varDecl        → (expression | lvalue ":=" expression | expression ":>>" lvalue) ";" ;
        let start = *self.peek_start_loc();
        let expr = self.expression()?; // We can't separate lvalues and assignmen here :/
        if self.match_token(Token::ColonEq) {
            let lvalue = self.expr_to_lvalue(expr, true)?;
            let rvalue = self.expression()?;
            let end = *self.peek_last_end_loc().unwrap();
            self.accept(Token::Semicolon, "Decl statement must end with ';'")?;
            Some(Either::Left(StmtNode::new(
                Stmt::Decl(lvalue, Some(rvalue)),
                start,
                end,
            )))
        } else if self.match_token(Token::ColonPipe) {
            let lvalue = self.lvalue(true)?;
            let end = *self.peek_end_loc();
            self.accept(Token::Semicolon, "Expect ';' after pipe decl statement")?;
            Some(Either::Left(StmtNode::new(
                Stmt::Decl(lvalue, Some(expr)),
                start,
                end,
            )))
        } else if self.match_token(Token::Semicolon) {
            let end = *self.peek_last_end_loc().unwrap();
            Some(Either::Left(StmtNode::new(Stmt::Expr(expr), start, end)))
        } else if !allow_expr && !semicolon_elision(&expr) {
            // A ; was expected, but not found
            self.error("Expect ';' after expression statement");
            None
        } else {
            Some(Either::Right(expr))
        }
    }
}

fn semicolon_elision(expr: &ExprNode) -> bool {
    match expr.node.as_ref() {
        Expr::While(_, block)
        | Expr::For(_, _, block)
        | Expr::If(_, block, None)
        | Expr::If(_, _, Some(block)) => matches!(block.node.as_ref(), Expr::Block(_)),
        Expr::Match(_, _) | Expr::Block(_) => true,
        _ => false,
    }
}
