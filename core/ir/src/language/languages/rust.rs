//! Rust SubDag: Rust programming language definition.
//!
//! # Composes
//!
//! - TuringComplete (via category)
//! - TypeSystemMapping
//! - NamingConventions
//!
//! # Configuration
//!
//! - File extension: `.rs`
//! - Comment prefix: `//`
//! - Doc comment prefix: `///`
//! - Types: String, i64, bool, Vec<T>, Option<T>, HashMap<K, V>
//! - Naming: snake_case functions, PascalCase types

use crate::dag::{Dag, Port};
use crate::node::Node;
use crate::language::LanguageOp;

/// Rust language static configuration.
pub struct RustConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub file_extensions: &'static [&'static str],
    pub comment_prefix: &'static str,
    pub doc_comment_prefix: &'static str,
    pub statement_terminator: &'static str,
    pub block_open: &'static str,
    pub block_close: &'static str,
}

/// Static Rust configuration.
pub const RUST: RustConfig = RustConfig {
    id: "rust",
    name: "Rust",
    file_extensions: &[".rs"],
    comment_prefix: "//",
    doc_comment_prefix: "///",
    statement_terminator: ";",
    block_open: "{",
    block_close: "}",
};

/// Build the Rust language SubDag node.
///
/// This SubDag composes TuringComplete category and provides
/// Rust-specific type mapping and naming conventions.
///
/// # I/O Contract
///
/// Inputs:
/// - `abstract_type`: String (optional) - Type to map
/// - `name`: String (optional) - Name to convert
/// - `context`: String (optional) - Naming context (type, function, etc.)
///
/// Outputs:
/// - `id`: String - Language ID ("rust")
/// - `extensions`: StrList - File extensions ([".rs"])
/// - `concrete_type`: String (optional) - Mapped type
/// - `converted_name`: String (optional) - Converted name
pub fn build_rust_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // Rust configuration node
    inner.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::scalar("name", "String"),
            Port::scalar("extensions", "StrList"),
            Port::scalar("comment_prefix", "String"),
            Port::scalar("doc_comment_prefix", "String"),
        ],
        LanguageOp::RustConfig,
    ));

    // Type mapping node (Rust-specific)
    inner.add_node(Node::opaque(
        "type_map",
        vec![Port::scalar("abstract_type", "String")],
        vec![
            Port::scalar("concrete_type", "String"),
            Port::scalar("optional_wrapper", "String"),
        ],
        LanguageOp::RustTypeMap,
    ));

    // Create the SubDag node with interface
    Node::subdag(
        "rust",
        vec![
            Port::optional("abstract_type", "String"),
            Port::optional("name", "String"),
            Port::optional("context", "String"),
        ],
        vec![
            Port::scalar("id", "String"),
            Port::scalar("extensions", "StrList"),
            Port::scalar("comment_prefix", "String"),
            Port::optional("concrete_type", "String"),
            Port::optional("converted_name", "String"),
        ],
        inner,
    )
}

/// Map an abstract type to Rust type.
pub fn rust_type(abstract_type: &str) -> String {
    match abstract_type {
        "String" => "String".to_string(),
        "Int" => "i64".to_string(),
        "Float" => "f64".to_string(),
        "Bool" => "bool".to_string(),
        "Bytes" => "Vec<u8>".to_string(),
        "Json" => "serde_json::Value".to_string(),
        _ if abstract_type.starts_with("List<") => {
            let inner = &abstract_type[5..abstract_type.len() - 1];
            format!("Vec<{}>", rust_type(inner))
        }
        _ if abstract_type.starts_with("Optional<") => {
            let inner = &abstract_type[9..abstract_type.len() - 1];
            format!("Option<{}>", rust_type(inner))
        }
        _ if abstract_type.starts_with("Map<") => {
            // Map<K, V> -> HashMap<K, V>
            let inner = &abstract_type[4..abstract_type.len() - 1];
            if let Some(comma_pos) = inner.find(',') {
                let k = inner[..comma_pos].trim();
                let v = inner[comma_pos + 1..].trim();
                format!("HashMap<{}, {}>", rust_type(k), rust_type(v))
            } else {
                abstract_type.to_string()
            }
        }
        _ => abstract_type.to_string(), // Pass through unknown types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_rust_subdag_is_subdag() {
        let node = build_rust_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "rust");
    }

    #[test]
    fn test_rust_subdag_interface() {
        let node = build_rust_subdag();

        // Check inputs
        assert!(node.inputs.iter().any(|p| p.name.0 == "abstract_type"));
        assert!(node.inputs.iter().any(|p| p.name.0 == "name"));

        // Check outputs
        assert!(node.outputs.iter().any(|p| p.name.0 == "id"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "extensions"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "comment_prefix"));
    }

    #[test]
    fn test_rust_subdag_structure() {
        let node = build_rust_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 2);

                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_ids.contains(&"config"));
                assert!(node_ids.contains(&"type_map"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_rust_type_primitives() {
        assert_eq!(rust_type("String"), "String");
        assert_eq!(rust_type("Int"), "i64");
        assert_eq!(rust_type("Float"), "f64");
        assert_eq!(rust_type("Bool"), "bool");
        assert_eq!(rust_type("Bytes"), "Vec<u8>");
    }

    #[test]
    fn test_rust_type_collections() {
        assert_eq!(rust_type("List<String>"), "Vec<String>");
        assert_eq!(rust_type("List<Int>"), "Vec<i64>");
        assert_eq!(rust_type("Optional<String>"), "Option<String>");
        assert_eq!(rust_type("Optional<Int>"), "Option<i64>");
    }

    #[test]
    fn test_rust_config() {
        assert_eq!(RUST.id, "rust");
        assert_eq!(RUST.file_extensions, &[".rs"]);
        assert_eq!(RUST.comment_prefix, "//");
    }
}
