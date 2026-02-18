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
}
