//! TOML SubDag: TOML configuration format definition.
//!
//! # Composes
//!
//! - ConfigFormat (via category)
//!
//! # Configuration
//!
//! - File extension: `.toml`
//! - Comment prefix: `#`

use crate::dag::{Dag, Port};
use crate::language::LanguageOp;
use crate::node::Node;

/// TOML format static configuration.
pub struct TomlConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub file_extensions: &'static [&'static str],
    pub comment_prefix: &'static str,
}

/// Static TOML configuration.
pub const TOML: TomlConfig = TomlConfig {
    id: "toml",
    name: "TOML",
    file_extensions: &[".toml"],
    comment_prefix: "#",
};

/// Build the TOML language SubDag node.
pub fn build_toml_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // TOML configuration node
    inner.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::scalar("name", "String"),
            Port::scalar("extensions", "List"),
            Port::scalar("comment_prefix", "String"),
        ],
        LanguageOp::TomlConfig,
    ));

    Node::subdag(
        "toml",
        inner,
    )
}

/// Generate TOML comment.
pub fn toml_comment(text: &str) -> String {
    format!("{} {}", TOML.comment_prefix, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_toml_subdag_is_subdag() {
        let node = build_toml_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "toml");
    }

    #[test]
    fn test_toml_subdag_structure() {
        let node = build_toml_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 1);
                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_ids.contains(&"config"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_toml_config() {
        assert_eq!(TOML.id, "toml");
        assert_eq!(TOML.file_extensions, &[".toml"]);
        assert_eq!(TOML.comment_prefix, "#");
    }

    #[test]
    fn test_toml_comment() {
        assert_eq!(toml_comment("test"), "# test");
    }
}
