//! gunbc-dag Workspace module.
//!
//! Workspace composition built on `DynOp` subdags.

pub(crate) mod convert;
pub mod subdags;

pub type WorkspaceOp = gunbc_exec::DynOp;
pub use subdags::bootstrap::build_bootstrap_subdag;
pub use subdags::build::build_build_subdag;
pub use subdags::build_workspace_dag;
pub use subdags::ci::build_ci_subdag;
pub use subdags::clippy::{build_clippy_lint_all_subdag, build_clippy_subdag};
pub use subdags::codegen::build_codegen_subdag;
pub use subdags::dag_viz::build_dag_viz_subdag;
pub use subdags::deps::{build_deps_generate_subdag, build_deps_install_subdag};
pub use subdags::docgen::build_docgen_subdag;
pub use subdags::gist::{build_gist_rust_subdag, build_gist_subdag};
pub use subdags::languages::build_languages_subdag;
pub use subdags::makegen::build_makegen_subdag;
pub use subdags::pragma::build_pragma_subdag;
pub use subdags::testgen::build_testgen_subdag;
