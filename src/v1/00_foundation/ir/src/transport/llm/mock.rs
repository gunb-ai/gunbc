//! Mock response builders for LLM provider testing.
//!
//! These builders create realistic mock responses for dry-run testing
//! of LLM DAG nodes. Each provider has its own response format, and
//! these helpers produce structurally valid responses without requiring
//! actual API calls.

use crate::transport::rest::RestResponse;

/// Build a mock OpenAI chat completion response.
///
/// Creates a structurally valid OpenAI API response for dry-run testing.
pub fn mock_openai_response(content: &str) -> RestResponse {
    mock_openai_response_full(content, "gpt-4o", "stop", 10, 20)
}

/// Build a mock OpenAI response with full control over fields.
pub fn mock_openai_response_full(
    content: &str,
    model: &str,
    finish_reason: &str,
    prompt_tokens: u64,
    completion_tokens: u64,
) -> RestResponse {
    RestResponse::ok(serde_json::json!({
        "id": "chatcmpl-mock-test",
        "object": "chat.completion",
        "model": model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content,
            },
            "finish_reason": finish_reason,
        }],
        "usage": {
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": prompt_tokens + completion_tokens,
        }
    }))
}

/// Build a mock OpenAI error response.
pub fn mock_openai_error(status: u16, error_type: &str, message: &str) -> RestResponse {
    RestResponse::new(
        status,
        serde_json::json!({
            "error": {
                "message": message,
                "type": error_type,
            }
        }),
    )
}

/// Build a mock Anthropic Messages API response.
///
/// Creates a structurally valid Anthropic API response for dry-run testing.
pub fn mock_anthropic_response(content: &str) -> RestResponse {
    mock_anthropic_response_full(content, "claude-sonnet-4-20250514", "end_turn", 10, 20)
}

/// Build a mock Anthropic response with full control over fields.
pub fn mock_anthropic_response_full(
    content: &str,
    model: &str,
    stop_reason: &str,
    input_tokens: u64,
    output_tokens: u64,
) -> RestResponse {
    RestResponse::ok(serde_json::json!({
        "id": "msg_mock_test",
        "type": "message",
        "role": "assistant",
        "content": [{
            "type": "text",
            "text": content,
        }],
        "model": model,
        "stop_reason": stop_reason,
        "usage": {
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
        }
    }))
}

/// Build a mock Anthropic error response.
pub fn mock_anthropic_error(status: u16, error_type: &str, message: &str) -> RestResponse {
    RestResponse::new(
        status,
        serde_json::json!({
            "type": "error",
            "error": {
                "type": error_type,
                "message": message,
            }
        }),
    )
}

/// Build a mock REST response wrapping a provider-specific response.
///
/// Dispatches to the appropriate mock builder based on provider ID.
pub fn mock_llm_response(provider_id: &str, content: &str) -> Option<RestResponse> {
    match provider_id {
        "openai" => Some(mock_openai_response(content)),
        "anthropic" => Some(mock_anthropic_response(content)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::llm::{anthropic, openai};

    #[test]
    fn test_mock_openai_response_parses() {
        let resp = mock_openai_response("Hello, world!");
        let parsed = openai::parse_openai_response(&resp).unwrap();
        assert_eq!(parsed.content, "Hello, world!");
        assert_eq!(parsed.model, "gpt-4o");
    }

    #[test]
    fn test_mock_openai_response_full_parses() {
        let resp = mock_openai_response_full("Test", "gpt-4o-mini", "length", 5, 50);
        let parsed = openai::parse_openai_response(&resp).unwrap();
        assert_eq!(parsed.content, "Test");
        assert_eq!(parsed.model, "gpt-4o-mini");
        assert_eq!(
            parsed.finish_reason,
            crate::transport::llm::FinishReason::Length
        );
        assert_eq!(parsed.usage.input_tokens, 5);
        assert_eq!(parsed.usage.output_tokens, 50);
    }

    #[test]
    fn test_mock_openai_error_parses() {
        let resp = mock_openai_error(401, "invalid_api_key", "bad key");
        let err = openai::parse_openai_response(&resp).unwrap_err();
        assert!(err.contains("bad key"));
    }

    #[test]
    fn test_mock_anthropic_response_parses() {
        let resp = mock_anthropic_response("Hello!");
        let parsed = anthropic::parse_anthropic_response(&resp).unwrap();
        assert_eq!(parsed.content, "Hello!");
        assert_eq!(parsed.model, "claude-sonnet-4-20250514");
    }

    #[test]
    fn test_mock_anthropic_response_full_parses() {
        let resp =
            mock_anthropic_response_full("Test", "claude-haiku-3-20250414", "max_tokens", 8, 100);
        let parsed = anthropic::parse_anthropic_response(&resp).unwrap();
        assert_eq!(parsed.content, "Test");
        assert_eq!(
            parsed.finish_reason,
            crate::transport::llm::FinishReason::Length
        );
    }

    #[test]
    fn test_mock_anthropic_error_parses() {
        let resp = mock_anthropic_error(429, "rate_limit_error", "too many requests");
        let err = anthropic::parse_anthropic_response(&resp).unwrap_err();
        assert!(err.contains("too many requests"));
    }

    #[test]
    fn test_mock_llm_response_dispatch() {
        assert!(mock_llm_response("openai", "hi").is_some());
        assert!(mock_llm_response("anthropic", "hi").is_some());
        assert!(mock_llm_response("unknown", "hi").is_none());
    }
}
