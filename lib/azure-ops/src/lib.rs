#![recursion_limit = "1024"]
//! Azure OIDC + Key Vault ops (dropped-provider facade).
//!
//! This crate currently exposes fail-closed builder facades so legacy call
//! sites compile while provider support is removed from the active branch.

mod unsupported;

pub use unsupported::{
    build_azure_key_vault_credential_graph, build_azure_key_vault_upsert_graph,
    AzureKeyVaultGraphOp,
};

// ============================================================================
// DagSpec Registry Helpers
// ============================================================================

/// Return DagSpec registrations originating from this crate.
pub fn dag_specs() -> Vec<&'static gunbc_testgen_registry::DagSpecDef> {
    gunbc_testgen_registry::iter_dag_specs()
        .filter(|spec| spec.origin_crate == env!("CARGO_CRATE_NAME"))
        .collect()
}
