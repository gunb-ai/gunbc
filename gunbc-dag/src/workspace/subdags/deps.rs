//! Deps SubDag builders.
//!
//! Wraps the deps DSL module as workspace subdags.

use crate::dsl_builder::build_deps_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

fn build_deps_subdag_with_id(id: &str) -> Result<Node<WorkspaceOp>, BuilderError> {
    let dsl_dag = build_deps_graph_dsl()?;
    Ok(Node::subdag(id, dsl_dag))
}

/// Build the deps install SubDag node.
pub fn build_deps_install_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    build_deps_subdag_with_id("deps_install")
}

/// Build the deps generate SubDag node.
pub fn build_deps_generate_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    build_deps_subdag_with_id("deps_generate")
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn deps_install_subdag_is_subdag() {
        let node = build_deps_install_subdag().expect("deps install subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "deps_install");
    }

    #[test]
    fn deps_generate_subdag_is_subdag() {
        let node = build_deps_generate_subdag().expect("deps generate subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "deps_generate");
    }

    #[test]
    fn deps_subdags_contain_nodes() {
        let install = build_deps_install_subdag().expect("deps install subdag should build");
        let generate = build_deps_generate_subdag().expect("deps generate subdag should build");
        for node in [install, generate] {
            match node.body {
                NodeBody::SubDag(ref dag) => {
                    assert!(
                        !dag.nodes.is_empty(),
                        "deps DSL subdag should contain nodes"
                    );
                }
                _ => panic!("Expected SubDag"),
            }
        }
    }
}
