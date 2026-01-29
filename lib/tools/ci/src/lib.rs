//! gunbc-ci: CI orchestration binary.
//!
//! This crate provides a CI runner that:
//! 1. Ensures tool dependencies are installed (via deps upsert)
//! 2. Runs CI steps (build, test, lint, etc.)
//!
//! The CI logic is in testable Rust code, not YAML.
//! The minimal YAML shim just calls this binary.
//!
//! # Mock Specifications
//!
//! Mock specs are in `graph_mock.rs` for test generation.

pub mod graph;
pub mod ops;

#[cfg(test)]
pub mod graph_mock;

pub use graph::build_ci_graph;
pub use ops::CIOp;
