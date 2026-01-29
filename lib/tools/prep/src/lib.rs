//! gunbc-prep: Repository preparation tool.
//!
//! This tool represents the "unwind" DAG - it runs all necessary code generation
//! to prepare the repository for other operations like testing, linting, etc.
//!
//! The prep DAG ensures that:
//! - CLI main.rs files are generated (codegen)
//! - graph.rs files are generated from declarative DAGs (daggen)
//! - Test files are generated from DAG analysis (testgen)
//! - The project builds successfully
//!
//! Other commands (test, check, clippy) can depend on prep to ensure
//! the repository is in a consistent state before running.
//!
//! # Pipeline
//!
//! ```text
//! CheckState -> RunCodegen -> RunDaggen -> Build
//!                  (sequential dependency chain)
//! ```
//!
//! # Mock Specifications
//!
//! Mock specs are in `graph_mock.rs` for test generation.

pub mod graph;
pub mod ops;

#[cfg(test)]
pub mod graph_mock;

pub use graph::{build_prep_graph, prep_signature};
pub use ops::PrepOp;
