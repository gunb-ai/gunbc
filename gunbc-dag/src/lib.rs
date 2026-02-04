//! gunbc-dag: Repo-specific DAG configuration for gunbc.
//!
//! This crate contains the gunbc repo's specific configuration, including:
//! - CI pipeline definition
//! - Makefile generation
//! - Bootstrap tools
//! - Workspace DAG composition
//!
//! # Distinction from lib/tools/
//!
//! The crates in `lib/tools/` are general-purpose tool wrappers that could be
//! used by any project. This crate (`gunbc-dag`) contains configuration
//! specific to the gunbc repository itself.
//!
//! For example:
//! - `gunbc-clippy` (in lib/tools/) wraps the clippy CLI tool (general)
//! - `gunbc-dag::ci` defines gunbc's CI pipeline (repo-specific)

pub mod bootstrap;
pub mod build;
pub mod ci;
pub mod makegen;
pub mod workspace;

// Re-exports for convenience
pub use bootstrap::{BootstrapOp, BootstrapGraphOp, build_bootstrap_graph, bootstrap_signature};
pub use build::{BuildOp, BuildGraphOp, build_build_graph, build_signature};
pub use ci::{CIOp, CIGraphOp, build_ci_graph, ci_signature, ci_workflow_config};
pub use makegen::{
    MakegenOp, MakegenGraphOp, build_makegen_graph, makegen_signature,
    BuildConfig, default_build_config, render_makefile, render_gitignore,
};
pub use workspace::{
    WorkspaceOp, build_workspace_dag,
    build_bootstrap_subdag, build_buck2_subdag, build_ci_subdag,
    build_clippy_lint_all_subdag, build_clippy_subdag,
    build_deps_generate_subdag, build_deps_install_subdag,
    build_gist_rust_subdag, build_gist_subdag,
    build_languages_subdag, build_makegen_subdag,
};
