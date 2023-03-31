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

#[derive(Debug, Clone, PartialEq)]
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
    Bang,

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
    BangEq,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeLoc {
    pub line: usize,
    pub col: usize,
    pub index: usize,
    pub index_end: usize,
}

lazy_static! {
    static ref PATTERNS: Vec<(Regex, Box<dyn Fn(&str) -> Token + Sync>)> = lex_rules![
        (r"[\w--\d]\w*", |str| Token::Identifier(str.to_owned())),
        // Not optimal that 001 is scanned as 0 0 1
        (r"(([1-9]\d*\.\d+)|(0\.\d+))", |str| Token::Float(str.parse().unwrap())),
        (r"([1-9]\d*|0)", |str| Token::Integer(str.parse().unwrap())),
        (r#"".*?""#, |str| Token::String(str[1..str.len()-1].to_owned())),
        (r#"'.*?'"#, |str| Token::String(str[1..str.len()-1].to_owned())),
        (r"//[^\n]*", |str| Token::Comment(str[2..].to_owned())),
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
        (r"!", |_| Token::Bang),
        (r"true", |_| Token::True),
        (r"false", |_| Token::False),
        (r"and", |_| Token::And),
        (r"or", |_| Token::Or),
        (r"\+", |_| Token::Plus),
        (r"-", |_| Token::Minus),
        (r"\*", |_| Token::Mult),
        (r"/", |_| Token::Div),
        (r"==", |_| Token::DoubleEq),
        (r"!=", |_| Token::BangEq),
        (r"<", |_| Token::Lt),
        (r">", |_| Token::Gt),
        (r"<=", |_| Token::Leq),
        (r">=", |_| Token::Geq),
    ];
}

pub fn tokenize(code: &str, error_reporter: &mut ErrorReporter) -> Vec<TokenInfo> {
    let mut tokens = vec![];
    let mut loc = CodeLoc {
        line: 1,
        col: 1,
        index: 0,
        index_end: 0,
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
                let scanned = &code[loc.index..].chars().next().unwrap();
                error_reporter.error(&loc, &format!("Unexpected character: {}", scanned));
                error_reporter.has_error = true;
                loc.adv_col(scanned.len_utf8());
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
        match &code[loc.index..].chars().next() {
            Some(' ') => loc.adv_col(1),
            Some('\n') => loc.adv_line(),
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
            loc.index_end = loc.index + cap.len() - 1;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers() {
        let mut reporter = ErrorReporter::new();
        let tokens = tokenize(
            "hejsor for_forforwhile\n notersteWeanfnåÅö áßãåãåøœđéđł",
            &mut reporter,
        );

        let expected_tokens = vec![
            Token::Identifier("hejsor".to_string()),
            Token::Identifier("for_forforwhile".to_string()),
            Token::Identifier("notersteWeanfnåÅö".to_string()),
            Token::Identifier("áßãåãåøœđéđł".to_string()),
            Token::EOF,
        ];
        let scanned_tokens: Vec<_> = tokens.iter().map(|info| info.token.clone()).collect();
        assert_eq!(scanned_tokens, expected_tokens);

        let first_loc = CodeLoc {
            line: 1,
            col: 1,
            index: 0,
            index_end: 5,
        };
        assert_eq!(&tokens[0].loc, &first_loc);

        let third_loc = CodeLoc {
            line: 2,
            col: 2,
            index: 24,
            index_end: 24 + 13 + 3 * 2,
        };
        assert_eq!(&tokens[2].loc, &third_loc);
    }

    #[test]
    fn numbers() {
        let mut reporter = ErrorReporter::new();
        let tokens = tokenize("123.123123.123 1234 0123.00 123.412", &mut reporter);

        // The double dot and especially leading 0 are a bit strange and may be changed
        let expected_tokens = vec![
            Token::Dot,
            Token::Integer(123),
            Token::Integer(1234),
            Token::Integer(0),
        ];
        let scanned_tokens: Vec<_> = tokens[1..5].iter().map(|info| info.token.clone()).collect();
        assert_eq!(scanned_tokens, expected_tokens);

        let third_loc = CodeLoc {
            line: 1,
            col: 12,
            index: 11,
            index_end: 13,
        };
        assert_eq!(&tokens[2].loc, &third_loc);
    }

    #[test]
    fn strings() {
        let mut reporter = ErrorReporter::new();
        // Cannot have a newline inside of a string
        let code = "\"first\" \n'secondthird' \"'inner'\" '\"inner2\"'";
        let tokens = tokenize(code, &mut reporter);

        let expected_tokens = vec![
            Token::String("first".to_string()),
            Token::String("secondthird".to_string()),
            Token::String("'inner'".to_string()),
            Token::String("\"inner2\"".to_string()),
            Token::EOF,
        ];
        let scanned_tokens: Vec<_> = tokens.iter().map(|info| info.token.clone()).collect();
        assert_eq!(scanned_tokens, expected_tokens);

        let third_loc = CodeLoc {
            line: 2,
            col: 15,
            index: 15 + 9 - 1,
            index_end: 23 + 8,
        };
        assert_eq!(&tokens[2].loc, &third_loc);
    }

    #[test]
    fn mixed() {
        let mut reporter = ErrorReporter::new();
        let code = "// Test []!\nif {+ = -} (==) else [match return for while .,;true false and or */ <> <=>=]";
        let tokens = tokenize(code, &mut reporter);

        let expected_tokens = vec![
            Token::Comment(" Test []!".to_string()),
            Token::If,
            Token::LBrace,
            Token::Plus,
            Token::Eq,
            Token::Minus,
            Token::RBrace,
            Token::LPar,
            Token::DoubleEq,
            Token::RPar,
            Token::Else,
            Token::LBrack,
            Token::Match,
            Token::Return,
            Token::For,
            Token::While,
            Token::Dot,
            Token::Comma,
            Token::Semicolon,
            Token::True,
            Token::False,
            Token::And,
            Token::Or,
            Token::Mult,
            Token::Div,
            Token::Lt,
            Token::Gt,
            Token::Leq,
            Token::Geq,
            Token::RBrack,
            Token::EOF,
        ];
        let scanned_tokens: Vec<_> = tokens.iter().map(|info| info.token.clone()).collect();
        assert_eq!(scanned_tokens, expected_tokens);
    }
}
