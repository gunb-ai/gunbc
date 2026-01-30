//! YAML SubDag: YAML configuration format definition.
//!
//! # Composes
//!
//! - ConfigFormat (via category)
//!
//! # Configuration
//!
//! - File extensions: `.yaml`, `.yml`
//! - Comment prefix: `#`

use crate::dag::{Dag, Port};
use crate::language::LanguageOp;
use crate::node::Node;

/// YAML format static configuration.
pub struct YamlConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub file_extensions: &'static [&'static str],
    pub comment_prefix: &'static str,
}

/// Static YAML configuration.
pub const YAML: YamlConfig = YamlConfig {
    id: "yaml",
    name: "YAML",
    file_extensions: &[".yaml", ".yml"],
    comment_prefix: "#",
};

/// Build the YAML language SubDag node.
pub fn build_yaml_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // YAML configuration node
    inner.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::scalar("name", "String"),
            Port::scalar("extensions", "StrList"),
            Port::scalar("comment_prefix", "String"),
        ],
        LanguageOp::YamlConfig,
    ));

    Node::subdag(
        "yaml",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::scalar("extensions", "StrList"),
            Port::scalar("comment_prefix", "String"),
        ],
        inner,
    )
}

/// Generate YAML comment.
pub fn yaml_comment(text: &str) -> String {
    format!("{} {}", YAML.comment_prefix, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_yaml_subdag_is_subdag() {
        let node = build_yaml_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "yaml");
    }

    #[test]
    fn test_yaml_subdag_structure() {
        let node = build_yaml_subdag();

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
    fn test_yaml_config() {
        assert_eq!(YAML.id, "yaml");
        assert_eq!(YAML.file_extensions, &[".yaml", ".yml"]);
        assert_eq!(YAML.comment_prefix, "#");
    }

    #[test]
    fn test_yaml_comment() {
        assert_eq!(yaml_comment("test"), "# test");
    }
}
