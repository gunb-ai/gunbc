#![recursion_limit = "1024"]
//! AWS OIDC + Secrets Manager ops (stub).
//!
//! This is a placeholder to keep provider-neutral modeling honest.
//! The implementation will follow the same subject-token → STS →
//! optional role chaining → Secrets Manager pattern.

mod graph;
pub mod graph_mock;
mod ops;
pub mod system_models;

pub use graph::{
    build_aws_secrets_manager_credential_graph, build_aws_secrets_manager_upsert_graph,
    AwsSecretManagerGraphOp,
};
pub use ops::AwsOps;

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
