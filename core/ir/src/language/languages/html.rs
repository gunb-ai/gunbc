//! HTML SubDag: HTML markup language definition.
//!
//! # Composes
//!
//! - ConfigFormat (via category)
//!
//! # Configuration
//!
//! - File extension: `.html`, `.htm`
//! - Comment prefix: `<!-- ` (with closing ` -->`
//! - Uses block comments only

use crate::dag::{Dag, Port};
use crate::language::LanguageOp;
use crate::node::Node;

/// HTML language static configuration.
pub struct HtmlConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub file_extensions: &'static [&'static str],
    pub comment_open: &'static str,
    pub comment_close: &'static str,
    pub doctype: &'static str,
}

/// Static HTML configuration.
pub const HTML: HtmlConfig = HtmlConfig {
    id: "html",
    name: "HTML",
    file_extensions: &[".html", ".htm"],
    comment_open: "<!-- ",
    comment_close: " -->",
    doctype: "<!DOCTYPE html>",
};

/// Build the HTML language SubDag node.
pub fn build_html_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // HTML configuration node
    inner.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::scalar("name", "String"),
            Port::scalar("extensions", "List"),
            Port::scalar("comment_open", "String"),
            Port::scalar("comment_close", "String"),
        ],
        LanguageOp::HtmlConfig,
    ));

    // HTML document rendering node
    inner.add_node(Node::opaque(
        "render",
        vec![
            Port::scalar("title", "String"),
            Port::optional("head", "String"),
            Port::scalar("body", "String"),
        ],
        vec![Port::scalar("document", "String")],
        LanguageOp::HtmlRender,
    ));

    Node::subdag(
        "html",
        inner,
    )
}

/// Render a basic HTML document.
pub fn render_html_document(title: &str, head: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
{head}
</head>
<body>
{body}
</body>
</html>
"#,
        title = title,
        head = head,
        body = body
    )
}

/// Generate HTML comment.
pub fn html_comment(text: &str) -> String {
    format!("{}{}{}", HTML.comment_open, text, HTML.comment_close)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_html_subdag_is_subdag() {
        let node = build_html_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "html");
    }

    #[test]
    fn test_html_subdag_structure() {
        let node = build_html_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 2);
                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_ids.contains(&"config"));
                assert!(node_ids.contains(&"render"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_html_config() {
        assert_eq!(HTML.id, "html");
        assert_eq!(HTML.file_extensions, &[".html", ".htm"]);
    }

    #[test]
    fn test_html_comment() {
        assert_eq!(html_comment("test"), "<!-- test -->");
    }

    #[test]
    fn test_render_html_document() {
        let doc = render_html_document("Test", "", "<p>Hello</p>");
        assert!(doc.contains("<!DOCTYPE html>"));
        assert!(doc.contains("<title>Test</title>"));
        assert!(doc.contains("<p>Hello</p>"));
    }
}
