use lazy_static::lazy_static;
use regex::Regex;

use crate::MainState;

macro_rules! lex_rules {
    ($(($pattern:expr, $boxed:expr)),* $(,)?) => {
        vec![
            $((Regex::new(&(r"\A".to_owned() + $pattern)).unwrap(), 
               Box::new($boxed)),)*
        ]
    };
}

#[derive(Debug)]
pub enum Token {
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Comment(String),
    Invalid(String),

    // Special constructs
    If,
    Else,
    Match,
    Return,
    For,
    While,

    // Single chars
    LPar,
    RPar,
    LBrace,
    RBrace,
    LBrack,
    RBrack,
    Dot,
    Semicolon,
    Comma,
    Eq,

    EOF,

    // Ones I might want to combine with identifier
    // But can do that later
    True,
    False,
    And,
    Or,
    Plus,
    Minus,
    Mult,
    Div,
    DoubleEq,

}

#[derive(Debug)]
pub struct TokenInfo {
    token: Token,
    loc: CodeLoc,

}

#[derive(Debug, Clone)]
pub struct CodeLoc {
    line: usize,
    col: usize,
    index: usize,
}

lazy_static!{
    static ref PATTERNS: Vec<(Regex, Box<dyn Fn(&str) -> Token + Sync>)> = lex_rules![
        (r"\(", |_| Token::LPar),
        (r"\)", |_| Token::RPar),
        (r"\{", |_| Token::LBrace),
        (r"\}", |_| Token::RBrace),
        (r"\[", |_| Token::LBrack),
        (r"\]", |_| Token::RBrack),
        (r"\.", |_| Token::Dot),
        (r";", |_| Token::Semicolon),
        (r",", |_| Token::Comma),
        (r"=", |_| Token::Eq),
    ];
}

pub fn tokenize(mut code: &str, main_state: &mut MainState) -> Vec<TokenInfo> {
    let mut tokens = vec![];
    let mut loc = CodeLoc {line: 0, col: 0, index: 0};

    while loc.index < code.len() {
        // Parse whitelines!!! No, Keep track of newlines...
        code = code.trim();

        for (re, closure) in PATTERNS.iter() {
            if let Some(len) = re.shortest_match(&code[loc.index..]) {
                let token = closure(&code[loc.index..loc.index+len]);
                tokens.push(TokenInfo{token: token, loc: loc.clone()});
                break;  // make a function
            }
        }
        loc.index += 1;
        
    }

    tokens
}
