use super::Parser;
use crate::errors::ErrorReporter;
use crate::scanner::{CodeLoc, Token, TokenInfo};

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [TokenInfo], error_reporter: &'a mut ErrorReporter) -> Self {
        Self {
            tokens: Vec::from_iter(tokens.iter()),
            current: 0,
            error_reporter,
        }
    }

    pub fn error(&mut self, str: &str) {
        // Ugly, makes sense why borrow checker argues unless copy, but still...
        // Should probably have the error reporter outside of the parser...
        let loc = self.peek_loc().clone();
        let error_string = format!("{} at '{}' {}", loc.line, &self.peek_string(), str);
        self.error_reporter.error(&loc, &error_string);
    }

    pub fn synchronize_error(&mut self) {
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

    pub fn at_end(&self) -> bool {
        self.current == self.tokens.len()
    }

    pub fn peek_info(&self) -> &TokenInfo {
        &self.tokens[self.current]
    }

    pub fn peek(&self) -> &Token {
        &self.peek_info().token
    }

    pub fn peek_loc(&self) -> &CodeLoc {
        &self.peek_info().loc
    }

    pub fn peek_string(&self) -> &str {
        &self.peek_info().string
    }

    pub fn accept(&mut self, expected: Token, error_str: &str, take: bool) -> Option<()> {
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

    pub fn take(&mut self) -> &Token {
        let ind = self.current;
        if !self.at_end() {
            self.current += 1
        }
        &self.tokens[ind].token
    }
}
