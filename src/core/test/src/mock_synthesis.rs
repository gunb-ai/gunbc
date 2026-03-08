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
    OpenAi,
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
            "etag": "mock-etag",
            "payload": { "data": "bW9jaw==" },
            "bindings": [],
            "access_token": "mock-access-token",
            "accessToken": "mock-access-token",
            "expires_in": 3600
        }),
        MockProvider::Anthropic => json!({
            "id": "msg_mock",
            "type": "message",
            "content": [{ "type": "text", "text": "Mock LLM response content." }],
            "stop_reason": "end_turn",
            "model": "claude-3-5-sonnet-20241022",
            "usage": { "input_tokens": 100, "output_tokens": 200 }
        }),
        MockProvider::OpenAi => json!({
            "id": "chatcmpl-mock",
            "object": "chat.completion",
            "model": "gpt-4",
            "choices": [{ "message": { "content": "Mock LLM response content." }, "finish_reason": "stop" }],
            "output": [{ "content": [{ "text": "Mock LLM response content." }] }],
            "usage": { "prompt_tokens": 100, "completion_tokens": 200 }
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
        MockProvider::OpenAi => json!({
            "error": {
                "type": openai_error_type(status, mode),
                "message": openai_error_message(status, mode),
                "code": openai_error_code(status, mode)
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

fn openai_error_type(status: u16, mode: &str) -> &'static str {
    match (status, mode) {
        (401, _) | (_, "auth") => "invalid_api_key",
        (429, _) | (_, "rate_limit") => "rate_limit_exceeded",
        (400, _) => "invalid_request_error",
        (404, _) | (_, "not_found") => "model_not_found",
        _ => "api_error",
    }
}

fn openai_error_message(status: u16, mode: &str) -> &'static str {
    match (status, mode) {
        (429, _) | (_, "rate_limit") => "Rate limit exceeded. Please retry after a short wait.",
        (401, _) | (_, "auth") => "Incorrect API key provided.",
        (404, _) | (_, "not_found") => "The model does not exist or you do not have access to it.",
        _ => "OpenAI API error",
    }
}

fn openai_error_code(status: u16, mode: &str) -> Option<&'static str> {
    match (status, mode) {
        (401, _) | (_, "auth") => Some("invalid_api_key"),
        (429, _) | (_, "rate_limit") => Some("rate_limit_exceeded"),
        (404, _) | (_, "not_found") => Some("model_not_found"),
        _ => None,
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

// ============================================================================
// TL-7: Response-block-driven mock synthesis
// ============================================================================

/// Infer the `MockProvider` from a response type name declared in `response {}` blocks.
///
/// This bridges the gap between DSL-declared response types (e.g., "GitHubErrorShape")
/// and the provider-aware mock synthesis system. When the response type name contains
/// a provider hint, we can generate realistic mock payloads.
pub fn provider_from_response_type(response_type: &str) -> MockProvider {
    let lower = response_type.to_lowercase();
    if lower.contains("github") || lower.contains("gist") {
        MockProvider::GitHub
    } else if lower.contains("gcp") {
        MockProvider::Gcp
    } else if lower.contains("anthropic") {
        MockProvider::Anthropic
    } else if lower.contains("openai") {
        MockProvider::OpenAi
    } else {
        MockProvider::Generic
    }
}

/// Infer the failure mode from an HTTP status code.
///
/// Maps common HTTP error codes to semantic failure mode tags used by
/// the provider-specific error shape generators.
pub fn failure_mode_from_status(status: u16) -> &'static str {
    match status {
        401 | 403 => "auth",
        404 => "not_found",
        429 => "rate_limit",
        409 => "conflict",
        422 => "validation",
        _ if status >= 500 => "server",
        _ => "unknown",
    }
}

/// Synthesize a mock REST response from a response mapping entry (TL-7).
///
/// Given a status code and response type name from a `response { STATUS => TYPE }`
/// declaration, generates a realistic mock response body that matches the
/// provider's error shape.
pub fn synthesize_from_response_entry(status: u16, response_type: &str) -> RestResponse {
    let provider = provider_from_response_type(response_type);
    let spec = if (200..300).contains(&status) {
        MockResponseSynthesis::success(provider)
    } else {
        let mode = failure_mode_from_status(status);
        MockResponseSynthesis {
            provider,
            status,
            failure_mode: Some(mode.to_string()),
            body_override: None,
        }
    };
    synthesize_rest_response(&spec)
}

// ============================================================================
// RT-1: mock_response block-driven synthesis
// ============================================================================

/// Synthesize a REST response directly from a `mock_response` block entry (RT-1).
///
/// Uses the exact JSON body declared in the DSL `mock_response { STATUS => { body } }`
/// block, providing operation-specific mock data instead of generic provider shapes.
///
/// Returns `None` if the body_json string cannot be parsed.
pub fn synthesize_from_mock_response_entry(status: u16, body_json: &str) -> Option<RestResponse> {
    let body: JsonValue = serde_json::from_str(body_json).ok()?;
    Some(RestResponse::new(status, body))
}

/// Find the best success mock response from a list of mock response entries.
///
/// Returns the first entry with a 2xx status code, or `None` if no success
/// response is declared.
pub fn find_success_mock(entries: &[(u16, String)]) -> Option<RestResponse> {
    entries
        .iter()
        .find(|(status, _)| (200..300).contains(status))
        .and_then(|(status, body_json)| synthesize_from_mock_response_entry(*status, body_json))
}

/// Find all error mock responses from a list of mock response entries.
///
/// Returns entries with non-2xx status codes.
pub fn find_error_mocks(entries: &[(u16, String)]) -> Vec<RestResponse> {
    entries
        .iter()
        .filter(|(status, _)| !(200..300).contains(status))
        .filter_map(|(status, body_json)| synthesize_from_mock_response_entry(*status, body_json))
        .collect()
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

    #[test]
    fn openai_success_synthesis_includes_choices_and_usage() {
        let response =
            synthesize_rest_response(&MockResponseSynthesis::success(MockProvider::OpenAi));
        assert_eq!(response.status, 200);
        assert!(response.body.get("choices").is_some());
        assert!(response.body.get("usage").is_some());
        assert_eq!(response.body["choices"][0]["finish_reason"], "stop");
    }

    #[test]
    fn openai_error_synthesis_uses_provider_shape() {
        let response = synthesize_rest_response(&MockResponseSynthesis::failure(
            MockProvider::OpenAi,
            429,
            "rate_limit",
        ));
        assert_eq!(response.status, 429);
        assert_eq!(response.body["error"]["type"], "rate_limit_exceeded");
    }

    // TL-7: Response-block-driven synthesis tests

    #[test]
    fn provider_from_response_type_infers_github() {
        assert_eq!(
            provider_from_response_type("GitHubErrorShape"),
            MockProvider::GitHub
        );
    }

    #[test]
    fn provider_from_response_type_infers_gcp() {
        assert_eq!(
            provider_from_response_type("GcpErrorShape"),
            MockProvider::Gcp
        );
    }

    #[test]
    fn provider_from_response_type_infers_anthropic() {
        assert_eq!(
            provider_from_response_type("AnthropicErrorShape"),
            MockProvider::Anthropic
        );
    }

    #[test]
    fn provider_from_response_type_infers_openai() {
        assert_eq!(
            provider_from_response_type("OpenAiErrorShape"),
            MockProvider::OpenAi
        );
    }

    #[test]
    fn provider_from_response_type_falls_back_to_generic() {
        assert_eq!(provider_from_response_type("Json"), MockProvider::Generic);
    }

    #[test]
    fn synthesize_from_response_entry_github_401() {
        let response = synthesize_from_response_entry(401, "GitHubErrorShape");
        assert_eq!(response.status, 401);
        assert_eq!(response.body["message"], "Bad credentials");
        assert!(response.body.get("documentation_url").is_some());
    }

    #[test]
    fn synthesize_from_response_entry_gcp_404() {
        let response = synthesize_from_response_entry(404, "GcpErrorShape");
        assert_eq!(response.status, 404);
        assert_eq!(response.body["error"]["status"], "NOT_FOUND");
    }

    #[test]
    fn synthesize_from_response_entry_success_200() {
        let response = synthesize_from_response_entry(200, "Json");
        assert_eq!(response.status, 200);
        assert_eq!(response.body["ok"], true);
    }

    // RT-1: mock_response entry synthesis tests

    #[test]
    fn synthesize_from_mock_response_entry_success() {
        let body_json = r#"{"id": "123", "name": "test"}"#;
        let response =
            synthesize_from_mock_response_entry(200, body_json).expect("valid JSON should parse");
        assert_eq!(response.status, 200);
        assert_eq!(response.body["id"], "123");
        assert_eq!(response.body["name"], "test");
    }

    #[test]
    fn synthesize_from_mock_response_entry_error() {
        let body_json = r#"{"error": "unauthorized", "message": "Bad credentials"}"#;
        let response =
            synthesize_from_mock_response_entry(401, body_json).expect("valid JSON should parse");
        assert_eq!(response.status, 401);
        assert_eq!(response.body["error"], "unauthorized");
    }

    #[test]
    fn synthesize_from_mock_response_entry_invalid_json_returns_none() {
        assert!(synthesize_from_mock_response_entry(200, "not json").is_none());
    }

    #[test]
    fn find_success_mock_picks_first_2xx() {
        let entries = vec![
            (401, r#"{"error": "auth"}"#.to_string()),
            (200, r#"{"id": "123"}"#.to_string()),
            (201, r#"{"id": "456"}"#.to_string()),
        ];
        let response = find_success_mock(&entries).expect("should find 200");
        assert_eq!(response.status, 200);
        assert_eq!(response.body["id"], "123");
    }

    #[test]
    fn find_error_mocks_returns_non_2xx() {
        let entries = vec![
            (200, r#"{"ok": true}"#.to_string()),
            (401, r#"{"error": "auth"}"#.to_string()),
            (500, r#"{"error": "server"}"#.to_string()),
        ];
        let errors = find_error_mocks(&entries);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].status, 401);
        assert_eq!(errors[1].status, 500);
    }
}
