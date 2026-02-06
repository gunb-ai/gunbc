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
//! - Request body: { model, messages, system?, max_tokens, temperature?, thinking? }
//! - Response body: { content: [{type, text}], model, stop_reason, usage }
//!
//! # Prompt Caching
//!
//! Anthropic supports explicit prompt caching via `cache_control` on content blocks:
//! - System messages as content block arrays with `cache_control: {"type": "ephemeral"}`
//! - Min cacheable: 1024 tokens (Sonnet/Opus 4), 2048 (Haiku 3), 4096 (Opus 4.5, Haiku 4.5)
//! - Up to 4 breakpoints; caches from request start to each breakpoint
//! - 5-minute default TTL; cache reads cost 10%, writes cost 125% of base
//!
//! Ref: <https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching>
//!
//! # Extended Thinking
//!
//! Enabled via `thinking: {type: "enabled", budget_tokens: N}` in the request.
//! Response includes `thinking` content blocks before `text` blocks.
//! Budget must be < max_tokens. Incompatible with temperature/top_k.
//! Supported: Claude Sonnet 3.7+, Haiku 4.5, Opus 4+.
//! Ref: <https://docs.anthropic.com/en/docs/build-with-claude/extended-thinking>
//!
//! # Key Differences from OpenAI
//!
//! - System messages are a separate top-level field, not in the messages array
//! - System supports content block arrays (required for cache_control)
//! - Auth uses a custom header (x-api-key) instead of Bearer token
//! - Requires anthropic-version header
//! - Response content is an array of content blocks, not a single string
//! - max_tokens is required (not optional)
//! - Extended thinking uses budget_tokens (vs OpenAI's effort level)

use super::chat::{
    CacheControl, ChatMessage, ChatRequest, ChatResponse, ContentBlock, FinishReason,
    MessageContent, ResponseBlock, Role, ThinkingConfig, Usage,
};
use super::provider::anthropic_provider;
use crate::transport::http::HttpMethod;
use crate::transport::rest::{RestRequest, RestResponse};

/// Default max_tokens for Anthropic requests when not specified.
///
/// Anthropic requires max_tokens to be set. This provides a reasonable default.
const DEFAULT_MAX_TOKENS: u64 = 4096;

/// Serialize a message's content to a JSON value for the Anthropic API.
///
/// Simple text becomes a plain string. Structured blocks become an array
/// of content block objects with optional `cache_control`.
fn serialize_content(content: &MessageContent) -> serde_json::Value {
    match content {
        MessageContent::Text(s) => serde_json::json!(s),
        MessageContent::Blocks(blocks) => {
            let items: Vec<serde_json::Value> = blocks
                .iter()
                .map(|block| match block {
                    ContentBlock::Text {
                        text,
                        cache_control,
                    } => {
                        let mut obj = serde_json::json!({"type": "text", "text": text});
                        if let Some(CacheControl { .. }) = cache_control {
                            obj["cache_control"] = serde_json::json!({"type": "ephemeral"});
                        }
                        obj
                    }
                    ContentBlock::Thinking {
                        thinking,
                        signature,
                    } => serde_json::json!({
                        "type": "thinking",
                        "thinking": thinking,
                        "signature": signature,
                    }),
                    ContentBlock::RedactedThinking { data } => {
                        serde_json::json!({"type": "redacted_thinking", "data": data})
                    }
                })
                .collect();
            serde_json::json!(items)
        }
    }
}

/// Build an Anthropic Messages API request from a chat request.
///
/// This is a PURE function - no I/O. The resulting `RestRequest` should be
/// executed via `TransportOps::Execute`.
///
/// Handles the Anthropic-specific format:
/// - System messages as content block array (supports cache_control breakpoints)
/// - Auth via `x-api-key` header (credential applied at boundary)
/// - Required `anthropic-version` header
/// - `max_tokens` is required (defaults to 4096 if not set)
/// - Extended thinking via `thinking` param when configured
pub fn build_anthropic_request(chat: &ChatRequest) -> RestRequest {
    let provider = anthropic_provider();

    // System messages → top-level `system` field.
    // If any system message uses content blocks (for cache_control), we emit
    // the full content block array format. Otherwise, a simple string.
    let system_msgs = chat.system_messages();
    let has_blocks = system_msgs.iter().any(|m| m.content.is_blocks());

    let system_value = if system_msgs.is_empty() {
        None
    } else if has_blocks {
        // Content block array format (required for cache_control)
        let mut blocks: Vec<serde_json::Value> = Vec::new();
        for msg in &system_msgs {
            match &msg.content {
                MessageContent::Text(s) => {
                    blocks.push(serde_json::json!({"type": "text", "text": s}));
                }
                MessageContent::Blocks(msg_blocks) => {
                    for block in msg_blocks {
                        if let ContentBlock::Text {
                            text,
                            cache_control,
                        } = block
                        {
                            let mut obj = serde_json::json!({"type": "text", "text": text});
                            if let Some(CacheControl { .. }) = cache_control {
                                obj["cache_control"] = serde_json::json!({"type": "ephemeral"});
                            }
                            blocks.push(obj);
                        }
                    }
                }
            }
        }
        Some(serde_json::json!(blocks))
    } else {
        // Simple string format (no cache_control needed)
        let text: String = system_msgs
            .iter()
            .map(|m| m.text())
            .collect::<Vec<_>>()
            .join("\n\n");
        Some(serde_json::json!(text))
    };

    let messages: Vec<serde_json::Value> = chat
        .non_system_messages()
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role.as_str(),
                "content": serialize_content(&m.content),
            })
        })
        .collect();

    let max_tokens = chat.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

    let mut body = serde_json::json!({
        "model": chat.model,
        "messages": messages,
        "max_tokens": max_tokens,
    });

    if let Some(system) = system_value {
        body["system"] = system;
    }
    if let Some(temp) = chat.temperature {
        body["temperature"] = serde_json::json!(temp);
    }
    if !chat.stop.is_empty() {
        body["stop_sequences"] = serde_json::json!(chat.stop);
    }

    // Extended thinking
    if let Some(ThinkingConfig::Anthropic { budget_tokens }) = &chat.thinking {
        body["thinking"] = serde_json::json!({
            "type": "enabled",
            "budget_tokens": budget_tokens,
        });
    }

    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    for (k, v) in &provider.extra_headers {
        headers.insert(k.clone(), v.clone());
    }

    RestRequest {
        url: provider.chat_url(),
        method: HttpMethod::Post,
        headers,
        body: Some(body),
        auth: None,
        query: Default::default(),
        timeout_ms: Some(120_000),
    }
}

/// Parse an Anthropic Messages API response.
///
/// Extracts the assistant's message content from the content blocks,
/// along with stop reason, usage statistics, and thinking blocks.
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

    // Parse content blocks
    let mut text_parts: Vec<String> = Vec::new();
    let mut thinking_parts: Vec<String> = Vec::new();
    let mut response_blocks: Vec<ResponseBlock> = Vec::new();

    let content_array = response
        .body
        .get("content")
        .and_then(|c| c.as_array())
        .ok_or_else(|| "Anthropic response missing content array".to_string())?;
    if content_array.is_empty() {
        return Err("Anthropic response content array is empty".to_string());
    }
    for block in content_array {
        let block_type = block
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| "Anthropic response content block missing type".to_string())?;
        match block_type {
            "text" => {
                let text = block
                    .get("text")
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| "Anthropic response text block missing text".to_string())?
                    .to_string();
                text_parts.push(text.clone());
                response_blocks.push(ResponseBlock::Text { text });
            }
            "thinking" => {
                let thinking = block
                    .get("thinking")
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| "Anthropic response thinking block missing thinking".to_string())?
                    .to_string();
                let signature = block
                    .get("signature")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Anthropic response thinking block missing signature".to_string())?
                    .to_string();
                thinking_parts.push(thinking.clone());
                response_blocks.push(ResponseBlock::Thinking {
                    thinking,
                    signature: Some(signature),
                });
            }
            "redacted_thinking" => {
                let data = block
                    .get("data")
                    .and_then(|d| d.as_str())
                    .ok_or_else(|| {
                        "Anthropic response redacted_thinking block missing data".to_string()
                    })?
                    .to_string();
                response_blocks.push(ResponseBlock::RedactedThinking { data });
            }
            other => {
                return Err(format!(
                    "Anthropic response unknown content block type: {}",
                    other
                ))
            }
        }
    }

    let content = text_parts.join("");

    let model = response
        .body
        .get("model")
        .and_then(|m| m.as_str())
        .ok_or_else(|| "Anthropic response missing model".to_string())?
        .to_string();

    let stop_reason = response
        .body
        .get("stop_reason")
        .and_then(|r| r.as_str())
        .ok_or_else(|| "Anthropic response missing stop_reason".to_string())?;
    let finish_reason = match stop_reason {
        "end_turn" => FinishReason::Stop,
        "max_tokens" => FinishReason::Length,
        other => {
            return Err(format!(
                "Anthropic response unknown stop_reason: {}",
                other
            ))
        }
    };

    let usage = response
        .body
        .get("usage")
        .ok_or_else(|| "Anthropic response missing usage".to_string())?;
    let input_tokens = usage
        .get("input_tokens")
        .and_then(|t| t.as_u64())
        .ok_or_else(|| "Anthropic response missing usage.input_tokens".to_string())?;
    let output_tokens = usage
        .get("output_tokens")
        .and_then(|t| t.as_u64())
        .ok_or_else(|| "Anthropic response missing usage.output_tokens".to_string())?;
    let usage = Usage {
        input_tokens,
        output_tokens,
        cache_creation_input_tokens: usage
            .get("cache_creation_input_tokens")
            .and_then(|t| t.as_u64()),
        cache_read_input_tokens: usage.get("cache_read_input_tokens").and_then(|t| t.as_u64()),
        cached_tokens: None,
        reasoning_tokens: None,
    };

    let thinking = if thinking_parts.is_empty() {
        None
    } else {
        Some(thinking_parts.join("\n"))
    };

    // Only include content_blocks if there were non-text blocks
    let has_non_text = response_blocks
        .iter()
        .any(|b| !matches!(b, ResponseBlock::Text { .. }));
    let content_blocks = if has_non_text {
        response_blocks
    } else {
        Vec::new()
    };

    Ok(ChatResponse {
        content,
        model,
        finish_reason,
        usage,
        thinking,
        content_blocks,
    })
}

/// Convert a `ChatResponse` back into a `ChatMessage` for multi-turn conversations.
pub fn response_to_message(response: &ChatResponse) -> ChatMessage {
    if response.content_blocks.is_empty() {
        ChatMessage {
            role: Role::Assistant,
            content: MessageContent::Text(response.content.clone()),
        }
    } else {
        // Preserve thinking blocks for multi-turn tool use
        let blocks = response
            .content_blocks
            .iter()
            .map(|b| match b {
                ResponseBlock::Text { text } => ContentBlock::text(text),
                ResponseBlock::Thinking {
                    thinking,
                    signature,
                } => ContentBlock::Thinking {
                    thinking: thinking.clone(),
                    signature: signature.clone().unwrap_or_default(),
                },
                ResponseBlock::RedactedThinking { data } => {
                    ContentBlock::RedactedThinking { data: data.clone() }
                }
            })
            .collect();
        ChatMessage::assistant_blocks(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::llm::chat::CacheControl;

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
    fn test_build_anthropic_request_cache_control() {
        let req = build_anthropic_request(&ChatRequest::new(
            "claude-sonnet-4-20250514",
            vec![
                ChatMessage::system_blocks(vec![
                    ContentBlock::text("Long system prompt here...")
                        .with_cache(CacheControl::ephemeral()),
                    ContentBlock::text("Variable instructions"),
                ]),
                ChatMessage::user("Hello"),
            ],
        ));

        let body = req.body.unwrap();
        let system = body["system"].as_array().unwrap();
        assert_eq!(system.len(), 2);
        assert_eq!(system[0]["type"], "text");
        assert_eq!(system[0]["text"], "Long system prompt here...");
        assert_eq!(
            system[0]["cache_control"],
            serde_json::json!({"type": "ephemeral"})
        );
        assert!(system[1].get("cache_control").is_none());
    }

    #[test]
    fn test_build_anthropic_request_thinking() {
        let req = build_anthropic_request(
            &ChatRequest::new("claude-sonnet-4-5", vec![ChatMessage::user("Think hard.")])
                .max_tokens(16000)
                .thinking(ThinkingConfig::anthropic(10000)),
        );

        let body = req.body.unwrap();
        assert_eq!(body["thinking"]["type"], "enabled");
        assert_eq!(body["thinking"]["budget_tokens"], 10000);
        // Temperature should not be set when thinking is enabled
        assert!(body.get("temperature").is_none());
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
        assert!(chat.thinking.is_none());
        assert!(chat.content_blocks.is_empty());
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
    fn test_parse_anthropic_response_with_thinking() {
        let response = RestResponse::ok(serde_json::json!({
            "content": [
                {
                    "type": "thinking",
                    "thinking": "Let me analyze step by step...",
                    "signature": "abc123sig"
                },
                {
                    "type": "text",
                    "text": "Based on my analysis..."
                }
            ],
            "model": "claude-sonnet-4-5",
            "stop_reason": "end_turn",
            "usage": { "input_tokens": 30, "output_tokens": 500 }
        }));

        let chat = parse_anthropic_response(&response).unwrap();
        assert_eq!(chat.content, "Based on my analysis...");
        assert_eq!(
            chat.thinking,
            Some("Let me analyze step by step...".to_string())
        );
        assert_eq!(chat.content_blocks.len(), 2);
        assert!(matches!(
            &chat.content_blocks[0],
            ResponseBlock::Thinking { thinking, signature: Some(sig) }
            if thinking == "Let me analyze step by step..." && sig == "abc123sig"
        ));
    }

    #[test]
    fn test_parse_anthropic_response_cache_usage() {
        let response = RestResponse::ok(serde_json::json!({
            "content": [{ "type": "text", "text": "Cached response" }],
            "model": "claude-sonnet-4-5",
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": 17,
                "output_tokens": 700,
                "cache_creation_input_tokens": 1370,
                "cache_read_input_tokens": 0
            }
        }));

        let chat = parse_anthropic_response(&response).unwrap();
        assert_eq!(chat.usage.cache_creation_input_tokens, Some(1370));
        assert_eq!(chat.usage.cache_read_input_tokens, Some(0));
        assert!(chat.usage.cached_tokens.is_none());
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
    fn test_response_to_message_simple() {
        let response = ChatResponse {
            content: "Hello!".to_string(),
            model: "test".to_string(),
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
            thinking: None,
            content_blocks: Vec::new(),
        };

        let msg = response_to_message(&response);
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.text(), "Hello!");
    }

    #[test]
    fn test_response_to_message_with_thinking() {
        let response = ChatResponse {
            content: "Answer".to_string(),
            model: "test".to_string(),
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
            thinking: Some("Reasoning...".to_string()),
            content_blocks: vec![
                ResponseBlock::Thinking {
                    thinking: "Reasoning...".to_string(),
                    signature: Some("sig123".to_string()),
                },
                ResponseBlock::Text {
                    text: "Answer".to_string(),
                },
            ],
        };

        let msg = response_to_message(&response);
        assert_eq!(msg.role, Role::Assistant);
        assert!(msg.content.is_blocks());
        let blocks = msg.content.blocks().unwrap();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], ContentBlock::Thinking { .. }));
    }
}
