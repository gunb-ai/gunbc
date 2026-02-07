//! GCP Workload Identity Federation + Secret Manager ops.
//!
//! This crate models:
//! - OIDC acquisition (runtime-provided tokens)
//! - STS token exchange
//! - Optional service account impersonation
//! - Secret Manager access + decoding
//! - Credential assembly for downstream transports

mod ops;
mod graph;
pub mod graph_mock;

pub use graph::{
    build_gcp_secret_manager_credential_graph,
    build_gcp_secret_manager_credential_graph_github,
    build_gcp_secret_manager_credential_graph_metadata,
    build_gcp_secret_manager_upsert_graph,
    build_gcp_secret_manager_upsert_graph_github,
    build_gcp_secret_manager_upsert_graph_metadata,
    GcpSecretManagerGraphOp,
};
pub use ops::{GcpOps, GcpRuntimeKind};

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

#[cfg(test)]
mod generated_tests_upsert {
    include!("generated_tests_upsert.rs");
}
