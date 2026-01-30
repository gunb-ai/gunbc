//! CSS SubDag: CSS stylesheet language definition.
//!
//! # Composes
//!
//! - ConfigFormat (via category)
//!
//! # Configuration
//!
//! - File extension: `.css`
//! - Comment: `/* ... */` (block comments only)

use crate::dag::{Dag, Port};
use crate::language::LanguageOp;
use crate::node::Node;

/// CSS language static configuration.
pub struct CssConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub file_extensions: &'static [&'static str],
    pub comment_open: &'static str,
    pub comment_close: &'static str,
}

/// Static CSS configuration.
pub const CSS: CssConfig = CssConfig {
    id: "css",
    name: "CSS",
    file_extensions: &[".css"],
    comment_open: "/* ",
    comment_close: " */",
};

/// Build the CSS language SubDag node.
pub fn build_css_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // CSS configuration node
    inner.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::scalar("name", "String"),
            Port::scalar("extensions", "StrList"),
            Port::scalar("comment_open", "String"),
            Port::scalar("comment_close", "String"),
        ],
        LanguageOp::CssConfig,
    ));

    Node::subdag(
        "css",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::scalar("extensions", "StrList"),
            Port::scalar("comment_open", "String"),
            Port::scalar("comment_close", "String"),
        ],
        inner,
    )
}

/// Generate CSS comment.
pub fn css_comment(text: &str) -> String {
    format!("{}{}{}", CSS.comment_open, text, CSS.comment_close)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_css_subdag_is_subdag() {
        let node = build_css_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "css");
    }

    #[test]
    fn test_css_subdag_structure() {
        let node = build_css_subdag();

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
    fn test_css_config() {
        assert_eq!(CSS.id, "css");
        assert_eq!(CSS.file_extensions, &[".css"]);
    }

    #[test]
    fn test_css_comment() {
        assert_eq!(css_comment("test"), "/* test */");
    }
}
