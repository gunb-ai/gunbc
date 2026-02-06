//! LLM provider definitions.
//!
//! Providers are data-driven structs identified by string IDs,
//! following the project convention of avoiding enums for extensible concepts.
//!
//! Each provider defines:
//! - API endpoint and authentication pattern
//! - How to build a `RestRequest` from a `ChatRequest` (pure)
//! - How to parse a `RestResponse` into a `ChatResponse` (pure)

use serde::{Deserialize, Serialize};

use crate::transport::credential::AuthScheme;

/// LLM provider definition.
///
/// Data-driven struct that describes how to interact with an LLM API.
/// Provider-specific request building and response parsing are handled
/// by the conversion functions in openai.rs / anthropic.rs / openai_responses.rs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmProvider {
    /// Provider identifier (e.g., "openai", "anthropic").
    pub id: String,
    /// Human-readable name (e.g., "OpenAI", "Anthropic").
    pub name: String,
    /// Base URL for the API.
    pub api_base: String,
    /// Chat completions endpoint path (e.g., "/v1/chat/completions", "/v1/messages").
    pub chat_endpoint: String,
    /// Responses API endpoint path, if supported (e.g., "/v1/responses").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses_endpoint: Option<String>,
    /// Authentication scheme for the provider.
    pub auth_scheme: AuthScheme,
    /// Environment variable containing the API key.
    pub api_key_env: String,
    /// Additional required headers (e.g., anthropic-version).
    #[serde(default)]
    pub extra_headers: Vec<(String, String)>,
}

impl LlmProvider {
    /// Full URL for the chat completion endpoint.
    pub fn chat_url(&self) -> String {
        format!("{}{}", self.api_base, self.chat_endpoint)
    }

    /// Full URL for the responses API endpoint, if supported.
    pub fn responses_url(&self) -> Option<String> {
        self.responses_endpoint
            .as_ref()
            .map(|ep| format!("{}{}", self.api_base, ep))
    }
}

/// OpenAI provider definition.
///
/// Supports both Chat Completions and Responses API endpoints.
pub fn openai_provider() -> LlmProvider {
    LlmProvider {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        api_base: "https://api.openai.com".to_string(),
        chat_endpoint: "/v1/chat/completions".to_string(),
        responses_endpoint: Some("/v1/responses".to_string()),
        auth_scheme: AuthScheme::Bearer,
        api_key_env: "OPENAI_API_KEY".to_string(),
        extra_headers: vec![],
    }
}

/// Anthropic provider definition.
pub fn anthropic_provider() -> LlmProvider {
    LlmProvider {
        id: "anthropic".to_string(),
        name: "Anthropic".to_string(),
        api_base: "https://api.anthropic.com".to_string(),
        chat_endpoint: "/v1/messages".to_string(),
        responses_endpoint: None,
        auth_scheme: AuthScheme::Header {
            name: "x-api-key".to_string(),
        },
        api_key_env: "ANTHROPIC_API_KEY".to_string(),
        extra_headers: vec![("anthropic-version".to_string(), "2023-06-01".to_string())],
    }
}

/// Look up a provider by ID.
///
/// Returns `None` for unknown provider IDs. Callers that need custom
/// providers should construct `LlmProvider` directly.
pub fn provider_by_id(id: &str) -> Option<LlmProvider> {
    match id {
        "openai" => Some(openai_provider()),
        "anthropic" => Some(anthropic_provider()),
        _ => None,
    }
}

/// List all built-in provider IDs.
pub fn builtin_provider_ids() -> &'static [&'static str] {
    &["openai", "anthropic"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider() {
        let p = openai_provider();
        assert_eq!(p.id, "openai");
        assert_eq!(p.chat_url(), "https://api.openai.com/v1/chat/completions");
        assert!(matches!(p.auth_scheme, AuthScheme::Bearer));
        assert_eq!(p.api_key_env, "OPENAI_API_KEY");
    }

    #[test]
    fn test_anthropic_provider() {
        let p = anthropic_provider();
        assert_eq!(p.id, "anthropic");
        assert_eq!(p.chat_url(), "https://api.anthropic.com/v1/messages");
        assert!(matches!(p.auth_scheme, AuthScheme::Header { .. }));
        assert_eq!(p.api_key_env, "ANTHROPIC_API_KEY");
        assert!(!p.extra_headers.is_empty());
    }

    #[test]
    fn test_provider_lookup() {
        assert!(provider_by_id("openai").is_some());
        assert!(provider_by_id("anthropic").is_some());
        assert!(provider_by_id("unknown").is_none());
    }

    #[test]
    fn test_builtin_ids() {
        let ids = builtin_provider_ids();
        assert!(ids.contains(&"openai"));
        assert!(ids.contains(&"anthropic"));
    }
}
