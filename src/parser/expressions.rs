use std::fmt::Debug;

use super::{AstNode, Parser, Stmts};
use crate::{code_loc::CodeLoc, scanner::Token};

// Cannot have more than this many arguments to a function
pub const MAX_ARGS: usize = 255;

// Exposes the data types and the expression method on parser
pub type ExprNode = AstNode<Expr>;
pub type BinOperNode = AstNode<BinOper>;
pub type UnOperNode = AstNode<UnOper>;
pub type LogicalOperNode = AstNode<LogicalOper>;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Call(Box<ExprNode>, Vec<ExprNode>), // TODO
    Binary(Box<ExprNode>, BinOperNode, Box<ExprNode>),
    Unary(UnOperNode, Box<ExprNode>),
    Logical(Box<ExprNode>, LogicalOperNode, Box<ExprNode>),
    Assign(String, Box<ExprNode>),
    Var(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Block(Stmts),
    If(Box<ExprNode>, Box<ExprNode>, Option<Box<ExprNode>>),
    While(Box<ExprNode>, Box<ExprNode>),
    Break, // TODO Do we want to return an optional value from this?
    Return(Option<Box<ExprNode>>),
    Nil,
    List(Vec<ExprNode>),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum BinOper {
    Add,
    Sub,
    Div,
    Mult,
    Mod,
    Pow,
    Eq,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UnOper {
    Not,
    Sub,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum LogicalOper {
    And,
    Or,
}

impl<'a> Parser<'a> {
    pub fn expression(&mut self) -> Option<ExprNode> {
        self.whole_expression()
    }

    fn whole_expression(&mut self) -> Option<ExprNode> {
        // For expressions like return/break, which shouldn't be part of larger composite expressions

        match self.peek() {
            Token::Return => self.accept_return(),
            Token::Break => self.accept_break(),
            _ => self.assignment(),
        }
    }

    fn assignment(&mut self) -> Option<ExprNode> {
        // assignment     → IDENTIFIER "=" assignment | equality ;
        // TODO At least assign to tuples
        let expr = self.pipe()?;
        if self.peek() == &Token::Eq {
            self.accept(Token::Eq, "Internal error, expected eq");
            if let AstNode {
                start_loc,
                end_loc: _,
                node: Expr::Var(id),
            } = expr
            {
                let rvalue = self.assignment()?;
                let end = rvalue.end_loc.clone();
                let assign = Expr::Assign(id, Box::new(rvalue));
                Some(ExprNode::new(assign, start_loc, end))
            } else {
                self.error("Invalid lvalue");
                None
            }
        } else {
            Some(expr)
        }
    }

    fn pipe(&mut self) -> Option<ExprNode> {
        // pipe       → or ( ">>" or )* ;
        let mut expr = self.or()?;

        while self.peek() == &Token::Pipe {
            self.accept(Token::Pipe, "Internal error at pipe");
            let start = expr.start_loc.clone();
            let (func, mut args, end) = self.accept_call()?;
            args.insert(0, expr); // Does this really work with ownership?
            expr = ExprNode::new(Expr::Call(func, args), start, end);
        }

        Some(expr)
    }

    fn accept_call(&mut self) -> Option<(Box<ExprNode>, Vec<ExprNode>, CodeLoc)> {
        // pipe_call   → IDENTIFIER | primary ( "(" exprs_list ")" )+
        // Just like a call, but must be a call or id, not boil down somehow

        let call = self.call()?;

        // Is it just a variable?
        if let &Expr::Var(_) = &call.node {
            let end = call.end_loc.clone();
            Some((Box::new(call), vec![], end))
        } else if let AstNode {
            // Or a real call
            start_loc: _,
            end_loc,
            node: Expr::Call(caller, args),
        } = call
        {
            Some((caller, args, end_loc))
        } else {
            self.error("Expected variable or call expression following pipe");
            None
        }
    }

    fn or(&mut self) -> Option<ExprNode> {
        // or       → and ( "or" and )* ;
        let mut expr = self.and()?;

        while let Some(op) = self.match_op([LogicalOper::Or]) {
            let right = self.and()?;
            expr = ExprNode::logical(expr, op, right);
        }

        Some(expr)
    }

    fn and(&mut self) -> Option<ExprNode> {
        // and       → equality ( "and" equality )* ;
        let mut expr = self.equality()?;

        while let Some(op) = self.match_op([LogicalOper::And]) {
            let right = self.equality()?;
            expr = ExprNode::logical(expr, op, right);
        }

        Some(expr)
    }

    fn equality(&mut self) -> Option<ExprNode> {
        // equality       → comparison ( ( "!=" | "==" ) comparison )* ;
        let mut expr = self.comparison()?;

        while let Some(op) = self.match_op([BinOper::Eq, BinOper::Neq]) {
            let right = self.comparison()?;
            expr = ExprNode::binary(expr, op, right);
        }

        Some(expr)
    }

    fn comparison(&mut self) -> Option<ExprNode> {
        // comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
        let mut term = self.term()?;

        while let Some(op) = self.match_op([BinOper::Gt, BinOper::Lt, BinOper::Geq, BinOper::Leq]) {
            let right = self.term()?;
            term = ExprNode::binary(term, op, right);
        }

        Some(term)
    }

    fn term(&mut self) -> Option<ExprNode> {
        // term           → factor ( ( "-" | "+" ) factor )* ;
        let mut factor = self.factor()?;

        while let Some(op) = self.match_op([BinOper::Add, BinOper::Sub]) {
            let right = self.factor()?;
            factor = ExprNode::binary(factor, op, right);
        }

        Some(factor)
    }

    fn factor(&mut self) -> Option<ExprNode> {
        // factor         → exponent ( ( "/" | "*" | "%" ) exponent )* ;
        let mut exponent = self.exponent()?;

        while let Some(op) = self.match_op([BinOper::Div, BinOper::Mult, BinOper::Mod]) {
            let right = self.exponent()?;
            exponent = ExprNode::binary(exponent, op, right);
        }

        Some(exponent)
    }

    fn exponent(&mut self) -> Option<ExprNode> {
        // exponent         → unary ( "^" unary )* ;
        let mut unary = self.unary()?;

        while let Some(op) = self.match_op([BinOper::Pow]) {
            let right = self.unary()?;
            unary = ExprNode::binary(unary, op, right);
        }

        Some(unary)
    }

    fn unary(&mut self) -> Option<ExprNode> {
        // unary          → ( "!" | "-" )? call ;
        if let Some(op) = self.match_op([UnOper::Sub, UnOper::Not]) {
            let right = self.call()?;
            Some(ExprNode::unary(op, right))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Option<ExprNode> {
        // call           → primary ( "(" exprs_list ")" )* ;
        let mut expr = self.primary()?;
        let start = expr.start_loc.clone();
        while self.peek() == &Token::LPar {
            self.accept(Token::LPar, "Internal error at call arguments");
            let args = self.accept_exprs_list(&Token::RPar)?;
            self.accept(Token::RPar, "Expect ')' to close call arguments");

            if args.len() >= MAX_ARGS {
                self.error("Can't have more than {MAX_ARGS} arguments");
            }
            let end = self.peek_last_end_loc()?.clone();
            expr = ExprNode::new(Expr::Call(Box::new(expr), args), start.clone(), end);
        }
        Some(expr)
    }

    fn accept_exprs_list(&mut self, terminator: &Token) -> Option<Vec<ExprNode>> {
        // exprs_list      → ( expression ( "," expression )* )? ;
        // The argument "terminator" will directly follow the optional list
        let mut args = if terminator == self.peek() {
            vec![]
        } else {
            vec![self.expression()?]
        };
        while self.peek() == &Token::Comma {
            self.accept(Token::Comma, "Internal error in accept_exprs_list");
            args.push(self.expression()?);
        }
        Some(args)
    }

    fn primary(&mut self) -> Option<ExprNode> {
        // primary        → "(" expression ")" | block | if | "break" expr? ;

        match self.peek() {
            Token::If => self.accept_if(),
            Token::LBrace => self.accept_block(),
            Token::While => self.accept_while(),
            Token::LBrack => self.accept_list(),
            Token::LPar => {
                // Dont have the correct location really for this
                self.accept(Token::LPar, "Internal error, should have peeked LPar");
                let expr = self.expression()?;
                self.accept(Token::RPar, "Expect ')' after expression.")?;
                // Why does the book create a grouping subclass?
                Some(expr)
            }
            _ => self.simple_primary(),
        }
    }

    fn simple_primary(&mut self) -> Option<ExprNode> {
        // simple_primary        → INT | FLOAT | STRING | "true" | "false" | "nil" | "break"
        //                       | Identifier ;
        let start = self.peek_start_loc().clone();
        let end = self.peek_end_loc().clone();
        match self.peek() {
            Token::False => some_node(Expr::Bool(false), start, end),
            Token::True => some_node(Expr::Bool(true), start, end),
            Token::Integer(int) => some_node(Expr::Int(*int), start, end),
            Token::Float(float) => some_node(Expr::Float(*float), start, end),
            Token::String(str) => some_node(Expr::String(str.to_string()), start, end),
            Token::Identifier(str) => some_node(Expr::Var(str.to_owned()), start, end),
            Token::Nil => some_node(Expr::Nil, start, end),
            _ => {
                self.error("Expect expression");
                None
            }
        }
        .map(|res| {
            // Not great to have side effect in map like this...
            self.take();
            res
        })
    }

    fn accept_list(&mut self) -> Option<ExprNode> {
        let start = self.peek_start_loc().clone();
        self.accept(Token::LBrack, "Internal error at list")?;

        let entries = self.accept_exprs_list(&Token::RBrack)?;

        self.accept(Token::RBrack, "Need to close list with ']'")?;
        let end = self.peek_last_end_loc()?.clone();
        Some(ExprNode::new(Expr::List(entries), start, end))
    }

    fn accept_return(&mut self) -> Option<ExprNode> {
        let start = self.peek_start_loc().clone();
        self.accept(Token::Return, "Internal error at return")?;

        // Ugly way, but if there is no expression we try to infer a nil return,
        // but only a simple check, which might miss things in strange expressions
        let expr = if ![
            Token::Semicolon,
            Token::Comma,
            Token::Else,
            Token::RBrace,
            Token::RPar,
            Token::RBrack,
        ]
        .contains(self.peek())
        {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        let end = self.peek_last_end_loc()?.clone();
        Some(ExprNode::new(Expr::Return(expr), start, end))
    }

    fn accept_break(&mut self) -> Option<ExprNode> {
        let start = self.peek_start_loc().clone();
        let end = self.peek_end_loc().clone();
        self.accept(Token::Break, "Internal error at break")?;
        Some(ExprNode::new(Expr::Break, start, end))
    }

    fn accept_if(&mut self) -> Option<ExprNode> {
        let start = self.peek_start_loc().clone();
        self.accept(Token::If, "Internal error at if")?;

        let cond = self.expression()?;
        let then = self.expression()?;
        let otherwise = if self.peek() == &Token::Else {
            self.accept(Token::Else, "Internal error at if")?;
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        let end = self.peek_last_end_loc()?.clone();
        Some(ExprNode::new(
            Expr::If(Box::new(cond), Box::new(then), otherwise),
            start,
            end,
        ))
    }

    fn accept_block(&mut self) -> Option<ExprNode> {
        let start = self.peek_start_loc().clone();
        self.accept(Token::LBrace, "Internal error at block")?;

        let stmts = self.statements(Token::RBrace).ok()?;

        let end = self.peek_last_end_loc()?.clone();
        self.accept(Token::RBrace, "Need to close block with '}'")?;
        Some(ExprNode::new(Expr::Block(stmts), start, end))
    }

    fn accept_while(&mut self) -> Option<ExprNode> {
        let start = self.peek_start_loc().clone();
        self.accept(Token::While, "Internal error at while")?;

        let cond = self.expression()?;
        let repeat = self.expression()?;

        let end = self.peek_last_end_loc()?.clone();
        Some(ExprNode::new(
            Expr::While(Box::new(cond), Box::new(repeat)),
            start,
            end,
        ))
    }

    fn match_op<F: FromToken + Eq + Debug, T: IntoIterator<Item = F>>(
        &mut self,
        expected: T,
    ) -> Option<AstNode<F>> {
        match F::try_from(self.peek())
            .filter(|peeked| expected.into_iter().any(|wanted| &wanted == peeked))
        {
            None => None,
            Some(matched) => {
                let start_loc = self.peek_start_loc().clone();
                let end_loc = self.peek_end_loc().clone();
                self.take();
                Some(AstNode::new(matched, start_loc, end_loc))
            }
        }
    }
}

// Some helper functions
impl ExprNode {
    fn binary(left: ExprNode, op: BinOperNode, right: ExprNode) -> ExprNode {
        let start_loc = left.start_loc.clone();
        let end_loc = right.end_loc.clone();
        let expr = Expr::Binary(Box::new(left), op, Box::new(right));
        AstNode::new(expr, start_loc, end_loc)
    }

    fn unary(op: UnOperNode, right: ExprNode) -> ExprNode {
        let start_loc = op.start_loc.clone();
        let end_loc = right.end_loc.clone();
        let expr = Expr::Unary(op, Box::new(right));
        AstNode::new(expr, start_loc, end_loc)
    }

    fn logical(left: ExprNode, op: LogicalOperNode, right: ExprNode) -> ExprNode {
        let start_loc = left.start_loc.clone();
        let end_loc = right.end_loc.clone();
        let expr = Expr::Logical(Box::new(left), op, Box::new(right));
        AstNode::new(expr, start_loc, end_loc)
    }
}

trait FromToken: Sized {
    fn try_from(token: &Token) -> Option<Self>;
}

impl FromToken for BinOper {
    fn try_from(token: &Token) -> Option<Self> {
        match token {
            Token::DoubleEq => Some(BinOper::Eq),
            Token::BangEq => Some(BinOper::Neq),
            Token::Gt => Some(BinOper::Gt),
            Token::Geq => Some(BinOper::Geq),
            Token::Lt => Some(BinOper::Lt),
            Token::Leq => Some(BinOper::Leq),
            Token::Mult => Some(BinOper::Mult),
            Token::Div => Some(BinOper::Div),
            Token::Plus => Some(BinOper::Add),
            Token::Minus => Some(BinOper::Sub),
            Token::UpArr => Some(BinOper::Pow),
            Token::Percent => Some(BinOper::Mod),
            _ => None,
        }
    }
}

impl FromToken for UnOper {
    fn try_from(token: &Token) -> Option<Self> {
        match token {
            Token::Minus => Some(UnOper::Sub),
            Token::Bang => Some(UnOper::Not),
            _ => None,
        }
    }
}

impl FromToken for LogicalOper {
    fn try_from(token: &Token) -> Option<Self> {
        match token {
            Token::And => Some(LogicalOper::And),
            Token::Or => Some(LogicalOper::Or),
            _ => None,
        }
    }
}

fn some_node<T: Debug>(grammar: T, start: CodeLoc, end: CodeLoc) -> Option<AstNode<T>> {
    Some(AstNode::new(grammar, start, end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{code_loc::CodeLoc, errors::ErrorReporter, scanner::TokenInfo};

    fn fake_token(token: Token) -> TokenInfo {
        TokenInfo {
            token,
            start_loc: CodeLoc::new(0, 0, 0),
            end_loc: CodeLoc::new(0, 0, 0),
            string: "fake string".to_string(),
        }
    }

    fn fake_node<T: Debug>(data: T) -> AstNode<T> {
        let loc = CodeLoc::new(0, 0, 0);
        AstNode::new(data, loc.clone(), loc)
    }

    #[test]
    fn basic_math() {
        let mut error_reporter = ErrorReporter::new();

        // Can't really test floats due to Rust not implementing Eq for them
        // "9 + 3 - 4 * 9 / (2 + -1)"
        let tokens = vec![
            fake_token(Token::Integer(9)),
            fake_token(Token::Plus),
            fake_token(Token::Integer(3)),
            fake_token(Token::Minus),
            fake_token(Token::Integer(4)),
            fake_token(Token::Mult),
            fake_token(Token::Integer(9)),
            fake_token(Token::Div),
            fake_token(Token::LPar),
            fake_token(Token::Integer(2)),
            fake_token(Token::Plus),
            fake_token(Token::Minus),
            fake_token(Token::Integer(1)),
            fake_token(Token::RPar),
            fake_token(Token::Eof),
        ];

        let mut parser = Parser::new(&tokens, &mut error_reporter);

        let expected = ExprNode::binary(
            ExprNode::binary(
                fake_node(Expr::Int(9)),
                fake_node(BinOper::Add),
                fake_node(Expr::Int(3)),
            ),
            fake_node(BinOper::Sub),
            ExprNode::binary(
                ExprNode::binary(
                    fake_node(Expr::Int(4)),
                    fake_node(BinOper::Mult),
                    fake_node(Expr::Int(9)),
                ),
                fake_node(BinOper::Div),
                ExprNode::binary(
                    fake_node(Expr::Int(2)),
                    fake_node(BinOper::Add),
                    ExprNode::unary(fake_node(UnOper::Sub), fake_node(Expr::Int(1))),
                ),
            ),
        );

        assert_eq!(parser.expression().unwrap(), expected);
    }

    #[test]
    fn int_comparisons() {
        // Ok, probably overkill, and more needed to mix arithmetic and comparisons...

        let mut error_reporter = ErrorReporter::new();

        // Can't really test floats due to Rust not implementing Eq for them
        // "1 < 3 1 <= 4 2 >= 9 3 > 3 2 == 3 5 != 6"
        let tokens = vec![
            fake_token(Token::Integer(1)),
            fake_token(Token::Lt),
            fake_token(Token::Integer(3)),
            fake_token(Token::Integer(1)),
            fake_token(Token::Leq),
            fake_token(Token::Integer(4)),
            fake_token(Token::Integer(2)),
            fake_token(Token::Geq),
            fake_token(Token::Integer(9)),
            fake_token(Token::Integer(3)),
            fake_token(Token::Gt),
            fake_token(Token::Integer(3)),
            fake_token(Token::Integer(2)),
            fake_token(Token::DoubleEq),
            fake_token(Token::Integer(3)),
            fake_token(Token::Integer(5)),
            fake_token(Token::BangEq),
            fake_token(Token::Integer(6)),
            fake_token(Token::Eof),
        ];

        let mut parser = Parser::new(&tokens, &mut error_reporter);

        assert_eq!(
            parser.expression().unwrap(),
            ExprNode::binary(
                fake_node(Expr::Int(1)),
                fake_node(BinOper::Lt),
                fake_node(Expr::Int(3))
            )
        );

        assert_eq!(
            parser.expression().unwrap(),
            ExprNode::binary(
                fake_node(Expr::Int(1)),
                fake_node(BinOper::Leq),
                fake_node(Expr::Int(4))
            )
        );

        assert_eq!(
            parser.expression().unwrap(),
            ExprNode::binary(
                fake_node(Expr::Int(2)),
                fake_node(BinOper::Geq),
                fake_node(Expr::Int(9))
            )
        );

        assert_eq!(
            parser.expression().unwrap(),
            ExprNode::binary(
                fake_node(Expr::Int(3)),
                fake_node(BinOper::Gt),
                fake_node(Expr::Int(3))
            )
        );

        assert_eq!(
            parser.expression().unwrap(),
            ExprNode::binary(
                fake_node(Expr::Int(2)),
                fake_node(BinOper::Eq),
                fake_node(Expr::Int(3))
            )
        );

        assert_eq!(
            parser.expression().unwrap(),
            ExprNode::binary(
                fake_node(Expr::Int(5)),
                fake_node(BinOper::Neq),
                fake_node(Expr::Int(6))
            )
        );
    }

    #[test]
    fn pipes() {
        // These tests are really going over board

        // Can't really test floats due to Rust not implementing Eq for them
        // "[1 >> print] >> print() >> fake(2)"
        let tokens = vec![
            fake_token(Token::LBrack),
            fake_token(Token::Integer(1)),
            fake_token(Token::Pipe),
            fake_token(Token::Identifier("print".to_string())),
            fake_token(Token::RBrack),
            fake_token(Token::Pipe),
            fake_token(Token::Identifier("print".to_string())),
            fake_token(Token::LPar),
            fake_token(Token::RPar),
            fake_token(Token::Pipe),
            fake_token(Token::Identifier("fake".to_string())),
            fake_token(Token::LPar),
            fake_token(Token::Integer(2)),
            fake_token(Token::RPar),
            fake_token(Token::Eof),
        ];
        let mut error_reporter = ErrorReporter::new();
        let mut pipe_parser = Parser::new(&tokens, &mut error_reporter);
        let pipe_expr = pipe_parser.expression().unwrap();

        // Logically equal to "fake(print([print(1)]), 2)"
        let tokens = vec![
            fake_token(Token::Identifier("fake".to_string())),
            fake_token(Token::LPar),
            fake_token(Token::Identifier("print".to_string())),
            fake_token(Token::LPar),
            fake_token(Token::LBrack),
            fake_token(Token::Identifier("print".to_string())),
            fake_token(Token::LPar),
            fake_token(Token::Integer(1)),
            fake_token(Token::RPar),
            fake_token(Token::RBrack),
            fake_token(Token::RPar),
            fake_token(Token::Comma),
            fake_token(Token::Integer(2)),
            fake_token(Token::RPar),
            fake_token(Token::Eof),
        ];
        let mut error_reporter = ErrorReporter::new();
        let mut normal_parser = Parser::new(&tokens, &mut error_reporter);
        let normal_expr = normal_parser.expression().unwrap();

        // Just assert that they are equal. Maybe should also spell out what is should be
        assert_eq!(pipe_expr, normal_expr);
    }
}
