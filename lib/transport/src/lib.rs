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
//! The ONLY way to perform I/O is through `TransportOps::Execute` nodes in a DAG.
//! This ensures all I/O is:
//! - Visible in the graph structure
//! - Interceptable by DryRun mode
//! - Auditable
//!
//! # Note
//!
//! This is the ONLY crate (besides codegen) that performs direct I/O operations
//! via std::fs and std::process::Command. All other crates MUST use
//! PrepareXxxOp + TransportOps::Execute.

// This crate IS the transport layer - it's allowed to use direct I/O
#![allow(clippy::disallowed_methods)]

pub mod credential;
pub mod env;
pub mod executor;
pub mod ops;

// STRUCTURAL ENFORCEMENT: Only export TransportOps
// execute_transport and execute_request are internal - not exported
pub use credential::{CredentialOp, GitHubEnvVarProvider, LlmEnvVarProvider, MockCredentialProvider};
pub use env::AuthEnv;
pub use ops::TransportOps;
