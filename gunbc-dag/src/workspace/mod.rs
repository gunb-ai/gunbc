//! gunbc-dag Workspace module.
//!
//! Unified WorkspaceOp and fractal DAG composition.

pub mod ops;
pub mod subdags;

pub use ops::WorkspaceOp;
pub use subdags::build_workspace_dag;
pub use subdags::bootstrap::build_bootstrap_subdag;
pub use subdags::buck2::build_buck2_subdag;
pub use subdags::ci::build_ci_subdag;
pub use subdags::clippy::{build_clippy_lint_all_subdag, build_clippy_subdag};
pub use subdags::deps::{build_deps_generate_subdag, build_deps_install_subdag};
pub use subdags::gist::{build_gist_rust_subdag, build_gist_subdag};
pub use subdags::languages::build_languages_subdag;
pub use subdags::makegen::build_makegen_subdag;
