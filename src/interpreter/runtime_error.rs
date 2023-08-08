use std::fmt;

use crate::code_loc::CodeLoc;

use super::value::Value;

pub type RunRes<T> = Result<T, RunError>;

pub trait RunResTrait {
    // pub fn error<T>(reason: String) -> Self;

    /// Adds the location where an error could take place
    fn add_loc(self, start: CodeLoc, end: CodeLoc) -> Self;

    /// Adds a call to the stack trace
    fn add_trace(self, function: String, start: CodeLoc, end: CodeLoc) -> Self;
}

impl<T> RunResTrait for RunRes<T> {
    fn add_trace(mut self, function: String, start: CodeLoc, end: CodeLoc) -> Self {
        if let Err(RunError::Error(box trace)) = &mut self {
            trace.add_call(function, start, end);
        }
        self
    }

    fn add_loc(mut self, start: CodeLoc, end: CodeLoc) -> Self {
        if let Err(RunError::Error(box trace)) = &mut self {
            trace.add_loc(start, end);
        }
        self
    }
}

#[derive(Debug)]
pub enum RunError {
    Error(Box<Trace>),
    Break, // Maybe include code loc for error messages? Or just handle that with static analysis?
    Continue,
    Return(Value),
}

impl RunError {
    pub fn error<T>(reason: String) -> Result<T, Self> {
        Err(Self::bare_error(reason))
    }

    pub fn bare_error(reason: String) -> Self {
        RunError::Error(Box::new(Trace::new(reason)))
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
            .expect("Should always have a location for a complete trace");
        // The error origin
        // TODO: Add different error types
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
            writeln!(
                f,
                "    [{i}] {function} at {}:{}",
                start.line(),
                start.col()
            )?;
        }
        Ok(())
    }
}
