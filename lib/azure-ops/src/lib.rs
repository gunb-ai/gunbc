#![recursion_limit = "1024"]
//! Azure OIDC + Key Vault ops (stub).
//!
//! Placeholder implementation to keep provider-neutral modeling honest.

mod graph;
mod ops;

pub use graph::{
    build_azure_key_vault_credential_graph, build_azure_key_vault_upsert_graph,
    AzureKeyVaultGraphOp,
};
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
