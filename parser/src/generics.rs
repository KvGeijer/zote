use std::fmt::Debug;

use super::{AstNode, Parser};
use crate::code_loc::CodeLoc;
use crate::errors::ErrorReporter;
use crate::scanner::{Token, TokenInfo};

// Module with different helper functions for the parsing.

impl<T: Debug> AstNode<T> {
    // This should not be visible outside the parser right, as this module is not pub.
    pub fn new(node: T, start_loc: CodeLoc, end_loc: CodeLoc) -> AstNode<T> {
        Self {
            node: Box::new(node),
            start_loc,
            end_loc,
        }
    }
}

impl<T: PartialEq> PartialEq for AstNode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<'a> Parser<'a> {
    pub fn new(
        filename: &'a str,
        tokens: &'a [TokenInfo],
        error_reporter: &'a mut ErrorReporter,
    ) -> Self {
        Self {
            scriptname: filename,
            tokens: Vec::from_iter(tokens.iter()),
            current: 0,
            error_reporter,
        }
    }

    pub fn error(&mut self, str: &str) {
        // Should probably have the error reporter outside of the parser...
        let loc = *self.peek_start_loc();
        let error_string = format!("{} at '{}' {}", loc.line(), &self.peek_string(), str);
        self.error_reporter
            .comp_error(&loc, &error_string, &self.scriptname);
    }

    pub fn synchronize_error(&mut self) {
        while !self.at_end() {
            // If we are at semicolon we take it and advance
            if self.take() == &Token::Semicolon {
                return;
            }

            // If the next token is the start of a new statement (declaration) we can also return
            match self.peek() {
                Token::Fn => return,
                _ => continue,
            }
        }
    }

    pub fn match_token(&mut self, token: Token) -> bool {
        if self.peek() == &token {
            if self.take() != &token {
                panic!("Internal error at match_token");
            }
            true
        } else {
            false
        }
    }

    // pub fn match_identifier(&mut self) -> Option<String> {
    //     if matches!(self.peek(), Token::Identifier(_)) {
    //         if let Token::Identifier(id) = self.take() {
    //             Some(id.clone())
    //         } else {
    //             panic!("Internal error at match_token");
    //         }
    //     } else {
    //         None
    //     }
    // }

    pub fn at_end(&self) -> bool {
        self.current == self.tokens.len() - 1
    }

    pub fn peek_info(&self) -> &TokenInfo {
        self.tokens[self.current]
    }

    pub fn peek(&self) -> &Token {
        &self.peek_info().token
    }

    pub fn peek2(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1).map(|info| &info.token)
    }

    pub fn peek_start_loc(&self) -> &CodeLoc {
        &self.peek_info().start_loc
    }

    pub fn peek_end_loc(&self) -> &CodeLoc {
        &self.peek_info().end_loc
    }

    pub fn peek_last_end_loc(&self) -> Option<&CodeLoc> {
        Some(&self.tokens.get(self.current - 1)?.end_loc)
    }

    pub fn peek_string(&self) -> &str {
        &self.peek_info().string
    }

    pub fn accept(&mut self, expected: Token, error_str: &str) -> Option<()> {
        if self.match_token(expected) {
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
