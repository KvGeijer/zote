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

#[derive(PartialEq, Debug)]
pub enum Expr {
    Call, // TODO
    Binary(Box<Expr>, BinOper, Box<Expr>),
    Unary(UnOper, Box<Expr>),
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

impl Expr {
    fn binary(left: Expr, op: BinOper, rigth: Expr) -> Expr {
        Expr::Binary(Box::new(left), op, Box::new(rigth))
    }

    fn unary(op: UnOper, rigth: Expr) -> Expr {
        Expr::Unary(op, Box::new(rigth))
    }
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

    fn accept(&mut self, expected: Token, error_str: &str, take: bool) -> Option<()> {
        if self.peek() == &expected {
            if take {
                // For edge case where we want to accept on peek (unconditional take after)
                self.take();
            }
            Some(())
        } else {
            self.error(error_str);
            None
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

    fn expression(&mut self) -> Option<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Option<Expr> {
        // equality       → comparison ( ( "!=" | "==" ) comparison )* ;
        let mut expr = self.comparison()?;

        while let Some(op) = self.match_op([BinOper::Eq, BinOper::Neq]) {
            let right = self.comparison()?;
            expr = Expr::binary(expr, op, right);
        }

        Some(expr)
    }

    fn comparison(&mut self) -> Option<Expr> {
        // comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
        let mut term = self.term()?;

        while let Some(op) = self.match_op([BinOper::Gt, BinOper::Lt, BinOper::Geq, BinOper::Leq]) {
            let right = self.term()?;
            term = Expr::binary(term, op, right);
        }

        Some(term)
    }

    fn term(&mut self) -> Option<Expr> {
        // term           → factor ( ( "-" | "+" ) factor )* ;
        let mut factor = self.factor()?;

        while let Some(op) = self.match_op([BinOper::Add, BinOper::Sub]) {
            let right = self.factor()?;
            factor = Expr::binary(factor, op, right);
        }

        Some(factor)
    }

    fn factor(&mut self) -> Option<Expr> {
        // factor         → unary ( ( "/" | "*" ) unary )* ;
        let mut unary = self.unary()?;

        while let Some(op) = self.match_op([BinOper::Div, BinOper::Mult]) {
            let right = self.unary()?;
            unary = Expr::binary(unary, op, right);
        }

        Some(unary)
    }

    fn unary(&mut self) -> Option<Expr> {
        // unary          → ( "!" | "-" )? primary ;
        if let Some(op) = self.match_op([UnOper::Sub, UnOper::Not]) {
            let right = self.primary()?;
            Some(Expr::unary(op, right))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Option<Expr> {
        // primary        → INT | FLOAT | STRING | "true" | "false" | "nil" | "(" expression ")" ;

        match self.peek() {
            Token::False => Some(Expr::Bool(false)),
            Token::True => Some(Expr::Bool(true)),
            Token::Integer(int) => Some(Expr::Int(*int)),
            Token::Float(float) => Some(Expr::Float(*float)),
            Token::String(str) => Some(Expr::String(str.to_string())),
            // Token::Nil => Expr::Nil(),
            Token::LPar => {
                self.accept(Token::LPar, "Internal error, should have peeked LPar", true);
                let expr = self.expression()?;
                self.accept(Token::RPar, "Expect ')' after expression.", false)?;
                // Why does the book create a grouping subclass?
                Some(expr)
            }
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
}

pub fn parse(tokens: &[TokenInfo], error_reporter: &mut ErrorReporter) -> Option<Expr> {
    let mut parser = Parser::new(tokens, error_reporter);
    parser.expression()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_token(token: Token) -> TokenInfo {
        TokenInfo {
            token,
            loc: CodeLoc {
                line: 0,
                col: 0,
                index: 0,
                index_end: 0,
            },
            string: "fake string".to_string(),
        }
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
            fake_token(Token::EOF),
        ];

        let mut parser = Parser::new(&tokens, &mut error_reporter);

        let expected = Expr::binary(
            Expr::binary(Expr::Int(9), BinOper::Add, Expr::Int(3)),
            BinOper::Sub,
            Expr::binary(
                Expr::binary(Expr::Int(4), BinOper::Mult, Expr::Int(9)),
                BinOper::Div,
                Expr::binary(
                    Expr::Int(2),
                    BinOper::Add,
                    Expr::unary(UnOper::Sub, Expr::Int(1)),
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
            fake_token(Token::EOF),
        ];

        let mut parser = Parser::new(&tokens, &mut error_reporter);

        assert_eq!(
            parser.expression().unwrap(),
            Expr::binary(Expr::Int(1), BinOper::Lt, Expr::Int(3))
        );

        assert_eq!(
            parser.expression().unwrap(),
            Expr::binary(Expr::Int(1), BinOper::Leq, Expr::Int(4))
        );

        assert_eq!(
            parser.expression().unwrap(),
            Expr::binary(Expr::Int(2), BinOper::Geq, Expr::Int(9))
        );

        assert_eq!(
            parser.expression().unwrap(),
            Expr::binary(Expr::Int(3), BinOper::Gt, Expr::Int(3))
        );

        assert_eq!(
            parser.expression().unwrap(),
            Expr::binary(Expr::Int(2), BinOper::Eq, Expr::Int(3))
        );

        assert_eq!(
            parser.expression().unwrap(),
            Expr::binary(Expr::Int(5), BinOper::Neq, Expr::Int(6))
        );
    }
}
