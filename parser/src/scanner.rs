use crate::code_loc::CodeLoc;
use lazy_static::lazy_static;
use regex::Regex;
use std::rc::Rc;

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
    String(Rc<String>),
    Comment(String),
    // Invalid(String),
    /// The name of a macro invocation, such as "include!" in "include!(path/to/file)"
    MacroInvocation(String),

    // Special constructs
    Struct,
    Fn,
    If,
    Else,
    Match,
    Return,
    For,
    While,
    // Var, // Might want to change?
    Break,
    Continue,
    // In,
    RArrow,
    // WideRArrow,
    BackslashPipe,

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
    Backslash,

    Eof,

    True,
    False,
    And,
    Or,
    Plus,
    Minus,
    Mult,
    Div,
    ColonEq,
    DoubleEq,
    BangEq,
    Lt,
    Gt,
    Leq,
    Geq,
    Nil,
    Pipe,
    // ColonPipe,
    // EqPipe,
    DoublePlus,
}

#[derive(Debug)]
pub struct TokenInfo {
    pub token: Token,
    pub start_loc: CodeLoc,
    pub end_loc: CodeLoc,
    pub string: String,  // Another slow thing. Could use a reference here...
    pub seperated: bool, // Is there some seperation (\n or " " eg) before the token? Used for separating calls/indexing
}

type Constructor = Box<dyn Fn(&str) -> Token + Sync>;
lazy_static! {
    static ref PATTERNS: Vec<(Regex, Constructor)> = lex_rules![
        (r"[\w--\d]\w*", |str| Token::Identifier(str.to_string())),
        // Not optimal that 001 is scanned as 0 0 1
        (r"(([1-9]\d*\.\d+)|(0\.\d+))", |str| Token::Float(str.parse().unwrap())),
        (r"([1-9]\d*|0)", |str| Token::Integer(str.parse().unwrap())),
        (r#"".*?""#, |str| Token::String(Rc::new(parse_string(&str[1..str.len()-1])))),
        (r#"'.*?'"#, |str| Token::String(Rc::new(parse_string(&str[1..str.len()-1])))),
        (r"//[^\n]*", |str| Token::Comment(str[2..].to_owned())),
        (r"[\w--\d]\w*!", |str| Token::MacroInvocation(str.to_string())),
        (r"struct", |_| Token::Struct),
        (r"fn", |_| Token::Fn),
        (r"if", |_| Token::If),
        (r"else", |_| Token::Else),
        (r"match", |_| Token::Match),
        (r"return", |_| Token::Return),
        (r"for", |_| Token::For),
        (r"while", |_| Token::While),
        // (r"var", |_| Token::Var),
        (r"break", |_| Token::Break),
        (r"continue", |_| Token::Continue),
        // (r"in", |_| Token::In),
        (r"->", |_| Token::RArrow),
        // (r"=>", |_| Token::WideRArrow),
        (r"\\>>", |_| Token::BackslashPipe),
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
        (r"\\", |_| Token::Backslash),
        (r"true", |_| Token::True),
        (r"false", |_| Token::False),
        (r"and", |_| Token::And),
        (r"or", |_| Token::Or),
        (r"\+", |_| Token::Plus),
        (r"-", |_| Token::Minus),
        (r"\*", |_| Token::Mult),
        (r"/", |_| Token::Div),
        (r":=", |_| Token::ColonEq),
        (r"==", |_| Token::DoubleEq),
        (r"!=", |_| Token::BangEq),
        (r"<", |_| Token::Lt),
        (r">", |_| Token::Gt),
        (r"<=", |_| Token::Leq),
        (r">=", |_| Token::Geq),
        (r"nil", |_| Token::Nil),
        (r">>", |_| Token::Pipe),
        // (r":>>", |_| Token::ColonPipe),
        // (r"=>>", |_| Token::EqPipe),
        (r"\+\+", |_| Token::DoublePlus),
    ];
}

pub fn tokenize(
    code: &str,
    scriptname: &str,
    error_reporter: &mut ErrorReporter,
) -> Vec<TokenInfo> {
    let mut tokens = vec![];
    let mut loc = CodeLoc::new(0, 1, 1);

    // Change to char indexes?
    while loc.index() < code.len() {
        let seperated = remove_separators(&mut loc, code);

        if loc.index() == code.len() {
            break;
        }

        match parse_token(code, &mut loc, seperated) {
            // For now just ignore all comments
            Some(token_info) if matches!(token_info.token, Token::Comment(_)) => continue,
            Some(token_info) => tokens.push(token_info),
            None => {
                let scanned = &code[loc.index()..].chars().next().unwrap();
                error_reporter.scan_error(
                    &loc,
                    &format!("Unexpected character: {}", scanned),
                    scriptname,
                );
                loc.adv_col(1, scanned.len_utf8());
            }
        }
    }
    tokens.push(TokenInfo {
        token: Token::Eof,
        start_loc: loc,
        end_loc: loc,
        string: "EOF".to_string(),
        seperated: true,
    });

    tokens
}

fn remove_separators(loc: &mut CodeLoc, code: &str) -> bool {
    let mut matched = false;
    while loc.index() < code.len() {
        match &code[loc.index()..].chars().next() {
            Some(' ') | Some('\t') | Some('\r') => {
                loc.adv_col(1, 1);
                matched = true;
            }
            Some('\n') => {
                loc.adv_line();
                matched = true;
            }
            _ => break,
        }
    }
    matched
}

fn parse_token(code: &str, loc: &mut CodeLoc, sep: bool) -> Option<TokenInfo> {
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
                seperated: sep,
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
            "test",
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
        let tokens = tokenize("123.123123.123 1234 0123.00 123.412", "test", &mut reporter);

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
        let tokens = tokenize(code, "test", &mut reporter);

        let expected_tokens = vec![
            Token::String(Rc::new("first".to_string())),
            Token::String(Rc::new("secondthird".to_string())),
            Token::String(Rc::new("'inner'".to_string())),
            Token::String(Rc::new("\"inner2\"".to_string())),
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
        let tokens = tokenize(code, "test", &mut reporter);

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
