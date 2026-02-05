//! Regex SubDag: Pattern matching and validation.
//!
//! Consolidates all regex usage in the codebase into a single SubDag.
//!
//! # I/O Contract
//!
//! Inputs:
//! - `pattern`: String - The regex pattern
//! - `text`: String (optional) - Text to match against
//!
//! Outputs:
//! - `valid`: Bool - Is the pattern valid?
//! - `matches`: List (optional) - Matched substrings

use crate::dag::{Dag, Port};
use crate::language::LanguageOp;
use crate::node::Node;

/// Build the Regex SubDag node.
///
/// This SubDag provides:
/// - Pattern validation (is the regex syntactically correct?)
/// - Pattern matching (find matches in text)
///
/// # Example
///
/// ```ignore
/// let regex_node = build_regex_subdag();
/// // Execute with pattern = "\\d+" → valid = true
/// // Execute with pattern = "\\d+", text = "abc123def" → matches = ["123"]
/// ```
pub fn build_regex_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // Validate node: checks if pattern is valid regex
    inner.add_node(Node::opaque(
        "validate",
        vec![Port::scalar("pattern", "String")],
        vec![Port::scalar("valid", "Bool")],
        LanguageOp::RegexValidate,
    ));

    // Match node: finds matches in text
    inner.add_node(Node::opaque(
        "match",
        vec![
            Port::scalar("pattern", "String"),
            Port::scalar("text", "String"),
        ],
        vec![Port::list("matches", "String")],
        LanguageOp::RegexMatch,
    ));

    // Create the SubDag node with interface
    Node::subdag("regex", inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_regex_subdag_is_subdag() {
        let node = build_regex_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "regex");
    }

    #[test]
    fn test_regex_subdag_interface() {
        let node = build_regex_subdag();

        // Check inputs (inferred from inner DAG entrypoints)
        assert!(node.inputs.iter().any(|p| p.name.0 == "pattern"));
        assert!(node.inputs.iter().any(|p| p.name.0 == "text"));

        // Check outputs (inferred from inner DAG boundaries)
        assert!(node.outputs.iter().any(|p| p.name.0 == "valid"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "matches"));
    }

    #[test]
    fn test_regex_subdag_structure() {
        let node = build_regex_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 2);

                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_ids.contains(&"validate"));
                assert!(node_ids.contains(&"match"));
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
