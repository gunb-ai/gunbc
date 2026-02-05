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
pub mod codegen;
pub mod makegen;
pub mod workspace;

// Re-exports for convenience
pub use bootstrap::{bootstrap_signature, build_bootstrap_graph, BootstrapGraphOp, BootstrapOp};
pub use build::{build_build_graph, build_signature, BuildGraphOp, BuildOp};
pub use ci::{build_ci_graph, ci_signature, ci_workflow_config, CIGraphOp, CIOp};
pub use codegen::{build_codegen_graph, codegen_signature, CodegenGraphOp, CodegenOp};
pub use gunbc_ir::CODEGEN_STAMP_PATH;
pub use makegen::{
    build_makegen_graph, default_build_config, makegen_signature, render_gitignore,
    render_makefile, BuildConfig, MakegenGraphOp, MakegenOp,
};
pub use workspace::{
    build_bootstrap_subdag, build_ci_subdag, build_clippy_lint_all_subdag, build_clippy_subdag,
    build_deps_generate_subdag, build_deps_install_subdag, build_gist_rust_subdag,
    build_gist_subdag, build_languages_subdag, build_makegen_subdag, build_workspace_dag,
    WorkspaceOp,
};
