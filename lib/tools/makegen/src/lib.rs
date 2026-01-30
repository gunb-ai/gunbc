//! gunbc-makegen: Makefile and .gitignore generation from repo layout.
//!
//! This crate generates Makefile targets and .gitignore patterns from the
//! repo's `BuildConfig` and `ToolRegistry`.
//!
//! # Generated Files
//!
//! - **Makefile**: Tool targets from DAG entrypoints, dev UX targets with -fix variants
//! - **.gitignore**: Patterns derived from build system (Cargo, Buck2, etc.)
//!
//! # Dev UX Convention (from the-gunbai)
//!
//! - `make <target>` - verify only (CI-safe, fails on issues)
//! - `make <target>-fix` - auto-fix then verify (for dev)
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

pub mod gitignore;
pub mod graph;
pub mod ops;
pub mod registry;
pub mod render;

#[cfg(test)]
pub mod graph_mock;

pub use gitignore::{derive_categories, render_gitignore, GitignoreRenderer, IgnoreCategory};
pub use graph::{build_makegen_graph, makegen_signature};
pub use ops::MakegenOp;
pub use registry::{
    default_build_config, default_meta_targets, BuildConfig, BuildSystem, ConfigField,
    EntrypointParam, MetaTarget, PrepLevel, ToolInfo, ToolRegistry,
};
pub use render::{render_makefile, render_makefile_with_config};
