//! OpenAI Responses API conversions.
//!
//! Pure functions that convert between gunbc chat types and the
//! OpenAI Responses API format (`POST /v1/responses`).
//!
//! # API Reference
//!
//! OpenAI Responses API:
//! - Endpoint: POST /v1/responses
//! - Auth: Bearer token (Authorization header)
//! - Request body: { model, input, instructions?, reasoning?, max_output_tokens? }
//! - Response body: { output: [{type, content}], output_text, usage }
//!
//! Ref: <https://platform.openai.com/docs/api-reference/responses/create>
//!
//! # Why Responses API?
//!
//! For reasoning models (o1, o3, o4-mini), the Responses API is recommended over
//! Chat Completions because it:
//! - Supports **reasoning summaries** (concise/detailed)
//! - Persists reasoning tokens between tool calls for better performance
//! - Provides 40-80% better cache utilization
//! - Returns richer output (reasoning items, structured output)
//!
//! Ref: <https://platform.openai.com/docs/guides/reasoning>
//!
//! # Key Differences from Chat Completions
//!
//! | Feature | Chat Completions | Responses |
//! |---------|-----------------|-----------|
//! | Endpoint | `/v1/chat/completions` | `/v1/responses` |
//! | System prompt | `messages[0].role="system"` | `instructions` field |
//! | Token limit | `max_completion_tokens` | `max_output_tokens` |
//! | Reasoning | `reasoning_effort` only | `reasoning.effort` + `reasoning.summary` |
//! | Output | `choices[0].message.content` | `output_text` / `output[]` items |
//! | Caching | Automatic prefix | Automatic, 40-80% better than CC |

use super::chat::{
    ChatRequest, ChatResponse, FinishReason, MessageContent, ResponseBlock, ThinkingConfig, Usage,
};
use super::provider::openai_provider;
use crate::transport::http::HttpMethod;
use crate::transport::rest::{RestRequest, RestResponse};

/// Build an OpenAI Responses API request from a chat request.
///
/// Converts the unified `ChatRequest` to the Responses API format:
/// - System messages → `instructions` field
/// - Non-system messages → `input` items
/// - Thinking config → `reasoning` parameter with effort and summary
/// - max_tokens → `max_output_tokens`
/// - Content blocks flattened to strings (OpenAI caching is automatic)
pub fn build_openai_responses_request(chat: &ChatRequest) -> RestRequest {
    let provider = openai_provider();
    let responses_url = provider
        .responses_url()
        .expect("OpenAI provider must have responses endpoint");

    // System messages → instructions
    let system_msgs = chat.system_messages();
    let instructions: Option<String> = if system_msgs.is_empty() {
        None
    } else {
        Some(
            system_msgs
                .iter()
                .map(|m| content_to_string(&m.content))
                .collect::<Vec<_>>()
                .join("\n\n"),
        )
    };

    // Non-system messages → input items
    let input: Vec<serde_json::Value> = chat
        .non_system_messages()
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role.as_str(),
                "content": content_to_string(&m.content),
            })
        })
        .collect();

    let mut body = serde_json::json!({
        "model": chat.model,
        "input": input,
    });

    if let Some(inst) = instructions {
        body["instructions"] = serde_json::json!(inst);
    }
    if let Some(max) = chat.max_tokens {
        body["max_output_tokens"] = serde_json::json!(max);
    }
    if let Some(temp) = chat.temperature {
        body["temperature"] = serde_json::json!(temp);
    }

    // Reasoning configuration (the main reason to use Responses API)
    if let Some(ThinkingConfig::OpenAI { effort, summary }) = &chat.thinking {
        let mut reasoning = serde_json::json!({
            "effort": effort.as_str(),
        });
        if let Some(s) = summary {
            reasoning["summary"] = serde_json::json!(s.as_str());
        }
        body["reasoning"] = reasoning;
    }

    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    RestRequest {
        url: responses_url,
        method: HttpMethod::Post,
        headers,
        body: Some(body),
        auth: None,
        query: Default::default(),
        timeout_ms: Some(120_000),
    }
}

fn content_to_string(content: &MessageContent) -> String {
    content.text()
}

/// Parse an OpenAI Responses API response.
///
/// The Responses API returns output as an array of items. This parser extracts:
/// - Text content from `output_text` (convenience field) or output items
/// - Reasoning summary from `reasoning` output items
/// - Token usage including `cached_tokens` and `reasoning_tokens`
/// - Finish status from the response `status` field
pub fn parse_openai_responses_response(response: &RestResponse) -> Result<ChatResponse, String> {
    if !response.is_success() {
        let error_msg = response
            .body
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        return Err(format!(
            "OpenAI Responses API error (status {}): {}",
            response.status, error_msg
        ));
    }

    // Use output_text convenience field if available
    let content = response
        .body
        .get("output_text")
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| extract_text_from_output(&response.body));

    let model = response
        .body
        .get("model")
        .and_then(|m| m.as_str())
        .ok_or_else(|| "OpenAI Responses API missing model".to_string())?
        .to_string();

    // Status field: "completed", "failed", "incomplete"
    let status = response
        .body
        .get("status")
        .and_then(|s| s.as_str())
        .ok_or_else(|| "OpenAI Responses API missing status".to_string())?;
    let finish_reason = match status {
        "completed" => FinishReason::Stop,
        "incomplete" => FinishReason::Length,
        other => FinishReason::Other(other.to_string()),
    };

    let usage_value = response
        .body
        .get("usage")
        .ok_or_else(|| "OpenAI Responses API missing usage".to_string())?;
    let input_tokens = usage_value
        .get("input_tokens")
        .and_then(|t| t.as_u64())
        .ok_or_else(|| "OpenAI Responses API missing usage.input_tokens".to_string())?;
    let output_tokens = usage_value
        .get("output_tokens")
        .and_then(|t| t.as_u64())
        .ok_or_else(|| "OpenAI Responses API missing usage.output_tokens".to_string())?;

    let cached_tokens = usage_value
        .get("input_tokens_details")
        .and_then(|d| d.get("cached_tokens"))
        .and_then(|t| t.as_u64());

    let reasoning_tokens = usage_value
        .get("output_tokens_details")
        .and_then(|d| d.get("reasoning_tokens"))
        .and_then(|t| t.as_u64());

    let usage = Usage {
        input_tokens,
        output_tokens,
        cache_creation_input_tokens: None,
        cache_read_input_tokens: None,
        cached_tokens,
        reasoning_tokens,
    };

    // Extract reasoning summary from output items
    let (thinking, content_blocks) = extract_reasoning_from_output(&response.body);

    Ok(ChatResponse {
        content,
        model,
        finish_reason,
        usage,
        thinking,
        content_blocks,
    })
}

/// Extract text content from the output array when output_text is not available.
fn extract_text_from_output(body: &serde_json::Value) -> String {
    let mut parts = Vec::new();
    if let Some(output) = body.get("output").and_then(|o| o.as_array()) {
        for item in output {
            let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if item_type == "message" {
                if let Some(content) = item.get("content").and_then(|c| c.as_array()) {
                    for block in content {
                        if block.get("type").and_then(|t| t.as_str()) == Some("output_text") {
                            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                parts.push(text.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    parts.join("")
}

/// Extract reasoning summary and build response blocks from output items.
fn extract_reasoning_from_output(body: &serde_json::Value) -> (Option<String>, Vec<ResponseBlock>) {
    let mut thinking_parts = Vec::new();
    let mut blocks = Vec::new();

    if let Some(output) = body.get("output").and_then(|o| o.as_array()) {
        for item in output {
            let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match item_type {
                "reasoning" => {
                    // Reasoning summary items
                    if let Some(summary) = item.get("summary").and_then(|s| s.as_array()) {
                        for s in summary {
                            if let Some(text) = s.get("text").and_then(|t| t.as_str()) {
                                thinking_parts.push(text.to_string());
                                blocks.push(ResponseBlock::Thinking {
                                    thinking: text.to_string(),
                                    signature: None,
                                });
                            }
                        }
                    }
                }
                "message" => {
                    if let Some(content) = item.get("content").and_then(|c| c.as_array()) {
                        for block in content {
                            if block.get("type").and_then(|t| t.as_str()) == Some("output_text") {
                                if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                    blocks.push(ResponseBlock::Text {
                                        text: text.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let thinking = if thinking_parts.is_empty() {
        None
    } else {
        Some(thinking_parts.join("\n"))
    };

    // Only return blocks if there was reasoning (otherwise keep it flat)
    if thinking.is_none() {
        blocks.clear();
    }

    (thinking, blocks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::llm::chat::{ChatMessage, ReasoningEffort, ReasoningSummary};

    #[test]
    fn test_build_responses_request_simple() {
        let req = build_openai_responses_request(&ChatRequest::new(
            "gpt-4o",
            vec![
                ChatMessage::system("You are helpful."),
                ChatMessage::user("Hello"),
            ],
        ));

        assert_eq!(req.url, "https://api.openai.com/v1/responses");
        let body = req.body.unwrap();
        assert_eq!(body["model"], "gpt-4o");
        assert_eq!(body["instructions"], "You are helpful.");

        let input = body["input"].as_array().unwrap();
        assert_eq!(input.len(), 1);
        assert_eq!(input[0]["role"], "user");
    }

    #[test]
    fn test_build_responses_request_reasoning() {
        let req = build_openai_responses_request(
            &ChatRequest::new("o3", vec![ChatMessage::user("Think about X.")])
                .max_tokens(8000)
                .thinking(ThinkingConfig::openai_with_summary(
                    ReasoningEffort::High,
                    ReasoningSummary::Concise,
                )),
        );

        let body = req.body.unwrap();
        assert_eq!(body["reasoning"]["effort"], "high");
        assert_eq!(body["reasoning"]["summary"], "concise");
        assert_eq!(body["max_output_tokens"], 8000);
        // Should NOT have max_tokens
        assert!(body.get("max_tokens").is_none());
    }

    #[test]
    fn test_parse_responses_simple() {
        let response = RestResponse::ok(serde_json::json!({
            "id": "resp_abc123",
            "object": "response",
            "status": "completed",
            "model": "gpt-4o",
            "output": [{
                "type": "message",
                "role": "assistant",
                "content": [{
                    "type": "output_text",
                    "text": "Hello! How can I help?"
                }]
            }],
            "output_text": "Hello! How can I help?",
            "usage": {
                "input_tokens": 20,
                "output_tokens": 10,
                "total_tokens": 30
            }
        }));

        let chat = parse_openai_responses_response(&response).unwrap();
        assert_eq!(chat.content, "Hello! How can I help?");
        assert_eq!(chat.model, "gpt-4o");
        assert_eq!(chat.finish_reason, FinishReason::Stop);
        assert_eq!(chat.usage.input_tokens, 20);
        assert!(chat.thinking.is_none());
    }

    #[test]
    fn test_parse_responses_with_reasoning() {
        let response = RestResponse::ok(serde_json::json!({
            "id": "resp_abc123",
            "status": "completed",
            "model": "o3",
            "output": [
                {
                    "type": "reasoning",
                    "id": "rs_abc",
                    "summary": [{
                        "type": "summary_text",
                        "text": "I analyzed the problem by considering..."
                    }]
                },
                {
                    "type": "message",
                    "role": "assistant",
                    "content": [{
                        "type": "output_text",
                        "text": "The answer is 42."
                    }]
                }
            ],
            "output_text": "The answer is 42.",
            "usage": {
                "input_tokens": 50,
                "output_tokens": 500,
                "total_tokens": 550,
                "output_tokens_details": {
                    "reasoning_tokens": 400
                }
            }
        }));

        let chat = parse_openai_responses_response(&response).unwrap();
        assert_eq!(chat.content, "The answer is 42.");
        assert_eq!(
            chat.thinking,
            Some("I analyzed the problem by considering...".to_string())
        );
        assert_eq!(chat.usage.reasoning_tokens, Some(400));
        assert_eq!(chat.content_blocks.len(), 2);
        assert!(matches!(
            &chat.content_blocks[0],
            ResponseBlock::Thinking { .. }
        ));
        assert!(matches!(
            &chat.content_blocks[1],
            ResponseBlock::Text { .. }
        ));
    }

    #[test]
    fn test_parse_responses_with_cache() {
        let response = RestResponse::ok(serde_json::json!({
            "id": "resp_abc",
            "status": "completed",
            "model": "gpt-4o",
            "output": [{
                "type": "message",
                "role": "assistant",
                "content": [{"type": "output_text", "text": "cached response"}]
            }],
            "output_text": "cached response",
            "usage": {
                "input_tokens": 500,
                "output_tokens": 10,
                "total_tokens": 510,
                "input_tokens_details": {
                    "cached_tokens": 450
                }
            }
        }));

        let chat = parse_openai_responses_response(&response).unwrap();
        assert_eq!(chat.usage.cached_tokens, Some(450));
    }

    #[test]
    fn test_parse_responses_error() {
        let response = RestResponse::new(
            400,
            serde_json::json!({
                "error": {
                    "message": "Invalid model",
                    "type": "invalid_request_error"
                }
            }),
        );

        let err = parse_openai_responses_response(&response).unwrap_err();
        assert!(err.contains("Invalid model"));
        assert!(err.contains("400"));
    }

    #[test]
    fn test_parse_responses_incomplete() {
        let response = RestResponse::ok(serde_json::json!({
            "id": "resp_abc",
            "status": "incomplete",
            "model": "gpt-4o",
            "output": [{
                "type": "message",
                "role": "assistant",
                "content": [{"type": "output_text", "text": "partial..."}]
            }],
            "output_text": "partial...",
            "usage": { "input_tokens": 10, "output_tokens": 100, "total_tokens": 110 }
        }));

        let chat = parse_openai_responses_response(&response).unwrap();
        assert_eq!(chat.finish_reason, FinishReason::Length);
    }

    #[test]
    fn test_parse_responses_missing_status_is_error() {
        let response = RestResponse::ok(serde_json::json!({
            "id": "resp_abc",
            "model": "gpt-4o",
            "output_text": "hello",
            "usage": { "input_tokens": 1, "output_tokens": 2, "total_tokens": 3 }
        }));

        let err = parse_openai_responses_response(&response).unwrap_err();
        assert!(err.contains("missing status"));
    }

    #[test]
    fn test_parse_responses_missing_usage_is_error() {
        let response = RestResponse::ok(serde_json::json!({
            "id": "resp_abc",
            "status": "completed",
            "model": "gpt-4o",
            "output_text": "hello"
        }));

        let err = parse_openai_responses_response(&response).unwrap_err();
        assert!(err.contains("missing usage"));
    }
}
