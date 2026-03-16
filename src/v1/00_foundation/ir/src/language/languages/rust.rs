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
use crate::language::traits::naming::LanguageNaming;
use crate::language::traits::type_system::TypeMapping;
use crate::language::LanguageOp;
use crate::language::NamingCase;
use crate::node::Node;

/// Rust language static configuration.
pub struct RustConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub file_extensions: &'static [&'static str],
    pub comment_prefix: &'static str,
    pub block_comment_open: &'static str,
    pub block_comment_close: &'static str,
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
    block_comment_open: "/*",
    block_comment_close: "*/",
    doc_comment_prefix: "///",
    statement_terminator: ";",
    block_open: "{",
    block_close: "}",
};

/// Rust type mappings.
pub const RUST_TYPES: TypeMapping = TypeMapping {
    string: "String",
    int: "i64",
    float: "f64",
    bool: "bool",
    bytes: "Vec<u8>",
    json: "serde_json::Value",
    list_template: "Vec<{0}>",
    optional_template: "Option<{0}>",
    map_template: "HashMap<{0}, {1}>",
};

/// Rust naming conventions.
pub const RUST_NAMING: LanguageNaming = LanguageNaming {
    type_case: NamingCase::PascalCase,
    function_case: NamingCase::SnakeCase,
    variable_case: NamingCase::SnakeCase,
    constant_case: NamingCase::ScreamingSnakeCase,
    module_case: NamingCase::SnakeCase,
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
/// - `extensions`: List - File extensions ([".rs"])
/// - `block_comment_open`: String - Block comment start ("/*")
/// - `block_comment_close`: String - Block comment end ("*/")
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
            Port::list("extensions", "List<String>"),
            Port::scalar("comment_prefix", "String"),
            Port::scalar("block_comment_open", "String"),
            Port::scalar("block_comment_close", "String"),
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
    Node::subdag("rust", inner)
}

/// Map an abstract type to Rust type.
///
/// Delegates to `map_type(_, "rust")` which reads from `RUST_TYPES`.
pub fn rust_type(abstract_type: &str) -> String {
    crate::language::map_type(abstract_type, "rust").unwrap_or_else(|| abstract_type.to_string())
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

        // Check inputs (inferred from inner DAG entrypoints)
        assert!(node.inputs.iter().any(|p| p.name.0 == "abstract_type"));

        // Check outputs (inferred from inner DAG boundaries)
        assert!(node.outputs.iter().any(|p| p.name.0 == "id"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "extensions"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "comment_prefix"));
        assert!(node
            .outputs
            .iter()
            .any(|p| p.name.0 == "block_comment_open"));
        assert!(node
            .outputs
            .iter()
            .any(|p| p.name.0 == "block_comment_close"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "concrete_type"));
    }

    #[test]
    fn test_rust_subdag_structure() {
        let node = build_rust_subdag();

        match &node.body {
            NodeBody::SubDag(dag, _) => {
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
        assert_eq!(rust_type("Json"), "serde_json::Value");
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
        assert_eq!(RUST.block_comment_open, "/*");
        assert_eq!(RUST.block_comment_close, "*/");
    }
}
