//! Canonical authenticate-chain contract model.
//!
//! This module defines the provider-neutral phase order for authentication
//! workflows. It does not prescribe concrete node implementations; instead it
//! provides a shared contract that graph builders can validate against.

use serde::{Deserialize, Serialize};

/// Canonical authenticate-chain phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthenticatePhase {
    ResolveContext,
    SelectFlow,
    AcquireBaseIdentity,
    ExchangeOrDerive,
    MaybeImpersonate,
    FinalizeCredential,
}

/// A concrete node binding for a canonical authenticate phase.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthenticatePhaseBinding {
    pub phase: AuthenticatePhase,
    pub node_id: String,
}

impl AuthenticatePhaseBinding {
    pub fn new(phase: AuthenticatePhase, node_id: impl Into<String>) -> Self {
        Self {
            phase,
            node_id: node_id.into(),
        }
    }
}

/// Return the canonical authenticate phase chain.
///
/// When `include_impersonation` is false, the `MaybeImpersonate` phase is
/// omitted while preserving canonical order for all remaining phases.
pub fn canonical_authenticate_chain(include_impersonation: bool) -> Vec<AuthenticatePhase> {
    let mut phases = vec![
        AuthenticatePhase::ResolveContext,
        AuthenticatePhase::SelectFlow,
        AuthenticatePhase::AcquireBaseIdentity,
        AuthenticatePhase::ExchangeOrDerive,
    ];
    if include_impersonation {
        phases.push(AuthenticatePhase::MaybeImpersonate);
    }
    phases.push(AuthenticatePhase::FinalizeCredential);
    phases
}

/// Validate that a phase list follows the canonical authenticate order.
pub fn validate_authenticate_chain(phases: &[AuthenticatePhase]) -> Result<(), String> {
    let include_impersonation = phases.contains(&AuthenticatePhase::MaybeImpersonate);
    let expected = canonical_authenticate_chain(include_impersonation);
    if phases == expected.as_slice() {
        return Ok(());
    }

    Err(format!(
        "invalid authenticate phase chain: expected {:?}, got {:?}",
        expected, phases
    ))
}

/// Validate canonical authenticate phase bindings.
///
/// Ensures:
/// - phase order matches the canonical authenticate chain
/// - all bound node IDs are non-empty
pub fn validate_authenticate_bindings(bindings: &[AuthenticatePhaseBinding]) -> Result<(), String> {
    let phases = bindings.iter().map(|b| b.phase).collect::<Vec<_>>();
    validate_authenticate_chain(&phases)?;

    for binding in bindings {
        if binding.node_id.trim().is_empty() {
            return Err(format!(
                "authenticate binding for phase {:?} has empty node_id",
                binding.phase
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_authenticate_chain_without_impersonation() {
        assert_eq!(
            canonical_authenticate_chain(false),
            vec![
                AuthenticatePhase::ResolveContext,
                AuthenticatePhase::SelectFlow,
                AuthenticatePhase::AcquireBaseIdentity,
                AuthenticatePhase::ExchangeOrDerive,
                AuthenticatePhase::FinalizeCredential
            ]
        );
    }

    #[test]
    fn canonical_authenticate_chain_with_impersonation() {
        assert_eq!(
            canonical_authenticate_chain(true),
            vec![
                AuthenticatePhase::ResolveContext,
                AuthenticatePhase::SelectFlow,
                AuthenticatePhase::AcquireBaseIdentity,
                AuthenticatePhase::ExchangeOrDerive,
                AuthenticatePhase::MaybeImpersonate,
                AuthenticatePhase::FinalizeCredential
            ]
        );
    }

    #[test]
    fn validate_authenticate_chain_accepts_canonical_orders() {
        assert!(validate_authenticate_chain(&canonical_authenticate_chain(false)).is_ok());
        assert!(validate_authenticate_chain(&canonical_authenticate_chain(true)).is_ok());
    }

    #[test]
    fn validate_authenticate_chain_rejects_non_canonical_order() {
        let invalid = vec![
            AuthenticatePhase::ResolveContext,
            AuthenticatePhase::AcquireBaseIdentity,
            AuthenticatePhase::SelectFlow,
            AuthenticatePhase::ExchangeOrDerive,
            AuthenticatePhase::FinalizeCredential,
        ];
        assert!(validate_authenticate_chain(&invalid).is_err());
    }

    #[test]
    fn validate_authenticate_bindings_accepts_canonical_bindings() {
        let bindings = vec![
            AuthenticatePhaseBinding::new(AuthenticatePhase::ResolveContext, "cloud_env"),
            AuthenticatePhaseBinding::new(AuthenticatePhase::SelectFlow, "resolve_auth"),
            AuthenticatePhaseBinding::new(
                AuthenticatePhase::AcquireBaseIdentity,
                "cloud_credential",
            ),
            AuthenticatePhaseBinding::new(AuthenticatePhase::ExchangeOrDerive, "cloud_credential"),
            AuthenticatePhaseBinding::new(AuthenticatePhase::MaybeImpersonate, "cloud_credential"),
            AuthenticatePhaseBinding::new(AuthenticatePhase::FinalizeCredential, "scope_preflight"),
        ];
        assert!(validate_authenticate_bindings(&bindings).is_ok());
    }

    #[test]
    fn validate_authenticate_bindings_rejects_empty_node_id() {
        let bindings = vec![
            AuthenticatePhaseBinding::new(AuthenticatePhase::ResolveContext, "cloud_env"),
            AuthenticatePhaseBinding::new(AuthenticatePhase::SelectFlow, "resolve_auth"),
            AuthenticatePhaseBinding::new(
                AuthenticatePhase::AcquireBaseIdentity,
                "cloud_credential",
            ),
            AuthenticatePhaseBinding::new(AuthenticatePhase::ExchangeOrDerive, "exchange"),
            AuthenticatePhaseBinding::new(AuthenticatePhase::FinalizeCredential, ""),
        ];
        assert!(validate_authenticate_bindings(&bindings).is_err());
    }
}
