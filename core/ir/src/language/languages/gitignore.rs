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

// Allow dead_code for infrastructure APIs provided for future use
#![allow(dead_code)]

use crate::dag::{Dag, Port};
use crate::node::Node;
use crate::language::LanguageOp;

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
            Port::scalar("file_patterns", "List"),
            Port::scalar("comment_prefix", "String"),
        ],
        LanguageOp::GitignoreConfig,
    ));

    // Render patterns to content
    inner.add_node(Node::opaque(
        "render",
        vec![Port::scalar("patterns", "List")],
        vec![Port::scalar("content", "String")],
        LanguageOp::GitignoreRender,
    ));

    // Create the SubDag node with interface
    Node::subdag(
        "gitignore",
        inner,
    )
}

/// Render gitignore patterns to file content.
pub fn render_gitignore_content(patterns: &[String]) -> String {
    patterns.join("\n")
}

/// Render gitignore content with sections.
pub fn render_gitignore_with_sections(sections: &[(String, Vec<String>)]) -> String {
    let mut content = String::new();

    for (section_name, patterns) in sections {
        if !content.is_empty() {
            content.push_str("\n\n");
        }
        content.push_str(&format!("# {}\n", section_name));
        for pattern in patterns {
            content.push_str(pattern);
            content.push('\n');
        }
    }

    content
}

/// Check if a file matches any gitignore pattern.
///
/// Note: This is a simplified implementation. For full glob matching,
/// consider using the `glob` or `globset` crate.
pub fn is_ignored(file: &str, patterns: &[String]) -> bool {
    use crate::language::patterns::glob::is_negated;

    let mut ignored = false;

    for pattern in patterns {
        // Skip comments and empty lines
        if pattern.starts_with('#') || pattern.trim().is_empty() {
            continue;
        }

        let is_neg = is_negated(pattern);
        let pattern_str = if is_neg { &pattern[1..] } else { pattern.as_str() };

        // Simple glob matching (supports * and **)
        if simple_glob_match(pattern_str, file) {
            ignored = !is_neg;
        }
    }

    ignored
}

/// Simple glob matching without regex dependency.
fn simple_glob_match(pattern: &str, file: &str) -> bool {
    // Handle ** (match any path)
    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');
            
            if !prefix.is_empty() && !file.starts_with(prefix) {
                return false;
            }
            if !suffix.is_empty() && !file.ends_with(suffix) {
                return false;
            }
            return true;
        }
    }

    // Handle single * (match within path segment)
    if pattern.contains('*') && !pattern.contains("**") {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            return file.starts_with(parts[0]) && file.ends_with(parts[1]);
        }
    }

    // Exact match or directory match
    file == pattern || file.starts_with(&format!("{}/", pattern))
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
    fn test_render_gitignore_content() {
        let patterns = vec![
            "*.rs".to_string(),
            "target/".to_string(),
        ];
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
