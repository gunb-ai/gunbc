#![recursion_limit = "1024"]
//! GCP Workload Identity Federation + Secret Manager ops.
//!
//! This crate models:
//! - OIDC acquisition (runtime-provided tokens)
//! - STS token exchange
//! - Optional service account impersonation
//! - Secret Manager access + decoding
//! - Credential assembly for downstream transports

pub mod discovery_graph;
pub mod discovery_ops;
mod ops;
pub mod services;
pub mod system_models;

pub use discovery_graph::{build_infra_discovery_dag, GcpDiscoveryGraphOp};
pub use discovery_ops::GcpDiscoveryOps;
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
