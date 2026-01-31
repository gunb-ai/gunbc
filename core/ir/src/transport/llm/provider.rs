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

/// Environment variable name for the API key.
///
/// Providers reference env vars by name; resolution happens at execution time
/// through the existing `AuthMethod::EnvVar` mechanism.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiKeyEnvVar(pub String);

/// Authentication style for the LLM API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LlmAuthStyle {
    /// Bearer token in Authorization header (OpenAI, most providers).
    BearerToken,
    /// Custom header for API key (Anthropic uses x-api-key).
    CustomHeader {
        /// Header name (e.g., "x-api-key").
        header: String,
    },
}

/// LLM provider definition.
///
/// Data-driven struct that describes how to interact with an LLM API.
/// Provider-specific request building and response parsing are handled
/// by the conversion functions in openai.rs / anthropic.rs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmProvider {
    /// Provider identifier (e.g., "openai", "anthropic").
    pub id: String,
    /// Human-readable name (e.g., "OpenAI", "Anthropic").
    pub name: String,
    /// Base URL for the chat completion endpoint.
    pub api_base: String,
    /// Chat completions endpoint path.
    pub chat_endpoint: String,
    /// Authentication style.
    pub auth_style: LlmAuthStyle,
    /// Environment variable containing the API key.
    pub api_key_env: ApiKeyEnvVar,
    /// Additional required headers (e.g., anthropic-version).
    #[serde(default)]
    pub extra_headers: Vec<(String, String)>,
}

impl LlmProvider {
    /// Full URL for the chat completion endpoint.
    pub fn chat_url(&self) -> String {
        format!("{}{}", self.api_base, self.chat_endpoint)
    }
}

/// OpenAI provider definition.
pub fn openai_provider() -> LlmProvider {
    LlmProvider {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        api_base: "https://api.openai.com".to_string(),
        chat_endpoint: "/v1/chat/completions".to_string(),
        auth_style: LlmAuthStyle::BearerToken,
        api_key_env: ApiKeyEnvVar("OPENAI_API_KEY".to_string()),
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
        auth_style: LlmAuthStyle::CustomHeader {
            header: "x-api-key".to_string(),
        },
        api_key_env: ApiKeyEnvVar("ANTHROPIC_API_KEY".to_string()),
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
        assert_eq!(
            p.chat_url(),
            "https://api.openai.com/v1/chat/completions"
        );
        assert!(matches!(p.auth_style, LlmAuthStyle::BearerToken));
        assert_eq!(p.api_key_env.0, "OPENAI_API_KEY");
    }

    #[test]
    fn test_anthropic_provider() {
        let p = anthropic_provider();
        assert_eq!(p.id, "anthropic");
        assert_eq!(p.chat_url(), "https://api.anthropic.com/v1/messages");
        assert!(matches!(
            p.auth_style,
            LlmAuthStyle::CustomHeader { .. }
        ));
        assert_eq!(p.api_key_env.0, "ANTHROPIC_API_KEY");
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
