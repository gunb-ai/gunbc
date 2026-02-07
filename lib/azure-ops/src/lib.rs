//! Azure OIDC + Key Vault ops (stub).
//!
//! Placeholder implementation to keep provider-neutral modeling honest.

mod ops;
mod graph;
pub mod graph_mock;

pub use graph::{build_azure_key_vault_credential_graph, AzureKeyVaultGraphOp};
pub use ops::AzureOps;

// ============================================================================
// DagSpec Registry Helpers
// ============================================================================

/// Return DagSpec registrations originating from this crate.
pub fn dag_specs() -> Vec<&'static gunbc_testgen_registry::DagSpecDef> {
    gunbc_testgen_registry::iter_dag_specs()
        .filter(|spec| spec.origin_crate == env!("CARGO_CRATE_NAME"))
        .collect()
}

// ============================================================================
// Generated Tests (from `make testgen`)
// ============================================================================

#[cfg(test)]
mod generated_tests {
    include!("generated_tests.rs");
}
