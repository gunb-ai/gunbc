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

pub mod graph;
pub mod ops;

#[cfg(test)]
pub mod graph_mock;

pub use graph::build_bootstrap_graph;
pub use ops::BootstrapOp;
