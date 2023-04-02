use crate::errors::ErrorReporter;
use crate::scanner::{CodeLoc, Token, TokenInfo};

// Example grammar from the crafting interpreter book
// Really want to be able to create my own binary functions though
// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;

#[derive(Debug)]
pub enum Expr {
    Call(), // TODO
    Binary(Box<Expr>, BinOper, Box<Expr>),
    Unary(UnOper, Box<Expr>),
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(PartialEq, Eq, Debug)]
pub enum BinOper {
    Add,
    Sub,
    Div,
    Mult,
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,
}

#[derive(PartialEq, Eq, Debug)]
pub enum UnOper {
    Not,
    Sub,
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

struct Parser<'a> {
    tokens: Vec<&'a TokenInfo>,
    current: usize,
    error_reporter: &'a mut ErrorReporter,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [TokenInfo], error_reporter: &'a mut ErrorReporter) -> Self {
        Self {
            tokens: Vec::from_iter(tokens.iter()),
            current: 0,
            error_reporter,
        }
    }

    fn error(&mut self, str: &str) {
        // Ugly, makes sense why borrow checker argues unless copy, but still...
        // Should probably have the error reporter outside of the parser...
        let loc = self.peek_loc().clone();
        let error_string = format!("{} at '{}' {}", loc.line, &self.peek_string(), str);
        self.error_reporter.error(&loc, &error_string);
    }

    fn synchronize_error(&mut self) {
        while !self.at_end() {
            // If we are at semicolon we take it and advance
            if self.take() == &Token::Semicolon {
                return;
            }

            // If the next token is the start of a new statement we can also return
            match self.peek() {
                Token::Struct => return,
                Token::Fn => return,
                Token::For => return,
                Token::While => return,
                Token::Return => return,
                _ => continue,
            }
        }
    }

    fn at_end(&self) -> bool {
        self.current == self.tokens.len()
    }

    fn peek_info(&self) -> &TokenInfo {
        &self.tokens[self.current]
    }

    fn peek(&self) -> &Token {
        &self.peek_info().token
    }

    fn peek_loc(&self) -> &CodeLoc {
        &self.peek_info().loc
    }

    fn peek_string(&self) -> &str {
        &self.peek_info().string
    }

    fn accept(&mut self, expected: Token, error_str: &str) -> Result<(), ()> {
        if self.peek() == &expected {
            self.take();
            Ok(())
        } else {
            self.error(error_str);
            Err(())
        }
    }

    fn take(&mut self) -> &Token {
        let ind = self.current;
        if !self.at_end() {
            self.current += 1
        }
        &self.tokens[ind].token
    }

    fn match_op<F: FromToken + Eq, T: IntoIterator<Item = F>>(&mut self, expected: T) -> Option<F> {
        match F::try_from(self.peek())
            .filter(|peeked| expected.into_iter().any(|wanted| &wanted == peeked))
        {
            None => None,
            matched => {
                self.take();
                matched
            }
        }
    }

    fn expression(&mut self) -> Result<Expr, ()> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ()> {
        // equality       → comparison ( ( "!=" | "==" ) comparison )* ;
        let mut expr = self.comparison()?;

        while let Some(op) = self.match_op([BinOper::Eq, BinOper::Neq]) {
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ()> {
        // comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
        let mut term = self.term()?;

        while let Some(op) = self.match_op([BinOper::Gt, BinOper::Lt, BinOper::Geq, BinOper::Leq]) {
            let right = self.term()?;
            term = Expr::Binary(Box::new(term), op, Box::new(right));
        }

        Ok(term)
    }

    fn term(&mut self) -> Result<Expr, ()> {
        // term           → factor ( ( "-" | "+" ) factor )* ;
        let mut factor = self.factor()?;

        while let Some(op) = self.match_op([BinOper::Add, BinOper::Sub]) {
            let right = self.factor()?;
            factor = Expr::Binary(Box::new(factor), op, Box::new(right));
        }

        Ok(factor)
    }

    fn factor(&mut self) -> Result<Expr, ()> {
        // factor         → unary ( ( "/" | "*" ) unary )* ;
        let mut unary = self.unary()?;

        while let Some(op) = self.match_op([BinOper::Div, BinOper::Mult]) {
            let right = self.unary()?;
            unary = Expr::Binary(Box::new(unary), op, Box::new(right));
        }

        Ok(unary)
    }

    fn unary(&mut self) -> Result<Expr, ()> {
        // unary          → ( "!" | "-" )? primary ;
        if let Some(op) = self.match_op([UnOper::Sub, UnOper::Not]) {
            let right = self.primary()?;
            Ok(Expr::Unary(op, Box::new(right)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ()> {
        // primary        → INT | FLOAT | STRING | "true" | "false" | "nil" | "(" expression ")" ;

        match self.peek() {
            Token::False => Ok(Expr::Bool(false)),
            Token::True => Ok(Expr::Bool(true)),
            Token::Integer(int) => Ok(Expr::Int(*int)),
            Token::Float(float) => Ok(Expr::Float(*float)),
            Token::String(str) => Ok(Expr::String(str.to_string())),
            // Token::Nil => Expr::Nil(),
            Token::LPar => {
                let expr = self.expression()?;
                self.accept(Token::RPar, "Expect ')' after expression.")?;
                // Why does the book create a grouping subclass?
                Ok(expr)
            }
            _ => {
                self.error("Expect expression");
                Err(())
            }
        }
        .map(|res| {
            // Not great to have side effect in map like this...
            self.take();
            res
        })
    }
}

pub fn parse(tokens: &[TokenInfo], error_reporter: &mut ErrorReporter) -> Option<Expr> {
    let mut parser = Parser::new(tokens, error_reporter);
    parser.expression().ok()
}
