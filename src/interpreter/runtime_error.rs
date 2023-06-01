use crate::code_loc::CodeLoc;

use super::expressions::Value;

pub type RunRes<T> = Result<T, RuntimeError>;
#[derive(Debug)]
pub enum RuntimeError {
    Error(CodeLoc, CodeLoc, String),
    ErrorReason(String), // Should combine this with Error to create a call stack starting with one of these, then just codelocs and extra info at each proper call
    Break, // Maybe include code loc for error messages? Or just handle that with static analysis?
    Return(Value),
}
