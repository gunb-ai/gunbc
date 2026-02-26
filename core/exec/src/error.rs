//! Structured error types for execution.
//!
//! Errors carry an ordered stack of [`ErrorLayer`] values that record which
//! transport/protocol layers were involved when the failure occurred. The
//! [`classify_layers`] function scans **all** layers and returns a
//! human-readable classification string by priority order.

use std::fmt;

// ---------------------------------------------------------------------------
// ErrorLayer + per-layer structs
// ---------------------------------------------------------------------------

/// A single layer in the error's diagnostic stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorLayer {
    Http(HttpErrorLayer),
    Rest(RestErrorLayer),
    Auth(AuthErrorLayer),
    Service(ServiceErrorLayer),
    Shell(ShellErrorLayer),
    File(FileErrorLayer),
}

/// HTTP-level failure information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpErrorLayer {
    pub status_code: u16,
    pub reason: Option<String>,
}

/// REST-level failure information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestErrorLayer {
    pub endpoint: String,
    pub method: String,
}

/// Auth-level failure information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthErrorLayer {
    pub scheme: String,
    pub credential_ref: Option<String>,
}

/// Service-level failure information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceErrorLayer {
    pub provider: String,
    pub operation: String,
}

/// Shell-level failure information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellErrorLayer {
    pub command: String,
    pub exit_code: Option<i32>,
}

/// File-level failure information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileErrorLayer {
    pub path: String,
    pub operation: String,
}

// ---------------------------------------------------------------------------
// classify_layers — priority-based, full-scan
// ---------------------------------------------------------------------------

/// Scan **all** layers and return a classification string by priority.
///
/// Priority order (highest first):
/// 1. Auth → `"AUTH"`
/// 2. Shell → `"SHELL"`
/// 3. File → `"FILE"`
/// 4. Http-specific:
///    - 429 → `"RATE_LIMIT"`
///    - 404 → `"NOT_FOUND"`
///    - 500+ → `"SERVER_ERROR"`
/// 5. Fallback → `"UNKNOWN"`
///
/// The function scans every layer first (setting boolean flags), then returns
/// by priority order — it does **not** short-circuit on the first match.
pub fn classify_layers(layers: &[ErrorLayer]) -> &'static str {
    let mut has_auth = false;
    let mut has_shell = false;
    let mut has_file = false;
    let mut http_status: Option<u16> = None;

    for layer in layers {
        match layer {
            ErrorLayer::Auth(_) => has_auth = true,
            ErrorLayer::Shell(_) => has_shell = true,
            ErrorLayer::File(_) => has_file = true,
            ErrorLayer::Http(h) => {
                // Keep the last HTTP status seen (could also keep highest).
                http_status = Some(h.status_code);
            }
            ErrorLayer::Rest(_) | ErrorLayer::Service(_) => {}
        }
    }

    // Return by priority.
    if has_auth {
        return "AUTH";
    }
    if has_shell {
        return "SHELL";
    }
    if has_file {
        return "FILE";
    }
    if let Some(code) = http_status {
        return match code {
            429 => "RATE_LIMIT",
            404 => "NOT_FOUND",
            c if c >= 500 => "SERVER_ERROR",
            _ => "HTTP_ERROR",
        };
    }
    "UNKNOWN"
}

// ---------------------------------------------------------------------------
// FailureDetail
// ---------------------------------------------------------------------------

/// A self-contained diagnostic snapshot suitable for serialization or display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureDetail {
    pub message: String,
    pub layers: Vec<ErrorLayer>,
}

impl FailureDetail {
    /// Human-readable classification derived from layers.
    pub fn classification(&self) -> &'static str {
        classify_layers(&self.layers)
    }

    /// The `provider.operation` label from the first [`ServiceErrorLayer`], if any.
    pub fn service_label(&self) -> Option<String> {
        self.layers.iter().find_map(|l| match l {
            ErrorLayer::Service(s) => Some(format!("{} → {}", s.provider, s.operation)),
            _ => None,
        })
    }
}

impl fmt::Display for FailureDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<&ExecError> for FailureDetail {
    fn from(err: &ExecError) -> Self {
        Self {
            message: err.message.clone(),
            layers: err.layers.clone(),
        }
    }
}

impl From<String> for FailureDetail {
    fn from(s: String) -> Self {
        Self {
            message: s,
            layers: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// ExecError
// ---------------------------------------------------------------------------

/// Error during DAG execution.
///
/// Carries a human-readable `message` and an ordered stack of [`ErrorLayer`]
/// values that describe which protocol/transport layers were involved.
#[derive(Debug, Clone)]
pub struct ExecError {
    pub message: String,
    pub layers: Vec<ErrorLayer>,
}

impl ExecError {
    /// Create a new error with no layers.
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            layers: Vec::new(),
        }
    }

    /// Append a layer to this error, returning `self` for chaining.
    pub fn with_layer(mut self, layer: ErrorLayer) -> Self {
        self.layers.push(layer);
        self
    }

    /// Add context to this error, producing a new error with the context
    /// prepended to the message. Layers are preserved.
    ///
    /// ```text
    /// let err = ExecError::new("file not found");
    /// let with_ctx = err.context("reading config");
    /// assert_eq!(with_ctx.to_string(), "reading config: file not found");
    /// ```
    pub fn context(self, ctx: impl fmt::Display) -> Self {
        Self {
            message: format!("{}: {}", ctx, self.message),
            layers: self.layers,
        }
    }

    /// Borrow the message string.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Borrow the layer stack.
    pub fn layers(&self) -> &[ErrorLayer] {
        &self.layers
    }

    /// Human-readable classification derived from layers.
    pub fn classification(&self) -> &'static str {
        classify_layers(&self.layers)
    }

    /// The `provider.operation` label from the first [`ServiceErrorLayer`], if any.
    pub fn service_label(&self) -> Option<String> {
        self.layers.iter().find_map(|l| match l {
            ErrorLayer::Service(s) => Some(format!("{} → {}", s.provider, s.operation)),
            _ => None,
        })
    }

    /// Convert to a [`FailureDetail`] snapshot.
    pub fn to_failure_detail(&self) -> FailureDetail {
        FailureDetail::from(self)
    }

    /// Create an `ExecError` from a [`FailureDetail`].
    pub fn from_failure_detail(detail: FailureDetail) -> Self {
        Self {
            message: detail.message,
            layers: detail.layers,
        }
    }
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ExecError {}

impl From<String> for ExecError {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for ExecError {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

// ---------------------------------------------------------------------------
// ResultExt
// ---------------------------------------------------------------------------

/// Extension trait for adding context to `Result<T, ExecError>`.
///
/// ```text
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

// ---------------------------------------------------------------------------
// IntoExecResult
// ---------------------------------------------------------------------------

/// Extension trait for converting any `Result<T, E>` into `Result<T, ExecError>`.
///
/// This trait provides `exec_context()` which converts any error type that implements
/// `Display` into an `ExecError` with a context message. This eliminates the common
/// boilerplate pattern:
///
/// ```text
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_error_creation() {
        let err = ExecError::new("something went wrong");
        assert_eq!(err.message(), "something went wrong");
        assert_eq!(err.to_string(), "something went wrong");
        assert!(err.layers().is_empty());
    }

    #[test]
    fn error_with_layers() {
        let err = ExecError::new("request failed")
            .with_layer(ErrorLayer::Http(HttpErrorLayer {
                status_code: 500,
                reason: Some("Internal Server Error".into()),
            }))
            .with_layer(ErrorLayer::Service(ServiceErrorLayer {
                provider: "github".into(),
                operation: "list_repos".into(),
            }));

        assert_eq!(err.layers().len(), 2);
        assert_eq!(err.to_string(), "request failed");
    }

    #[test]
    fn error_classification_auth() {
        let err = ExecError::new("forbidden").with_layer(ErrorLayer::Auth(AuthErrorLayer {
            scheme: "BearerToken".into(),
            credential_ref: Some("GITHUB_TOKEN".into()),
        }));

        assert_eq!(err.classification(), "AUTH");
    }

    #[test]
    fn error_service_label() {
        let err = ExecError::new("not found").with_layer(ErrorLayer::Service(ServiceErrorLayer {
            provider: "github".into(),
            operation: "get_gist".into(),
        }));

        assert_eq!(err.service_label(), Some("github → get_gist".into()));
    }

    #[test]
    fn error_classification_priority() {
        // Auth should win over Http 500.
        let err = ExecError::new("multi-layer failure")
            .with_layer(ErrorLayer::Http(HttpErrorLayer {
                status_code: 500,
                reason: None,
            }))
            .with_layer(ErrorLayer::Auth(AuthErrorLayer {
                scheme: "BearerToken".into(),
                credential_ref: None,
            }));

        assert_eq!(err.classification(), "AUTH");
    }

    #[test]
    fn error_classification_shell() {
        let err = ExecError::new("command failed").with_layer(ErrorLayer::Shell(ShellErrorLayer {
            command: "cargo test".into(),
            exit_code: Some(101),
        }));

        assert_eq!(err.classification(), "SHELL");
    }

    #[test]
    fn error_classification_http_specific() {
        // 429 → RATE_LIMIT
        let rate_limit = ExecError::new("too many requests").with_layer(ErrorLayer::Http(
            HttpErrorLayer {
                status_code: 429,
                reason: None,
            },
        ));
        assert_eq!(rate_limit.classification(), "RATE_LIMIT");

        // 404 → NOT_FOUND
        let not_found =
            ExecError::new("missing").with_layer(ErrorLayer::Http(HttpErrorLayer {
                status_code: 404,
                reason: None,
            }));
        assert_eq!(not_found.classification(), "NOT_FOUND");

        // 500 → SERVER_ERROR
        let server_err =
            ExecError::new("kaboom").with_layer(ErrorLayer::Http(HttpErrorLayer {
                status_code: 500,
                reason: None,
            }));
        assert_eq!(server_err.classification(), "SERVER_ERROR");

        // 502 → SERVER_ERROR (>= 500)
        let gateway_err =
            ExecError::new("bad gateway").with_layer(ErrorLayer::Http(HttpErrorLayer {
                status_code: 502,
                reason: None,
            }));
        assert_eq!(gateway_err.classification(), "SERVER_ERROR");
    }

    #[test]
    fn error_context_preserves_layers() {
        let err = ExecError::new("timeout")
            .with_layer(ErrorLayer::Http(HttpErrorLayer {
                status_code: 504,
                reason: None,
            }))
            .context("calling GitHub API");

        assert_eq!(err.to_string(), "calling GitHub API: timeout");
        assert_eq!(err.layers().len(), 1);
        assert_eq!(err.classification(), "SERVER_ERROR");
    }

    #[test]
    fn failure_detail_conversion() {
        let err = ExecError::new("auth failed")
            .with_layer(ErrorLayer::Auth(AuthErrorLayer {
                scheme: "Header".into(),
                credential_ref: Some("X-API-KEY".into()),
            }))
            .with_layer(ErrorLayer::Service(ServiceErrorLayer {
                provider: "gcp".into(),
                operation: "upload".into(),
            }));

        let detail = err.to_failure_detail();
        assert_eq!(detail.message, "auth failed");
        assert_eq!(detail.layers.len(), 2);
        assert_eq!(detail.classification(), "AUTH");
        assert_eq!(detail.service_label(), Some("gcp → upload".into()));

        // Round-trip back to ExecError.
        let roundtrip = ExecError::from_failure_detail(detail);
        assert_eq!(roundtrip.message(), "auth failed");
        assert_eq!(roundtrip.layers().len(), 2);
        assert_eq!(roundtrip.classification(), "AUTH");
    }

    #[test]
    fn failure_detail_from_string() {
        let detail = FailureDetail::from("plain error".to_string());
        assert_eq!(detail.message, "plain error");
        assert!(detail.layers.is_empty());
        assert_eq!(detail.classification(), "UNKNOWN");
    }

    #[test]
    fn from_string_creates_empty_layers() {
        let err: ExecError = "bare string".into();
        assert_eq!(err.message(), "bare string");
        assert!(err.layers().is_empty());
    }

    #[test]
    fn from_str_creates_empty_layers() {
        let err = ExecError::from("bare &str");
        assert_eq!(err.message(), "bare &str");
        assert!(err.layers().is_empty());
    }

    // Preserve backward-compat tests from the original file.

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

    #[test]
    fn error_no_layers_classifies_unknown() {
        let err = ExecError::new("generic error");
        assert_eq!(err.classification(), "UNKNOWN");
        assert_eq!(err.service_label(), None);
    }

    #[test]
    fn file_error_classification() {
        let err = ExecError::new("read failed").with_layer(ErrorLayer::File(FileErrorLayer {
            path: "/tmp/data.json".into(),
            operation: "read".into(),
        }));
        assert_eq!(err.classification(), "FILE");
    }

    #[test]
    fn auth_credential_ref() {
        let layer = AuthErrorLayer {
            scheme: "BearerToken".into(),
            credential_ref: Some("GITHUB_TOKEN".into()),
        };
        assert_eq!(layer.credential_ref, Some("GITHUB_TOKEN".to_string()));
    }
}
