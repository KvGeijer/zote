use either::Either;

use super::{AstNode, ExprNode, Parser};
use crate::scanner::Token;

pub type StmtNode = AstNode<Stmt>;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Decl(String, Option<ExprNode>),
    Expr(ExprNode),
    Print(ExprNode), // TODO Integrate into standard lib func
    Invalid,
}

impl<'a> Parser<'a> {
    /// Only for top level parsing of global statements
    pub fn statements(&mut self) -> Result<Vec<StmtNode>, Vec<StmtNode>> {
        // If a single expression it is wrapped in a print statement
        let mut stmts = Vec::new();
        match self.statement(true) {
            Either::Left(stmt) => stmts.push(stmt),
            Either::Right(expr) => {
                self.accept(Token::EOF, "Expect singleton expression")
                    .ok_or(stmts)?;
                let start = expr.start_loc.clone();
                let end = expr.end_loc.clone();
                let print_stmt = StmtNode::new(Stmt::Print(expr), start, end);
                return Ok(vec![print_stmt]);
            }
        }

        while !self.at_end() {
            let stmt = self
                .statement(false)
                .expect_left("Internal error: allow_expr is false, so should get a statement");
            stmts.push(stmt);
        }

        if stmts.iter().any(|stmt| stmt.node == Stmt::Invalid) {
            Err(stmts) // In case we still want to be able so see the ast
        } else {
            Ok(stmts)
        }
    }

    // Maybe not ideal to have pub, but statements can be inside blocks...
    // If allow_expr is on, it will match an expression instead of causing error if there is no closing ;
    pub fn statement(&mut self, allow_expr: bool) -> Either<StmtNode, ExprNode> {
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
            _ => self.non_decl_stmt(allow_expr),
        }
    }

    fn decl_stmt(&mut self) -> Option<StmtNode> {
        // TODO Add declaring multiple in a row and/or tuple based init
        // varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
        self.accept(Token::Var, "Expect 'var' at start of declaration");
        if let Token::Identifier(id) = self.peek().clone() {
            let start = self.peek_start_loc().clone();
            self.take();
            let expr = if self.peek() == &Token::Eq {
                // Also assign it a value
                self.take();
                let expr = self.expression()?;
                Some(expr)
            } else {
                // Really None?
                None
            };
            let end = self.peek_end_loc().clone();
            self.accept(Token::Semicolon, "Decl statement must end with ';'")?;
            Some(StmtNode::new(Stmt::Decl(id, expr), start, end))
        } else {
            self.error("A declaration statement must start with an id");
            None
        }
    }

    // To not allow declarations in for example single arm of if stmt
    fn non_decl_stmt(&mut self, allow_expr: bool) -> Option<Either<StmtNode, ExprNode>> {
        match self.peek() {
            Token::Identifier(id) if id.as_str() == "print" => {
                Some(Either::Left(self.print_stmt()?))
            }
            _ => self.expr_stmt(allow_expr),
        }
    }

    fn print_stmt(&mut self) -> Option<StmtNode> {
        // Special case for print functions/statments
        self.take();
        self.accept(Token::LPar, "Expect parentheses around print expression")?;
        let expr = self.expression()?;
        let start = expr.start_loc.clone();
        self.accept(Token::RPar, "Expect closing parentheses after print")?;
        let end = self.peek_end_loc().clone();
        self.accept(Token::Semicolon, "Expect ';' after print call")?;
        Some(StmtNode::new(Stmt::Print(expr), start, end))
    }

    fn expr_stmt(&mut self, allow_expr: bool) -> Option<Either<StmtNode, ExprNode>> {
        let expr = self.expression()?;
        if !allow_expr || self.peek() == &Token::Semicolon {
            let start = expr.start_loc.clone();
            let end = self.peek_end_loc().clone();
            self.accept(Token::Semicolon, "Expect ';' after expression statement")?;
            Some(Either::Left(StmtNode::new(Stmt::Expr(expr), start, end)))
        } else {
            Some(Either::Right(expr))
        }
    }
}
