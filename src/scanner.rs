use crate::code_loc::CodeLoc;
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
    Struct,
    Fn,
    If,
    Else,
    Match,
    Return,
    For,
    While,
    Var, // Might want to change?
    Break,
    // In,
    RArrow,

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
    UpArr,
    Percent,
    Colon,

    Eof,

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
    Nil,
    Pipe,
    PipeColon,
    DoublePlus,
}

#[derive(Debug)]
pub struct TokenInfo {
    pub token: Token,
    pub start_loc: CodeLoc,
    pub end_loc: CodeLoc,
    pub string: String, // Another slow thing. Could use a reference here...
}

type Constructor = Box<dyn Fn(&str) -> Token + Sync>;
lazy_static! {
    static ref PATTERNS: Vec<(Regex, Constructor)> = lex_rules![
        (r"[\w--\d]\w*", |str| Token::Identifier(str.to_string())),
        // Not optimal that 001 is scanned as 0 0 1
        (r"(([1-9]\d*\.\d+)|(0\.\d+))", |str| Token::Float(str.parse().unwrap())),
        (r"([1-9]\d*|0)", |str| Token::Integer(str.parse().unwrap())),
        (r#"".*?""#, |str| Token::String(parse_string(&str[1..str.len()-1]))),
        (r#"'.*?'"#, |str| Token::String(parse_string(&str[1..str.len()-1]))),
        (r"//[^\n]*", |str| Token::Comment(str[2..].to_owned())),
        (r"struct", |_| Token::Struct),
        (r"fn", |_| Token::Fn),
        (r"if", |_| Token::If),
        (r"else", |_| Token::Else),
        (r"match", |_| Token::Match),
        (r"return", |_| Token::Return),
        (r"for", |_| Token::For),
        (r"while", |_| Token::While),
        (r"var", |_| Token::Var),
        (r"break", |_| Token::Break),
        // (r"in", |_| Token::In),
        (r"->", |_| Token::RArrow),
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
        (r"\^", |_| Token::UpArr),
        (r"%", |_| Token::Percent),
        (r":", |_| Token::Colon),
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
        (r"Nil", |_| Token::Nil),
        (r">>", |_| Token::Pipe),
        (r">>:", |_| Token::PipeColon),
        (r"\+\+", |_| Token::DoublePlus),
    ];
}

pub fn tokenize(code: &str, error_reporter: &mut ErrorReporter) -> Vec<TokenInfo> {
    let mut tokens = vec![];
    let mut loc = CodeLoc::new(0, 1, 1);

    // Change to char indexes?
    while loc.index() < code.len() {
        remove_separators(&mut loc, code);

        if loc.index() == code.len() {
            break;
        }

        match parse_token(code, &mut loc) {
            // For now just ignore all comments
            Some(token_info) if matches!(token_info.token, Token::Comment(_)) => continue,
            Some(token_info) => tokens.push(token_info),
            None => {
                let scanned = &code[loc.index()..].chars().next().unwrap();
                error_reporter.error(&loc, &format!("Unexpected character: {}", scanned));
                error_reporter.had_compilation_error = true;
                loc.adv_col(1, scanned.len_utf8());
            }
        }
    }
    tokens.push(TokenInfo {
        token: Token::Eof,
        start_loc: loc,
        end_loc: loc,
        string: "EOF".to_string(),
    });

    tokens
}

fn remove_separators(loc: &mut CodeLoc, code: &str) {
    while loc.index() < code.len() {
        match &code[loc.index()..].chars().next() {
            Some(' ') | Some('\t') | Some('\r') => loc.adv_col(1, 1),
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
            re.captures(&code[loc.index()..])
                .map(|caps| (caps[0].to_owned(), transform))
        })
        .max_by_key(|(cap, _transform)| cap.len())
        .map(|(cap, transform)| {
            let start_loc = *loc;
            loc.adv_col(cap.chars().count(), cap.len()); // Not very nice looking to mutate in here
            let end_loc = *loc;

            TokenInfo {
                token: transform(&cap),
                start_loc,
                end_loc,
                string: cap.to_string(),
            }
        })
}

// Want to parse escape sequences properly
fn parse_string(string: &str) -> String {
    string
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
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
            Token::Eof,
        ];
        let scanned_tokens: Vec<_> = tokens.iter().map(|info| info.token.clone()).collect();
        assert_eq!(scanned_tokens, expected_tokens);

        let first_start = CodeLoc::new(0, 1, 1);
        assert_eq!(&tokens[0].start_loc, &first_start);

        let third_start = CodeLoc::new(24, 2, 2);
        let third_end = CodeLoc::new(44, 2, 19);
        assert_eq!(&tokens[2].start_loc, &third_start);
        assert_eq!(&tokens[2].end_loc, &third_end);
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

        let third_start = CodeLoc::new(11, 1, 12);
        let third_end = CodeLoc::new(14, 1, 15);
        assert_eq!(&tokens[2].start_loc, &third_start);
        assert_eq!(&tokens[2].end_loc, &third_end);
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
            Token::Eof,
        ];
        let scanned_tokens: Vec<_> = tokens.iter().map(|info| info.token.clone()).collect();
        assert_eq!(scanned_tokens, expected_tokens);

        let third_start = CodeLoc::new(23, 2, 15);
        let third_end = CodeLoc::new(32, 2, 24);
        assert_eq!(&tokens[2].start_loc, &third_start);
        assert_eq!(&tokens[2].end_loc, &third_end);
    }

    #[test]
    fn mixed() {
        let mut reporter = ErrorReporter::new();
        let code = "// Test []!\nif {+ = -} (==) else [match return for while .,;true false and or */ <> <=>=]Nil>>";
        let tokens = tokenize(code, &mut reporter);

        let expected_tokens = vec![
            // Token::Comment(" Test []!".to_string()), // No longer emitted as tokens
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
            Token::Nil,
            Token::Pipe,
            Token::Eof,
        ];
        let scanned_tokens: Vec<_> = tokens.iter().map(|info| info.token.clone()).collect();
        assert_eq!(scanned_tokens, expected_tokens);
    }
}
