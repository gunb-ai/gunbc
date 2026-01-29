//! gunbc-makegen: Makefile generation from DAG entrypoints.
//!
//! This crate generates Makefile targets from gunbc tool entrypoints.
//! Entrypoints are inputs with no upstream edge — they come from the world
//! and become Make variables.
//!
//! # Example Generated Makefile
//!
//! ```makefile
//! # gunbc-gist entrypoints: repo_path (String)
//! gist:
//!     @cargo run -p gunbc-gist -- $(if $(REPO),--repo $(REPO))
//! ```
//!
//! # Mock Specifications
//!
//! Mock specs are in `graph_mock.rs` for test generation.

pub mod ops;
pub mod graph;
pub mod registry;
pub mod render;

#[cfg(test)]
pub mod graph_mock;

pub use graph::{build_makegen_graph, makegen_signature};
pub use ops::MakegenOp;
pub use registry::{
    default_build_config, default_meta_targets, BuildConfig, BuildSystem, ConfigField,
    EntrypointParam, MetaTarget, PrepLevel, ToolInfo, ToolRegistry,
};
pub use render::{render_makefile, render_makefile_with_config};
