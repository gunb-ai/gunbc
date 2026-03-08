//! Gitignore SubDag: .gitignore file format definition.
//!
//! # Composes
//!
//! - ConfigFormat (via category)
//! - GlobPatterns (for pattern matching)
//! - Regex (underlying pattern engine)
//!
//! # Configuration
//!
//! - File patterns: `.gitignore`, `.dockerignore`
//! - Comment prefix: `#`
//! - Pattern syntax: glob with negation (!)

use crate::dag::{Dag, Port};
use crate::language::LanguageOp;
use crate::node::Node;

/// Default gitignore filename - the canonical name for generated .gitignore files.
pub const DEFAULT_GITIGNORE_FILENAME: &str = ".gitignore";

/// Gitignore format static configuration.
pub struct GitignoreConfig {
    pub id: &'static str,
    /// The default filename for generated gitignore files.
    pub default_filename: &'static str,
    /// All file patterns that identify gitignore files.
    pub file_patterns: &'static [&'static str],
    pub comment_prefix: &'static str,
}

/// Static gitignore configuration.
pub const GITIGNORE: GitignoreConfig = GitignoreConfig {
    id: "gitignore",
    default_filename: DEFAULT_GITIGNORE_FILENAME,
    file_patterns: &[".gitignore", ".dockerignore"],
    comment_prefix: "#",
};

/// Build the Gitignore format SubDag node.
///
/// This SubDag composes ConfigFormat category and GlobPatterns,
/// providing gitignore-specific pattern handling.
///
/// # I/O Contract
///
/// Inputs:
/// - `patterns`: List - Gitignore patterns to render
/// - `files`: List (optional) - Files to check for ignore
///
/// Outputs:
/// - `id`: String - Format ID ("gitignore")
/// - `content`: String - Rendered .gitignore content
/// - `ignored`: List (optional) - Files that would be ignored
pub fn build_gitignore_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // Gitignore configuration node
    inner.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::list("file_patterns", "StringList"),
            Port::scalar("comment_prefix", "String"),
        ],
        LanguageOp::GitignoreConfig,
    ));

    // Render patterns to content
    inner.add_node(Node::opaque(
        "render",
        vec![Port::list("patterns", "StringList")],
        vec![Port::scalar("content", "String")],
        LanguageOp::GitignoreRender,
    ));

    // Create the SubDag node with interface
    Node::subdag("gitignore", inner)
}

/// Render gitignore patterns to file content.
#[cfg(test)]
pub fn render_gitignore_content(patterns: &[String]) -> String {
    patterns.join("\n")
}

/// Render gitignore content with sections.
#[cfg(test)]
pub fn render_gitignore_with_sections(sections: &[(String, Vec<String>)]) -> String {
    use std::fmt::Write as _;

    let mut content = String::new();

    for (section_name, patterns) in sections {
        if !content.is_empty() {
            content.push_str("\n\n");
        }
        writeln!(content, "# {}", section_name).unwrap();
        for pattern in patterns {
            content.push_str(pattern);
            content.push('\n');
        }
    }

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_gitignore_subdag_is_subdag() {
        let node = build_gitignore_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "gitignore");
    }

    #[test]
    fn test_gitignore_subdag_interface() {
        let node = build_gitignore_subdag();

        // Check inputs (inferred from inner DAG entrypoints)
        assert!(node.inputs.iter().any(|p| p.name.0 == "patterns"));

        // Check outputs (inferred from inner DAG boundaries)
        assert!(node.outputs.iter().any(|p| p.name.0 == "id"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "content"));
    }

    #[test]
    fn test_gitignore_subdag_structure() {
        let node = build_gitignore_subdag();

        match &node.body {
            NodeBody::SubDag(dag, _) => {
                assert_eq!(dag.nodes.len(), 2);

                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_ids.contains(&"config"));
                assert!(node_ids.contains(&"render"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_render_gitignore_content() {
        let patterns = vec!["*.rs".to_string(), "target/".to_string()];
        assert_eq!(render_gitignore_content(&patterns), "*.rs\ntarget/");
    }

    #[test]
    fn test_render_gitignore_with_sections() {
        let sections = vec![
            ("Build artifacts".to_string(), vec!["target/".to_string()]),
            ("IDE".to_string(), vec![".idea/".to_string()]),
        ];
        let content = render_gitignore_with_sections(&sections);
        assert!(content.contains("# Build artifacts"));
        assert!(content.contains("target/"));
        assert!(content.contains("# IDE"));
        assert!(content.contains(".idea/"));
    }

    #[test]
    fn test_gitignore_config() {
        assert_eq!(GITIGNORE.id, "gitignore");
        assert_eq!(GITIGNORE.comment_prefix, "#");
        assert!(GITIGNORE.file_patterns.contains(&".gitignore"));
    }
}
