//! Structured error types for execution.
//!
//! Errors carry an ordered stack of [`ErrorLayer`] values that record which
//! transport/protocol layers were involved when the failure occurred. The
//! [`classify_layers`] function scans **all** layers and returns a
//! human-readable classification string by priority order.

use crate::diagnostic::{AcquisitionDiagnostic, KeyIdentity, LockIdentity};
use std::fmt;

// ---------------------------------------------------------------------------
// ErrorLayer + per-layer structs
// ---------------------------------------------------------------------------

/// A single layer in the error's diagnostic stack.
///
/// **Domain layers** (Http, Rest, Acquisition, Service, Shell, File) are pushed by
/// individual ops that understand their transport/protocol context.
///
/// **NodeTrace** is pushed automatically by the executor for every node
/// failure — like a stack frame, it records which node was executing when
/// the error occurred. This is structural: no hand-wiring needed per op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorLayer {
    Http(HttpErrorLayer),
    Rest(RestErrorLayer),
    /// Resource acquisition diagnostic — replaces the old `Auth` variant.
    /// Carries a self-describing [`AcquisitionDiagnostic`] with lock+key identity.
    Acquisition(AcquisitionErrorLayer),
    Service(ServiceErrorLayer),
    Shell(ShellErrorLayer),
    File(FileErrorLayer),
    /// Automatically pushed by the executor — identifies the failing node.
    NodeTrace(NodeTraceLayer),
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

/// Resource acquisition failure — carries a self-describing diagnostic.
///
/// The [`AcquisitionDiagnostic`] contains both the lock (what resource) and
/// key (what credential) identity, so the error system delegates formatting
/// to the types themselves.
///
/// `required_permissions` surfaces the scopes/permissions the operation
/// requires (from `permissions` declarations on operations). Empty when
/// the operation declares no permission requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcquisitionErrorLayer {
    pub diagnostic: AcquisitionDiagnostic,
    pub required_permissions: Vec<String>,
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

/// Execution trace — automatically pushed by the executor for every node failure.
///
/// Like a stack frame: records which node was executing when the error occurred.
/// For nested execution (loop bodies), multiple NodeTrace layers form a trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeTraceLayer {
    /// The internal node ID (e.g., "parse_transport_services_github_gist_github_Gist_Create").
    pub node_id: String,
    /// Structural role inferred from the node's port types (not from string parsing).
    pub role: NodeRole,
}

/// Structural role of a node, inferred from its port types.
///
/// This classification comes from the node's actual ports, not from naming
/// conventions. Adding a new node type with `TransportRequest` inputs
/// automatically classifies it as `TransportExecutor` — no hand-wiring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeRole {
    /// Node consumes `TransportRequest` — performs actual I/O.
    TransportExecutor,
    /// Node emits `ToolHandle`, `FilesystemHandle`, etc. — environment provider.
    ResourceProvider,
    /// Node consumes `ToolHandle` — tool runner.
    ToolConsumer,
    /// Pure computation node — no transport or resource ports.
    Pure,
}

// ---------------------------------------------------------------------------
// classify_layers — priority-based, full-scan
// ---------------------------------------------------------------------------

/// Normalized error classification used across display, CI, and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorClass {
    Auth,
    Shell,
    File,
    RateLimit,
    NotFound,
    ServerError,
    HttpError,
    Unknown,
}

impl ErrorClass {
    /// Stable uppercase label used in non-TTY output.
    pub fn label(self) -> &'static str {
        match self {
            Self::Auth => "AUTH",
            Self::Shell => "SHELL",
            Self::File => "FILE",
            Self::RateLimit => "RATE_LIMIT",
            Self::NotFound => "NOT_FOUND",
            Self::ServerError => "SERVER_ERROR",
            Self::HttpError => "HTTP_ERROR",
            Self::Unknown => "UNKNOWN",
        }
    }

    /// Box tag used in rich error boxes.
    pub fn box_tag(self) -> &'static str {
        match self {
            Self::Auth => "[AUTH]",
            Self::Shell => "[SHELL]",
            Self::File => "[FILE]",
            Self::RateLimit => "[RATE_LIMIT]",
            Self::NotFound => "[NOT_FOUND]",
            Self::ServerError => "[SERVER_ERROR]",
            Self::HttpError => "[HTTP_ERROR]",
            Self::Unknown => "[ERROR]",
        }
    }
}

impl fmt::Display for ErrorClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Scan **all** layers and return a normalized [`ErrorClass`] by priority.
///
/// Priority order (highest first):
/// 1. Acquisition (with or without key) → `"AUTH"`
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
pub fn classify_layers(layers: &[ErrorLayer]) -> ErrorClass {
    let mut has_acquisition = false;
    let mut has_shell = false;
    let mut has_file = false;
    let mut http_status: Option<u16> = None;

    for layer in layers {
        match layer {
            ErrorLayer::Acquisition(_) => has_acquisition = true,
            ErrorLayer::Shell(_) => has_shell = true,
            ErrorLayer::File(_) => has_file = true,
            ErrorLayer::Http(h) => {
                // Keep the last HTTP status seen (could also keep highest).
                http_status = Some(h.status_code);
            }
            ErrorLayer::Rest(_) | ErrorLayer::Service(_) | ErrorLayer::NodeTrace(_) => {}
        }
    }

    // Return by priority.
    if has_acquisition {
        return ErrorClass::Auth;
    }
    if has_shell {
        return ErrorClass::Shell;
    }
    if has_file {
        return ErrorClass::File;
    }
    if let Some(code) = http_status {
        return match code {
            401 | 403 => ErrorClass::Auth,
            429 => ErrorClass::RateLimit,
            404 => ErrorClass::NotFound,
            c if c >= 500 => ErrorClass::ServerError,
            _ => ErrorClass::HttpError,
        };
    }
    ErrorClass::Unknown
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
    pub fn classification(&self) -> ErrorClass {
        classify_layers(&self.layers)
    }

    /// Stable uppercase classification label.
    pub fn classification_label(&self) -> &'static str {
        self.classification().label()
    }

    /// The `provider.operation` label from the first [`ServiceErrorLayer`], if any.
    pub fn service_label(&self) -> Option<String> {
        self.layers.iter().find_map(|l| match l {
            ErrorLayer::Service(s) => Some(format!("{} → {}", s.provider, s.operation)),
            _ => None,
        })
    }

    /// The node ID from the first [`NodeTraceLayer`], if any.
    pub fn node_trace(&self) -> Option<&NodeTraceLayer> {
        self.layers.iter().find_map(|l| match l {
            ErrorLayer::NodeTrace(t) => Some(t),
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
    pub fn classification(&self) -> ErrorClass {
        classify_layers(&self.layers)
    }

    /// Stable uppercase classification label.
    pub fn classification_label(&self) -> &'static str {
        self.classification().label()
    }

    /// The `provider.operation` label from the first [`ServiceErrorLayer`], if any.
    pub fn service_label(&self) -> Option<String> {
        self.layers.iter().find_map(|l| match l {
            ErrorLayer::Service(s) => Some(format!("{} → {}", s.provider, s.operation)),
            _ => None,
        })
    }

    /// The node ID from the first [`NodeTraceLayer`], if any.
    pub fn node_trace(&self) -> Option<&NodeTraceLayer> {
        self.layers.iter().find_map(|l| match l {
            ErrorLayer::NodeTrace(t) => Some(t),
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
// Service failure decoration helpers
// ---------------------------------------------------------------------------

/// Service identity for structured failure decoration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceCallMetadata {
    pub provider: String,
    pub operation: String,
}

/// Transport-specific context for service failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportContext {
    Rest {
        endpoint: String,
        method: String,
        status_code: u16,
        reason: Option<String>,
    },
    Shell {
        command: String,
        exit_code: Option<i32>,
    },
}

/// Optional auth/acquisition context for service failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthContext {
    pub scheme: Option<String>,
    pub credential_ref: Option<String>,
    pub required_permissions: Vec<String>,
    pub lock_target: String,
}

/// Attach consistent service/transport/auth layers to an [`ExecError`].
///
/// Layer order is canonical and stable:
/// 1. Service
/// 2. Transport-specific layers (HTTP/REST or Shell)
/// 3. Acquisition/auth context (when provided)
pub fn decorate_service_failure(
    mut err: ExecError,
    service: ServiceCallMetadata,
    transport: TransportContext,
    auth: Option<AuthContext>,
) -> ExecError {
    err = err.with_layer(ErrorLayer::Service(ServiceErrorLayer {
        provider: service.provider,
        operation: service.operation,
    }));

    match transport {
        TransportContext::Rest {
            endpoint,
            method,
            status_code,
            reason,
        } => {
            err = err
                .with_layer(ErrorLayer::Http(HttpErrorLayer {
                    status_code,
                    reason,
                }))
                .with_layer(ErrorLayer::Rest(RestErrorLayer { endpoint, method }));
        }
        TransportContext::Shell { command, exit_code } => {
            err = err.with_layer(ErrorLayer::Shell(ShellErrorLayer { command, exit_code }));
        }
    }

    if let Some(auth) = auth {
        let key = auth.scheme.map(|scheme| KeyIdentity {
            scheme,
            hint: auth
                .credential_ref
                .as_deref()
                .map(|_| "\"***\"".to_string())
                .unwrap_or_else(|| "(none)".into()),
            source: auth
                .credential_ref
                .map(|c| format!("env:{c}"))
                .unwrap_or_else(|| "(no credential provided)".into()),
        });
        err = err.with_layer(ErrorLayer::Acquisition(AcquisitionErrorLayer {
            diagnostic: AcquisitionDiagnostic {
                lock: LockIdentity {
                    resource: "AuthContext".into(),
                    mode: "Read".into(),
                    target: auth.lock_target,
                },
                key,
            },
            required_permissions: auth.required_permissions,
        }));
    }

    err
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
    use crate::diagnostic::{KeyIdentity, LockIdentity};

    /// Helper: build an Acquisition layer from scheme + optional credential ref.
    fn acquisition_layer(scheme: &str, credential_ref: Option<&str>) -> ErrorLayer {
        ErrorLayer::Acquisition(AcquisitionErrorLayer {
            diagnostic: AcquisitionDiagnostic {
                lock: LockIdentity {
                    resource: "AuthContext".into(),
                    mode: "Read".into(),
                    target: "test".into(),
                },
                key: Some(KeyIdentity {
                    scheme: scheme.into(),
                    hint: credential_ref
                        .map(|_| "\"***\"".to_string())
                        .unwrap_or_else(|| "\"\"".into()),
                    source: credential_ref
                        .map(|c| format!("env:{c}"))
                        .unwrap_or_else(|| "static".into()),
                }),
            },
            required_permissions: vec![],
        })
    }

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
        let err = ExecError::new("forbidden")
            .with_layer(acquisition_layer("BearerToken", Some("GITHUB_TOKEN")));

        assert_eq!(err.classification(), ErrorClass::Auth);
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
        // Acquisition should win over Http 500.
        let err = ExecError::new("multi-layer failure")
            .with_layer(ErrorLayer::Http(HttpErrorLayer {
                status_code: 500,
                reason: None,
            }))
            .with_layer(acquisition_layer("BearerToken", None));

        assert_eq!(err.classification(), ErrorClass::Auth);
    }

    #[test]
    fn error_classification_shell() {
        let err = ExecError::new("command failed").with_layer(ErrorLayer::Shell(ShellErrorLayer {
            command: "cargo test".into(),
            exit_code: Some(101),
        }));

        assert_eq!(err.classification(), ErrorClass::Shell);
    }

    #[test]
    fn error_classification_http_specific() {
        // 429 → RATE_LIMIT
        let rate_limit =
            ExecError::new("too many requests").with_layer(ErrorLayer::Http(HttpErrorLayer {
                status_code: 429,
                reason: None,
            }));
        assert_eq!(rate_limit.classification(), ErrorClass::RateLimit);

        // 404 → NOT_FOUND
        let not_found = ExecError::new("missing").with_layer(ErrorLayer::Http(HttpErrorLayer {
            status_code: 404,
            reason: None,
        }));
        assert_eq!(not_found.classification(), ErrorClass::NotFound);

        // 500 → SERVER_ERROR
        let server_err = ExecError::new("kaboom").with_layer(ErrorLayer::Http(HttpErrorLayer {
            status_code: 500,
            reason: None,
        }));
        assert_eq!(server_err.classification(), ErrorClass::ServerError);

        // 502 → SERVER_ERROR (>= 500)
        let gateway_err =
            ExecError::new("bad gateway").with_layer(ErrorLayer::Http(HttpErrorLayer {
                status_code: 502,
                reason: None,
            }));
        assert_eq!(gateway_err.classification(), ErrorClass::ServerError);
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
        assert_eq!(err.classification(), ErrorClass::ServerError);
    }

    #[test]
    fn failure_detail_conversion() {
        let err = ExecError::new("auth failed")
            .with_layer(acquisition_layer("Header", Some("X-API-KEY")))
            .with_layer(ErrorLayer::Service(ServiceErrorLayer {
                provider: "gcp".into(),
                operation: "upload".into(),
            }));

        let detail = err.to_failure_detail();
        assert_eq!(detail.message, "auth failed");
        assert_eq!(detail.layers.len(), 2);
        assert_eq!(detail.classification(), ErrorClass::Auth);
        assert_eq!(detail.service_label(), Some("gcp → upload".into()));

        // Round-trip back to ExecError.
        let roundtrip = ExecError::from_failure_detail(detail);
        assert_eq!(roundtrip.message(), "auth failed");
        assert_eq!(roundtrip.layers().len(), 2);
        assert_eq!(roundtrip.classification(), ErrorClass::Auth);
    }

    #[test]
    fn failure_detail_from_string() {
        let detail = FailureDetail::from("plain error".to_string());
        assert_eq!(detail.message, "plain error");
        assert!(detail.layers.is_empty());
        assert_eq!(detail.classification(), ErrorClass::Unknown);
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
        assert_eq!(err.classification(), ErrorClass::Unknown);
        assert_eq!(err.service_label(), None);
    }

    #[test]
    fn node_trace_layer() {
        let err = ExecError::new("request failed")
            .with_layer(ErrorLayer::Http(HttpErrorLayer {
                status_code: 401,
                reason: Some("Unauthorized".into()),
            }))
            .with_layer(ErrorLayer::NodeTrace(NodeTraceLayer {
                node_id: "parse_transport_services_github_gist".into(),
                role: NodeRole::Pure,
            }));

        // 401 classifies as AUTH even without an Acquisition layer
        assert_eq!(err.classification(), ErrorClass::Auth);
        let trace = err.node_trace().unwrap();
        assert_eq!(trace.node_id, "parse_transport_services_github_gist");
        assert_eq!(trace.role, NodeRole::Pure);
    }

    #[test]
    fn node_trace_only_classifies_unknown() {
        // When the only layer is NodeTrace, classification falls through to UNKNOWN
        let err =
            ExecError::new("something broke").with_layer(ErrorLayer::NodeTrace(NodeTraceLayer {
                node_id: "some_node".into(),
                role: NodeRole::TransportExecutor,
            }));
        assert_eq!(err.classification(), ErrorClass::Unknown);
        assert!(err.node_trace().is_some());
    }

    #[test]
    fn file_error_classification() {
        let err = ExecError::new("read failed").with_layer(ErrorLayer::File(FileErrorLayer {
            path: "/tmp/data.json".into(),
            operation: "read".into(),
        }));
        assert_eq!(err.classification(), ErrorClass::File);
    }

    #[test]
    fn acquisition_diagnostic_display() {
        let layer = AcquisitionErrorLayer {
            diagnostic: AcquisitionDiagnostic {
                lock: LockIdentity {
                    resource: "AuthContext".into(),
                    mode: "Read".into(),
                    target: "POST https://api.github.com/gists".into(),
                },
                key: Some(KeyIdentity {
                    scheme: "Bearer".into(),
                    hint: "\"***\"".into(),
                    source: "env:GITHUB_TOKEN".into(),
                }),
            },
            required_permissions: vec![],
        };
        assert_eq!(
            layer.diagnostic.to_string(),
            "AuthContext (Read): POST https://api.github.com/gists with Bearer (key: \"***\", source: env:GITHUB_TOKEN)"
        );
    }

    #[test]
    fn http_401_classifies_as_auth_without_acquisition_layer() {
        // A bare 401 response (no Acquisition layer) should still classify as AUTH.
        let err = ExecError::new("Unauthorized").with_layer(ErrorLayer::Http(HttpErrorLayer {
            status_code: 401,
            reason: Some("Unauthorized".into()),
        }));
        assert_eq!(err.classification(), ErrorClass::Auth);
    }

    #[test]
    fn http_403_classifies_as_auth_without_acquisition_layer() {
        let err = ExecError::new("Forbidden").with_layer(ErrorLayer::Http(HttpErrorLayer {
            status_code: 403,
            reason: Some("Forbidden".into()),
        }));
        assert_eq!(err.classification(), ErrorClass::Auth);
    }

    #[test]
    fn acquisition_layer_with_permissions() {
        let layer = AcquisitionErrorLayer {
            diagnostic: AcquisitionDiagnostic {
                lock: LockIdentity {
                    resource: "AuthContext".into(),
                    mode: "Read".into(),
                    target: "GET https://storage.googleapis.com/bucket".into(),
                },
                key: None,
            },
            required_permissions: vec!["storage.read".into(), "storage.inspect".into()],
        };
        assert_eq!(
            layer.required_permissions,
            vec!["storage.read", "storage.inspect"]
        );
    }

    #[test]
    fn acquisition_layer_wins_over_401_http() {
        // When both Acquisition and HTTP 401 are present, AUTH from Acquisition
        // (higher priority) should still win.
        let err = ExecError::new("auth failed")
            .with_layer(ErrorLayer::Http(HttpErrorLayer {
                status_code: 401,
                reason: None,
            }))
            .with_layer(acquisition_layer("Bearer", Some("GITHUB_TOKEN")));
        assert_eq!(err.classification(), ErrorClass::Auth);
    }

    #[test]
    fn decorate_service_failure_rest_layers_are_canonical() {
        let err = decorate_service_failure(
            ExecError::new("request failed"),
            ServiceCallMetadata {
                provider: "github".into(),
                operation: "create_gist".into(),
            },
            TransportContext::Rest {
                endpoint: "https://api.github.com".into(),
                method: "POST".into(),
                status_code: 401,
                reason: Some("Unauthorized".into()),
            },
            Some(AuthContext {
                scheme: Some("BearerToken".into()),
                credential_ref: Some("GITHUB_TOKEN".into()),
                required_permissions: vec!["gist:write".into()],
                lock_target: "POST https://api.github.com/gists".into(),
            }),
        );

        assert_eq!(err.classification(), ErrorClass::Auth);
        assert_eq!(err.layers.len(), 4);
        assert!(matches!(err.layers[0], ErrorLayer::Service(_)));
        assert!(matches!(err.layers[1], ErrorLayer::Http(_)));
        assert!(matches!(err.layers[2], ErrorLayer::Rest(_)));
        assert!(matches!(err.layers[3], ErrorLayer::Acquisition(_)));
    }

    #[test]
    fn decorate_service_failure_shell_layers_are_canonical() {
        let err = decorate_service_failure(
            ExecError::new("shell parse failed"),
            ServiceCallMetadata {
                provider: "git".into(),
                operation: "status".into(),
            },
            TransportContext::Shell {
                command: "git status".into(),
                exit_code: Some(1),
            },
            None,
        );

        assert_eq!(err.classification(), ErrorClass::Shell);
        assert_eq!(err.layers.len(), 2);
        assert!(matches!(err.layers[0], ErrorLayer::Service(_)));
        assert!(matches!(err.layers[1], ErrorLayer::Shell(_)));
    }
}
