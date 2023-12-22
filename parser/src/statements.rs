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
            match self.statement(&terminator) {
                Either::Left(parsed_stmts) => {
                    stmts.stmts.extend(parsed_stmts.into_iter());
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
    fn statement(&mut self, terminator: &Token) -> Either<Vec<StmtNode>, ExprNode> {
        if let Some(nodes) = self.fn_statement(terminator) {
            nodes
        } else {
            // Should we propagate a result to here instead?
            self.synchronize_error();
            Either::Left(vec![StmtNode::new(
                Stmt::Invalid,
                *self.peek_start_loc(),
                *self.peek_start_loc(),
            )])
        }
    }

    fn fn_statement(&mut self, terminator: &Token) -> Option<Either<Vec<StmtNode>, ExprNode>> {
        // decl_stmt | "fn" var "(" parameters? ")" "->" expression ;
        let start = *self.peek_start_loc();
        if !self.match_token(Token::Fn) {
            self.macro_stmt(terminator)
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
            // let id = format!("fn {name}/{}", params.len());
            let func = ExprNode::new(
                super::Expr::FunctionDefinition(name.to_owned(), params, body),
                start,
                end,
            );

            Some(Either::Left(vec![StmtNode::new(
                Stmt::Decl(LValue::Var(name), Some(func)),
                start,
                end,
            )]))
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

    /// Checks for a macro statement, before delegating to leading with an expression
    fn macro_stmt(&mut self, terminator: &Token) -> Option<Either<Vec<StmtNode>, ExprNode>> {
        // macro_stmt -> macro_invocation ( '(' args ')' )
        if let Some(name) = self.match_macro_invocation() {
            let res = match name {
                "include!" => Some(Either::Left(self.macro_include_statement()?)),
                otherwise => {
                    let reason = &format!("Could not resolve statement macro '{otherwise}'");
                    self.error(&reason);
                    None
                }
            };

            self.accept(
                Token::Semicolon,
                "Must end macro statement with a semicolon",
            )?;

            res
        } else {
            self.leading_expr_stmt(terminator)
        }
    }

    fn leading_expr_stmt(&mut self, terminator: &Token) -> Option<Either<Vec<StmtNode>, ExprNode>> {
        // varDecl        → (expression | lvalue ":=" expression | expression ":>>" lvalue) ";" ;
        let start = *self.peek_start_loc();
        let expr = self.expression()?; // We can't separate lvalues and assignmen here :/
        if self.match_token(Token::ColonEq) {
            let lvalue = self.expr_to_lvalue(expr, true)?;
            let rvalue = self.expression()?;
            let end = *self.peek_last_end_loc().unwrap();
            self.accept(Token::Semicolon, "Decl statement must end with ';'")?;
            Some(Either::Left(vec![StmtNode::new(
                Stmt::Decl(lvalue, Some(rvalue)),
                start,
                end,
            )]))
        // } else if self.match_token(Token::ColonPipe) { // Removed from language
        //     let lvalue = self.lvalue(true)?;
        //     let end = *self.peek_end_loc();
        //     self.accept(Token::Semicolon, "Expect ';' after pipe decl statement")?;
        //     Some(Either::Left(vec![StmtNode::new(
        //         Stmt::Decl(lvalue, Some(expr)),
        //         start,
        //         end,
        //     )]))
        } else if self.match_token(Token::Semicolon) {
            let end = *self.peek_last_end_loc().unwrap();
            Some(Either::Left(vec![StmtNode::new(
                Stmt::Expr(expr),
                start,
                end,
            )]))
        } else if !semicolon_elision(&expr) && self.peek() != terminator {
            // A ; was expected, but not found
            self.error("Expect ';' after expression statement");
            None
        } else {
            Some(Either::Right(expr))
        }
    }

    fn match_macro_invocation(&mut self) -> Option<&str> {
        if matches!(self.peek(), Token::MacroInvocation(_)) {
            let Token::MacroInvocation(name) = self.take() else {
                panic!("Internal error at match macro invocation");
            };
            Some(name.as_ref())
        } else {
            None
        }
    }
}

fn semicolon_elision(expr: &ExprNode) -> bool {
    match expr.node.as_ref() {
        Expr::While(_, block)
        | Expr::For(_, _, block)
        | Expr::If(_, block, None)
        | Expr::If(_, _, Some(block)) => {
            matches!(block.node.as_ref(), Expr::Block(_)) || semicolon_elision(block)
        }
        Expr::Match(_, _) | Expr::Block(_) => true,
        _ => false,
    }
}
