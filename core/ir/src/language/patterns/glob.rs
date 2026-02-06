//! GlobPatterns SubDag: File matching with glob patterns.
//!
//! Composes the Regex SubDag internally to convert globs to regex.
//!
//! # I/O Contract
//!
//! Inputs:
//! - `pattern`: String - Glob pattern (e.g., "*.rs", "**/test/*")
//! - `files`: List - Files to match against
//!
//! Outputs:
//! - `matched`: List - Files matching the pattern
//! - `negated`: Bool - Is this a negation pattern (starts with !)

use crate::dag::{Dag, Port};
use crate::language::LanguageOp;
use crate::node::Node;

/// Build the GlobPatterns SubDag node.
///
/// This SubDag provides glob-style pattern matching for files.
/// Internally, it composes the Regex SubDag (globs are converted to regex).
///
/// # Supported Patterns
///
/// - `*` - Match any characters except path separator
/// - `**` - Match any characters including path separator
/// - `?` - Match single character
/// - `[abc]` - Match character class
/// - `!pattern` - Negation (exclude matching files)
///
/// # Example
///
/// ```ignore
/// let glob_node = build_glob_subdag();
/// // Execute with pattern = "*.rs", files = ["main.rs", "lib.py"]
/// // → matched = ["main.rs"]
/// ```
pub fn build_glob_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // Glob match node: matches files against glob pattern
    inner.add_node(Node::opaque(
        "glob_match",
        vec![
            Port::scalar("pattern", "String"),
            Port::list("files", "String"),
        ],
        vec![
            Port::list("matched", "String"),
            Port::scalar("negated", "Bool"),
        ],
        LanguageOp::GlobMatch,
    ));

    // Create the SubDag node with interface
    Node::subdag("glob", inner)
}

/// Convert a glob pattern to a regex pattern.
///
/// This is used internally by the GlobPatterns SubDag.
pub fn glob_to_regex(glob: &str) -> String {
    let mut regex = String::from("^");
    let mut chars = glob.chars().peekable();
    let mut negated = false;

    // Check for negation
    if glob.starts_with('!') {
        negated = true;
        chars.next();
    }

    while let Some(c) = chars.next() {
        match c {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    // Skip optional path separator after **
                    if chars.peek() == Some(&'/') {
                        chars.next();
                    }
                    regex.push_str(".*");
                } else {
                    regex.push_str("[^/]*");
                }
            }
            '?' => regex.push('.'),
            '.' | '+' | '^' | '$' | '(' | ')' | '{' | '}' | '|' | '\\' => {
                regex.push('\\');
                regex.push(c);
            }
            '[' => {
                regex.push('[');
                // Handle character class contents
                for cc in chars.by_ref() {
                    if cc == ']' {
                        regex.push(']');
                        break;
                    }
                    regex.push(cc);
                }
            }
            _ => regex.push(c),
        }
    }

    regex.push('$');

    if negated {
        // Return negated regex (lookahead)
        format!("^(?!{}).*$", &regex[1..regex.len() - 1])
    } else {
        regex
    }
}

/// Check if a pattern is a negation pattern.
pub fn is_negated(pattern: &str) -> bool {
    pattern.starts_with('!')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_subdag_is_subdag() {
        let node = build_glob_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "glob");
    }

    #[test]
    fn test_glob_subdag_interface() {
        let node = build_glob_subdag();

        // Check inputs (inferred from inner DAG entrypoints)
        assert_eq!(node.inputs.len(), 2);
        assert!(node.inputs.iter().any(|p| p.name.0 == "pattern"));
        assert!(node.inputs.iter().any(|p| p.name.0 == "files"));

        // Check outputs (inferred from inner DAG boundaries)
        assert_eq!(node.outputs.len(), 2);
        assert!(node.outputs.iter().any(|p| p.name.0 == "matched"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "negated"));
    }

    #[test]
    fn test_glob_to_regex_simple() {
        assert_eq!(glob_to_regex("*.rs"), "^[^/]*\\.rs$");
        assert_eq!(glob_to_regex("test_*.py"), "^test_[^/]*\\.py$");
    }

    #[test]
    fn test_glob_to_regex_double_star() {
        assert_eq!(glob_to_regex("**/test/*.rs"), "^.*test/[^/]*\\.rs$");
        assert_eq!(glob_to_regex("src/**/*.rs"), "^src/.*[^/]*\\.rs$");
    }

    #[test]
    fn test_glob_to_regex_question_mark() {
        assert_eq!(glob_to_regex("file?.txt"), "^file.\\.txt$");
    }

    #[test]
    fn test_is_negated() {
        assert!(is_negated("!*.rs"));
        assert!(!is_negated("*.rs"));
    }
}
