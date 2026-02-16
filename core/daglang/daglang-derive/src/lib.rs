//! daglang-derive: Derives ProgressManifest, TestObligations, and ToolMetadata.
//!
//! After lowering to GraphIR, the derive phase extracts higher-level
//! information needed by renderers, test generation, and tooling:
//!
//! - **ProgressManifest**: topology, waves, SubDag boundaries, parallel
//!   groups, scatter points, stage groups — used by all progress renderers
//! - **TestObligations**: 4-bucket test obligations derived from DAG structure
//!   and `@mock_response` / `@contract` annotations
//! - **ToolMetadata**: CLI entrypoints, Makefile targets, tool descriptions
//!
//! # Pipeline position
//!
//! ```text
//! Validated GraphIR → [daglang-derive] → ProgressManifest
//!                                      → TestObligations
//!                                      → ToolMetadata
//! ```

/// Errors during derivation.
#[derive(Debug)]
pub enum DeriveError {
    /// The IR graph is not valid for manifest derivation.
    InvalidGraph(String),
}
