//! Transport response/error classification.
//!
//! Classification is used by middleware (retry, metrics, diagnostics) to make
//! transport decisions before provider-specific parse stages run.

use gunbc_ir::transport::{HttpResponse, ResponseClassification, ResponseProvider, RestResponse};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Normalized transport error kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassifiedErrorKind {
    Auth,
    RateLimit,
    Client,
    Server,
    Network,
    Unknown,
}

/// Classification output with optional provider diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedResponse {
    pub kind: ClassifiedErrorKind,
    pub provider: ResponseProvider,
    pub status: Option<u16>,
    pub message: Option<String>,
    pub retry_after_ms: Option<u64>,
}

impl ClassifiedResponse {
    /// Whether retry middleware should treat this classification as transient.
    pub fn retryable(&self) -> bool {
        matches!(
            self.kind,
            ClassifiedErrorKind::RateLimit
                | ClassifiedErrorKind::Server
                | ClassifiedErrorKind::Network
        )
    }
}

/// Classify a REST response. Returns `None` for 2xx success responses.
pub fn classify_rest_response(
    response: &RestResponse,
    policy: &ResponseClassification,
) -> Option<ClassifiedResponse> {
    if response.is_success() {
        return None;
    }

    let message = if policy.parse_provider_error_shapes {
        provider_error_message(policy.provider, &response.body)
    } else {
        None
    };

    Some(ClassifiedResponse {
        kind: classify_status(
            response.status,
            message.as_deref(),
            policy.prioritize_auth_errors,
        ),
        provider: policy.provider,
        status: Some(response.status),
        message,
        retry_after_ms: parse_retry_after_ms(&response.headers),
    })
}

/// Classify a raw HTTP response. Returns `None` for 2xx success responses.
pub fn classify_http_response(
    response: &HttpResponse,
    policy: &ResponseClassification,
) -> Option<ClassifiedResponse> {
    if response.is_success() {
        return None;
    }

    let parsed = serde_json::from_str::<JsonValue>(&response.body).ok();
    let message = if policy.parse_provider_error_shapes {
        parsed
            .as_ref()
            .and_then(|body| provider_error_message(policy.provider, body))
    } else {
        None
    };

    Some(ClassifiedResponse {
        kind: classify_status(
            response.status,
            message.as_deref(),
            policy.prioritize_auth_errors,
        ),
        provider: policy.provider,
        status: Some(response.status),
        message,
        retry_after_ms: parse_retry_after_ms(&response.headers),
    })
}

/// Classify a transport-layer execution failure (no HTTP status available).
pub fn classify_transport_error(message: &str) -> ClassifiedResponse {
    ClassifiedResponse {
        kind: ClassifiedErrorKind::Network,
        provider: ResponseProvider::Generic,
        status: None,
        message: Some(message.to_string()),
        retry_after_ms: None,
    }
}

fn classify_status(
    status: u16,
    message: Option<&str>,
    prioritize_auth: bool,
) -> ClassifiedErrorKind {
    let auth_hint = message_has_auth_indicator(message);
    if prioritize_auth && (matches!(status, 401 | 403) || auth_hint) {
        return ClassifiedErrorKind::Auth;
    }
    if status == 429 || message_has_rate_limit_indicator(message) {
        return ClassifiedErrorKind::RateLimit;
    }
    if matches!(status, 401 | 403) {
        return ClassifiedErrorKind::Auth;
    }
    if (400..500).contains(&status) {
        return ClassifiedErrorKind::Client;
    }
    if status >= 500 {
        return ClassifiedErrorKind::Server;
    }
    ClassifiedErrorKind::Unknown
}

/// Provider-specific error details parsed from response body.
#[derive(Debug, Clone, Default)]
pub struct ProviderDiagnostics {
    /// Primary error message.
    pub message: Option<String>,
    /// Error type/code (provider-specific).
    pub error_type: Option<String>,
    /// HTTP status string (GCP).
    pub status: Option<String>,
    /// Documentation URL (GitHub).
    pub documentation_url: Option<String>,
    /// Numeric error code (GCP, OpenAI).
    pub code: Option<i64>,
}

impl ProviderDiagnostics {
    /// Combine fields into a human-readable message.
    pub fn to_message(&self) -> Option<String> {
        if let Some(msg) = &self.message {
            let mut result = msg.clone();
            if let Some(doc) = &self.documentation_url {
                result.push_str(" (see: ");
                result.push_str(doc);
                result.push(')');
            }
            Some(result)
        } else {
            self.error_type.clone()
        }
    }
}

/// Parse provider-specific error shape from response body.
pub fn parse_provider_error(provider: ResponseProvider, body: &JsonValue) -> ProviderDiagnostics {
    match provider {
        ResponseProvider::GitHub => parse_github_error(body),
        ResponseProvider::Gcp => parse_gcp_error(body),
        ResponseProvider::Anthropic => parse_anthropic_error(body),
        ResponseProvider::OpenAi => parse_openai_error(body),
        ResponseProvider::Generic => ProviderDiagnostics::default(),
    }
}

/// GitHub error shape: `{ message, documentation_url }`
fn parse_github_error(body: &JsonValue) -> ProviderDiagnostics {
    ProviderDiagnostics {
        message: body.get("message").and_then(JsonValue::as_str).map(String::from),
        documentation_url: body.get("documentation_url").and_then(JsonValue::as_str).map(String::from),
        ..Default::default()
    }
}

/// GCP error shape: `{ error: { code, message, status } }`
fn parse_gcp_error(body: &JsonValue) -> ProviderDiagnostics {
    let error = body.get("error");
    ProviderDiagnostics {
        message: error.and_then(|e| e.get("message")).and_then(JsonValue::as_str).map(String::from),
        status: error.and_then(|e| e.get("status")).and_then(JsonValue::as_str).map(String::from),
        code: error.and_then(|e| e.get("code")).and_then(JsonValue::as_i64),
        ..Default::default()
    }
}

/// Anthropic error shape: `{ type, error: { type, message } }`
fn parse_anthropic_error(body: &JsonValue) -> ProviderDiagnostics {
    let error = body.get("error");
    ProviderDiagnostics {
        message: error.and_then(|e| e.get("message")).and_then(JsonValue::as_str).map(String::from),
        error_type: error
            .and_then(|e| e.get("type"))
            .and_then(JsonValue::as_str)
            .map(String::from)
            .or_else(|| body.get("type").and_then(JsonValue::as_str).map(String::from)),
        ..Default::default()
    }
}

/// OpenAI error shape: `{ error: { message, type, code } }`
fn parse_openai_error(body: &JsonValue) -> ProviderDiagnostics {
    let error = body.get("error");
    ProviderDiagnostics {
        message: error.and_then(|e| e.get("message")).and_then(JsonValue::as_str).map(String::from),
        error_type: error.and_then(|e| e.get("type")).and_then(JsonValue::as_str).map(String::from),
        code: error.and_then(|e| e.get("code")).and_then(JsonValue::as_i64),
        ..Default::default()
    }
}

fn provider_error_message(provider: ResponseProvider, body: &JsonValue) -> Option<String> {
    parse_provider_error(provider, body).to_message()
}

fn message_has_auth_indicator(message: Option<&str>) -> bool {
    message
        .map(|m| {
            let lower = m.to_ascii_lowercase();
            lower.contains("auth")
                || lower.contains("unauthorized")
                || lower.contains("forbidden")
                || lower.contains("invalid api key")
                || lower.contains("permission")
        })
        .unwrap_or(false)
}

fn message_has_rate_limit_indicator(message: Option<&str>) -> bool {
    message
        .map(|m| {
            let lower = m.to_ascii_lowercase();
            lower.contains("rate limit")
                || lower.contains("rate_limit")
                || lower.contains("too many requests")
        })
        .unwrap_or(false)
}

fn parse_retry_after_ms(headers: &HashMap<String, String>) -> Option<u64> {
    let raw = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("retry-after"))
        .map(|(_, v)| v.trim())?;
    // Retry-After commonly uses seconds. We keep parsing intentionally strict
    // for deterministic tests and middleware behavior.
    raw.parse::<u64>().ok().map(|seconds| seconds * 1000)
}

// ---------------------------------------------------------------------------
// Middleware integration
// ---------------------------------------------------------------------------

use gunbc_ir::transport::TransportResponse;

/// Default classification policy when none is configured.
fn default_policy() -> ResponseClassification {
    ResponseClassification {
        provider: ResponseProvider::Generic,
        prioritize_auth_errors: true,
        parse_provider_error_shapes: true,
    }
}

/// Classify a transport response for middleware decisions.
///
/// This is the primary entry point for middleware (retry, metrics) to classify
/// responses. It handles the `TransportResponse` enum and dispatches to the
/// appropriate type-specific classifier.
///
/// Returns `None` for:
/// - Successful HTTP responses (2xx)
/// - Non-HTTP transports (File, Shell, Tcp, Local)
///
/// # Arguments
///
/// * `response` - The transport response to classify
/// * `policy` - Optional classification policy; uses a sensible default if None
///
/// # Example
///
/// ```ignore
/// use gunbc_lib_transport::classify::{classify_for_middleware, ClassifiedErrorKind};
///
/// let classified = classify_for_middleware(&response, None);
/// if let Some(c) = classified {
///     if c.retryable() {
///         // Handle retry
///     }
/// }
/// ```
pub fn classify_for_middleware(
    response: &TransportResponse,
    policy: Option<&ResponseClassification>,
) -> Option<ClassifiedResponse> {
    let default = default_policy();
    let policy = policy.unwrap_or(&default);
    match response {
        TransportResponse::Rest(r) => classify_rest_response(r, policy),
        TransportResponse::Http(r) => classify_http_response(r, policy),
        // Non-HTTP transports don't have HTTP-style classification
        TransportResponse::File(_)
        | TransportResponse::Shell(_)
        | TransportResponse::Tcp(_)
        | TransportResponse::Local(_) => None,
    }
}

/// Extract HTTP status code from a transport response if applicable.
pub fn extract_status_code(response: &TransportResponse) -> Option<u16> {
    match response {
        TransportResponse::Rest(r) => Some(r.status),
        TransportResponse::Http(r) => Some(r.status),
        _ => None,
    }
}

/// Check if a response indicates success (2xx for HTTP, success field for others).
pub fn is_success(response: &TransportResponse) -> bool {
    match response {
        TransportResponse::Rest(r) => r.is_success(),
        TransportResponse::Http(r) => r.is_success(),
        TransportResponse::File(r) => r.success,
        TransportResponse::Shell(r) => r.success(),
        TransportResponse::Tcp(_) => true, // TCP connections that succeed don't error
        TransportResponse::Local(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::{HttpResponse, RestResponse};

    #[test]
    fn github_rate_limit_uses_retry_after_header() {
        let mut response = RestResponse::new(
            429,
            serde_json::json!({
                "message": "API rate limit exceeded",
                "documentation_url": "https://docs.github.com/rest/overview/resources-in-the-rest-api"
            }),
        );
        response
            .headers
            .insert("Retry-After".to_string(), "42".to_string());
        let policy = ResponseClassification {
            provider: ResponseProvider::GitHub,
            prioritize_auth_errors: true,
            parse_provider_error_shapes: true,
        };

        let classified =
            classify_rest_response(&response, &policy).expect("non-2xx should classify");
        assert_eq!(classified.kind, ClassifiedErrorKind::RateLimit);
        assert_eq!(classified.retry_after_ms, Some(42_000));
        assert!(classified
            .message
            .as_deref()
            .is_some_and(|m| m.contains("rate limit")));
    }

    #[test]
    fn gcp_client_error_extracts_nested_message() {
        let response = RestResponse::new(
            400,
            serde_json::json!({
                "error": {
                    "code": 400,
                    "message": "Invalid project id",
                    "status": "INVALID_ARGUMENT"
                }
            }),
        );
        let policy = ResponseClassification {
            provider: ResponseProvider::Gcp,
            prioritize_auth_errors: true,
            parse_provider_error_shapes: true,
        };
        let classified = classify_rest_response(&response, &policy).expect("classification");
        assert_eq!(classified.kind, ClassifiedErrorKind::Client);
        assert_eq!(classified.message, Some("Invalid project id".to_string()));
    }

    #[test]
    fn anthropic_auth_priority_beats_generic_client() {
        let response = RestResponse::new(
            400,
            serde_json::json!({
                "type": "error",
                "error": {
                    "type": "authentication_error",
                    "message": "authentication failed"
                }
            }),
        );
        let policy = ResponseClassification {
            provider: ResponseProvider::Anthropic,
            prioritize_auth_errors: true,
            parse_provider_error_shapes: true,
        };
        let classified = classify_rest_response(&response, &policy).expect("classification");
        assert_eq!(classified.kind, ClassifiedErrorKind::Auth);
    }

    #[test]
    fn malformed_provider_shape_falls_back_to_status_classification() {
        let response = RestResponse::new(500, serde_json::json!({"unexpected": true}));
        let policy = ResponseClassification {
            provider: ResponseProvider::Gcp,
            prioritize_auth_errors: true,
            parse_provider_error_shapes: true,
        };
        let classified = classify_rest_response(&response, &policy).expect("classification");
        assert_eq!(classified.kind, ClassifiedErrorKind::Server);
        assert_eq!(classified.message, None);
    }

    #[test]
    fn http_response_classification_parses_json_body_for_message() {
        let response = HttpResponse {
            status: 401,
            headers: HashMap::new(),
            body: r#"{"error":{"message":"invalid api key"}}"#.to_string(),
        };
        let policy = ResponseClassification {
            provider: ResponseProvider::OpenAi,
            prioritize_auth_errors: true,
            parse_provider_error_shapes: true,
        };
        let classified = classify_http_response(&response, &policy).expect("classification");
        assert_eq!(classified.kind, ClassifiedErrorKind::Auth);
        assert_eq!(classified.message, Some("invalid api key".to_string()));
    }

    #[test]
    fn classify_transport_error_marks_network_retryable() {
        let classified = classify_transport_error("connect timeout");
        assert_eq!(classified.kind, ClassifiedErrorKind::Network);
        assert!(classified.retryable());
    }

    // Middleware integration tests

    #[test]
    fn classify_for_middleware_handles_rest_response() {
        use gunbc_ir::transport::TransportResponse;

        let rest_response = RestResponse::new(429, serde_json::json!({"message": "rate limit"}));
        let response = TransportResponse::Rest(rest_response);

        let classified = classify_for_middleware(&response, None);
        assert!(classified.is_some());
        let c = classified.unwrap();
        assert_eq!(c.kind, ClassifiedErrorKind::RateLimit);
        assert!(c.retryable());
    }

    #[test]
    fn classify_for_middleware_returns_none_for_success() {
        use gunbc_ir::transport::TransportResponse;

        let rest_response = RestResponse::new(200, serde_json::json!({"ok": true}));
        let response = TransportResponse::Rest(rest_response);

        let classified = classify_for_middleware(&response, None);
        assert!(classified.is_none());
    }

    #[test]
    fn classify_for_middleware_returns_none_for_non_http() {
        use gunbc_ir::transport::{LocalResponse, ShellResponse, TransportResponse};

        let shell = TransportResponse::Shell(ShellResponse {
            stdout: "ok".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
        });
        assert!(classify_for_middleware(&shell, None).is_none());

        let local = TransportResponse::Local(LocalResponse {
            outputs: serde_json::json!({}),
        });
        assert!(classify_for_middleware(&local, None).is_none());
    }

    #[test]
    fn extract_status_code_works_for_http_types() {
        use gunbc_ir::transport::{LocalResponse, TransportResponse};

        let rest = TransportResponse::Rest(RestResponse::new(404, serde_json::json!({})));
        assert_eq!(extract_status_code(&rest), Some(404));

        let http = TransportResponse::Http(HttpResponse {
            status: 500,
            headers: HashMap::new(),
            body: "".to_string(),
        });
        assert_eq!(extract_status_code(&http), Some(500));

        let local = TransportResponse::Local(LocalResponse {
            outputs: serde_json::json!({}),
        });
        assert_eq!(extract_status_code(&local), None);
    }

    #[test]
    fn is_success_checks_all_transport_types() {
        use gunbc_ir::transport::{FileResponse, LocalResponse, ShellResponse, TransportResponse};

        // REST success
        let rest = TransportResponse::Rest(RestResponse::new(200, serde_json::json!({})));
        assert!(is_success(&rest));

        // REST failure
        let rest_fail = TransportResponse::Rest(RestResponse::new(500, serde_json::json!({})));
        assert!(!is_success(&rest_fail));

        // Shell success
        let shell = TransportResponse::Shell(ShellResponse {
            stdout: "".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
        });
        assert!(is_success(&shell));

        // Shell failure
        let shell_fail = TransportResponse::Shell(ShellResponse {
            stdout: "".to_string(),
            stderr: "error".to_string(),
            exit_code: 1,
        });
        assert!(!is_success(&shell_fail));

        // File success is determined by the success field
        use gunbc_ir::transport::FileOp;
        let file_ok = TransportResponse::File(FileResponse {
            path: "/tmp/test".to_string(),
            operation: FileOp::Read,
            success: true,
            content: None,
            bytes: None,
            exists: Some(true),
            error: None,
        });
        assert!(is_success(&file_ok));

        // File failure (success=false) should return false
        let file_fail = TransportResponse::File(FileResponse {
            path: "/tmp/missing".to_string(),
            operation: FileOp::Read,
            success: false,
            content: None,
            bytes: None,
            exists: Some(false),
            error: Some("file not found".to_string()),
        });
        assert!(!is_success(&file_fail));

        // Local always success
        let local = TransportResponse::Local(LocalResponse {
            outputs: serde_json::json!({}),
        });
        assert!(is_success(&local));
    }
}
