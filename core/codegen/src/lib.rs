//! gunbc-codegen: Shared code generation utilities.
//!
//! This crate provides common utilities for code generation tools:
//! - [`Template`]: Simple string template rendering
//! - [`FileWriter`]: File writing with dry-run support
//! - [`DagInfo`]: Combined boundary and entrypoint information
//! - [`cli_gen`]: CLI generation from DAG entrypoints
//! - [`testgen`]: Test generation from proof obligations
//! - Testgen target registry (see gunbc-testgen-registry)
//!
//! # Note
//!
//! This crate is the bootstrapper - it generates code for other tools.
//! As such, it cannot use the transport pattern (circular dependency).
//! It uses direct filesystem operations by design.

#![deny(dead_code)]
// Codegen is the bootstrapper - can't use transport layer (circular dependency)
#![allow(clippy::disallowed_methods)]

pub mod cli_gen;
pub mod dag_gen;
pub mod file_writer;
pub mod registry;
pub mod template;
pub mod testgen;

pub use cli_gen::{
    generate_cli, generate_cli_with_import, CliBoundary, CliEntrypoint, GraphBuilderId, ToolMeta,
};
pub use dag_gen::generate_graph_rs;
pub use file_writer::{FileWriter, WriteResult};
pub use registry::{
    all_cleanable_outputs, all_tools, core_outputs, DagDef, EdgeDef, NodeDef, PortDef,
    TestgenTargetDef, ToolDef,
};
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
