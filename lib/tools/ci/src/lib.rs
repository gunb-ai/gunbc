//! gunbc-ci: CI orchestration binary.
//!
//! This crate provides a CI runner that:
//! 1. Ensures tool dependencies are installed (via deps upsert)
//! 2. Runs CI steps (build, test, lint, etc.)
//!
//! The CI logic is in testable Rust code, not YAML.
//! The minimal YAML shim just calls this binary.
//!
//! # Transport Pattern
//!
//! This crate follows the "every node is pure" pattern:
//! - `CIGraphOp` is a union of pure CI ops, primitives, and transport
//! - All I/O happens through explicit `TransportOps::Execute` nodes
//! - DryRun can intercept all transport nodes
//!
//! # GitHub Actions Integration
//!
//! The workflow configuration is typed via [`WorkflowConfig`], which declares
//! integrations used and computes required permissions automatically.
//!
//! ```ignore
//! let config = ci_workflow_config();
//! assert!(config.runner.has_tool("cargo"));
//! assert!(!config.permissions.is_empty()); // needs contents:read
//! ```

pub mod graph;
pub mod ops;

#[cfg(test)]
pub mod graph_mock;

pub use graph::{
    build_ci_graph, ci_integrations, ci_signature, ci_workflow_config, ci_workflow_permissions,
    CIGraphOp,
};
// Re-export embedded ops from primitives for convenience
pub use gunbc_primitives::EmbeddedFileExistsOp;
// Re-export WorkflowConfig from github_actions for convenience
pub use gunbc_ir::transport::github_actions::WorkflowConfig;
pub use ops::CIOp;
