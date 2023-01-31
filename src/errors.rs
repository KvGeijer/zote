pub struct ErrorReporter {
    // Add struct for many errors
    has_error: bool,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self { has_error: false }
    }

    pub fn reset(&mut self) {
        self.has_error = false;
    }

    // Should be expanded and changed when more is clear
    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    fn report(&mut self, line: usize, place: &str, message: &str) {
        eprintln!("[line {line}] Error{place}: {message}");
        self.has_error = true;
    }
}
