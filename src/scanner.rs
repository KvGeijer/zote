use lazy_static::lazy_static;
use regex::Regex;

use crate::errors::ErrorReporter;

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
    // Change to &str
    Identifier(String),
    Float(f64),
    Integer(i64),
    String(String),
    Comment(String),
    // Invalid(String),

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
    Lt,
    Gt,
    Leq,
    Geq,
}

#[derive(Debug)]
pub struct TokenInfo {
    token: Token,
    loc: CodeLoc,
}

#[derive(Debug, Clone)]
pub struct CodeLoc {
    pub line: usize,
    pub col: usize,
    pub index: usize,
}

lazy_static! {
    static ref PATTERNS: Vec<(Regex, Box<dyn Fn(&str) -> Token + Sync>)> = lex_rules![
        (r"[\w--\d]\w*", |str| Token::Identifier(str.to_owned())),
        // Not optimal that 001 is scanned as 0 0 1
        (r"(([1-9]\d*\.\d+)|(0\.\d+))", |str| Token::Float(str.parse().unwrap())),
        (r"([1-9]\d*|0)", |str| Token::Integer(str.parse().unwrap())),
        (r#"".*?""#, |str| Token::String(str.to_owned())),
        (r"//[^\n]*", |str| Token::Comment(str.to_owned())),
        (r"if", |_| Token::If),
        (r"else", |_| Token::Else),
        (r"match", |_| Token::Match),
        (r"return", |_| Token::Return),
        (r"for", |_| Token::For),
        (r"while", |_| Token::While),
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
        (r"true", |_| Token::True),
        (r"false", |_| Token::False),
        (r"and", |_| Token::And),
        (r"or", |_| Token::Or),
        (r"\+", |_| Token::Plus),
        (r"-", |_| Token::Minus),
        (r"\*", |_| Token::Mult),
        (r"/", |_| Token::Div),
        (r"==", |_| Token::DoubleEq),
        (r"/", |_| Token::Lt),
        (r"/", |_| Token::Gt),
        (r"/", |_| Token::Leq),
        (r"/", |_| Token::Geq),
    ];
}

pub fn tokenize(mut code: &str, error_reporter: &mut ErrorReporter) -> Vec<TokenInfo> {
    let mut tokens = vec![];
    let mut loc = CodeLoc {
        line: 1,
        col: 1,
        index: 0,
    };

    // Change to char indexes?
    while loc.index < code.len() {
        remove_separators(&mut loc, code);

        if loc.index == code.len() {
            break;
        }

        match parse_token(code, &mut loc) {
            Some(token_info) => tokens.push(token_info),
            None => {
                error_reporter.error(
                    &loc,
                    &format!("Unexpected character: {}", &code[loc.index..loc.index + 1]),
                );
                error_reporter.has_error = true;
                loc.adv_col(1);
            }
        }
    }
    tokens.push(TokenInfo {
        token: Token::EOF,
        loc,
    });

    tokens
}

fn remove_separators(loc: &mut CodeLoc, code: &str) {
    while loc.index < code.len() {
        match &code[loc.index..loc.index + 1] {
            " " => loc.adv_col(1),
            "\n" => loc.adv_line(),
            _ => return,
        }
    }
}

fn parse_token(code: &str, loc: &mut CodeLoc) -> Option<TokenInfo> {
    // Basically I want a longest match regex tool.
    // But it is not part of the package, so instead we have this inefficient loop
    PATTERNS
        .iter()
        .filter_map(|(re, transform)| {
            // Can replace with find for speed
            re.captures(&code[loc.index..])
                .and_then(|caps| Some((caps[0].to_owned(), transform)))
        })
        .max_by_key(|(cap, _transform)| cap.len())
        .and_then(|(cap, transform)| {
            let token_loc = loc.clone();
            loc.adv_col(cap.len()); // Not very nice looking to mutate in here

            Some(TokenInfo {
                token: transform(&cap),
                loc: token_loc,
            })
        })
}

impl CodeLoc {
    fn adv_col(&mut self, nbr_chars: usize) {
        self.index += nbr_chars;
        self.col += nbr_chars;
    }

    fn adv_line(&mut self) {
        self.index += 1;
        self.line += 1;
        self.col = 1;
    }
}
