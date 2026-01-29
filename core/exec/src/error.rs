//! Error types for execution.

use std::fmt;

/// Error during DAG execution.
#[derive(Debug, Clone)]
pub struct ExecError(pub String);

impl ExecError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ExecError {}

impl From<String> for ExecError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ExecError {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
