//! Scope contracts for credentialed actions.
//!
//! Credentialed interfaces should declare required permissions as typed
//! contracts. Runtime acquisition strategy is derived from these contracts.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A resolved credential intent for an action.
///
/// This is the provider-neutral contract consumed by credential lifecycle DAGs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialIntent {
    /// Credential provider identifier (e.g., "github").
    pub provider: String,
    /// Service/source identifier used by secret binding.
    pub service: String,
    /// Secret Manager secret ID (without namespace prefix).
    ///
    /// Required: `BindSecretName` will error if this is `None`.
    /// Must be set via `.with_secret_name()` on every `CredentialIntent`.
    /// Example: `Some("github-token")` for the GitHub PAT stored as
    /// `dev-github-token` in Secret Manager.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_name: Option<String>,
    /// Auth scheme expected by execute boundary (e.g., "bearer").
    pub scheme: String,
    /// Optional header name when `scheme` requires a custom header.
    pub header_name: String,
    /// Required permission scopes for this action.
    pub required_scopes: Vec<String>,
    /// Whether interactive acquisition is allowed for this action.
    pub interactive_allowed: bool,
}

impl CredentialIntent {
    /// Create a new credential intent.
    pub fn new(
        provider: impl Into<String>,
        service: impl Into<String>,
        scheme: impl Into<String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            service: service.into(),
            secret_name: None,
            scheme: scheme.into(),
            header_name: String::new(),
            required_scopes: Vec::new(),
            interactive_allowed: false,
        }
    }

    /// Set the Secret Manager secret ID (without namespace prefix).
    ///
    /// Use this when the secret ID differs from `service`.
    /// Example: service="github", secret_name="github-token".
    pub fn with_secret_name(mut self, secret_name: impl Into<String>) -> Self {
        self.secret_name = Some(secret_name.into());
        self
    }

    /// Set a custom header name (for header-based schemes).
    pub fn with_header_name(mut self, header_name: impl Into<String>) -> Self {
        self.header_name = header_name.into();
        self
    }

    /// Set required scopes.
    pub fn with_required_scopes<I, S>(mut self, scopes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_scopes = scopes.into_iter().map(Into::into).collect();
        self
    }

    /// Set whether interactive acquisition is allowed.
    pub fn with_interactive_allowed(mut self, interactive_allowed: bool) -> Self {
        self.interactive_allowed = interactive_allowed;
        self
    }

    /// Validate that this contract is actionable.
    pub fn validate(&self) -> Result<(), ScopeContractError> {
        if self.provider.trim().is_empty() {
            return Err(ScopeContractError::MissingProvider);
        }
        if self.service.trim().is_empty() {
            return Err(ScopeContractError::MissingService);
        }
        if self.scheme.trim().is_empty() {
            return Err(ScopeContractError::MissingScheme);
        }
        if self.required_scopes.is_empty() {
            return Err(ScopeContractError::MissingRequiredScopes);
        }
        for scope in &self.required_scopes {
            if scope.trim().is_empty() {
                return Err(ScopeContractError::InvalidScope(
                    "scope entries cannot be empty".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// A typed scope contract for a credentialed action.
pub trait ScopeContract {
    /// Resolve this contract into a provider-neutral intent.
    fn credential_intent(&self) -> CredentialIntent;
}

/// Contract validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeContractError {
    MissingProvider,
    MissingService,
    MissingScheme,
    MissingRequiredScopes,
    InvalidScope(String),
}

impl fmt::Display for ScopeContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScopeContractError::MissingProvider => {
                write!(f, "credential intent missing provider")
            }
            ScopeContractError::MissingService => {
                write!(f, "credential intent missing service")
            }
            ScopeContractError::MissingScheme => {
                write!(f, "credential intent missing scheme")
            }
            ScopeContractError::MissingRequiredScopes => {
                write!(f, "credential intent missing required scopes")
            }
            ScopeContractError::InvalidScope(msg) => write!(f, "invalid scope: {msg}"),
        }
    }
}

impl std::error::Error for ScopeContractError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_intent_requires_non_empty_scope_list() {
        let intent = CredentialIntent::new("github", "github", "bearer");
        let err = intent.validate().unwrap_err();
        assert_eq!(err, ScopeContractError::MissingRequiredScopes);
    }

    #[test]
    fn test_credential_intent_validation_ok() {
        let intent = CredentialIntent::new("github", "github", "bearer")
            .with_required_scopes(["gist:write"]);
        assert!(intent.validate().is_ok());
    }
}
