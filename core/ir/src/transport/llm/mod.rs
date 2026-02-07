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
}
