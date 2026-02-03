//! Languages SubDag builder.
//!
//! Wraps the languages DAG as a SubDag node using WorkspaceOp.

use crate::workspace::WorkspaceOp;
use gunbc_ir::language::{
    build_comment_prefix_subdag, build_config_format_subdag, build_gitignore_subdag,
    build_glob_subdag, build_makefile_subdag, build_naming_conventions_subdag, build_regex_subdag,
    build_rust_subdag, build_turing_complete_subdag, build_type_system_mapping_subdag,
    build_variable_syntax_subdag, LanguageOp,
};
use gunbc_ir::{Dag, Node};

/// Convert a Node<LanguageOp> to Node<WorkspaceOp>.
fn convert_language_node(node: Node<LanguageOp>) -> Node<WorkspaceOp> {
    Node {
        id: node.id,
        inputs: node.inputs,
        outputs: node.outputs,
        body: match node.body {
            gunbc_ir::NodeBody::Opaque(op) => {
                gunbc_ir::NodeBody::Opaque(WorkspaceOp::Language(op))
            }
            gunbc_ir::NodeBody::SubDag(dag) => {
                gunbc_ir::NodeBody::SubDag(convert_language_dag(dag))
            }
        },
        examples: Vec::new(),
    }
}

/// Convert a Dag<LanguageOp> to Dag<WorkspaceOp>.
fn convert_language_dag(dag: Dag<LanguageOp>) -> Dag<WorkspaceOp> {
    Dag {
        nodes: dag.nodes.into_iter().map(convert_language_node).collect(),
        edges: dag.edges,
    }
}

/// Build the languages SubDag node.
///
/// This wraps the Languages DAG as a `Node<WorkspaceOp>` containing all
/// language, format, and pattern SubDags.
///
/// # I/O Interface
///
/// The Languages SubDag is primarily a model DAG with no I/O.
/// Its child SubDags provide language characteristics like:
/// - Comment syntax
/// - Naming conventions
/// - Type mappings
/// - File patterns
pub fn build_languages_subdag() -> Node<WorkspaceOp> {
    let mut inner: Dag<WorkspaceOp> = Dag::new();

    // Pattern SubDags (foundations)
    inner.add_node(convert_language_node(build_regex_subdag()));
    inner.add_node(convert_language_node(build_glob_subdag()));
    inner.add_node(convert_language_node(build_variable_syntax_subdag()));

    // Trait SubDags (composable characteristics)
    inner.add_node(convert_language_node(build_type_system_mapping_subdag()));
    inner.add_node(convert_language_node(build_naming_conventions_subdag()));
    inner.add_node(convert_language_node(build_comment_prefix_subdag()));

    // Category SubDags
    inner.add_node(convert_language_node(build_turing_complete_subdag()));
    inner.add_node(convert_language_node(build_config_format_subdag()));

    // Language/Format SubDags
    inner.add_node(convert_language_node(build_rust_subdag()));
    inner.add_node(convert_language_node(build_gitignore_subdag()));
    inner.add_node(convert_language_node(build_makefile_subdag()));

    // Wrap as SubDag with explicit interface
    Node::subdag(
        "languages",
        vec![], // No inputs - pure model
        vec![], // No outputs - accessed via child SubDags
        inner,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_languages_subdag_is_subdag() {
        let node = build_languages_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "languages");
    }

    #[test]
    fn test_languages_subdag_contains_all_subdags() {
        let node = build_languages_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();

                // Pattern SubDags
                assert!(node_ids.contains(&"regex"));
                assert!(node_ids.contains(&"glob"));
                assert!(node_ids.contains(&"variable_syntax"));

                // Trait SubDags
                assert!(node_ids.contains(&"type_system"));
                assert!(node_ids.contains(&"naming"));
                assert!(node_ids.contains(&"comment_prefix"));

                // Category SubDags
                assert!(node_ids.contains(&"turing_complete"));
                assert!(node_ids.contains(&"config_format"));

                // Language SubDags
                assert!(node_ids.contains(&"rust"));
                assert!(node_ids.contains(&"gitignore"));
                assert!(node_ids.contains(&"makefile"));
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
