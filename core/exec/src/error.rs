//! Error types for execution.

use std::fmt;

/// Error during DAG execution.
#[derive(Debug, Clone)]
pub struct ExecError(pub String);

impl ExecError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }

    /// Add context to this error, producing a new error with the context prepended.
    ///
    /// ```ignore
    /// let err = ExecError::new("file not found");
    /// let with_ctx = err.context("reading config");
    /// assert_eq!(with_ctx.to_string(), "reading config: file not found");
    /// ```
    pub fn context(self, ctx: impl fmt::Display) -> Self {
        Self(format!("{}: {}", ctx, self.0))
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

/// Extension trait for adding context to `Result<T, ExecError>`.
///
/// ```ignore
/// use gunbc_exec::{ExecError, ResultExt};
///
/// fn load_config() -> Result<String, ExecError> {
///     Err(ExecError::new("file not found"))
/// }
///
/// let result = load_config().context("loading config");
/// assert_eq!(result.unwrap_err().to_string(), "loading config: file not found");
/// ```
pub trait ResultExt<T> {
    /// Add context to the error case.
    fn context(self, ctx: impl fmt::Display) -> Result<T, ExecError>;

    /// Add lazily-evaluated context to the error case.
    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T, ExecError>;
}

impl<T> ResultExt<T> for Result<T, ExecError> {
    fn context(self, ctx: impl fmt::Display) -> Result<T, ExecError> {
        self.map_err(|e| e.context(ctx))
    }

    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T, ExecError> {
        self.map_err(|e| e.context(f()))
    }
}
