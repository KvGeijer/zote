use crate::scanner::Token;

use super::{AstNode, ExprNode, Parser};

pub type StmtNode = AstNode<Stmt>;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expr(ExprNode),
    Print(ExprNode), // TODO Integrate into standard lib func
    Invalid,
}

impl<'a> Parser<'a> {
    pub fn statement(&mut self) -> StmtNode {
        if let Some(stmt) = self.try_statement() {
            stmt
        } else {
            // Should we propagate a result to here instead?
            StmtNode::new(Stmt::Invalid, self.peek_loc(), self.peek_loc())
        }
    }

    fn try_statement(&mut self) -> Option<StmtNode> {
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
        let start = expr.loc;
        self.accept(Token::RPar, "Expect closing parentheses after print")?;
        let end = self.peek_loc().into();
        self.accept(Token::Semicolon, "Expect ';' after print call")?;
        Some(StmtNode::new(Stmt::Print(expr), start, end))
    }

    fn expr_stmt(&mut self) -> Option<StmtNode> {
        let expr = self.expression()?;
        let start = expr.loc;
        let end = self.peek_loc().into();
        self.accept(Token::Semicolon, "Expect ';' after expression statement")?;
        Some(StmtNode::new(Stmt::Expr(expr), start, end))
    }
}
