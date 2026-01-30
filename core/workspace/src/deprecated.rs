//! Deprecated aliases for backward compatibility.
//!
//! These functions provide backward compatibility for code that uses the
//! old `build_*_graph()` function signatures. They extract the inner DAG
//! from the new SubDag nodes.
//!
//! **Migration Guide**: Replace `build_*_graph()` calls with `build_*_subdag()`
//! and update your code to work with `Node<WorkspaceOp>` instead of `Dag<*GraphOp>`.

use crate::subdags::{bootstrap, buck2, ci, clippy, deps, gist, makegen};
use crate::WorkspaceOp;
use gunbc_ir::{BuilderError, Dag, NodeBody};

/// Extract inner DAG from a SubDag node.
fn extract_inner_dag<T>(node: gunbc_ir::Node<T>) -> Dag<T> {
    match node.body {
        NodeBody::SubDag(dag) => dag,
        NodeBody::Opaque(_) => panic!("Expected SubDag node"),
    }
}

/// Build the makegen graph (deprecated).
///
/// **Deprecated**: Use `build_makegen_subdag()` instead.
#[deprecated(since = "0.2.0", note = "Use build_makegen_subdag() instead")]
pub fn build_makegen_graph() -> Result<Dag<WorkspaceOp>, BuilderError> {
    Ok(extract_inner_dag(makegen::build_makegen_subdag()))
}

/// Build the clippy graph with arguments (deprecated).
///
/// **Deprecated**: Use `build_clippy_subdag()` instead.
#[deprecated(since = "0.2.0", note = "Use build_clippy_subdag() instead")]
pub fn build_clippy_graph(args: &[&str]) -> Dag<WorkspaceOp> {
    extract_inner_dag(clippy::build_clippy_subdag(args))
}

/// Build the clippy lint-all graph (deprecated).
///
/// **Deprecated**: Use `build_clippy_lint_all_subdag()` instead.
#[deprecated(since = "0.2.0", note = "Use build_clippy_lint_all_subdag() instead")]
pub fn build_clippy_lint_all_graph() -> Dag<WorkspaceOp> {
    extract_inner_dag(clippy::build_clippy_lint_all_subdag())
}

/// Build the deps install graph (deprecated).
///
/// **Deprecated**: Use `build_deps_install_subdag()` instead.
#[deprecated(since = "0.2.0", note = "Use build_deps_install_subdag() instead")]
pub fn build_deps_install_graph() -> Dag<WorkspaceOp> {
    extract_inner_dag(deps::build_deps_install_subdag())
}

/// Build the deps generate graph (deprecated).
///
/// **Deprecated**: Use `build_deps_generate_subdag()` instead.
#[deprecated(since = "0.2.0", note = "Use build_deps_generate_subdag() instead")]
pub fn build_deps_generate_graph() -> Dag<WorkspaceOp> {
    extract_inner_dag(deps::build_deps_generate_subdag())
}

/// Build the bootstrap graph (deprecated).
///
/// **Deprecated**: Use `build_bootstrap_subdag()` instead.
#[deprecated(since = "0.2.0", note = "Use build_bootstrap_subdag() instead")]
pub fn build_bootstrap_graph() -> Dag<WorkspaceOp> {
    extract_inner_dag(bootstrap::build_bootstrap_subdag())
}

/// Build the buck2 graph (deprecated).
///
/// **Deprecated**: Use `build_buck2_subdag()` instead.
#[deprecated(since = "0.2.0", note = "Use build_buck2_subdag() instead")]
pub fn build_buck2_graph() -> Dag<WorkspaceOp> {
    extract_inner_dag(buck2::build_buck2_subdag())
}

/// Build the CI graph (deprecated).
///
/// **Deprecated**: Use `build_ci_subdag()` instead.
#[deprecated(since = "0.2.0", note = "Use build_ci_subdag() instead")]
pub fn build_ci_graph() -> Dag<WorkspaceOp> {
    extract_inner_dag(ci::build_ci_subdag())
}

/// Build the gist graph (deprecated).
///
/// **Deprecated**: Use `build_gist_subdag()` instead.
#[deprecated(since = "0.2.0", note = "Use build_gist_subdag() instead")]
pub fn build_gist_graph(extensions: Vec<String>, create_gist: bool) -> Dag<WorkspaceOp> {
    extract_inner_dag(gist::build_gist_subdag(extensions, create_gist))
}

/// Build the default gist rust graph (deprecated).
///
/// **Deprecated**: Use `build_gist_rust_subdag()` instead.
#[deprecated(since = "0.2.0", note = "Use build_gist_rust_subdag() instead")]
pub fn build_gist_rust_graph() -> Dag<WorkspaceOp> {
    extract_inner_dag(gist::build_gist_rust_subdag())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_deprecated_makegen_graph() {
        let dag = build_makegen_graph().expect("should build");
        assert_eq!(dag.nodes.len(), 4);
    }

    #[test]
    #[allow(deprecated)]
    fn test_deprecated_bootstrap_graph() {
        let dag = build_bootstrap_graph();
        assert_eq!(dag.nodes.len(), 9);
    }

    #[test]
    #[allow(deprecated)]
    fn test_deprecated_buck2_graph() {
        let dag = build_buck2_graph();
        assert_eq!(dag.nodes.len(), 7);
    }
}
