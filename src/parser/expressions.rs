use super::{AstLoc, AstNode, Parser};
use crate::scanner::Token;

// Exposes the data types and the expression method on parser
pub type ExprNode = AstNode<Expr>;
pub type BinOperNode = AstNode<BinOper>;
pub type UnOperNode = AstNode<UnOper>;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Call, // TODO
    Binary(Box<ExprNode>, BinOperNode, Box<ExprNode>),
    Unary(UnOperNode, Box<ExprNode>),
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

impl<'a> Parser<'a> {
    pub fn expression(&mut self) -> Option<ExprNode> {
        self.equality()
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
        // factor         → unary ( ( "/" | "*" ) unary )* ;
        let mut unary = self.unary()?;

        while let Some(op) = self.match_op([BinOper::Div, BinOper::Mult]) {
            let right = self.unary()?;
            unary = ExprNode::binary(unary, op, right);
        }

        Some(unary)
    }

    fn unary(&mut self) -> Option<ExprNode> {
        // unary          → ( "!" | "-" )? primary ;
        if let Some(op) = self.match_op([UnOper::Sub, UnOper::Not]) {
            let right = self.primary()?;
            Some(ExprNode::unary(op, right))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Option<ExprNode> {
        // primary        → INT | FLOAT | STRING | "true" | "false" | "nil" | "(" expression ")" ;

        let start = self.peek_loc();
        match self.peek() {
            Token::False => some_node(Expr::Bool(false), start, start),
            Token::True => some_node(Expr::Bool(true), start, start),
            Token::Integer(int) => some_node(Expr::Int(*int), start, start),
            Token::Float(float) => some_node(Expr::Float(*float), start, start),
            Token::String(str) => some_node(Expr::String(str.to_string()), start, start),
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

    fn match_op<F: FromToken + Eq, T: IntoIterator<Item = F>>(
        &mut self,
        expected: T,
    ) -> Option<AstNode<F>> {
        match F::try_from(self.peek())
            .filter(|peeked| expected.into_iter().any(|wanted| &wanted == peeked))
        {
            None => None,
            Some(matched) => {
                let loc: AstLoc = self.peek_loc().into();
                self.take();
                Some(AstNode::new(matched, loc, loc))
            }
        }
    }
}

// Some helper functions
impl ExprNode {
    fn binary(left: ExprNode, op: BinOperNode, right: ExprNode) -> ExprNode {
        let lloc: AstLoc = left.loc.into();
        let rloc: AstLoc = right.loc.into();
        let expr = Expr::Binary(Box::new(left), op, Box::new(right));
        AstNode::new(expr, lloc, rloc)
    }

    fn unary(op: UnOperNode, right: ExprNode) -> ExprNode {
        let rloc: AstLoc = right.loc.into();
        let oloc: AstLoc = op.loc.into();
        let expr = Expr::Unary(op, Box::new(right));
        AstNode::new(expr, oloc, rloc)
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

// Ugly without references, but could not get it to work :(
fn some_node<T, F: Into<AstLoc>>(grammar: T, start: F, end: F) -> Option<AstNode<T>> {
    Some(AstNode::new(grammar, start, end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        errors::ErrorReporter,
        scanner::{CodeLoc, TokenInfo},
    };

    fn fake_token(token: Token) -> TokenInfo {
        TokenInfo {
            token,
            loc: CodeLoc {
                line: 0,
                col: 0,
                index: 0,
                len: 1,
            },
            string: "fake string".to_string(),
        }
    }

    fn fake_node<T>(data: T) -> AstNode<T> {
        let loc = AstLoc::new(0, 0, 0, 0);
        AstNode { loc, node: data }
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
            fake_token(Token::EOF),
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
}
