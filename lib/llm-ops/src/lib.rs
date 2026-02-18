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

#![deny(dead_code)]
pub mod graph;

pub mod graph_mock;

use gunbc_exec::{
    optional_int_strict, optional_json_strict, optional_str_strict, propagate_skipped,
    require_response, require_str, ExecError, Executable, IntoExecResult, OutputMap,
    TransportResponseExt,
};
use gunbc_ir::transport::llm::{self, ChatMessage, ChatRequest, MessageContent, Role};
use gunbc_ir::transport::{ScopeContract, TransportRequest};
use gunbc_ir::Value;
use gunbc_lib_cloud_ops::{bind_credential_intent_policy, policy_allows_impersonation};
use std::collections::HashMap;

/// LLM operations for use in DAG nodes.
///
/// All operations are PURE - no I/O. Use `TransportOps::Execute` for actual I/O.
#[derive(Debug, Clone)]
pub enum LlmOps {
    /// Resolve provider auth requirements (PURE).
    ///
    /// Inputs:
    /// - `provider`: String - provider ID (e.g., "openai", "anthropic")
    ///
    /// Outputs:
    /// - `service`: String - canonical provider/service ID
    /// - `secret_name`: OptionalString - policy-bound secret override
    /// - `scheme`: String - auth scheme ("bearer" or "header")
    /// - `header_name`: String - header name for "header" scheme (e.g., "x-api-key"), empty for "bearer"
    /// - `required_scopes`: List<String> - required capability scopes for this request class
    /// - `interactive_allowed`: Bool - whether interactive recovery is allowed
    /// - `allow_impersonation`: Bool - policy gate for SA impersonation branch
    ResolveAuth,
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

    // ========================================================================
    // Simple request/response (string in → string out)
    // ========================================================================
    /// Prepare a simple LLM request: content + question → request.
    ///
    /// This is a convenience wrapper that builds a single-turn chat request.
    ///
    /// Inputs:
    /// - `content`: String - the content/context to analyze
    /// - `question`: String - what to ask about the content
    /// - `provider`: String - provider ID (e.g., "openai", "anthropic")
    /// - `model`: String - model identifier
    /// - `system_prompt` (optional): String - system instructions
    ///
    /// Outputs:
    /// - `request`: TransportRequest (Rest)
    /// - `provider`: String - echoed for use in ParseSimpleResponse
    PrepareSimpleRequest,

    /// Parse a simple LLM response: response → answer string.
    ///
    /// Inputs:
    /// - `provider`: String - provider ID
    /// - `response`: TransportResponse (Rest)
    ///
    /// Outputs:
    /// - `answer`: String - the generated text
    ParseSimpleResponse,
}

impl Executable for LlmOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            LlmOps::ResolveAuth => execute_resolve_auth(inputs),
            LlmOps::PrepareChatRequest => execute_prepare_chat_request(inputs),
            LlmOps::ParseChatResponse => execute_parse_chat_response(inputs),
            LlmOps::PrepareSimpleRequest => execute_prepare_simple_request(inputs),
            LlmOps::ParseSimpleResponse => execute_parse_simple_response(inputs),
        }
    }
}

fn execute_resolve_auth(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    use gunbc_ir::AuthScheme;

    let provider_id = require_str(&inputs, "provider")?;
    let provider = llm::provider_by_id(provider_id)
        .ok_or_else(|| ExecError::new(format!("unknown provider '{}'", provider_id)))?;

    let (scheme, header_name) = match &provider.auth_scheme {
        AuthScheme::Bearer => ("bearer".to_string(), String::new()),
        AuthScheme::Header { name } => ("header".to_string(), name.clone()),
        AuthScheme::Basic { .. } => {
            return Err(ExecError::new(
                "basic auth is not supported for LLM providers",
            ))
        }
    };
    let provider_id = provider.id.clone();
    let fallback_intent = llm::LlmScopeContract::new(provider_id.clone()).credential_intent();
    let intent_key = format!("llm.{}.chat_completion", provider_id);
    let bound = bind_credential_intent_policy(&intent_key, &fallback_intent)
        .or_else(|_| bind_credential_intent_policy("llm.chat_completion", &fallback_intent))
        .map_err(|e| ExecError::new(format!("credential policy binding failed: {e}")))?;
    let allow_impersonation = policy_allows_impersonation(bound.impersonation.as_ref());
    let intent = bound.intent;
    intent.validate().map_err(|e| {
        ExecError::new(format!(
            "invalid llm credential contract for '{}': {e}",
            provider_id
        ))
    })?;
    let mut out = OutputMap::new()
        .str("service", provider_id)
        .str("scheme", scheme)
        .str("header_name", header_name)
        .str_list("required_scopes", intent.required_scopes)
        .bool("interactive_allowed", intent.interactive_allowed)
        .bool("allow_impersonation", allow_impersonation);
    if let Some(secret_name) = intent.secret_name {
        out = out.str("secret_name", secret_name);
    }
    out.ok()
}

/// Build a `ChatRequest` from DAG inputs and convert it to a `RestRequest`.
fn execute_prepare_chat_request(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let provider_id = require_str(&inputs, "provider")?.to_string();
    let model = require_str(&inputs, "model")?;

    // Build messages list
    let mut messages = Vec::new();

    // Optional system prompt (convenience: added as first system message)
    if let Some(system_prompt) = optional_str_strict(&inputs, "system_prompt")? {
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

    if let Some(json) = optional_json_strict(&inputs, "temperature")? {
        if let Some(t) = json.as_f64() {
            chat = chat.temperature(t);
        }
    }

    if let Some(n) = optional_int_strict(&inputs, "max_tokens")? {
        chat = chat.max_tokens(n as u64);
    }

    // Convert to REST request via provider-specific builder
    let rest_request =
        llm::build_chat_request(&provider_id, &chat).exec_context("build chat request")?;

    OutputMap::new()
        .request("request", TransportRequest::Rest(rest_request))
        .str("provider", provider_id)
        .bool("skip", false)
        .ok()
}

/// Parse a provider-specific REST response into structured chat output.
fn execute_parse_chat_response(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(
        &inputs,
        "response",
        &[
            "content",
            "model",
            "finish_reason",
            "input_tokens",
            "output_tokens",
        ],
    ) {
        return result;
    }

    let provider_id = require_str(&inputs, "provider")?;
    let response = require_response(&inputs, "response")?;
    let rest_response = response.require_rest()?;

    let chat_response =
        llm::parse_chat_response(provider_id, rest_response).exec_context("parse chat response")?;

    OutputMap::new()
        .str("content", chat_response.content)
        .str("model", chat_response.model)
        .str(
            "finish_reason",
            format!("{:?}", chat_response.finish_reason),
        )
        .int("input_tokens", chat_response.usage.input_tokens as i64)
        .int("output_tokens", chat_response.usage.output_tokens as i64)
        .ok()
}

/// Build a simple request from content + question.
///
/// String in → String out convenience wrapper.
fn execute_prepare_simple_request(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let provider_id = require_str(&inputs, "provider")?.to_string();
    let model = require_str(&inputs, "model")?;
    let content = require_str(&inputs, "content")?;
    let question = require_str(&inputs, "question")?;

    // Build the user message combining content and question
    let user_message = format!(
        "Given the following content:\n\n{}\n\n{}",
        content, question
    );

    let mut messages = Vec::new();

    // Optional system prompt
    if let Some(system_prompt) = optional_str_strict(&inputs, "system_prompt")? {
        if !system_prompt.is_empty() {
            messages.push(ChatMessage::system(system_prompt));
        }
    }

    messages.push(ChatMessage::user(user_message));

    // Build the chat request
    let chat = ChatRequest::new(model, messages);

    // Convert to REST request
    let rest_request =
        llm::build_chat_request(&provider_id, &chat).exec_context("build chat request")?;

    OutputMap::new()
        .request("request", TransportRequest::Rest(rest_request))
        .str("provider", provider_id)
        .bool("skip", false)
        .ok()
}

/// Parse a simple response: just extract the answer string.
fn execute_parse_simple_response(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(&inputs, "response", &["answer"]) {
        return result;
    }

    let provider_id = require_str(&inputs, "provider")?;
    let response = require_response(&inputs, "response")?;
    let rest_response = response.require_rest()?;

    let chat_response =
        llm::parse_chat_response(provider_id, rest_response).exec_context("parse chat response")?;

    OutputMap::new().str("answer", chat_response.content).ok()
}

/// Parse chat messages from a JSON value.
///
/// Accepts either:
/// - A JSON array of `{role: string, content: string}` objects
/// - A single string (interpreted as a user message)
fn parse_messages_from_json(json: &serde_json::Value) -> Result<Vec<ChatMessage>, ExecError> {
    match json {
        serde_json::Value::Array(arr) => {
            let mut messages = Vec::with_capacity(arr.len());
            for (i, item) in arr.iter().enumerate() {
                let role_str = item.get("role").and_then(|r| r.as_str()).ok_or_else(|| {
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
                    content: MessageContent::Text(content.to_string()),
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
    llm::provider_by_id(provider_id).ok_or_else(|| format!("unknown provider: {}", provider_id))?;

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
        vec![ChatMessage::system(system), ChatMessage::user(user_message)],
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
    llm::provider_by_id(provider_id).ok_or_else(|| format!("unknown provider: {}", provider_id))?;

    let system = format!(
        "You are an expert {} programmer. Generate clean, idiomatic code \
         that follows best practices. Include appropriate error handling. \
         Return only the code without explanation unless asked.",
        language
    );

    Ok(ChatRequest::new(
        model,
        vec![ChatMessage::system(system), ChatMessage::user(description)],
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
    use gunbc_ir::transport::TransportResponse;
    use gunbc_lib_cloud_ops::{ENV_CREDENTIAL_POLICY_JSON, ENV_CREDENTIAL_POLICY_PROFILE};
    use std::sync::{Mutex, OnceLock};

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
        inputs.insert("provider".to_string(), Value::Str("anthropic".to_string()));
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
        inputs.insert("model".to_string(), Value::Str("gpt-4o-mini".to_string()));
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
        let response =
            TransportResponse::Rest(gunbc_ir::transport::RestResponse::ok(serde_json::json!({
                "model": "gpt-4o",
                "choices": [{
                    "message": {"role": "assistant", "content": "Hello!"},
                    "finish_reason": "stop"
                }],
                "usage": {"prompt_tokens": 5, "completion_tokens": 2, "total_tokens": 7}
            })));

        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("openai".to_string()));
        inputs.insert("response".to_string(), Value::Response(response));

        let result = LlmOps::ParseChatResponse.execute(inputs).unwrap();

        assert_eq!(
            result.get("content"),
            Some(&Value::Str("Hello!".to_string()))
        );
        assert_eq!(result.get("model"), Some(&Value::Str("gpt-4o".to_string())));
        assert_eq!(result.get("input_tokens"), Some(&Value::Int(5)));
        assert_eq!(result.get("output_tokens"), Some(&Value::Int(2)));
    }

    #[test]
    fn test_parse_chat_response_anthropic() {
        let response =
            TransportResponse::Rest(gunbc_ir::transport::RestResponse::ok(serde_json::json!({
                "content": [{"type": "text", "text": "Hello!"}],
                "model": "claude-sonnet-4-20250514",
                "stop_reason": "end_turn",
                "usage": {"input_tokens": 5, "output_tokens": 2}
            })));

        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("anthropic".to_string()));
        inputs.insert("response".to_string(), Value::Response(response));

        let result = LlmOps::ParseChatResponse.execute(inputs).unwrap();
        assert_eq!(
            result.get("content"),
            Some(&Value::Str("Hello!".to_string()))
        );
    }

    #[test]
    fn test_resolve_auth_openai() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("openai".to_string()));

        let result = LlmOps::ResolveAuth.execute(inputs).unwrap();
        assert_eq!(
            result.get("service"),
            Some(&Value::Str("openai".to_string()))
        );
        assert_eq!(
            result.get("scheme"),
            Some(&Value::Str("bearer".to_string()))
        );
        assert_eq!(result.get("header_name"), Some(&Value::Str(String::new())));
        assert_eq!(
            result.get("required_scopes"),
            Some(&Value::str_list(vec!["llm:chat_completion".to_string()]))
        );
        assert_eq!(result.get("interactive_allowed"), Some(&Value::Bool(true)));
        assert_eq!(result.get("allow_impersonation"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_resolve_auth_anthropic_scheme() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("anthropic".to_string()));

        let result = LlmOps::ResolveAuth.execute(inputs).unwrap();
        assert_eq!(
            result.get("scheme"),
            Some(&Value::Str("header".to_string()))
        );
        assert_eq!(
            result.get("header_name"),
            Some(&Value::Str("x-api-key".to_string()))
        );
        assert_eq!(
            result.get("required_scopes"),
            Some(&Value::str_list(vec!["llm:chat_completion".to_string()]))
        );
    }

    #[test]
    fn test_resolve_auth_applies_policy_secret_binding() {
        with_env_lock(|| {
            std::env::set_var(
                ENV_CREDENTIAL_POLICY_JSON,
                serde_json::json!({
                    "version": 0,
                    "profiles": [{
                        "name": "prod",
                        "defaults": {
                            "provider": "Gcp",
                            "runtime": "GitHubActions"
                        },
                        "intents": [{
                            "intent": "llm.openai.chat_completion",
                            "secret": { "name": "prod-openai-token" },
                            "required_scopes": ["llm:chat_completion"]
                        }]
                    }]
                })
                .to_string(),
            );
            std::env::set_var(ENV_CREDENTIAL_POLICY_PROFILE, "prod");

            let mut inputs = HashMap::new();
            inputs.insert("provider".to_string(), Value::Str("openai".to_string()));
            let result = LlmOps::ResolveAuth.execute(inputs).expect("resolve auth");

            assert_eq!(
                result.get("secret_name"),
                Some(&Value::Str("prod-openai-token".to_string()))
            );
            assert_eq!(
                result.get("required_scopes"),
                Some(&Value::str_list(vec!["llm:chat_completion".to_string()]))
            );
            assert_eq!(result.get("allow_impersonation"), Some(&Value::Bool(true)));
        });
    }

    #[test]
    fn test_resolve_auth_policy_never_disables_impersonation() {
        with_env_lock(|| {
            std::env::set_var(
                ENV_CREDENTIAL_POLICY_JSON,
                serde_json::json!({
                    "version": 0,
                    "profiles": [{
                        "name": "prod",
                        "defaults": {
                            "provider": "Gcp",
                            "runtime": "GitHubActions"
                        },
                        "intents": [{
                            "intent": "llm.openai.chat_completion",
                            "secret": { "name": "prod-openai-token" },
                            "required_scopes": ["llm:chat_completion"],
                            "impersonation": { "mode": "never" }
                        }]
                    }]
                })
                .to_string(),
            );
            std::env::set_var(ENV_CREDENTIAL_POLICY_PROFILE, "prod");

            let mut inputs = HashMap::new();
            inputs.insert("provider".to_string(), Value::Str("openai".to_string()));
            let result = LlmOps::ResolveAuth.execute(inputs).expect("resolve auth");
            assert_eq!(result.get("allow_impersonation"), Some(&Value::Bool(false)));
        });
    }

    #[test]
    fn test_resolve_auth_unknown_provider_errors() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("unknown".to_string()));

        let err = LlmOps::ResolveAuth.execute(inputs).unwrap_err();
        assert!(err.0.contains("unknown provider"));
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
        assert_eq!(messages[0].text(), "Hello!");
    }

    #[test]
    fn test_parse_messages_invalid_role() {
        let json = serde_json::json!([{"role": "villain", "content": "mwahaha"}]);

        let err = parse_messages_from_json(&json).unwrap_err();
        assert!(err.0.contains("invalid role"));
    }

    fn with_env_lock<F>(f: F)
    where
        F: FnOnce() + std::panic::UnwindSafe,
    {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        clear_policy_env();
        let result = std::panic::catch_unwind(f);
        clear_policy_env();
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }

    fn clear_policy_env() {
        std::env::remove_var(ENV_CREDENTIAL_POLICY_JSON);
        std::env::remove_var(ENV_CREDENTIAL_POLICY_PROFILE);
    }

    #[test]
    fn test_code_review_request() {
        let req = code_review_request(
            "openai",
            "gpt-4o",
            "fn add(a: i32, b: i32) -> i32 { a + b }",
            "",
        )
        .unwrap();

        assert_eq!(req.model, "gpt-4o");
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.messages[0].role, Role::System);
        assert!(req.messages[1].text().contains("fn add"));
        assert_eq!(req.temperature, Some(0.3));
    }

    #[test]
    fn test_code_review_request_with_context() {
        let req = code_review_request(
            "anthropic",
            "claude-sonnet-4-20250514",
            "x = 1",
            "Python code",
        )
        .unwrap();

        assert!(req.messages[1].text().contains("Python code"));
    }

    #[test]
    fn test_code_generation_request() {
        let req = code_generation_request("openai", "gpt-4o", "A function to sort a list", "Rust")
            .unwrap();

        assert_eq!(req.messages.len(), 2);
        assert!(req.messages[0].text().contains("Rust"));
        assert_eq!(req.temperature, Some(0.2));
    }

    #[test]
    fn test_code_review_request_unknown_provider() {
        let err = code_review_request("unknown", "test", "code", "").unwrap_err();
        assert!(err.contains("unknown provider"));
    }

    // ========================================================================
    // Simple request/response tests
    // ========================================================================

    #[test]
    fn test_prepare_simple_request() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("openai".to_string()));
        inputs.insert("model".to_string(), Value::Str("gpt-4o".to_string()));
        inputs.insert(
            "content".to_string(),
            Value::Str("fn add(a: i32, b: i32) -> i32 { a + b }".to_string()),
        );
        inputs.insert(
            "question".to_string(),
            Value::Str("What does this function do?".to_string()),
        );

        let result = LlmOps::PrepareSimpleRequest.execute(inputs).unwrap();

        assert!(result.contains_key("request"));
        assert_eq!(
            result.get("provider"),
            Some(&Value::Str("openai".to_string()))
        );

        match result.get("request") {
            Some(Value::Request(TransportRequest::Rest(req))) => {
                assert_eq!(req.url, "https://api.openai.com/v1/chat/completions");
                let body = req.body.as_ref().unwrap();
                let messages = body["messages"].as_array().unwrap();
                // Should have one user message combining content and question
                assert_eq!(messages.len(), 1);
                let msg = &messages[0];
                assert_eq!(msg["role"], "user");
                let content = msg["content"].as_str().unwrap();
                assert!(content.contains("fn add"));
                assert!(content.contains("What does this function do?"));
            }
            _ => panic!("expected REST request"),
        }
    }

    #[test]
    fn test_prepare_simple_request_with_system_prompt() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("anthropic".to_string()));
        inputs.insert(
            "model".to_string(),
            Value::Str("claude-sonnet-4-20250514".to_string()),
        );
        inputs.insert("content".to_string(), Value::Str("some code".to_string()));
        inputs.insert(
            "question".to_string(),
            Value::Str("Review this".to_string()),
        );
        inputs.insert(
            "system_prompt".to_string(),
            Value::Str("You are a code reviewer.".to_string()),
        );

        let result = LlmOps::PrepareSimpleRequest.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Rest(req))) => {
                let body = req.body.as_ref().unwrap();
                // Anthropic puts system prompt in 'system' field
                assert_eq!(body["system"], "You are a code reviewer.");
            }
            _ => panic!("expected REST request"),
        }
    }

    #[test]
    fn test_prepare_simple_request_missing_content() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("openai".to_string()));
        inputs.insert("model".to_string(), Value::Str("gpt-4o".to_string()));
        inputs.insert(
            "question".to_string(),
            Value::Str("What is this?".to_string()),
        );
        // Missing 'content'

        let err = LlmOps::PrepareSimpleRequest.execute(inputs).unwrap_err();
        assert!(err.0.contains("content"));
    }

    #[test]
    fn test_prepare_simple_request_missing_question() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("openai".to_string()));
        inputs.insert("model".to_string(), Value::Str("gpt-4o".to_string()));
        inputs.insert("content".to_string(), Value::Str("some code".to_string()));
        // Missing 'question'

        let err = LlmOps::PrepareSimpleRequest.execute(inputs).unwrap_err();
        assert!(err.0.contains("question"));
    }

    #[test]
    fn test_parse_simple_response_openai() {
        let response =
            TransportResponse::Rest(gunbc_ir::transport::RestResponse::ok(serde_json::json!({
                "model": "gpt-4o",
                "choices": [{
                    "message": {"role": "assistant", "content": "This function adds two integers."},
                    "finish_reason": "stop"
                }],
                "usage": {"prompt_tokens": 20, "completion_tokens": 10, "total_tokens": 30}
            })));

        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("openai".to_string()));
        inputs.insert("response".to_string(), Value::Response(response));

        let result = LlmOps::ParseSimpleResponse.execute(inputs).unwrap();

        // Simple response only returns 'answer'
        assert_eq!(
            result.get("answer"),
            Some(&Value::Str("This function adds two integers.".to_string()))
        );
        // Should not include other fields like 'content', 'model', 'input_tokens'
        assert!(!result.contains_key("content"));
        assert!(!result.contains_key("model"));
        assert!(!result.contains_key("input_tokens"));
    }

    #[test]
    fn test_parse_simple_response_anthropic() {
        let response =
            TransportResponse::Rest(gunbc_ir::transport::RestResponse::ok(serde_json::json!({
                "content": [{"type": "text", "text": "The function returns the sum."}],
                "model": "claude-sonnet-4-20250514",
                "stop_reason": "end_turn",
                "usage": {"input_tokens": 15, "output_tokens": 8}
            })));

        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("anthropic".to_string()));
        inputs.insert("response".to_string(), Value::Response(response));

        let result = LlmOps::ParseSimpleResponse.execute(inputs).unwrap();

        assert_eq!(
            result.get("answer"),
            Some(&Value::Str("The function returns the sum.".to_string()))
        );
    }

    #[test]
    fn test_parse_simple_response_missing_response() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("openai".to_string()));
        // Missing 'response'

        let err = LlmOps::ParseSimpleResponse.execute(inputs).unwrap_err();
        assert!(err.0.contains("response"));
    }
}

// ============================================================================
// DagSpec Registry Helpers
// ============================================================================

/// Return DagSpec registrations originating from this crate.
pub fn dag_specs() -> Vec<&'static gunbc_testgen_registry::DagSpecDef> {
    gunbc_testgen_registry::iter_dag_specs()
        .filter(|spec| spec.origin_crate == env!("CARGO_CRATE_NAME"))
        .collect()
}

// ============================================================================
// Generated Tests (from `make testgen`)
// ============================================================================

#[cfg(test)]
mod generated_tests {
    include!("generated_tests.rs");
}

#[cfg(test)]
mod generated_tests_anthropic {
    include!("generated_tests_anthropic.rs");
}

#[cfg(test)]
mod generated_tests_code_review {
    include!("generated_tests_code_review.rs");
}

#[cfg(test)]
mod generated_tests_secrets {
    include!("generated_tests_secrets.rs");
}

#[cfg(test)]
mod generated_tests_credential {
    include!("generated_tests_credential.rs");
}

#[cfg(test)]
mod generated_tests_credential_anthropic {
    include!("generated_tests_credential_anthropic.rs");
}
