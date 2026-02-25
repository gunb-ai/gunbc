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
//! This crate is a library. The codegen binary lives in gunbc-dag
//! (see `gunbc-dag/src/bin/codegen_cli.rs`) and performs source/DSL-driven
//! discovery there while this crate stays a leaf utility layer.

#![deny(dead_code)]

pub mod cli_gen;
pub mod entrypoint;
pub mod file_writer;
pub mod lambda_gen;
pub mod registry;
pub mod rest_gen;
pub mod template;
pub mod testgen;

pub use cli_gen::{generate_cli, generate_cli_with_import, CliEntrypoint, ToolMeta};
pub use file_writer::{FileWriter, WriteResult};
pub use registry::{core_outputs, TestgenTargetDef, ToolDef};
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
