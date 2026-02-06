//! gunbc-dag Makegen module.
//!
//! Makefile generation from gunbc DAG entrypoints.

pub mod gitignore;
pub mod graph;
pub mod ops;
pub mod registry;
pub mod render;

pub mod graph_mock;

pub use gitignore::{derive_categories, render_gitignore, GitignoreRenderer};
pub use graph::{build_makegen_graph, makegen_signature, MakegenGraphOp};
pub use ops::MakegenOp;
pub use registry::{
    default_build_config, default_meta_targets, BuildConfig, BuildSystem, ConfigField,
    EntrypointParam, MetaTarget, PrepLevel, ToolInfo, ToolRegistry,
};
pub use render::{render_makefile, render_makefile_with_config};

#[cfg(test)]
mod generated_tests {
    include!("generated_tests.rs");
}
