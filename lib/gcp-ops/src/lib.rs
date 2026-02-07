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
mod graph_mock;

pub use graph::{
    build_gcp_secret_manager_credential_graph,
    build_gcp_secret_manager_credential_graph_github,
    build_gcp_secret_manager_credential_graph_metadata,
    GcpSecretManagerGraphOp,
};
pub use ops::{GcpOps, GcpRuntimeKind};
