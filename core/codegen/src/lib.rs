//! gunbc-codegen: Shared code generation utilities.
//!
//! This crate provides common utilities for code generation tools:
//! - [`Template`]: Simple string template rendering
//! - [`FileWriter`]: File writing with dry-run support
//! - [`DagInfo`]: Combined boundary and entrypoint information

pub mod file_writer;
pub mod template;

pub use file_writer::{FileWriter, WriteResult};
pub use template::Template;

use gunbc_ir::{detect_boundaries, detect_entrypoints, BoundaryInfo, Dag, EntrypointInfo};

/// Combined DAG analysis information for code generation.
#[derive(Debug)]
pub struct DagInfo {
    /// Boundary information (world writes)
    pub boundaries: BoundaryInfo,
    /// Entrypoint information (world reads)
    pub entrypoints: EntrypointInfo,
}

impl DagInfo {
    /// Analyze a DAG for code generation.
    pub fn analyze<T>(dag: &Dag<T>) -> Self {
        Self {
            boundaries: detect_boundaries(dag),
            entrypoints: detect_entrypoints(dag),
        }
    }
}
