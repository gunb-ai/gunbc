//! Transport operations and executors for gunbc I/O.
//!
//! This library provides:
//! - `TransportOps` - DAG node operations for transport execution
//!
//! The transport layer separates pure business logic from I/O:
//! - Pure ops prepare `TransportRequest` values
//! - `TransportOps::Execute` is the boundary that does actual I/O
//!
//! In dry-run mode, the boundary is mocked to intercept I/O.
//!
//! # Structural I/O Enforcement
//!
//! `execute_transport()` and `execute_request()` are NOT exported from this crate.
//! The primary I/O boundary is `TransportOps::Execute` nodes in a DAG. Tool
//! acquisition/execution helpers live here as well, so CLI tool I/O stays in
//! the transport layer rather than leaking into pure crates.
//!
//! This ensures I/O is:
//! - Visible in the graph structure (for transport requests)
//! - Interceptable by DryRun mode
//! - Auditable
//!
//! # Note
//!
//! This is the ONLY crate (besides codegen) that performs direct I/O operations
//! via std::fs and std::process::Command. All other crates MUST use
//! PrepareXxxOp + TransportOps::Execute.

#![deny(dead_code)]
// This crate IS the transport layer - it's allowed to use direct I/O
#![allow(clippy::disallowed_methods)]

pub mod credential;
pub mod credential_graph;
pub mod backend;
pub mod cli;
pub mod executor;
pub mod ops;
pub mod resource_io;
pub mod test_backend;

// STRUCTURAL ENFORCEMENT: TransportOps + transport-layer CLI helpers only
// execute_transport and execute_request are internal - not exported
pub use credential::{CredentialOp, GitHubEnvVarProvider, LlmEnvVarProvider, MockCredentialProvider};
pub use backend::{TransportBackend, TransportBackendGuard};
pub use ops::TransportOps;
pub use resource_io::TransportIo;

#[cfg(test)]
mod pragma_lint;
