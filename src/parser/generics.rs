use std::fmt::Debug;

use super::{AstLoc, AstNode, Parser};
use crate::errors::ErrorReporter;
use crate::scanner::{CodeLoc, Token, TokenInfo};

// Module with different helper functions for the parsing.

impl<T: Debug> AstNode<T> {
    // This should not be visible outside the parser right, as this module is not pub.
    pub fn new<F: Into<AstLoc>>(node: T, start: F, end: F) -> AstNode<T> {
        let start_astloc: AstLoc = start.into();
        let end_astloc: AstLoc = end.into();
        let loc = AstLoc::new(
            start_astloc.row_start(),
            end_astloc.row_end(),
            start_astloc.col_start(),
            end_astloc.col_end(),
        );
        AstNode { node, loc }
    }
}

impl<T: PartialEq> PartialEq for AstNode<T> {
    fn eq(&self, other: &Self) -> bool {
        &self.node == &other.node
    }
}

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
                Token::Var
                | Token::Struct
                | Token::Fn
                | Token::For
                | Token::While
                | Token::Return => return,
                _ => continue,
            }
        }
    }

    pub fn at_end(&self) -> bool {
        self.current == self.tokens.len() - 1
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

    pub fn accept(&mut self, expected: Token, error_str: &str) -> Option<()> {
        if self.peek() == &expected {
            self.take();
            Some(())
        } else {
            self.error(error_str);
            None
        }
    }

    pub fn accept_peek(&mut self, expected: Token, error_str: &str) -> Option<()> {
        if self.peek() == &expected {
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
