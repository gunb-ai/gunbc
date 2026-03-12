//! TuringComplete SubDag: Category for programming languages.
//!
//! Programming languages (Rust, Python, TypeScript) share:
//! - Type systems
//! - Naming conventions
//! - Control flow constructs
//! - Module systems
//!
//! # Composes
//!
//! - TypeSystemMapping SubDag
//! - NamingConventions SubDag

use crate::dag::{Dag, Port};
use crate::language::LanguageOp;
use crate::node::Node;

/// Build the TuringComplete category SubDag node.
///
/// This SubDag composes TypeSystemMapping and NamingConventions,
/// providing shared functionality for programming languages.
///
/// # Example
///
/// ```text
/// let tc_node = build_turing_complete_subdag();
/// // Languages like Rust, Python compose this SubDag
/// ```
pub fn build_turing_complete_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // Configuration node for TuringComplete category
    inner.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            Port::scalar("has_type_system", "Bool"),
            Port::scalar("has_control_flow", "Bool"),
            Port::scalar("has_modules", "Bool"),
        ],
        LanguageOp::TuringCompleteConfig,
    ));

    // Type mapping node (would compose TypeSystemMapping in full impl)
    inner.add_node(Node::opaque(
        "type_mapping",
        vec![
            Port::scalar("abstract_type", "String"),
            Port::scalar("language", "String"),
        ],
        vec![Port::scalar("concrete_type", "String")],
        LanguageOp::MapType,
    ));

    // Naming node (would compose NamingConventions in full impl)
    inner.add_node(Node::opaque(
        "naming",
        vec![
            Port::scalar("name", "String"),
            Port::scalar("context", "String"),
        ],
        vec![Port::scalar("converted", "String")],
        LanguageOp::ConvertCase,
    ));

    // Create the SubDag node with interface
    Node::subdag("turing_complete", inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_turing_complete_subdag_is_subdag() {
        let node = build_turing_complete_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "turing_complete");
    }

    #[test]
    fn test_turing_complete_subdag_interface() {
        let node = build_turing_complete_subdag();

        // Check inputs
        assert!(node.inputs.iter().any(|p| p.name.0 == "language"));
        assert!(node.inputs.iter().any(|p| p.name.0 == "abstract_type"));
        assert!(node.inputs.iter().any(|p| p.name.0 == "name"));

        // Check outputs
        assert!(node.outputs.iter().any(|p| p.name.0 == "has_type_system"));
    }

    #[test]
    fn test_turing_complete_subdag_structure() {
        let node = build_turing_complete_subdag();

        match &node.body {
            NodeBody::SubDag(dag, _) => {
                assert_eq!(dag.nodes.len(), 3);

                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_ids.contains(&"config"));
                assert!(node_ids.contains(&"type_mapping"));
                assert!(node_ids.contains(&"naming"));
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
