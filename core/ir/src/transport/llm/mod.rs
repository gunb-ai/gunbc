//! LLM provider integration types.
//!
//! This module provides a unified interface for interacting with LLM APIs:
//! - **OpenAI Chat Completions** (`/v1/chat/completions`)
//! - **OpenAI Responses** (`/v1/responses`) — recommended for reasoning models
//! - **Anthropic Messages** (`/v1/messages`)
//!
//! # Architecture
//!
//! Following the transport pattern: Prepare (pure) -> Execute (boundary) -> Parse (pure)
//!
//! - **Chat types** (`chat.rs`): Unified `ChatRequest`/`ChatResponse` types with
//!   content blocks, cache hints, and thinking/reasoning configuration
//! - **Provider definitions** (`provider.rs`): Data-driven provider structs
//! - **OpenAI Chat Completions** (`openai.rs`): Standard chat completion endpoint
//! - **OpenAI Responses** (`openai_responses.rs`): Responses API with reasoning summaries
//! - **Anthropic** (`anthropic.rs`): Messages API with prompt caching and extended thinking
//!
//! # Usage
//!
//! ```ignore
//! use gunbc_ir::transport::llm::*;
//!
//! // Simple chat completion
//! let chat = ChatRequest::new("gpt-4o", vec![
//!     ChatMessage::system("You are a code reviewer."),
//!     ChatMessage::user("Review this function."),
//! ]).temperature(0.3);
//! let rest_request = build_chat_request("openai", &chat).unwrap();
//!
//! // Anthropic with caching and extended thinking
//! let chat = ChatRequest::new("claude-sonnet-4-5", vec![
//!     ChatMessage::system_blocks(vec![
//!         ContentBlock::text("Long context...").with_cache(CacheControl::ephemeral()),
//!     ]),
//!     ChatMessage::user("Analyze this."),
//! ]).max_tokens(16000).thinking(ThinkingConfig::anthropic(10000));
//! let rest_request = build_chat_request("anthropic", &chat).unwrap();
//! ```

pub mod anthropic;
pub mod chat;
pub mod mock;
pub mod openai;
pub mod openai_responses;
pub mod provider;

pub use chat::{
    CacheControl, CacheType, ChatMessage, ChatRequest, ChatResponse, ContentBlock, FinishReason,
    MessageContent, ReasoningEffort, ReasoningSummary, ResponseBlock, Role, ThinkingConfig, Usage,
};
pub use provider::{
    anthropic_provider, builtin_provider_ids, openai_provider, provider_by_id, LlmProvider,
};

use crate::transport::rest::{RestRequest, RestResponse};
use crate::transport::scope::{CredentialIntent, ScopeContract};
use serde::{Deserialize, Serialize};

// ============================================================================
// LLM Scope Contracts
// ============================================================================

/// LLM-specific permission scopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmScope {
    /// Send chat completion / message requests.
    ChatCompletion,
}

impl LlmScope {
    /// Canonical scope identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            LlmScope::ChatCompletion => "llm:chat_completion",
        }
    }
}

/// Scope contract for LLM chat completion actions.
///
/// This contract is parameterized by provider ID so it can resolve
/// the correct auth scheme (bearer for OpenAI, header for Anthropic).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmScopeContract {
    provider_id: String,
}

impl LlmScopeContract {
    /// Create a scope contract for the given LLM provider.
    pub fn new(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
        }
    }

    /// Scope contract for OpenAI.
    pub fn openai() -> Self {
        Self::new("openai")
    }

    /// Scope contract for Anthropic.
    pub fn anthropic() -> Self {
        Self::new("anthropic")
    }
}

impl ScopeContract for LlmScopeContract {
    fn credential_intent(&self) -> CredentialIntent {
        let provider = provider_by_id(&self.provider_id);
        let (scheme, header_name) = match provider.as_ref().map(|p| &p.auth_scheme) {
            Some(crate::AuthScheme::Bearer) => ("bearer", ""),
            Some(crate::AuthScheme::Header { name }) => ("header", name.as_str()),
            _ => ("bearer", ""),
        };

        let mut intent =
            CredentialIntent::new(&self.provider_id, &self.provider_id, scheme)
                .with_required_scopes([LlmScope::ChatCompletion.as_str()])
                .with_interactive_allowed(true);

        if !header_name.is_empty() {
            intent = intent.with_header_name(header_name);
        }

        intent
    }
}

/// Build a REST request for the Chat Completions / Messages endpoint.
///
/// Dispatches to the appropriate provider-specific builder:
/// - `"openai"` → OpenAI Chat Completions (`/v1/chat/completions`)
/// - `"anthropic"` → Anthropic Messages (`/v1/messages`)
///
/// # Errors
///
/// Returns `Err` if the provider ID is not recognized.
pub fn build_chat_request(provider_id: &str, chat: &ChatRequest) -> Result<RestRequest, String> {
    match provider_id {
        "openai" => Ok(openai::build_openai_request(chat)),
        "anthropic" => Ok(anthropic::build_anthropic_request(chat)),
        _ => Err(format!("unknown LLM provider: '{}'", provider_id)),
    }
}

/// Parse a REST response from the Chat Completions / Messages endpoint.
///
/// Dispatches to the appropriate provider-specific parser.
///
/// # Errors
///
/// Returns `Err` if the provider ID is not recognized or the response
/// cannot be parsed.
pub fn parse_chat_response(
    provider_id: &str,
    response: &RestResponse,
) -> Result<ChatResponse, String> {
    match provider_id {
        "openai" => openai::parse_openai_response(response),
        "anthropic" => anthropic::parse_anthropic_response(response),
        _ => Err(format!("unknown LLM provider: '{}'", provider_id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_chat_request_dispatch() {
        let chat = ChatRequest::new("gpt-4o", vec![ChatMessage::user("Hello")]);

        assert!(build_chat_request("openai", &chat).is_ok());
        assert!(build_chat_request("anthropic", &chat).is_ok());
        assert!(build_chat_request("unknown", &chat).is_err());
    }

    #[test]
    fn test_parse_chat_response_dispatch() {
        let openai_resp = RestResponse::ok(serde_json::json!({
            "model": "gpt-4o",
            "choices": [{
                "message": { "role": "assistant", "content": "Hi" },
                "finish_reason": "stop"
            }],
            "usage": { "prompt_tokens": 5, "completion_tokens": 1, "total_tokens": 6 }
        }));

        let anthropic_resp = RestResponse::ok(serde_json::json!({
            "content": [{ "type": "text", "text": "Hi" }],
            "model": "claude-sonnet-4-20250514",
            "stop_reason": "end_turn",
            "usage": { "input_tokens": 5, "output_tokens": 1 }
        }));

        assert!(parse_chat_response("openai", &openai_resp).is_ok());
        assert!(parse_chat_response("anthropic", &anthropic_resp).is_ok());
        assert!(parse_chat_response("unknown", &openai_resp).is_err());
    }

    #[test]
    fn test_llm_scope_contract_openai() {
        let intent = LlmScopeContract::openai().credential_intent();
        assert_eq!(intent.provider, "openai");
        assert_eq!(intent.service, "openai");
        assert_eq!(intent.scheme, "bearer");
        assert!(intent.header_name.is_empty());
        assert_eq!(
            intent.required_scopes,
            vec!["llm:chat_completion".to_string()]
        );
        assert!(intent.interactive_allowed);
        assert!(intent.validate().is_ok());
    }

    #[test]
    fn test_llm_scope_contract_anthropic() {
        let intent = LlmScopeContract::anthropic().credential_intent();
        assert_eq!(intent.provider, "anthropic");
        assert_eq!(intent.service, "anthropic");
        assert_eq!(intent.scheme, "header");
        assert_eq!(intent.header_name, "x-api-key");
        assert_eq!(
            intent.required_scopes,
            vec!["llm:chat_completion".to_string()]
        );
        assert!(intent.validate().is_ok());
    }
}
