//! LLM operations for gunbc DAGs.
//!
//! Pure operations for preparing LLM chat requests and parsing responses.
//! All operations are PURE (no I/O). I/O happens through `TransportOps::Execute` nodes.
//!
//! # Transport Pattern
//!
//! ```text
//! PrepareChatRequest (pure) → TransportOps::Execute (I/O) → ParseChatResponse (pure)
//! ```
//!
//! # Example
//!
//! ```ignore
//! use gunbc_lib_llm_ops::LlmOps;
//!
//! // In a DAG:
//! // 1. PrepareChatRequest builds a RestRequest from provider_id + model + messages
//! // 2. TransportOps::Execute sends the request (I/O boundary)
//! // 3. ParseChatResponse extracts content from the provider-specific response
//! ```

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::llm::{self, ChatMessage, ChatRequest, Role};
use gunbc_ir::transport::{TransportRequest, TransportResponse};
use gunbc_ir::Value;
use std::collections::HashMap;

/// LLM operations for use in DAG nodes.
///
/// All operations are PURE - no I/O. Use `TransportOps::Execute` for actual I/O.
#[derive(Debug, Clone)]
pub enum LlmOps {
    /// Build a chat completion REST request from inputs.
    ///
    /// Inputs:
    /// - `provider`: String - provider ID (e.g., "openai", "anthropic")
    /// - `model`: String - model identifier (e.g., "gpt-4o", "claude-sonnet-4-20250514")
    /// - `messages`: JSON array of {role, content} objects
    /// - `temperature` (optional): f64
    /// - `max_tokens` (optional): i64
    /// - `system_prompt` (optional): String - prepended as a system message
    ///
    /// Outputs:
    /// - `request`: TransportRequest (Rest)
    /// - `provider`: String - echoed for use in ParseChatResponse
    PrepareChatRequest,

    /// Parse a chat completion response.
    ///
    /// Inputs:
    /// - `provider`: String - provider ID (must match the one used in PrepareChatRequest)
    /// - `response`: TransportResponse (Rest)
    ///
    /// Outputs:
    /// - `content`: String - the generated text
    /// - `model`: String - model that generated the response
    /// - `finish_reason`: String - why generation stopped
    /// - `input_tokens`: i64 - tokens in the prompt
    /// - `output_tokens`: i64 - tokens in the completion
    ParseChatResponse,
}

impl Executable for LlmOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            LlmOps::PrepareChatRequest => execute_prepare_chat_request(inputs),
            LlmOps::ParseChatResponse => execute_parse_chat_response(inputs),
        }
    }
}

/// Build a `ChatRequest` from DAG inputs and convert it to a `RestRequest`.
fn execute_prepare_chat_request(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let provider_id = inputs
        .get("provider")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExecError::new("missing or invalid 'provider' input"))?
        .to_string();

    let model = inputs
        .get("model")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExecError::new("missing or invalid 'model' input"))?;

    // Build messages list
    let mut messages = Vec::new();

    // Optional system prompt (convenience: added as first system message)
    if let Some(system_prompt) = inputs.get("system_prompt").and_then(|v| v.as_str()) {
        if !system_prompt.is_empty() {
            messages.push(ChatMessage::system(system_prompt));
        }
    }

    // Parse messages from JSON array or string list
    if let Some(Value::Json(json_messages)) = inputs.get("messages") {
        let parsed = parse_messages_from_json(json_messages)?;
        messages.extend(parsed);
    } else if let Some(Value::Str(user_message)) = inputs.get("messages") {
        // Single string treated as a user message
        messages.push(ChatMessage::user(user_message));
    } else if messages.is_empty() {
        return Err(ExecError::new(
            "missing 'messages' input: provide JSON array of {role, content} or a string",
        ));
    }

    // Build the chat request
    let mut chat = ChatRequest::new(model, messages);

    if let Some(Value::Json(serde_json::Value::Number(n))) = inputs.get("temperature") {
        if let Some(t) = n.as_f64() {
            chat = chat.temperature(t);
        }
    }

    if let Some(Value::Int(n)) = inputs.get("max_tokens") {
        chat = chat.max_tokens(*n as u64);
    }

    // Convert to REST request via provider-specific builder
    let rest_request = llm::build_chat_request(&provider_id, &chat)
        .map_err(|e| ExecError::new(e))?;

    let mut out = HashMap::new();
    out.insert(
        "request".to_string(),
        Value::Request(TransportRequest::Rest(rest_request)),
    );
    out.insert("provider".to_string(), Value::Str(provider_id));
    Ok(out)
}

/// Parse a provider-specific REST response into structured chat output.
fn execute_parse_chat_response(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let provider_id = inputs
        .get("provider")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExecError::new("missing or invalid 'provider' input"))?;

    let response = inputs
        .get("response")
        .and_then(|v| v.as_response())
        .ok_or_else(|| ExecError::new("missing or invalid 'response' input"))?;

    let rest_response = match response {
        TransportResponse::Rest(r) => r,
        _ => {
            return Err(ExecError::new(
                "expected REST response from LLM API, got different transport type",
            ))
        }
    };

    let chat_response = llm::parse_chat_response(provider_id, rest_response)
        .map_err(|e| ExecError::new(e))?;

    let mut out = HashMap::new();
    out.insert("content".to_string(), Value::Str(chat_response.content));
    out.insert("model".to_string(), Value::Str(chat_response.model));
    out.insert(
        "finish_reason".to_string(),
        Value::Str(format!("{:?}", chat_response.finish_reason)),
    );
    out.insert(
        "input_tokens".to_string(),
        Value::Int(chat_response.usage.input_tokens as i64),
    );
    out.insert(
        "output_tokens".to_string(),
        Value::Int(chat_response.usage.output_tokens as i64),
    );
    Ok(out)
}

/// Parse chat messages from a JSON value.
///
/// Accepts either:
/// - A JSON array of `{role: string, content: string}` objects
/// - A single string (interpreted as a user message)
fn parse_messages_from_json(
    json: &serde_json::Value,
) -> Result<Vec<ChatMessage>, ExecError> {
    match json {
        serde_json::Value::Array(arr) => {
            let mut messages = Vec::with_capacity(arr.len());
            for (i, item) in arr.iter().enumerate() {
                let role_str = item
                    .get("role")
                    .and_then(|r| r.as_str())
                    .ok_or_else(|| {
                        ExecError::new(format!("message[{}]: missing or invalid 'role'", i))
                    })?;

                let role = Role::parse(role_str).ok_or_else(|| {
                    ExecError::new(format!(
                        "message[{}]: invalid role '{}' (expected system, user, or assistant)",
                        i, role_str
                    ))
                })?;

                let content = item
                    .get("content")
                    .and_then(|c| c.as_str())
                    .ok_or_else(|| {
                        ExecError::new(format!("message[{}]: missing or invalid 'content'", i))
                    })?;

                messages.push(ChatMessage {
                    role,
                    content: content.to_string(),
                });
            }
            Ok(messages)
        }
        serde_json::Value::String(s) => Ok(vec![ChatMessage::user(s)]),
        _ => Err(ExecError::new(
            "messages must be a JSON array of {role, content} objects or a string",
        )),
    }
}

// ============================================================================
// Convenience functions for building common LLM requests
// ============================================================================

/// Build a code review chat request.
///
/// Creates a `ChatRequest` configured for code review with appropriate
/// system prompting and parameters.
pub fn code_review_request(
    provider_id: &str,
    model: &str,
    code: &str,
    context: &str,
) -> Result<ChatRequest, String> {
    llm::provider_by_id(provider_id)
        .ok_or_else(|| format!("unknown provider: {}", provider_id))?;

    let system = "You are an expert code reviewer. Analyze the code for:\n\
                  - Correctness and potential bugs\n\
                  - Security vulnerabilities\n\
                  - Performance issues\n\
                  - Code style and readability\n\
                  Provide specific, actionable feedback.";

    let user_message = if context.is_empty() {
        format!("Review this code:\n\n```\n{}\n```", code)
    } else {
        format!(
            "Context: {}\n\nReview this code:\n\n```\n{}\n```",
            context, code
        )
    };

    Ok(ChatRequest::new(
        model,
        vec![
            ChatMessage::system(system),
            ChatMessage::user(user_message),
        ],
    )
    .temperature(0.3)
    .max_tokens(4096))
}

/// Build a code generation chat request.
///
/// Creates a `ChatRequest` configured for code generation.
pub fn code_generation_request(
    provider_id: &str,
    model: &str,
    description: &str,
    language: &str,
) -> Result<ChatRequest, String> {
    llm::provider_by_id(provider_id)
        .ok_or_else(|| format!("unknown provider: {}", provider_id))?;

    let system = format!(
        "You are an expert {} programmer. Generate clean, idiomatic code \
         that follows best practices. Include appropriate error handling. \
         Return only the code without explanation unless asked.",
        language
    );

    Ok(ChatRequest::new(
        model,
        vec![
            ChatMessage::system(system),
            ChatMessage::user(description),
        ],
    )
    .temperature(0.2)
    .max_tokens(4096))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_chat_request_openai() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("openai".to_string()));
        inputs.insert("model".to_string(), Value::Str("gpt-4o".to_string()));
        inputs.insert(
            "messages".to_string(),
            Value::Json(serde_json::json!([
                {"role": "user", "content": "Hello!"}
            ])),
        );

        let op = LlmOps::PrepareChatRequest;
        let result = op.execute(inputs).unwrap();

        assert!(result.contains_key("request"));
        assert_eq!(
            result.get("provider"),
            Some(&Value::Str("openai".to_string()))
        );

        match result.get("request") {
            Some(Value::Request(TransportRequest::Rest(req))) => {
                assert_eq!(req.url, "https://api.openai.com/v1/chat/completions");
            }
            _ => panic!("expected REST request"),
        }
    }

    #[test]
    fn test_prepare_chat_request_anthropic() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "provider".to_string(),
            Value::Str("anthropic".to_string()),
        );
        inputs.insert(
            "model".to_string(),
            Value::Str("claude-sonnet-4-20250514".to_string()),
        );
        inputs.insert(
            "system_prompt".to_string(),
            Value::Str("Be helpful.".to_string()),
        );
        inputs.insert(
            "messages".to_string(),
            Value::Json(serde_json::json!([
                {"role": "user", "content": "Hello!"}
            ])),
        );

        let result = LlmOps::PrepareChatRequest.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Rest(req))) => {
                assert_eq!(req.url, "https://api.anthropic.com/v1/messages");
                let body = req.body.as_ref().unwrap();
                assert_eq!(body["system"], "Be helpful.");
            }
            _ => panic!("expected REST request"),
        }
    }

    #[test]
    fn test_prepare_chat_request_with_string_message() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("openai".to_string()));
        inputs.insert(
            "model".to_string(),
            Value::Str("gpt-4o-mini".to_string()),
        );
        inputs.insert(
            "messages".to_string(),
            Value::Str("What is 2+2?".to_string()),
        );

        let result = LlmOps::PrepareChatRequest.execute(inputs).unwrap();
        assert!(result.contains_key("request"));
    }

    #[test]
    fn test_prepare_chat_request_missing_provider() {
        let mut inputs = HashMap::new();
        inputs.insert("model".to_string(), Value::Str("gpt-4o".to_string()));

        let err = LlmOps::PrepareChatRequest.execute(inputs).unwrap_err();
        assert!(err.0.contains("provider"));
    }

    #[test]
    fn test_prepare_chat_request_unknown_provider() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("unknown".to_string()));
        inputs.insert("model".to_string(), Value::Str("test".to_string()));
        inputs.insert(
            "messages".to_string(),
            Value::Json(serde_json::json!([{"role": "user", "content": "hi"}])),
        );

        let err = LlmOps::PrepareChatRequest.execute(inputs).unwrap_err();
        assert!(err.0.contains("unknown"));
    }

    #[test]
    fn test_parse_chat_response_openai() {
        let response = TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(serde_json::json!({
                "model": "gpt-4o",
                "choices": [{
                    "message": {"role": "assistant", "content": "Hello!"},
                    "finish_reason": "stop"
                }],
                "usage": {"prompt_tokens": 5, "completion_tokens": 2, "total_tokens": 7}
            })),
        );

        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("openai".to_string()));
        inputs.insert("response".to_string(), Value::Response(response));

        let result = LlmOps::ParseChatResponse.execute(inputs).unwrap();

        assert_eq!(
            result.get("content"),
            Some(&Value::Str("Hello!".to_string()))
        );
        assert_eq!(
            result.get("model"),
            Some(&Value::Str("gpt-4o".to_string()))
        );
        assert_eq!(result.get("input_tokens"), Some(&Value::Int(5)));
        assert_eq!(result.get("output_tokens"), Some(&Value::Int(2)));
    }

    #[test]
    fn test_parse_chat_response_anthropic() {
        let response = TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(serde_json::json!({
                "content": [{"type": "text", "text": "Hello!"}],
                "model": "claude-sonnet-4-20250514",
                "stop_reason": "end_turn",
                "usage": {"input_tokens": 5, "output_tokens": 2}
            })),
        );

        let mut inputs = HashMap::new();
        inputs.insert(
            "provider".to_string(),
            Value::Str("anthropic".to_string()),
        );
        inputs.insert("response".to_string(), Value::Response(response));

        let result = LlmOps::ParseChatResponse.execute(inputs).unwrap();
        assert_eq!(
            result.get("content"),
            Some(&Value::Str("Hello!".to_string()))
        );
    }

    #[test]
    fn test_parse_messages_from_json_array() {
        let json = serde_json::json!([
            {"role": "system", "content": "Be helpful"},
            {"role": "user", "content": "Hello"}
        ]);

        let messages = parse_messages_from_json(&json).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, Role::System);
        assert_eq!(messages[1].role, Role::User);
    }

    #[test]
    fn test_parse_messages_from_json_string() {
        let json = serde_json::json!("Hello!");

        let messages = parse_messages_from_json(&json).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, Role::User);
        assert_eq!(messages[0].content, "Hello!");
    }

    #[test]
    fn test_parse_messages_invalid_role() {
        let json = serde_json::json!([{"role": "villain", "content": "mwahaha"}]);

        let err = parse_messages_from_json(&json).unwrap_err();
        assert!(err.0.contains("invalid role"));
    }

    #[test]
    fn test_code_review_request() {
        let req = code_review_request("openai", "gpt-4o", "fn add(a: i32, b: i32) -> i32 { a + b }", "")
            .unwrap();

        assert_eq!(req.model, "gpt-4o");
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.messages[0].role, Role::System);
        assert!(req.messages[1].content.contains("fn add"));
        assert_eq!(req.temperature, Some(0.3));
    }

    #[test]
    fn test_code_review_request_with_context() {
        let req =
            code_review_request("anthropic", "claude-sonnet-4-20250514", "x = 1", "Python code").unwrap();

        assert!(req.messages[1].content.contains("Python code"));
    }

    #[test]
    fn test_code_generation_request() {
        let req =
            code_generation_request("openai", "gpt-4o", "A function to sort a list", "Rust")
                .unwrap();

        assert_eq!(req.messages.len(), 2);
        assert!(req.messages[0].content.contains("Rust"));
        assert_eq!(req.temperature, Some(0.2));
    }

    #[test]
    fn test_code_review_request_unknown_provider() {
        let err = code_review_request("unknown", "test", "code", "").unwrap_err();
        assert!(err.contains("unknown provider"));
    }
}
