use crate::code_loc::CodeLoc;

pub struct ErrorReporter {
    pub had_error: bool,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self { had_error: false }
    }

    pub fn scan_error(&mut self, loc: &CodeLoc, message: &str, scriptname: &str) {
        self.error("Scanning", loc, message, scriptname)
    }

    pub fn comp_error(&mut self, loc: &CodeLoc, message: &str, scriptname: &str) {
        self.error("Compilation", loc, message, scriptname)
    }

    fn error(&mut self, error_type: &str, loc: &CodeLoc, message: &str, scriptname: &str) {
        eprintln!(
            "[line: {}, col: {}] {error_type} Error: {message}. In {scriptname}",
            loc.line(),
            loc.col()
        );

        self.had_error = true;
    }
}
