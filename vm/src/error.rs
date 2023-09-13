use std::fmt;

use parser::CodeLoc;

pub type RunRes<T> = Result<T, RuntimeError>;

pub trait RunResTrait {
    // pub fn error<T>(reason: String) -> Self;

    /// Adds the location where an error could take place
    fn add_loc(self, start: CodeLoc, end: CodeLoc) -> Self;

    /// Adds a call to the stack trace
    fn add_trace(self, function: String, start: CodeLoc, end: CodeLoc) -> Self;

    /// Create a new bare error
    fn new_err(reason: String) -> Self;
}

impl<T> RunResTrait for RunRes<T> {
    fn add_trace(mut self, function: String, start: CodeLoc, end: CodeLoc) -> Self {
        if let Err(RuntimeError { trace }) = &mut self {
            trace.add_call(function, start, end);
        }
        self
    }

    fn add_loc(mut self, start: CodeLoc, end: CodeLoc) -> Self {
        if let Err(RuntimeError { trace }) = &mut self {
            trace.add_loc(start, end);
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
    loc: Option<(CodeLoc, CodeLoc)>,
    stack_trace: Vec<(String, CodeLoc, CodeLoc)>,
}

impl Trace {
    fn new(reason: String) -> Self {
        Self {
            reason,
            loc: None,
            stack_trace: vec![],
        }
    }

    fn add_loc(&mut self, start: CodeLoc, end: CodeLoc) {
        if self.loc.is_none() {
            self.loc = Some((start, end));
        }
    }

    fn add_call(&mut self, function: String, start: CodeLoc, end: CodeLoc) {
        self.stack_trace.push((function, start, end));
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (start, end) = self
            .loc
            .expect("Should always have an original location for a complete trace");

        // TODO: Change up how these errors work when functions are implemented in vm
        writeln!(
            f,
            "ERROR [{}:{} - {}:{}] {}",
            start.line(),
            start.col(),
            end.line(),
            end.col(),
            self.reason
        )?;
        for (i, (function, start, _end)) in self.stack_trace.iter().enumerate() {
            writeln!(f, "    ({i}) [{}:{}] {function}", start.line(), start.col())?;
        }
        Ok(())
    }
}
