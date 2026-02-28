//! Provider-aware mock response synthesis helpers.
//!
//! This module centralizes transport mock body shapes so tests can derive
//! provider-specific success/error payloads from behavioral intent instead of
//! relying on one kitchen-sink response blob.

use gunbc_ir::transport::RestResponse;
use serde_json::{json, Value as JsonValue};

/// Supported provider families for synthesized mock responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockProvider {
    Generic,
    GitHub,
    Gcp,
    Anthropic,
}

/// Behavior-driven synthesis input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockResponseSynthesis {
    pub provider: MockProvider,
    pub status: u16,
    /// Optional failure mode tag (e.g. `rate_limit`, `auth`, `not_found`).
    pub failure_mode: Option<String>,
    /// Optional body override merged on top of synthesized body.
    pub body_override: Option<JsonValue>,
}

impl MockResponseSynthesis {
    pub fn success(provider: MockProvider) -> Self {
        Self {
            provider,
            status: 200,
            failure_mode: None,
            body_override: None,
        }
    }

    pub fn failure(provider: MockProvider, status: u16, mode: impl Into<String>) -> Self {
        Self {
            provider,
            status,
            failure_mode: Some(mode.into()),
            body_override: None,
        }
    }
}

/// Synthesize a provider-aware REST response body for tests.
pub fn synthesize_rest_response(spec: &MockResponseSynthesis) -> RestResponse {
    let mut body = if (200..300).contains(&spec.status) {
        success_shape(spec.provider)
    } else {
        error_shape(spec.provider, spec.status, spec.failure_mode.as_deref())
    };
    if let Some(override_body) = &spec.body_override {
        merge_json(&mut body, override_body);
    }
    RestResponse::new(spec.status, body)
}

fn success_shape(provider: MockProvider) -> JsonValue {
    match provider {
        MockProvider::GitHub => json!({
            "id": "mock-id",
            "html_url": "https://gist.github.com/mock-id",
            "state": "open"
        }),
        MockProvider::Gcp => json!({
            "name": "projects/mock-project/secrets/mock-secret/versions/1",
            "etag": "mock-etag"
        }),
        MockProvider::Anthropic => json!({
            "id": "msg_mock",
            "type": "message",
            "content": [{ "type": "text", "text": "Mock response" }],
            "stop_reason": "end_turn"
        }),
        MockProvider::Generic => json!({ "ok": true }),
    }
}

fn error_shape(provider: MockProvider, status: u16, mode: Option<&str>) -> JsonValue {
    let mode = mode.unwrap_or("unknown");
    match provider {
        MockProvider::GitHub => json!({
            "message": github_error_message(status, mode),
            "documentation_url": "https://docs.github.com/rest"
        }),
        MockProvider::Gcp => json!({
            "error": {
                "code": status,
                "status": gcp_error_status(status, mode),
                "message": gcp_error_message(status, mode)
            }
        }),
        MockProvider::Anthropic => json!({
            "type": "error",
            "error": {
                "type": anthropic_error_type(status, mode),
                "message": anthropic_error_message(status, mode)
            }
        }),
        MockProvider::Generic => json!({
            "error": mode,
            "status": status
        }),
    }
}

fn github_error_message(status: u16, mode: &str) -> &'static str {
    match (status, mode) {
        (401, _) | (_, "auth") => "Bad credentials",
        (403, _) => "Forbidden",
        (404, _) | (_, "not_found") => "Not Found",
        (429, _) | (_, "rate_limit") => "API rate limit exceeded",
        _ => "GitHub API error",
    }
}

fn gcp_error_status(status: u16, mode: &str) -> &'static str {
    match (status, mode) {
        (401, _) | (_, "auth") => "UNAUTHENTICATED",
        (403, _) => "PERMISSION_DENIED",
        (404, _) | (_, "not_found") => "NOT_FOUND",
        (429, _) | (_, "rate_limit") => "RESOURCE_EXHAUSTED",
        (400, _) => "INVALID_ARGUMENT",
        _ => "INTERNAL",
    }
}

fn gcp_error_message(status: u16, mode: &str) -> &'static str {
    match (status, mode) {
        (429, _) | (_, "rate_limit") => "Quota exceeded",
        (401, _) | (_, "auth") => "Request is missing valid authentication credentials",
        (404, _) | (_, "not_found") => "Resource not found",
        _ => "GCP API error",
    }
}

fn anthropic_error_type(status: u16, mode: &str) -> &'static str {
    match (status, mode) {
        (401, _) | (_, "auth") => "authentication_error",
        (429, _) | (_, "rate_limit") => "rate_limit_error",
        (400, _) => "invalid_request_error",
        _ => "api_error",
    }
}

fn anthropic_error_message(status: u16, mode: &str) -> &'static str {
    match (status, mode) {
        (429, _) | (_, "rate_limit") => "rate limit exceeded",
        (401, _) | (_, "auth") => "authentication failed",
        _ => "Anthropic API error",
    }
}

fn merge_json(target: &mut JsonValue, patch: &JsonValue) {
    match (target, patch) {
        (JsonValue::Object(target_map), JsonValue::Object(patch_map)) => {
            for (key, patch_value) in patch_map {
                match target_map.get_mut(key) {
                    Some(target_value) => merge_json(target_value, patch_value),
                    None => {
                        target_map.insert(key.clone(), patch_value.clone());
                    }
                }
            }
        }
        (target_value, patch_value) => {
            *target_value = patch_value.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_rate_limit_synthesis_uses_github_error_shape() {
        let response = synthesize_rest_response(&MockResponseSynthesis::failure(
            MockProvider::GitHub,
            429,
            "rate_limit",
        ));
        assert_eq!(response.status, 429);
        assert_eq!(response.body["message"], "API rate limit exceeded");
        assert!(response.body.get("documentation_url").is_some());
    }

    #[test]
    fn gcp_error_synthesis_uses_nested_error_object() {
        let response = synthesize_rest_response(&MockResponseSynthesis::failure(
            MockProvider::Gcp,
            404,
            "not_found",
        ));
        assert_eq!(response.status, 404);
        assert_eq!(response.body["error"]["status"], "NOT_FOUND");
    }

    #[test]
    fn anthropic_error_synthesis_uses_provider_shape() {
        let response = synthesize_rest_response(&MockResponseSynthesis::failure(
            MockProvider::Anthropic,
            401,
            "auth",
        ));
        assert_eq!(response.status, 401);
        assert_eq!(response.body["error"]["type"], "authentication_error");
    }

    #[test]
    fn body_override_merges_into_synthesized_shape() {
        let mut spec = MockResponseSynthesis::success(MockProvider::GitHub);
        spec.body_override = Some(json!({"id": "override", "nested": {"x": 1}}));
        let response = synthesize_rest_response(&spec);
        assert_eq!(response.body["id"], "override");
        assert_eq!(response.body["nested"]["x"], 1);
    }
}
