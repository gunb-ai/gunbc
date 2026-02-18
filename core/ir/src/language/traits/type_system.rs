//! TypeSystemMapping SubDag: Map abstract types to language-specific types.
//!
//! # I/O Contract
//!
//! Inputs:
//! - `abstract_type`: String - Abstract type name (e.g., "String", "Int", "List<String>")
//! - `language`: String - Target language ID (e.g., "rust", "python")
//!
//! Outputs:
//! - `concrete_type`: String - Language-specific type (e.g., "String", "str", "string")
//! - `optional_wrapper`: String - How to wrap optional types (e.g., "Option<{0}>")

use crate::dag::{Dag, Port};
use crate::language::LanguageOp;
use crate::node::Node;

/// Type mappings for a specific language.
/// Aligned with `gunbai-integrations-contracts::TypeSystemMapping` from
/// the-gunbai for cross-repo compatibility (F2.3).
#[derive(Debug, Clone)]
pub struct TypeMapping {
    pub string: &'static str,
    pub int: &'static str,
    pub float: &'static str,
    pub bool: &'static str,
    pub bytes: &'static str,
    pub list_template: &'static str,
    pub optional_template: &'static str,
    pub map_template: &'static str,
}

/// Rust type mappings.
pub const RUST_TYPES: TypeMapping = TypeMapping {
    string: "String",
    int: "i64",
    float: "f64",
    bool: "bool",
    bytes: "Vec<u8>",
    list_template: "Vec<{0}>",
    optional_template: "Option<{0}>",
    map_template: "HashMap<{0}, {1}>",
};

/// Python type mappings.
pub const PYTHON_TYPES: TypeMapping = TypeMapping {
    string: "str",
    int: "int",
    float: "float",
    bool: "bool",
    bytes: "bytes",
    list_template: "list[{0}]",
    optional_template: "{0} | None",
    map_template: "dict[{0}, {1}]",
};

/// TypeScript type mappings.
pub const TYPESCRIPT_TYPES: TypeMapping = TypeMapping {
    string: "string",
    int: "number",
    float: "number",
    bool: "boolean",
    bytes: "Uint8Array",
    list_template: "{0}[]",
    optional_template: "{0} | undefined",
    map_template: "Record<{0}, {1}>",
};

/// Build the TypeSystemMapping SubDag node.
///
/// This SubDag maps abstract types to language-specific representations.
///
/// # Example
///
/// ```ignore
/// let tsm_node = build_type_system_mapping_subdag();
/// // Execute with abstract_type = "String", language = "rust"
/// // → concrete_type = "String"
/// // Execute with abstract_type = "String", language = "python"
/// // → concrete_type = "str"
/// ```
pub fn build_type_system_mapping_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // Map type node
    inner.add_node(Node::opaque(
        "map_type",
        vec![
            Port::scalar("abstract_type", "String"),
            Port::scalar("language", "String"),
        ],
        vec![
            Port::scalar("concrete_type", "String"),
            Port::scalar("optional_wrapper", "String"),
        ],
        LanguageOp::MapType,
    ));

    // Create the SubDag node with interface
    Node::subdag("type_system", inner)
}

/// Map an abstract type to a language-specific type.
pub fn map_type(abstract_type: &str, language: &str) -> Option<String> {
    let mapping = match language {
        "rust" => &RUST_TYPES,
        "python" => &PYTHON_TYPES,
        "typescript" | "javascript" => &TYPESCRIPT_TYPES,
        _ => return None,
    };

    let result = match abstract_type {
        "String" => mapping.string.to_string(),
        "Int" => mapping.int.to_string(),
        "Float" => mapping.float.to_string(),
        "Bool" => mapping.bool.to_string(),
        "Bytes" => mapping.bytes.to_string(),
        _ if abstract_type.starts_with("List<") => {
            let inner = &abstract_type[5..abstract_type.len() - 1];
            let inner_type = map_type(inner, language)?;
            mapping.list_template.replace("{0}", &inner_type)
        }
        _ if abstract_type.starts_with("Optional<") => {
            let inner = &abstract_type[9..abstract_type.len() - 1];
            let inner_type = map_type(inner, language)?;
            mapping.optional_template.replace("{0}", &inner_type)
        }
        _ if abstract_type.starts_with("Map<") => {
            let inner = &abstract_type[4..abstract_type.len() - 1];
            if let Some(comma_pos) = inner.find(',') {
                let key = inner[..comma_pos].trim();
                let val = inner[comma_pos + 1..].trim();
                let key_type = map_type(key, language)?;
                let val_type = map_type(val, language)?;
                mapping
                    .map_template
                    .replace("{0}", &key_type)
                    .replace("{1}", &val_type)
            } else {
                abstract_type.to_string()
            }
        }
        _ => abstract_type.to_string(), // Pass through unknown types
    };

    Some(result)
}

/// Get the optional wrapper template for a language.
pub fn optional_wrapper(language: &str) -> Option<&'static str> {
    match language {
        "rust" => Some(RUST_TYPES.optional_template),
        "python" => Some(PYTHON_TYPES.optional_template),
        "typescript" | "javascript" => Some(TYPESCRIPT_TYPES.optional_template),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_system_subdag_is_subdag() {
        let node = build_type_system_mapping_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "type_system");
    }

    #[test]
    fn test_type_system_subdag_interface() {
        let node = build_type_system_mapping_subdag();

        // Check inputs
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.inputs[0].name.0, "abstract_type");
        assert_eq!(node.inputs[1].name.0, "language");

        // Check outputs
        assert_eq!(node.outputs.len(), 2);
        assert_eq!(node.outputs[0].name.0, "concrete_type");
        assert_eq!(node.outputs[1].name.0, "optional_wrapper");
    }

    #[test]
    fn test_map_type_rust() {
        assert_eq!(map_type("String", "rust"), Some("String".to_string()));
        assert_eq!(map_type("Int", "rust"), Some("i64".to_string()));
        assert_eq!(map_type("Bool", "rust"), Some("bool".to_string()));
        assert_eq!(
            map_type("List<String>", "rust"),
            Some("Vec<String>".to_string())
        );
        assert_eq!(
            map_type("Optional<Int>", "rust"),
            Some("Option<i64>".to_string())
        );
    }

    #[test]
    fn test_map_type_python() {
        assert_eq!(map_type("String", "python"), Some("str".to_string()));
        assert_eq!(map_type("Int", "python"), Some("int".to_string()));
        assert_eq!(map_type("Bool", "python"), Some("bool".to_string()));
        assert_eq!(
            map_type("List<String>", "python"),
            Some("list[str]".to_string())
        );
        assert_eq!(
            map_type("Optional<Int>", "python"),
            Some("int | None".to_string())
        );
    }

    #[test]
    fn test_map_type_typescript() {
        assert_eq!(map_type("String", "typescript"), Some("string".to_string()));
        assert_eq!(map_type("Int", "typescript"), Some("number".to_string()));
        assert_eq!(
            map_type("List<String>", "typescript"),
            Some("string[]".to_string())
        );
    }

    #[test]
    fn test_optional_wrapper() {
        assert_eq!(optional_wrapper("rust"), Some("Option<{0}>"));
        assert_eq!(optional_wrapper("python"), Some("{0} | None"));
        assert_eq!(optional_wrapper("typescript"), Some("{0} | undefined"));
    }
}
