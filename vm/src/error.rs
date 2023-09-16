use std::fmt;

use parser::CodeRange;

pub type RunRes<T> = Result<T, RuntimeError>;

pub trait RunResTrait {
    /// Adds a call to the stack trace
    fn add_trace(self, function: String, range: CodeRange) -> Self;

    /// Create a new bare error
    fn new_err(reason: String) -> Self;
}

impl<T> RunResTrait for RunRes<T> {
    fn add_trace(mut self, function: String, range: CodeRange) -> Self {
        if let Err(RuntimeError { trace }) = &mut self {
            trace.add_call(function, range);
        }
        self
    }

    fn new_err(reason: String) -> Self {
        RuntimeError::error(reason)
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    trace: Box<Trace>,
}

impl RuntimeError {
    pub fn error<T>(reason: String) -> Result<T, Self> {
        Err(Self::bare_error(reason))
    }

    pub fn bare_error(reason: String) -> Self {
        Self {
            trace: Box::new(Trace::new(reason)),
        }
    }
}

#[derive(Debug)]
pub struct Trace {
    reason: String,
    stack_trace: Vec<(String, CodeRange)>,
}

impl Trace {
    fn new(reason: String) -> Self {
        Self {
            reason,
            stack_trace: vec![],
        }
    }

    fn add_call(&mut self, function: String, range: CodeRange) {
        self.stack_trace.push((function, range));
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Make this really nice, with error types
        writeln!(f, "ERROR: {}", self.reason)?;
        // Write exact location for first error?

        for (i, (function, range)) in self.stack_trace.iter().enumerate() {
            writeln!(f, "    ({i}) [line {}] in {function}", range.sl())?;
        }
        Ok(())
    }
}
