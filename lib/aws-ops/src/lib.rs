#![recursion_limit = "1024"]
//! AWS OIDC + Secrets Manager ops (dropped-provider facade).
//!
//! This crate currently exposes fail-closed builder facades so legacy call
//! sites compile while provider support is removed from the active branch.

pub mod system_models;
mod unsupported;

pub use unsupported::{
    build_aws_secrets_manager_credential_graph, build_aws_secrets_manager_upsert_graph,
    AwsSecretManagerGraphOp,
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
