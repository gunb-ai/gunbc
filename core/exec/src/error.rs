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

/// Extension trait for converting any `Result<T, E>` into `Result<T, ExecError>`.
///
/// This trait provides `exec_context()` which converts any error type that implements
/// `Display` into an `ExecError` with a context message. This eliminates the common
/// boilerplate pattern:
///
/// ```ignore
/// // Before:
/// serde_json::from_str(&text)
///     .map_err(|e| ExecError::new(format!("JSON parse error: {}", e)))?;
///
/// // After:
/// use gunbc_exec::IntoExecResult;
/// serde_json::from_str(&text).exec_context("JSON parse error")?;
/// ```
pub trait IntoExecResult<T> {
    /// Convert error to `ExecError` with context.
    ///
    /// The resulting error message is formatted as `"context: original_error"`.
    fn exec_context(self, ctx: impl fmt::Display) -> Result<T, ExecError>;

    /// Convert error to `ExecError` with lazily-evaluated context.
    ///
    /// Use this when the context string needs to be computed (e.g., includes
    /// loop indices or other dynamic values).
    fn with_exec_context<F: FnOnce() -> String>(self, f: F) -> Result<T, ExecError>;
}

impl<T, E: fmt::Display> IntoExecResult<T> for Result<T, E> {
    fn exec_context(self, ctx: impl fmt::Display) -> Result<T, ExecError> {
        self.map_err(|e| ExecError::new(format!("{}: {}", ctx, e)))
    }

    fn with_exec_context<F: FnOnce() -> String>(self, f: F) -> Result<T, ExecError> {
        self.map_err(|e| ExecError::new(format!("{}: {}", f(), e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_error_context() {
        let err = ExecError::new("file not found");
        let with_ctx = err.context("reading config");
        assert_eq!(with_ctx.to_string(), "reading config: file not found");
    }

    #[test]
    fn result_ext_context() {
        let result: Result<(), ExecError> = Err(ExecError::new("file not found"));
        let with_ctx = result.context("reading config");
        assert_eq!(
            with_ctx.unwrap_err().to_string(),
            "reading config: file not found"
        );
    }

    #[test]
    fn result_ext_context_preserves_ok() {
        let result: Result<i32, ExecError> = Ok(42);
        let with_ctx = result.context("some context");
        assert_eq!(with_ctx.unwrap(), 42);
    }

    #[test]
    fn into_exec_result_converts_any_error() {
        // Simulate a serde error
        let result: Result<serde_json::Value, _> = serde_json::from_str("invalid json");
        let with_ctx = result.exec_context("JSON parse error");

        let err = with_ctx.unwrap_err();
        assert!(err.to_string().starts_with("JSON parse error:"));
        assert!(err.to_string().contains("expected"));
    }

    #[test]
    fn into_exec_result_preserves_ok() {
        let result: Result<i32, std::io::Error> = Ok(42);
        let with_ctx = result.exec_context("some context");
        assert_eq!(with_ctx.unwrap(), 42);
    }

    #[test]
    fn into_exec_result_with_string_error() {
        let result: Result<(), String> = Err("something went wrong".to_string());
        let with_ctx = result.exec_context("operation failed");
        assert_eq!(
            with_ctx.unwrap_err().to_string(),
            "operation failed: something went wrong"
        );
    }
}
