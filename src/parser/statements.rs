use crate::scanner::Token;

use super::{AstNode, ExprNode, Parser};

pub type StmtNode = AstNode<Stmt>;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Decl(String, Option<ExprNode>),
    Expr(ExprNode),
    Print(ExprNode), // TODO Integrate into standard lib func
    Invalid,
}

impl<'a> Parser<'a> {
    pub fn statements(&mut self) -> Result<Vec<StmtNode>, Vec<StmtNode>> {
        let mut stmts = Vec::new();
        while !self.at_end() {
            stmts.push(self.statement());
        }

        if stmts.iter().any(|stmt| stmt.node == Stmt::Invalid) {
            Err(stmts) // In case we still want to be able so see the ast
        } else {
            Ok(stmts)
        }
    }

    // Maybe not ideal to have pub, but statements can be inside blocks...
    pub fn statement(&mut self) -> StmtNode {
        if let Some(stmt) = self.top_statement() {
            stmt
        } else {
            // Should we propagate a result to here instead?
            self.synchronize_error();
            StmtNode::new(
                Stmt::Invalid,
                self.peek_start_loc().clone(),
                self.peek_start_loc().clone(),
            )
        }
    }

    fn top_statement(&mut self) -> Option<StmtNode> {
        // Statements and decl statements
        match self.peek() {
            Token::Var => self.decl_stmt(),
            _ => self.non_decl_stmt(),
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
    fn non_decl_stmt(&mut self) -> Option<StmtNode> {
        match self.peek() {
            Token::Identifier(id) if id.as_str() == "print" => self.print_stmt(),
            _ => self.expr_stmt(),
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

    fn expr_stmt(&mut self) -> Option<StmtNode> {
        let expr = self.expression()?;
        let start = expr.start_loc.clone();
        let end = self.peek_end_loc().clone();
        self.accept(Token::Semicolon, "Expect ';' after expression statement")?;
        Some(StmtNode::new(Stmt::Expr(expr), start, end))
    }
}
