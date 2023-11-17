use std::fmt;

pub type RunRes<T> = Result<T, RuntimeError>;

pub trait RunResTrait {
    /// Create a new bare error
    fn new_err(reason: String) -> Self;
}

impl<T> RunResTrait for RunRes<T> {
    fn new_err(reason: String) -> Self {
        RuntimeError::error(reason)
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    /// The bottommost error reason
    reason: Box<String>,
}

impl RuntimeError {
    pub fn error<T>(reason: String) -> Result<T, Self> {
        Err(Self::bare_error(reason))
    }

    pub fn bare_error(reason: String) -> Self {
        Self {
            reason: Box::new(reason),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}
