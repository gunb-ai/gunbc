//! LLM provider integration types.
//!
//! This module provides a unified interface for interacting with LLM chat
//! completion APIs (OpenAI, Anthropic, and OpenAI-compatible providers).
//!
//! # Architecture
//!
//! Following the transport pattern: Prepare (pure) -> Execute (boundary) -> Parse (pure)
//!
//! - **Chat types** (`chat.rs`): Unified `ChatRequest`/`ChatResponse` types
//! - **Provider definitions** (`provider.rs`): Data-driven provider structs
//! - **OpenAI conversions** (`openai.rs`): `ChatRequest` -> `RestRequest` and back
//! - **Anthropic conversions** (`anthropic.rs`): `ChatRequest` -> `RestRequest` and back
//!
//! # Usage
//!
//! ```ignore
//! use gunbc_ir::transport::llm::{ChatRequest, ChatMessage, build_chat_request, parse_chat_response};
//!
//! // Build a chat request
//! let chat = ChatRequest::new("gpt-4o", vec![
//!     ChatMessage::system("You are a code reviewer."),
//!     ChatMessage::user("Review this function."),
//! ]).temperature(0.3);
//!
//! // Convert to REST request (pure - no I/O)
//! let rest_request = build_chat_request("openai", &chat).unwrap();
//!
//! // Execute via TransportOps::Execute (I/O boundary)
//! // let rest_response = execute_transport(rest_request);
//!
//! // Parse response (pure - no I/O)
//! // let chat_response = parse_chat_response("openai", &rest_response).unwrap();
//! ```

pub mod anthropic;
pub mod chat;
pub mod mock;
pub mod openai;
pub mod provider;

pub use chat::{ChatMessage, ChatRequest, ChatResponse, FinishReason, Role, Usage};
pub use provider::{
    anthropic_provider, builtin_provider_ids, openai_provider, provider_by_id, ApiKeyEnvVar,
    LlmAuthStyle, LlmProvider,
};

use crate::transport::rest::{RestRequest, RestResponse};

/// Build a REST request for the given provider from a chat request.
///
/// This is the main entry point for preparing LLM requests. It dispatches
/// to the appropriate provider-specific builder based on the provider ID.
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

/// Parse a REST response from the given provider into a chat response.
///
/// Dispatches to the appropriate provider-specific parser based on the
/// provider ID.
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
