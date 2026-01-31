//! OpenAI chat completion API conversions.
//!
//! Pure functions that convert between gunbc chat types and
//! OpenAI-specific REST request/response formats.
//!
//! # API Reference
//!
//! OpenAI chat completions:
//! - Endpoint: POST /v1/chat/completions
//! - Auth: Bearer token via Authorization header
//! - Request body: { model, messages: [{role, content}], temperature?, max_tokens?, stop? }
//! - Response body: { choices: [{message: {content}, finish_reason}], model, usage }

use super::chat::{ChatRequest, ChatResponse, FinishReason, Usage};
use super::provider::openai_provider;
use crate::transport::http::HttpMethod;
use crate::transport::rest::{AuthMethod, RestRequest, RestResponse};

/// Build an OpenAI-compatible REST request from a chat request.
///
/// This is a PURE function - no I/O. The resulting `RestRequest` should be
/// executed via `TransportOps::Execute`.
///
/// Auth uses `AuthMethod::EnvVar` so the actual API key is resolved at
/// execution time, not embedded in the request.
pub fn build_openai_request(chat: &ChatRequest) -> RestRequest {
    let provider = openai_provider();

    let mut body = serde_json::json!({
        "model": chat.model,
        "messages": chat.messages.iter().map(|m| {
            serde_json::json!({
                "role": m.role.as_str(),
                "content": m.content,
            })
        }).collect::<Vec<_>>(),
    });

    if let Some(temp) = chat.temperature {
        body["temperature"] = serde_json::json!(temp);
    }
    if let Some(max) = chat.max_tokens {
        body["max_tokens"] = serde_json::json!(max);
    }
    if !chat.stop.is_empty() {
        body["stop"] = serde_json::json!(chat.stop);
    }

    RestRequest {
        url: provider.chat_url(),
        method: HttpMethod::Post,
        headers: [("Content-Type".to_string(), "application/json".to_string())]
            .into_iter()
            .collect(),
        body: Some(body),
        auth: Some(AuthMethod::EnvVar(provider.api_key_env.0)),
        query: Default::default(),
        timeout_ms: Some(120_000),
    }
}

/// Parse an OpenAI chat completion response.
///
/// Extracts the assistant's message content, finish reason, and usage
/// from the OpenAI response JSON format.
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
        .unwrap_or("")
        .to_string();

    let finish_reason = first
        .get("finish_reason")
        .and_then(|r| r.as_str())
        .map(FinishReason::from_openai)
        .unwrap_or(FinishReason::Stop);

    let model = response
        .body
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown")
        .to_string();

    let usage = response
        .body
        .get("usage")
        .map(|u| Usage {
            input_tokens: u
                .get("prompt_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0),
            output_tokens: u
                .get("completion_tokens")
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

/// Build a request for an OpenAI-compatible provider with a custom base URL.
///
/// This enables use of OpenAI-compatible APIs (e.g., local models via Ollama,
/// Azure OpenAI, etc.) by overriding the endpoint URL while keeping the
/// same request format.
pub fn build_openai_compatible_request(
    chat: &ChatRequest,
    base_url: &str,
    api_key_env: &str,
) -> RestRequest {
    let mut req = build_openai_request(chat);
    req.url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));
    req.auth = Some(AuthMethod::EnvVar(api_key_env.to_string()));
    req
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::chat::ChatMessage;

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
        assert!(matches!(req.auth, Some(AuthMethod::EnvVar(ref v)) if v == "OPENAI_API_KEY"));

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
    fn test_parse_openai_response_length_finish() {
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
        let req = build_openai_compatible_request(&chat, "http://localhost:11434", "OLLAMA_KEY");

        assert_eq!(
            req.url,
            "http://localhost:11434/v1/chat/completions"
        );
        assert!(matches!(req.auth, Some(AuthMethod::EnvVar(ref v)) if v == "OLLAMA_KEY"));
    }
}
