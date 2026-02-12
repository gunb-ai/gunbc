//! SubDag builders for each tool.
//!
//! Each tool has a `build_*_subdag()` function that returns `Node<WorkspaceOp>`,
//! enabling fractal composition into the Workspace DAG.

pub mod bootstrap;
pub mod ci;
pub mod clippy;
pub mod deps;
pub mod gist;
pub mod languages;
pub mod makegen;

use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Dag};

/// Build the Workspace DAG containing all tool and language SubDags.
///
/// This is the root DAG that composes all functionality in the workspace
/// using the fractal SubDag pattern.
///
/// # Structure
///
/// ```text
/// Workspace DAG
/// ├── makegen SubDag (Makefile generation)
/// ├── languages SubDag
/// │   ├── rust
/// │   ├── makefile
/// │   ├── gitignore
/// │   └── ...
/// └── (more tools as they're migrated)
/// ```
pub fn build_workspace_dag() -> Result<Dag<WorkspaceOp>, BuilderError> {
    let mut dag = Dag::new();

    // Tool SubDags (all migrated to fractal pattern)
    dag.add_node(makegen::build_makegen_subdag());
    dag.add_node(clippy::build_clippy_lint_all_subdag());
    dag.add_node(deps::build_deps_install_subdag()?);
    dag.add_node(deps::build_deps_generate_subdag()?);
    dag.add_node(bootstrap::build_bootstrap_subdag()?);
    dag.add_node(ci::build_ci_subdag());
    dag.add_node(gist::build_gist_rust_subdag());

    // Language SubDag (already fractal)
    dag.add_node(languages::build_languages_subdag());

    Ok(dag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_dag_structure() {
        let dag = build_workspace_dag().expect("workspace dag should build");

        // Should have all tool subdags plus languages
        let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
        assert!(node_ids.contains(&"makegen"));
        assert!(node_ids.contains(&"clippy"));
        assert!(node_ids.contains(&"deps_install"));
        assert!(node_ids.contains(&"deps_generate"));
        assert!(node_ids.contains(&"bootstrap"));
        assert!(node_ids.contains(&"ci"));
        assert!(node_ids.contains(&"gist"));
        assert!(node_ids.contains(&"languages"));
    }

    #[test]
    fn test_workspace_dag_nodes_are_subdags() {
        let dag = build_workspace_dag().expect("workspace dag should build");

        for node in &dag.nodes {
            assert!(node.is_subdag(), "Node {} should be a SubDag", node.id.0);
        }
    }
}
