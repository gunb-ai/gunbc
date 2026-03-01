//! Transport response/error classification.
//!
//! Classification is used by middleware (retry, metrics, diagnostics) to make
//! transport decisions before provider-specific parse stages run.

use gunbc_ir::transport::{HttpResponse, ResponseClassification, ResponseProvider, RestResponse};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Normalized transport error kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

fn provider_error_message(provider: ResponseProvider, body: &JsonValue) -> Option<String> {
    match provider {
        ResponseProvider::GitHub => body
            .get("message")
            .and_then(JsonValue::as_str)
            .map(|s| s.to_string()),
        ResponseProvider::Gcp => body
            .get("error")
            .and_then(|v| v.get("message").or_else(|| v.get("status")))
            .and_then(JsonValue::as_str)
            .map(|s| s.to_string()),
        ResponseProvider::Anthropic => body
            .get("error")
            .and_then(|v| v.get("message"))
            .and_then(JsonValue::as_str)
            .map(|s| s.to_string())
            .or_else(|| {
                body.get("type")
                    .and_then(JsonValue::as_str)
                    .map(|s| s.to_string())
            }),
        ResponseProvider::OpenAi => body
            .get("error")
            .and_then(|v| v.get("message"))
            .and_then(JsonValue::as_str)
            .map(|s| s.to_string()),
        ResponseProvider::Generic => None,
    }
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
}
