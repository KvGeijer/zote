use std::fs::read;

use crate::{parse, scanner::Token, Parser, StmtNode};

impl<'a> Parser<'a> {
    /// Parse a whole other file to a sequence of statements
    pub(crate) fn macro_include_statement(&mut self) -> Option<Vec<StmtNode>> {
        self.accept(Token::LPar, "Expect parenthesis after 'include!'")?;

        let Token::String(path) = self.take() else {
            self.error("Expect file path as string in include macro");
            return None;
        };
        let path = path.clone();

        // TODO: This is a really bad way to do this. Stdlib should always be included for the vm in some nice way...
        let included_bytes = match path.as_str() {
            "stdlib" => include_str!("../../vm/stdlib.zote").as_bytes().to_vec(),
            otherwise => {
                let Ok(included_bytes) = read(otherwise) else {
                    self.error(&format!("Could not open {path} for including code"));
                    return None;
                };
                included_bytes
            }
        };

        let Ok(included_code) = String::from_utf8(included_bytes) else {
            self.error(&format!("Could not read {path} as utf8 for including code"));
            return None;
        };

        self.accept(
            Token::RPar,
            "Expect parenthesis after file path in 'include!'",
        )?;

        let statements = parse(&path, &included_code)?;
        if statements.output {
            self.error(&format!(
                "Cannot import file with implicit output of last statement. In file {path}."
            ));
            None
        } else {
            Some(statements.stmts)
        }
    }
}
