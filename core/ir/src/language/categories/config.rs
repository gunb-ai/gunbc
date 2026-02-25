//! ConfigFormat SubDag: Category for configuration file formats.
//!
//! Configuration formats (Makefile, gitignore, YAML, TOML) share:
//! - Comment syntax (usually #)
//! - No type system (or very limited)
//! - Declarative structure
//!
//! # Composes
//!
//! - CommentPrefix SubDag

use crate::dag::{Dag, Port};
use crate::language::LanguageOp;
use crate::node::Node;

/// Build the ConfigFormat category SubDag node.
///
/// This SubDag composes CommentPrefix, providing shared functionality
/// for configuration file formats.
///
/// # Example
///
/// ```text
/// let cf_node = build_config_format_subdag();
/// // Formats like Makefile, gitignore compose this SubDag
/// ```
pub fn build_config_format_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // Configuration node for ConfigFormat category
    inner.add_node(Node::opaque(
        "config",
        vec![Port::scalar("format_id", "String")],
        vec![
            Port::scalar("comment_prefix", "String"),
            Port::scalar("is_declarative", "Bool"),
        ],
        LanguageOp::ConfigFormatConfig,
    ));

    // Comment prefix node (would compose CommentPrefix in full impl)
    inner.add_node(Node::opaque(
        "add_comment",
        vec![
            Port::scalar("content", "String"),
            Port::scalar("prefix", "String"),
        ],
        vec![Port::scalar("commented", "String")],
        LanguageOp::AddComment,
    ));

    // Create the SubDag node with interface
    Node::subdag("config_format", inner)
}

/// Get the comment prefix for a config format.
#[cfg(test)]
pub fn config_comment_prefix(format_id: &str) -> &'static str {
    match format_id {
        "makefile" | "gitignore" | "yaml" | "toml" | "shell" | "dockerfile" => "#",
        _ => "#", // Default to hash comments for config formats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_config_format_subdag_is_subdag() {
        let node = build_config_format_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "config_format");
    }

    #[test]
    fn test_config_format_subdag_interface() {
        let node = build_config_format_subdag();

        // Check inputs
        assert!(node.inputs.iter().any(|p| p.name.0 == "format_id"));
        assert!(node.inputs.iter().any(|p| p.name.0 == "content"));

        // Check outputs
        assert!(node.outputs.iter().any(|p| p.name.0 == "comment_prefix"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "is_declarative"));
    }

    #[test]
    fn test_config_format_subdag_structure() {
        let node = build_config_format_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 2);

                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_ids.contains(&"config"));
                assert!(node_ids.contains(&"add_comment"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_config_comment_prefix() {
        assert_eq!(config_comment_prefix("makefile"), "#");
        assert_eq!(config_comment_prefix("gitignore"), "#");
        assert_eq!(config_comment_prefix("yaml"), "#");
    }
}
