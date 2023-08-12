pub struct ErrorReporter {
    pub had_error: bool,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self { had_error: false }
    }

    pub fn reset(&mut self) {
        self.had_error = false;
    }

    // TODO: This is bad
    pub fn runtime_error(&mut self, message: &str) {
        self.had_error = true;
        eprintln!("{message}");
    }

    pub fn runtime_panic(&mut self, message: &str) {
        self.had_error = true;
        eprintln!("Panic! {message}")
    }
}
