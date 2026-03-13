//! Review-specific scope contracts.
//!
//! The review pipeline delegates to LLM providers for analysis. Its scope
//! contract wraps an `LlmScopeContract` parameterised by the chosen provider.

use super::llm::LlmScopeContract;
use super::scope::{CredentialIntent, ScopeContract};
use serde::{Deserialize, Serialize};

/// Review-specific permission scopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewScope {
    /// Perform code review via LLM analysis.
    CodeReview,
}

impl ReviewScope {
    /// Canonical scope identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            ReviewScope::CodeReview => "review:code_review",
        }
    }
}

/// Scope contract for review actions.
///
/// Review delegates credentialed I/O to an LLM provider, so this contract
/// resolves to the same credential intent as the underlying `LlmScopeContract`
/// with the addition of the review-specific scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewScopeContract {
    /// Underlying LLM scope contract (review delegates to LLM).
    llm_contract: LlmScopeContract,
}

impl ReviewScopeContract {
    /// Create a review scope contract for the given LLM provider.
    pub fn new(provider_id: impl Into<String>) -> Self {
        Self {
            llm_contract: LlmScopeContract::new(provider_id),
        }
    }
}

impl ScopeContract for ReviewScopeContract {
    fn credential_intent(&self) -> CredentialIntent {
        // Start from the LLM intent (which has provider-specific auth info)
        // and add the review scope.
        let mut intent = self.llm_contract.credential_intent();
        intent
            .required_scopes
            .push(ReviewScope::CodeReview.as_str().to_string());
        intent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_scope_contract_openai() {
        let intent = ReviewScopeContract::new("openai").credential_intent();
        assert_eq!(intent.provider, "openai");
        assert_eq!(intent.scheme, "bearer");
        assert!(intent
            .required_scopes
            .contains(&"llm:chat_completion".to_string()));
        assert!(intent
            .required_scopes
            .contains(&"review:code_review".to_string()));
        assert!(intent.validate().is_ok());
    }

    #[test]
    fn test_review_scope_contract_anthropic() {
        let intent = ReviewScopeContract::new("anthropic").credential_intent();
        assert_eq!(intent.provider, "anthropic");
        assert_eq!(intent.scheme, "header");
        assert_eq!(intent.header_name, "x-api-key");
        assert!(intent
            .required_scopes
            .contains(&"review:code_review".to_string()));
        assert!(intent.validate().is_ok());
    }
}
