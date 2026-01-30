//! gunbc-bootstrap: Generate all build infrastructure.
//!
//! This tool generates:
//! - Makefile (via gunbc-makegen logic)
//! - .gitignore
//! - deps.toml template
//! - CI workflow
//!
//! All outputs are boundaries (file writes), all dry-runnable.
//!
//! # Mock Specifications
//!
//! Mock specs are in `graph_mock.rs` for test generation.
//!
//! # Note
//!
//! `ScanWorkspace` currently uses direct filesystem reads (`std::fs::read_dir`).
//! This is a known deviation from the transport pattern and is documented
//! for future migration to PrepareDirectoryListOp + TransportOps::Execute.

// ScanWorkspace uses direct filesystem reads (future migration target)
#![allow(clippy::disallowed_methods)]

pub mod graph;
pub mod ops;

#[cfg(test)]
pub mod graph_mock;

pub use graph::{build_bootstrap_graph, bootstrap_signature};
pub use ops::BootstrapOp;
