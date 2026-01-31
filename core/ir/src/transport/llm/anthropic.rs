//! Anthropic Messages API conversions.
//!
//! Pure functions that convert between gunbc chat types and
//! Anthropic-specific REST request/response formats.
//!
//! # API Reference
//!
//! Anthropic Messages API:
//! - Endpoint: POST /v1/messages
//! - Auth: x-api-key header
//! - Request body: { model, messages: [{role, content}], system?, max_tokens, temperature? }
//! - Response body: { content: [{type, text}], model, stop_reason, usage }
//!
//! # Key Differences from OpenAI
//!
//! - System messages are a separate top-level field, not in the messages array
//! - Auth uses a custom header (x-api-key) instead of Bearer token
//! - Requires anthropic-version header
//! - Response content is an array of content blocks, not a single string
//! - max_tokens is required (not optional)

use super::chat::{ChatMessage, ChatRequest, ChatResponse, FinishReason, Role, Usage};
use super::provider::anthropic_provider;
use crate::transport::http::HttpMethod;
use crate::transport::rest::{AuthMethod, RestRequest, RestResponse};

/// Default max_tokens for Anthropic requests when not specified.
///
/// Anthropic requires max_tokens to be set. This provides a reasonable default.
const DEFAULT_MAX_TOKENS: u64 = 4096;

/// Build an Anthropic Messages API request from a chat request.
///
/// This is a PURE function - no I/O. The resulting `RestRequest` should be
/// executed via `TransportOps::Execute`.
///
/// Handles the Anthropic-specific format differences:
/// - System messages extracted to a top-level `system` field
/// - Auth via `x-api-key` header (using `AuthMethod::ApiKey`)
/// - Required `anthropic-version` header
/// - `max_tokens` is required (defaults to 4096 if not set)
pub fn build_anthropic_request(chat: &ChatRequest) -> RestRequest {
    let provider = anthropic_provider();

    // Anthropic expects system as a separate top-level field
    let system_text: String = chat
        .system_messages()
        .iter()
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    let messages: Vec<serde_json::Value> = chat
        .non_system_messages()
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role.as_str(),
                "content": m.content,
            })
        })
        .collect();

    let max_tokens = chat.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

    let mut body = serde_json::json!({
        "model": chat.model,
        "messages": messages,
        "max_tokens": max_tokens,
    });

    if !system_text.is_empty() {
        body["system"] = serde_json::json!(system_text);
    }
    if let Some(temp) = chat.temperature {
        body["temperature"] = serde_json::json!(temp);
    }
    if !chat.stop.is_empty() {
        body["stop_sequences"] = serde_json::json!(chat.stop);
    }

    let mut headers = std::collections::HashMap::new();
    headers.insert(
        "Content-Type".to_string(),
        "application/json".to_string(),
    );
    for (k, v) in &provider.extra_headers {
        headers.insert(k.clone(), v.clone());
    }

    RestRequest {
        url: provider.chat_url(),
        method: HttpMethod::Post,
        headers,
        body: Some(body),
        auth: Some(AuthMethod::ApiKey {
            header: "x-api-key".to_string(),
            key: format!("${{{}}}", provider.api_key_env.0),
        }),
        query: Default::default(),
        timeout_ms: Some(120_000),
    }
}

/// Parse an Anthropic Messages API response.
///
/// Extracts the assistant's message content from the content blocks,
/// along with stop reason and usage statistics.
///
/// Returns `Err` if the response body doesn't match the expected format.
pub fn parse_anthropic_response(response: &RestResponse) -> Result<ChatResponse, String> {
    if !response.is_success() {
        let error_msg = response
            .body
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        return Err(format!(
            "Anthropic API error (status {}): {}",
            response.status, error_msg
        ));
    }

    // Anthropic returns content as an array of blocks: [{type: "text", text: "..."}]
    let content = response
        .body
        .get("content")
        .and_then(|c| c.as_array())
        .map(|blocks| {
            blocks
                .iter()
                .filter_map(|block| {
                    if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                        block.get("text").and_then(|t| t.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    let model = response
        .body
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown")
        .to_string();

    let finish_reason = response
        .body
        .get("stop_reason")
        .and_then(|r| r.as_str())
        .map(FinishReason::from_anthropic)
        .unwrap_or(FinishReason::Stop);

    let usage = response
        .body
        .get("usage")
        .map(|u| Usage {
            input_tokens: u
                .get("input_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0),
            output_tokens: u
                .get("output_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0),
        })
        .unwrap_or_default();

    Ok(ChatResponse {
        content,
        model,
        finish_reason,
        usage,
    })
}

/// Convert a `ChatResponse` back into a `ChatMessage` for multi-turn conversations.
pub fn response_to_message(response: &ChatResponse) -> ChatMessage {
    ChatMessage {
        role: Role::Assistant,
        content: response.content.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> ChatRequest {
        ChatRequest::new(
            "claude-sonnet-4-20250514",
            vec![
                ChatMessage::system("You are a code reviewer."),
                ChatMessage::user("Review this function."),
            ],
        )
        .temperature(0.3)
        .max_tokens(2048)
    }

    #[test]
    fn test_build_anthropic_request() {
        let req = build_anthropic_request(&sample_request());

        assert_eq!(req.url, "https://api.anthropic.com/v1/messages");
        assert_eq!(req.method, HttpMethod::Post);

        // Check anthropic-version header
        assert_eq!(
            req.headers.get("anthropic-version"),
            Some(&"2023-06-01".to_string())
        );

        let body = req.body.unwrap();
        assert_eq!(body["model"], "claude-sonnet-4-20250514");
        assert_eq!(body["max_tokens"], 2048);
        assert_eq!(body["temperature"], 0.3);

        // System should be a top-level field, not in messages
        assert_eq!(body["system"], "You are a code reviewer.");

        // Messages should only contain non-system messages
        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
    }

    #[test]
    fn test_build_anthropic_request_no_system() {
        let req = build_anthropic_request(&ChatRequest::new(
            "claude-sonnet-4-20250514",
            vec![ChatMessage::user("Hello")],
        ));

        let body = req.body.unwrap();
        assert!(body.get("system").is_none());
    }

    #[test]
    fn test_build_anthropic_request_default_max_tokens() {
        let req = build_anthropic_request(&ChatRequest::new(
            "claude-sonnet-4-20250514",
            vec![ChatMessage::user("Hello")],
        ));

        let body = req.body.unwrap();
        assert_eq!(body["max_tokens"], DEFAULT_MAX_TOKENS);
    }

    #[test]
    fn test_parse_anthropic_response_success() {
        let response = RestResponse::ok(serde_json::json!({
            "id": "msg_abc123",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "The function looks good overall."
                }
            ],
            "model": "claude-sonnet-4-20250514",
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": 30,
                "output_tokens": 15
            }
        }));

        let chat = parse_anthropic_response(&response).unwrap();
        assert_eq!(chat.content, "The function looks good overall.");
        assert_eq!(chat.model, "claude-sonnet-4-20250514");
        assert_eq!(chat.finish_reason, FinishReason::Stop);
        assert_eq!(chat.usage.input_tokens, 30);
        assert_eq!(chat.usage.output_tokens, 15);
    }

    #[test]
    fn test_parse_anthropic_response_multi_block() {
        let response = RestResponse::ok(serde_json::json!({
            "content": [
                { "type": "text", "text": "First part. " },
                { "type": "text", "text": "Second part." }
            ],
            "model": "claude-sonnet-4-20250514",
            "stop_reason": "end_turn",
            "usage": { "input_tokens": 10, "output_tokens": 20 }
        }));

        let chat = parse_anthropic_response(&response).unwrap();
        assert_eq!(chat.content, "First part. Second part.");
    }

    #[test]
    fn test_parse_anthropic_response_error() {
        let response = RestResponse::new(
            401,
            serde_json::json!({
                "type": "error",
                "error": {
                    "type": "authentication_error",
                    "message": "invalid x-api-key"
                }
            }),
        );

        let err = parse_anthropic_response(&response).unwrap_err();
        assert!(err.contains("invalid x-api-key"));
        assert!(err.contains("401"));
    }

    #[test]
    fn test_parse_anthropic_response_max_tokens() {
        let response = RestResponse::ok(serde_json::json!({
            "content": [{ "type": "text", "text": "partial..." }],
            "model": "claude-sonnet-4-20250514",
            "stop_reason": "max_tokens",
            "usage": { "input_tokens": 10, "output_tokens": 100 }
        }));

        let chat = parse_anthropic_response(&response).unwrap();
        assert_eq!(chat.finish_reason, FinishReason::Length);
    }

    #[test]
    fn test_response_to_message() {
        let response = ChatResponse {
            content: "Hello!".to_string(),
            model: "test".to_string(),
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
        };

        let msg = response_to_message(&response);
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content, "Hello!");
    }
}
