//! LLM provider definitions.
//!
//! Providers are data-driven structs identified by string IDs,
//! following the project convention of avoiding enums for extensible concepts.
//!
//! Each provider defines:
//! - API endpoint and authentication pattern
//! - How to build a `RestRequest` from a `ChatRequest` (pure)
//! - How to parse a `RestResponse` into a `ChatResponse` (pure)
//!
//! # Single-source registry
//!
//! All built-in providers are defined once in [`BUILTIN_PROVIDERS`].
//! Adding a new provider requires exactly **one** edit: append an
//! `LlmProviderMeta` entry to that array. All lookup, listing, and
//! convenience constructor functions derive from the registry.

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

// ---------------------------------------------------------------------------
// Static registry — single source of truth
// ---------------------------------------------------------------------------

/// Static metadata for a built-in LLM provider.
///
/// All fields are `&'static` so the registry can live in a `const` array.
/// Use [`LlmProviderMeta::to_provider`] to obtain a heap-allocated
/// [`LlmProvider`] when needed.
#[derive(Debug, Clone, Copy)]
pub struct LlmProviderMeta {
    /// Provider identifier (e.g., "openai", "anthropic").
    pub id: &'static str,
    /// Human-readable name (e.g., "OpenAI", "Anthropic").
    pub name: &'static str,
    /// Base URL for the API.
    pub api_base: &'static str,
    /// Chat completions endpoint path.
    pub chat_endpoint: &'static str,
    /// Responses API endpoint path, if supported.
    pub responses_endpoint: Option<&'static str>,
    /// Authentication scheme (static representation).
    pub auth: LlmAuthMeta,
    /// Environment variable containing the API key.
    pub api_key_env: &'static str,
    /// Additional required headers (static representation).
    pub extra_headers: &'static [(&'static str, &'static str)],
}

/// Static authentication scheme metadata.
#[derive(Debug, Clone, Copy)]
pub enum LlmAuthMeta {
    /// `Authorization: Bearer {token}`
    Bearer,
    /// Custom header: `{name}: {token}`
    Header { name: &'static str },
}

impl LlmProviderMeta {
    /// Convert to a heap-allocated [`LlmProvider`].
    pub fn to_provider(self) -> LlmProvider {
        LlmProvider {
            id: self.id.to_string(),
            name: self.name.to_string(),
            api_base: self.api_base.to_string(),
            chat_endpoint: self.chat_endpoint.to_string(),
            responses_endpoint: self.responses_endpoint.map(|s| s.to_string()),
            auth_scheme: match self.auth {
                LlmAuthMeta::Bearer => AuthScheme::Bearer,
                LlmAuthMeta::Header { name } => AuthScheme::Header {
                    name: name.to_string(),
                },
            },
            api_key_env: self.api_key_env.to_string(),
            extra_headers: self
                .extra_headers
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

/// The single source of truth for all built-in LLM providers.
///
/// To add a new provider, append an entry here. All lookup, listing,
/// and convenience functions derive from this array automatically.
pub static BUILTIN_PROVIDERS: &[LlmProviderMeta] = &[
    // ----- OpenAI -----
    LlmProviderMeta {
        id: "openai",
        name: "OpenAI",
        api_base: "https://api.openai.com",
        chat_endpoint: "/v1/chat/completions",
        responses_endpoint: Some("/v1/responses"),
        auth: LlmAuthMeta::Bearer,
        api_key_env: "OPENAI_API_KEY",
        extra_headers: &[],
    },
    // ----- Anthropic -----
    LlmProviderMeta {
        id: "anthropic",
        name: "Anthropic",
        api_base: "https://api.anthropic.com",
        chat_endpoint: "/v1/messages",
        responses_endpoint: None,
        auth: LlmAuthMeta::Header { name: "x-api-key" },
        api_key_env: "ANTHROPIC_API_KEY",
        extra_headers: &[("anthropic-version", "2023-06-01")],
    },
];

// ---------------------------------------------------------------------------
// Convenience constructors (derived from registry)
// ---------------------------------------------------------------------------

/// OpenAI provider definition.
///
/// Supports both Chat Completions and Responses API endpoints.
pub fn openai_provider() -> LlmProvider {
    provider_by_id("openai").expect("openai is a built-in provider")
}

/// Anthropic provider definition.
pub fn anthropic_provider() -> LlmProvider {
    provider_by_id("anthropic").expect("anthropic is a built-in provider")
}

// ---------------------------------------------------------------------------
// Lookup helpers (derived from registry)
// ---------------------------------------------------------------------------

/// Look up a provider's static metadata by ID.
///
/// Returns `None` for unknown provider IDs.
pub fn provider_meta_by_id(id: &str) -> Option<&'static LlmProviderMeta> {
    BUILTIN_PROVIDERS.iter().find(|p| p.id == id)
}

/// Look up a provider by ID.
///
/// Returns `None` for unknown provider IDs. Callers that need custom
/// providers should construct `LlmProvider` directly.
pub fn provider_by_id(id: &str) -> Option<LlmProvider> {
    provider_meta_by_id(id).map(|m| m.to_provider())
}

/// List all built-in provider IDs.
pub fn builtin_provider_ids() -> Vec<&'static str> {
    BUILTIN_PROVIDERS.iter().map(|p| p.id).collect()
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

    #[test]
    fn test_registry_is_single_source_of_truth() {
        // Every provider returned by builtin_provider_ids() must be resolvable
        // via provider_by_id(), and vice versa.
        for id in builtin_provider_ids() {
            assert!(
                provider_by_id(id).is_some(),
                "provider_by_id({id:?}) should resolve"
            );
        }
        // The count must match.
        assert_eq!(builtin_provider_ids().len(), BUILTIN_PROVIDERS.len());
    }

    #[test]
    fn test_provider_meta_by_id() {
        let meta = provider_meta_by_id("openai").unwrap();
        assert_eq!(meta.id, "openai");
        assert_eq!(meta.name, "OpenAI");
        assert!(matches!(meta.auth, LlmAuthMeta::Bearer));

        let meta = provider_meta_by_id("anthropic").unwrap();
        assert_eq!(meta.id, "anthropic");
        assert!(matches!(
            meta.auth,
            LlmAuthMeta::Header { name: "x-api-key" }
        ));

        assert!(provider_meta_by_id("unknown").is_none());
    }
}
