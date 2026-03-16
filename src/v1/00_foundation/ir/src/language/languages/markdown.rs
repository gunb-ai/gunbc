//! Markdown SubDag: Markdown document format definition.
//!
//! # Composes
//!
//! - ConfigFormat (via category)
//!
//! # Configuration
//!
//! - File extension: `.md`, `.markdown`
//! - Comment: HTML comments `<!-- ... -->`
//! - Supports fenced code blocks with language hints

use crate::dag::{Dag, Port};
use crate::language::LanguageOp;
use crate::node::Node;

/// Markdown format static configuration.
pub struct MarkdownConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub file_extensions: &'static [&'static str],
    pub comment_open: &'static str,
    pub comment_close: &'static str,
    pub code_fence: &'static str,
}

/// Static Markdown configuration.
pub const MARKDOWN: MarkdownConfig = MarkdownConfig {
    id: "markdown",
    name: "Markdown",
    file_extensions: &[".md", ".markdown"],
    comment_open: "<!-- ",
    comment_close: " -->",
    code_fence: "```",
};

/// Build the Markdown language SubDag node.
pub fn build_markdown_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // Markdown configuration node
    inner.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::scalar("name", "String"),
            Port::list("extensions", "List<String>"),
            Port::scalar("comment_open", "String"),
            Port::scalar("comment_close", "String"),
            Port::scalar("code_fence", "String"),
        ],
        LanguageOp::MarkdownConfig,
    ));

    // Code block rendering node
    inner.add_node(Node::opaque(
        "render_code_block",
        vec![
            Port::scalar("code", "String"),
            Port::optional("language", "Optional<String>"),
        ],
        vec![Port::scalar("block", "String")],
        LanguageOp::MarkdownRenderCodeBlock,
    ));

    Node::subdag("markdown", inner)
}

/// Render a fenced code block.
pub fn render_code_block(code: &str, language: Option<&str>) -> String {
    let lang = language.unwrap_or("");
    let fence = code_fence_for(code);
    format!(
        "{}{}\n{}\n{}",
        fence, lang, code, fence
    )
}

/// Generate Markdown comment.
pub fn markdown_comment(text: &str) -> String {
    format!(
        "{}{}{}",
        MARKDOWN.comment_open, text, MARKDOWN.comment_close
    )
}

fn code_fence_for(code: &str) -> String {
    let base_len = MARKDOWN.code_fence.len();
    let longest_backtick_run = code
        .chars()
        .fold((0usize, 0usize), |(max_run, current_run), ch| {
            if ch == '`' {
                let next_run = current_run + 1;
                (max_run.max(next_run), next_run)
            } else {
                (max_run, 0)
            }
        })
        .0;
    let fence_len = base_len.max(longest_backtick_run + 1);

    "`".repeat(fence_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_markdown_subdag_is_subdag() {
        let node = build_markdown_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "markdown");
    }

    #[test]
    fn test_markdown_subdag_structure() {
        let node = build_markdown_subdag();

        match &node.body {
            NodeBody::SubDag(dag, _) => {
                assert_eq!(dag.nodes.len(), 2);
                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_ids.contains(&"config"));
                assert!(node_ids.contains(&"render_code_block"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_markdown_config() {
        assert_eq!(MARKDOWN.id, "markdown");
        assert_eq!(MARKDOWN.file_extensions, &[".md", ".markdown"]);
    }

    #[test]
    fn test_render_code_block() {
        let block = render_code_block("fn main() {}", Some("rust"));
        assert_eq!(block, "```rust\nfn main() {}\n```");
    }

    #[test]
    fn test_render_code_block_no_language() {
        let block = render_code_block("hello", None);
        assert_eq!(block, "```\nhello\n```");
    }

    #[test]
    fn test_render_code_block_with_embedded_backticks() {
        let block = render_code_block("before\n```\nafter", Some("markdown"));
        assert_eq!(block, "````markdown\nbefore\n```\nafter\n````");
    }
}
