//! Transport middleware configuration types.
//!
//! These IR-level types describe how transport execution should be shaped at
//! runtime (rate limiting, retry, credential policy, and response
//! classification). They are designed to round-trip through serde so lowering,
//! resolution, and execution layers can share one configuration model.

use serde::{Deserialize, Serialize};

/// End-to-end middleware configuration for one transport operation.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TransportMiddlewareConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential: Option<CredentialConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_classification: Option<ResponseClassification>,
}

/// Rate limiting configuration for an operation/provider scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Shared scope key (e.g. "github:core", "github:search", "gcp:iam").
    pub scope_key: String,
    pub algorithm: RateLimitAlgorithm,
    /// Maximum immediate burst before throttling.
    pub max_burst: u32,
    /// Total requests allowed in the window.
    pub requests: u32,
    /// Window duration in seconds (e.g. 3600 for per-hour limits).
    pub window_seconds: u32,
    /// Respect `Retry-After` headers when present.
    #[serde(default)]
    pub honor_retry_after: bool,
}

/// Rate limiting algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitAlgorithm {
    TokenBucket,
    SlidingWindow,
}

/// Retry policy for transient failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Total attempts including the first request.
    pub max_attempts: u32,
    /// Base delay before retry #2.
    pub base_delay_ms: u64,
    /// Upper bound for any single delay.
    pub max_delay_ms: u64,
    pub backoff: RetryBackoff,
    /// HTTP statuses considered retryable.
    #[serde(default)]
    pub retry_statuses: Vec<u16>,
    /// Retry network/transport failures that don't have an HTTP status.
    #[serde(default)]
    pub retry_network_errors: bool,
    /// Guardrail: only auto-retry idempotent/readonly operations.
    #[serde(default)]
    pub require_idempotent_or_readonly: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circuit_breaker: Option<CircuitBreakerConfig>,
}

/// Retry backoff strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryBackoff {
    Fixed,
    Exponential,
    ExponentialJitter,
}

/// Circuit breaker behavior for persistent failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub reset_timeout_ms: u64,
    pub half_open_max_requests: u32,
}

/// Credential middleware configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialConfig {
    pub provider: CredentialProvider,
    pub injection: CredentialInjection,
    /// Shared cache key across operations needing the same credential.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<String>,
    /// Optional hard cache TTL override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_ttl_ms: Option<u64>,
    /// Refresh threshold as percent of observed TTL (80 = proactive refresh at 80%).
    #[serde(default = "default_refresh_threshold_pct")]
    pub refresh_threshold_pct: u8,
}

fn default_refresh_threshold_pct() -> u8 {
    80
}

/// Credential provider family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialProvider {
    #[serde(rename = "oauth_bearer", alias = "o_auth_bearer")]
    OAuthBearer,
    GcpWorkloadIdentityFederation,
    ApiKey,
}

/// Where/how credential material is injected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CredentialInjection {
    AuthorizationBearer,
    Header { name: String },
    QueryParam { name: String },
    RequestAuthField,
}

/// Response classification policy and provider hint.
///
/// # TL-15: Provider-agnostic classification
///
/// Error extraction uses only the `error_shape` JSON-path rules. The
/// `parse_provider_error_shapes` field is retained for serialization backward
/// compatibility but is not used by the transport layer. New code should
/// always populate `error_shape`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseClassification {
    pub provider: ResponseProvider,
    /// Prefer auth-classification over generic 4xx when both signals exist.
    #[serde(default)]
    pub prioritize_auth_errors: bool,
    /// Legacy field retained for serialization backward compatibility.
    /// The transport layer ignores this field — all error extraction uses
    /// `error_shape` JSON-path rules (TL-15).
    #[serde(default, skip_serializing_if = "is_false")]
    pub parse_provider_error_shapes: bool,
    /// JSON-path based error shape extraction rules (TL-16).
    /// The transport layer uses these paths to extract error details from the
    /// response body. This is the sole error extraction mechanism (TL-15).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_shape: Option<ErrorShapeExtraction>,
    /// JSON-path based output shape extraction rules (C29).
    /// When present, the parse op uses these declarative rules instead of
    /// hardcoded field extraction. Populated from `output {}` blocks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_shape: Option<OutputShapeExtraction>,
}

fn is_false(v: &bool) -> bool {
    !v
}

/// JSON-path based error shape extraction (TL-16).
///
/// Replaces hardcoded `parse_github_error`/`parse_gcp_error`/etc. with
/// declarative extraction rules from `error_shape {}` blocks in `.dag` files.
/// The transport layer blindly executes these JSON-path extractions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorShapeExtraction {
    /// JSON path to extract the error message (e.g., ".message", ".error.message").
    pub message_path: String,
    /// JSON path to extract the error code (e.g., ".status", ".error.code").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_path: Option<String>,
    /// JSON path to extract additional details (e.g., ".documentation_url", ".error.errors").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details_path: Option<String>,
}

/// JSON-path based output shape extraction (C29).
///
/// Parallel to `ErrorShapeExtraction`: declares how to extract typed output
/// fields from a response body using JSON-path rules. Each field specifies
/// a name, type, and extraction path. The transport layer uses these rules
/// instead of hardcoded field extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputShapeExtraction {
    /// Per-field extraction rules.
    pub fields: Vec<OutputFieldExtraction>,
}

/// A single output field extraction rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputFieldExtraction {
    /// Output field name (as exposed on the parse node's output port).
    pub name: String,
    /// Expected type for type-aware deserialization.
    pub type_id: String,
    /// JSON path for extraction (e.g., "choices/0/message/content").
    /// When ".", extracts the entire response body.
    pub json_path: String,
    /// Whether this field contains a secret value.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_secret: bool,
    /// Whether to use the raw response body string instead of JSON extraction.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_raw_body: bool,
    /// Whether the field is optional (missing → Value::Unit).
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_optional: bool,
}

/// Provider-specific error-shape hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseProvider {
    Generic,
    #[serde(rename = "github", alias = "git_hub")]
    GitHub,
    Gcp,
    Anthropic,
    #[serde(rename = "openai", alias = "open_ai")]
    OpenAi,
}

impl std::str::FromStr for ResponseProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "generic" | "Generic" => Ok(Self::Generic),
            "github" | "GitHub" | "git_hub" => Ok(Self::GitHub),
            "gcp" | "Gcp" | "GCP" => Ok(Self::Gcp),
            "anthropic" | "Anthropic" => Ok(Self::Anthropic),
            "openai" | "OpenAi" | "open_ai" => Ok(Self::OpenAi),
            other => Err(format!("unknown response provider: `{other}`")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_middleware_config_round_trip_json() {
        let config = TransportMiddlewareConfig {
            rate_limit: Some(RateLimitConfig {
                scope_key: "github:core".to_string(),
                algorithm: RateLimitAlgorithm::TokenBucket,
                max_burst: 20,
                requests: 5000,
                window_seconds: 3600,
                honor_retry_after: true,
            }),
            retry: Some(RetryConfig {
                max_attempts: 4,
                base_delay_ms: 100,
                max_delay_ms: 2_000,
                backoff: RetryBackoff::ExponentialJitter,
                retry_statuses: vec![429, 500, 502, 503, 504],
                retry_network_errors: true,
                require_idempotent_or_readonly: true,
                circuit_breaker: Some(CircuitBreakerConfig {
                    failure_threshold: 5,
                    reset_timeout_ms: 30_000,
                    half_open_max_requests: 1,
                }),
            }),
            credential: Some(CredentialConfig {
                provider: CredentialProvider::OAuthBearer,
                injection: CredentialInjection::AuthorizationBearer,
                cache_key: Some("github-token".to_string()),
                cache_ttl_ms: None,
                refresh_threshold_pct: 80,
            }),
            response_classification: Some(ResponseClassification {
                provider: ResponseProvider::GitHub,
                prioritize_auth_errors: true,
                parse_provider_error_shapes: false,
                error_shape: Some(ErrorShapeExtraction {
                    message_path: ".message".to_string(),
                    code_path: None,
                    details_path: Some(".documentation_url".to_string()),
                }),
                output_shape: None,
            }),
        };

        let json = serde_json::to_string_pretty(&config).expect("serialize middleware config");
        let round_trip: TransportMiddlewareConfig =
            serde_json::from_str(&json).expect("deserialize middleware config");
        assert_eq!(round_trip, config);
    }

    #[test]
    fn credential_refresh_threshold_defaults_to_eighty_percent() {
        let json = r#"{
            "provider":"oauth_bearer",
            "injection":{"kind":"authorization_bearer"}
        }"#;
        let cfg: CredentialConfig = serde_json::from_str(json).expect("credential config");
        assert_eq!(cfg.refresh_threshold_pct, 80);
    }

    #[test]
    fn response_classification_defaults_parse_provider_shapes_to_false() {
        let json = r#"{
            "provider":"gcp",
            "prioritize_auth_errors":true
        }"#;
        let cfg: ResponseClassification = serde_json::from_str(json).expect("classification");
        // TL-15: parse_provider_error_shapes defaults to false; the transport
        // layer uses only error_shape JSON-path extraction.
        assert!(!cfg.parse_provider_error_shapes);
        assert!(cfg.error_shape.is_none());
    }

    #[test]
    fn error_shape_extraction_round_trips_through_json() {
        let config = TransportMiddlewareConfig {
            rate_limit: None,
            retry: None,
            credential: None,
            response_classification: Some(ResponseClassification {
                provider: ResponseProvider::GitHub,
                prioritize_auth_errors: true,
                parse_provider_error_shapes: false,
                error_shape: Some(ErrorShapeExtraction {
                    message_path: ".message".to_string(),
                    code_path: Some(".status".to_string()),
                    details_path: Some(".documentation_url".to_string()),
                }),
                output_shape: None,
            }),
        };

        let json = serde_json::to_string_pretty(&config).expect("serialize");
        let round_trip: TransportMiddlewareConfig =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(round_trip, config);
        let shape = round_trip
            .response_classification
            .unwrap()
            .error_shape
            .unwrap();
        assert_eq!(shape.message_path, ".message");
        assert_eq!(shape.code_path, Some(".status".to_string()));
        assert_eq!(shape.details_path, Some(".documentation_url".to_string()));
    }
}
