//! Gist SubDag builder.
//!
//! Wraps the gist DSL module as a Workspace subdag.

use crate::dsl_builder::build_gist_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_gist::GistMode;
use gunbc_ir::transport::cloud::CloudSecretConfig;
use gunbc_ir::Node;

/// Build the gist SubDag node.
pub fn build_gist_subdag(
    mode: GistMode,
    extensions: Vec<String>,
    create_gist: bool,
) -> Node<WorkspaceOp> {
    build_gist_subdag_with_config(mode, extensions, create_gist, None)
}

/// Build a gist SubDag with explicit cloud config.
///
/// The current DSL-backed subdag is mode-agnostic at composition time; mode
/// and credential routing are represented inside the DSL module.
pub fn build_gist_subdag_with_config(
    _mode: GistMode,
    _extensions: Vec<String>,
    _create_gist: bool,
    _cloud_config: Option<CloudSecretConfig>,
) -> Node<WorkspaceOp> {
    let dsl_dag = build_gist_graph_dsl().expect("gist DSL graph should build");
    Node::subdag("gist", dsl_dag)
}

/// Build a default gist SubDag for Rust files (snapshot mode).
pub fn build_gist_rust_subdag() -> Node<WorkspaceOp> {
    build_gist_subdag(GistMode::Snapshot, vec![".rs".to_string()], false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_gist_subdag_is_subdag() {
        let node = build_gist_rust_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "gist");
    }

    #[test]
    fn test_gist_subdag_has_nodes() {
        let node = build_gist_rust_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert!(
                    !dag.nodes.is_empty(),
                    "gist DSL subdag should contain nodes"
                );
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
