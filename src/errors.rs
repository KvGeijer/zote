use crate::scanner::CodeLoc;

pub struct ErrorReporter {
    // Add struct for many errors
    pub has_error: bool,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self { has_error: false }
    }

    pub fn reset(&mut self) {
        self.has_error = false;
    }

    // Should be expanded and changed when more is clear
    pub fn error(&mut self, loc: &CodeLoc, message: &str) {
        self.report(loc, "", message);
    }

    fn report(&mut self, loc: &CodeLoc, place: &str, message: &str) {
        eprintln!(
            "[line: {}, col: {}] Error{place}: {message}",
            loc.line, loc.col
        );

        self.has_error = true;
    }
}
