use crate::code_loc::CodeLoc;

pub struct ErrorReporter {
    // Add struct for many errors
    pub had_compilation_error: bool,
    pub had_runtime_error: bool,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            had_compilation_error: false,
            had_runtime_error: false,
        }
    }

    pub fn reset(&mut self) {
        self.had_compilation_error = false;
        self.had_runtime_error = false;
    }

    pub fn runtime_error(&mut self, start: &CodeLoc, end: &CodeLoc, message: &str) {
        self.had_runtime_error = true;
        eprintln!(
            "ERROR [{}:{} - {}-{}] {message}",
            start.line(),
            start.col(),
            end.line(),
            end.col()
        )
    }

    // Should be expanded and changed when more is clear
    pub fn error(&mut self, loc: &CodeLoc, message: &str) {
        self.report(loc, "", message);
    }

    fn report(&mut self, loc: &CodeLoc, place: &str, message: &str) {
        eprintln!(
            "[line: {}, col: {}] Error{place}: {message}",
            loc.line(),
            loc.col()
        );

        self.had_compilation_error = true;
    }

    pub fn runtime_panic(&mut self, message: &str) {
        self.had_runtime_error = true;
        eprintln!("Panic! {message}")
    }
}
