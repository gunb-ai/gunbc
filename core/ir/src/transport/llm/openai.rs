//! OpenAI Chat Completions API conversions.
//!
//! Pure functions that convert between gunbc chat types and
//! OpenAI-specific REST request/response formats.
//!
//! # API Reference
//!
//! OpenAI Chat Completions API:
//! - Endpoint: POST /v1/chat/completions
//! - Auth: Bearer token (Authorization header)
//! - Request body: { model, messages, temperature?, max_completion_tokens?, reasoning_effort? }
//! - Response body: { choices: [{message, finish_reason}], usage }
//!
//! # Prompt Caching
//!
//! OpenAI applies automatic prefix caching for identical prefixes >= 1024 tokens.
//! No API changes needed; place static content (system prompt, examples) first
//! for best cache hit rate. Cached tokens cost 50% of input price.
//! Response includes `usage.prompt_tokens_details.cached_tokens`.
//! Ref: <https://platform.openai.com/docs/guides/prompt-caching>
//!
//! # Reasoning Models
//!
//! Reasoning models (o1, o3, o4-mini) use the `reasoning_effort` parameter:
//! - `reasoning_effort`: "low" | "medium" | "high" (default: "medium")
//! - Uses `max_completion_tokens` instead of `max_tokens` to include reasoning tokens
//! - Chat Completions does NOT support reasoning summaries (use Responses API for that)
//!
//! Ref: <https://platform.openai.com/docs/guides/reasoning>
//!
//! # Note on the Responses API
//!
//! For reasoning models, the Responses API (`/v1/responses`) is recommended over
//! Chat Completions. It supports reasoning summaries, persisted reasoning between
//! tool calls, and built-in tools. See `openai_responses.rs` for that endpoint.

use super::chat::{ChatRequest, ChatResponse, FinishReason, MessageContent, ThinkingConfig, Usage};
use super::provider::openai_provider;
use crate::transport::http::HttpMethod;
use crate::transport::rest::{RestRequest, RestResponse};

/// Build an OpenAI Chat Completions API request from a chat request.
///
/// This is a PURE function - no I/O.
///
/// Handles OpenAI-specific format:
/// - System messages stay in the messages array (unlike Anthropic)
/// - Auth via Bearer token
/// - Reasoning models use `max_completion_tokens` and `reasoning_effort`
/// - Content blocks with cache hints are flattened to strings (OpenAI caching
///   is automatic; no API-level cache control)
pub fn build_openai_request(chat: &ChatRequest) -> RestRequest {
    let provider = openai_provider();

    let messages: Vec<serde_json::Value> = chat
        .messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role.as_str(),
                "content": content_to_string(&m.content),
            })
        })
        .collect();

    let has_reasoning = matches!(chat.thinking, Some(ThinkingConfig::OpenAI { .. }));

    let mut body = serde_json::json!({
        "model": chat.model,
        "messages": messages,
    });

    // Reasoning models use max_completion_tokens (includes reasoning tokens)
    if let Some(max) = chat.max_tokens {
        if has_reasoning {
            body["max_completion_tokens"] = serde_json::json!(max);
        } else {
            body["max_tokens"] = serde_json::json!(max);
        }
    }

    if let Some(temp) = chat.temperature {
        body["temperature"] = serde_json::json!(temp);
    }
    if !chat.stop.is_empty() {
        body["stop"] = serde_json::json!(chat.stop);
    }

    // Reasoning effort for o1/o3/o4-mini
    if let Some(ThinkingConfig::OpenAI { effort, .. }) = &chat.thinking {
        body["reasoning_effort"] = serde_json::json!(effort.as_str());
    }

    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

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

/// Flatten message content to a plain string for the OpenAI Chat Completions API.
///
/// OpenAI Chat Completions expects string content. Content blocks are
/// concatenated; cache hints are silently ignored (caching is automatic).
fn content_to_string(content: &MessageContent) -> String {
    content.text()
}

/// Parse an OpenAI Chat Completions API response.
///
/// Extracts the assistant message content, finish reason, and token usage.
/// Includes cached_tokens and reasoning_tokens from detailed usage when present.
///
/// Returns `Err` if the response body doesn't match the expected format.
pub fn parse_openai_response(response: &RestResponse) -> Result<ChatResponse, String> {
    if !response.is_success() {
        let error_msg = response
            .body
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        return Err(format!(
            "OpenAI API error (status {}): {}",
            response.status, error_msg
        ));
    }

    let choices = response
        .body
        .get("choices")
        .and_then(|c| c.as_array())
        .ok_or("missing 'choices' array in response")?;

    let first = choices.first().ok_or("empty 'choices' array")?;

    let content = first
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or("OpenAI response missing message.content")?
        .to_string();

    let model = response
        .body
        .get("model")
        .and_then(|m| m.as_str())
        .ok_or("OpenAI response missing model")?
        .to_string();

    let finish_reason_str = first
        .get("finish_reason")
        .and_then(|r| r.as_str())
        .ok_or("OpenAI response missing finish_reason")?;
    let finish_reason = match finish_reason_str {
        "stop" => FinishReason::Stop,
        "length" => FinishReason::Length,
        "content_filter" => FinishReason::ContentFilter,
        other => {
            return Err(format!(
                "OpenAI response has unknown finish_reason: {}",
                other
            ))
        }
    };

    let usage = response
        .body
        .get("usage")
        .ok_or("OpenAI response missing usage")?;
    // Detailed token breakdown (if present)
    let cached_tokens = usage
        .get("prompt_tokens_details")
        .and_then(|d| d.get("cached_tokens"))
        .and_then(|t| t.as_u64());

    let reasoning_tokens = usage
        .get("completion_tokens_details")
        .and_then(|d| d.get("reasoning_tokens"))
        .and_then(|t| t.as_u64());

    let input_tokens = usage
        .get("prompt_tokens")
        .and_then(|t| t.as_u64())
        .ok_or("OpenAI response missing usage.prompt_tokens")?;
    let output_tokens = usage
        .get("completion_tokens")
        .and_then(|t| t.as_u64())
        .ok_or("OpenAI response missing usage.completion_tokens")?;
    let usage = Usage {
        input_tokens,
        output_tokens,
        cache_creation_input_tokens: None,
        cache_read_input_tokens: None,
        cached_tokens,
        reasoning_tokens,
    };

    Ok(ChatResponse {
        content,
        model,
        finish_reason,
        usage,
        thinking: None, // Chat Completions doesn't expose reasoning text
        content_blocks: Vec::new(),
    })
}

/// Build a request for an OpenAI-compatible provider with a custom base URL.
///
/// This enables use of OpenAI-compatible APIs (e.g., local models via Ollama,
/// Azure OpenAI, etc.) by overriding the endpoint URL while keeping the
/// same request format.
pub fn build_openai_compatible_request(
    chat: &ChatRequest,
    base_url: &str,
) -> RestRequest {
    let mut req = build_openai_request(chat);
    req.url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));
    req
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::llm::chat::{
        ChatMessage, ContentBlock, ReasoningEffort, ReasoningSummary,
    };

    fn sample_request() -> ChatRequest {
        ChatRequest::new(
            "gpt-4o",
            vec![
                ChatMessage::system("You are a code reviewer."),
                ChatMessage::user("Review this function."),
            ],
        )
        .temperature(0.3)
        .max_tokens(2048)
    }

    #[test]
    fn test_build_openai_request() {
        let req = build_openai_request(&sample_request());

        assert_eq!(req.url, "https://api.openai.com/v1/chat/completions");
        assert_eq!(req.method, HttpMethod::Post);
        assert!(req.auth.is_none());

        let body = req.body.unwrap();
        assert_eq!(body["model"], "gpt-4o");
        assert_eq!(body["temperature"], 0.3);
        assert_eq!(body["max_tokens"], 2048);

        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[1]["role"], "user");
    }

    #[test]
    fn test_build_openai_request_minimal() {
        let req = build_openai_request(&ChatRequest::new(
            "gpt-4o-mini",
            vec![ChatMessage::user("Hello")],
        ));

        let body = req.body.unwrap();
        assert!(body.get("temperature").is_none());
        assert!(body.get("max_tokens").is_none());
        assert!(body.get("stop").is_none());
    }

    #[test]
    fn test_build_openai_request_reasoning() {
        let req = build_openai_request(
            &ChatRequest::new("o3", vec![ChatMessage::user("Reason about this.")])
                .max_tokens(8000)
                .thinking(ThinkingConfig::openai_with_summary(
                    ReasoningEffort::High,
                    ReasoningSummary::Concise,
                )),
        );

        let body = req.body.unwrap();
        // Should use max_completion_tokens for reasoning models
        assert!(body.get("max_tokens").is_none());
        assert_eq!(body["max_completion_tokens"], 8000);
        assert_eq!(body["reasoning_effort"], "high");
    }

    #[test]
    fn test_build_openai_request_content_blocks_flattened() {
        let req = build_openai_request(&ChatRequest::new(
            "gpt-4o",
            vec![
                ChatMessage::system_blocks(vec![
                    ContentBlock::text("Part 1"),
                    ContentBlock::text("Part 2"),
                ]),
                ChatMessage::user("Hello"),
            ],
        ));

        let body = req.body.unwrap();
        let messages = body["messages"].as_array().unwrap();
        // Content blocks flattened to concatenated string
        assert_eq!(messages[0]["content"], "Part 1Part 2");
    }

    #[test]
    fn test_parse_openai_response_success() {
        let response = RestResponse::ok(serde_json::json!({
            "id": "chatcmpl-abc123",
            "object": "chat.completion",
            "model": "gpt-4o-2024-08-06",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "The function looks good."
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 25,
                "completion_tokens": 10,
                "total_tokens": 35
            }
        }));

        let chat = parse_openai_response(&response).unwrap();
        assert_eq!(chat.content, "The function looks good.");
        assert_eq!(chat.model, "gpt-4o-2024-08-06");
        assert_eq!(chat.finish_reason, FinishReason::Stop);
        assert_eq!(chat.usage.input_tokens, 25);
        assert_eq!(chat.usage.output_tokens, 10);
        assert_eq!(chat.usage.total(), 35);
        assert!(chat.thinking.is_none());
    }

    #[test]
    fn test_parse_openai_response_with_cache_and_reasoning_tokens() {
        let response = RestResponse::ok(serde_json::json!({
            "model": "o3",
            "choices": [{
                "message": { "role": "assistant", "content": "Result" },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 500,
                "completion_tokens": 1200,
                "total_tokens": 1700,
                "prompt_tokens_details": {
                    "cached_tokens": 400
                },
                "completion_tokens_details": {
                    "reasoning_tokens": 900
                }
            }
        }));

        let chat = parse_openai_response(&response).unwrap();
        assert_eq!(chat.usage.input_tokens, 500);
        assert_eq!(chat.usage.output_tokens, 1200);
        assert_eq!(chat.usage.cached_tokens, Some(400));
        assert_eq!(chat.usage.reasoning_tokens, Some(900));
        assert!(chat.usage.cache_creation_input_tokens.is_none());
    }

    #[test]
    fn test_parse_openai_response_error() {
        let response = RestResponse::new(
            401,
            serde_json::json!({
                "error": {
                    "message": "Incorrect API key provided",
                    "type": "invalid_request_error"
                }
            }),
        );

        let err = parse_openai_response(&response).unwrap_err();
        assert!(err.contains("Incorrect API key"));
        assert!(err.contains("401"));
    }

    #[test]
    fn test_parse_openai_response_length() {
        let response = RestResponse::ok(serde_json::json!({
            "model": "gpt-4o",
            "choices": [{
                "message": { "role": "assistant", "content": "partial..." },
                "finish_reason": "length"
            }],
            "usage": { "prompt_tokens": 10, "completion_tokens": 100, "total_tokens": 110 }
        }));

        let chat = parse_openai_response(&response).unwrap();
        assert_eq!(chat.finish_reason, FinishReason::Length);
    }

    #[test]
    fn test_openai_compatible_request() {
        let chat = sample_request();
        let req = build_openai_compatible_request(&chat, "http://localhost:11434");

        assert_eq!(req.url, "http://localhost:11434/v1/chat/completions");
        assert!(req.auth.is_none());
    }
}
