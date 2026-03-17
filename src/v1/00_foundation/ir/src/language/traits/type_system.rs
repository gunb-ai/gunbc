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
use crate::language::{language_metadata_for, LanguageOp};
use crate::node::Node;
use crate::types::BuiltinType;

/// A single builtin primitive -> target-language type mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimitiveTypeMapping {
    pub builtin_type: &'static str,
    pub concrete_type: &'static str,
}

/// Type mappings for a specific language.
/// Aligned with `gunbai-integrations-contracts::TypeSystemMapping` from
/// the-gunbai for cross-repo compatibility (F2.3).
#[derive(Debug, Clone)]
pub struct TypeMapping {
    /// Primitive spellings keyed by builtin names from `std.types`.
    pub primitive_mappings: &'static [PrimitiveTypeMapping],
    pub list_template: &'static str,
    pub optional_template: &'static str,
    pub map_template: &'static str,
}

impl TypeMapping {
    fn primitive_mapping(&self, builtin_type: &str) -> Option<&'static str> {
        self.primitive_mappings
            .iter()
            .find(|mapping| mapping.builtin_type == builtin_type)
            .map(|mapping| mapping.concrete_type)
    }
}

/// Build the TypeSystemMapping SubDag node.
///
/// This SubDag maps abstract types to language-specific representations.
///
/// # Example
///
/// ```text
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
    let mapping = language_metadata_for(language).and_then(|m| m.type_mapping)?;

    if let Some(builtin) = BuiltinType::lookup(abstract_type) {
        if builtin.supports_target_language_primitive_mapping() {
            return mapping
                .primitive_mapping(builtin.name)
                .map(|concrete_type| concrete_type.to_string());
        }
    }

    let result = match abstract_type {
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
    language_metadata_for(language)
        .and_then(|m| m.type_mapping)
        .map(|m| m.optional_template)
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
            map_type("Json", "rust"),
            Some("serde_json::Value".to_string())
        );
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
        assert_eq!(map_type("Json", "python"), Some("Any".to_string()));
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
        assert_eq!(map_type("Json", "typescript"), Some("unknown".to_string()));
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

    #[test]
    fn language_mappings_cover_all_builtin_target_primitives() {
        let expected: std::collections::BTreeSet<_> = BuiltinType::all()
            .iter()
            .filter(|builtin| builtin.supports_target_language_primitive_mapping())
            .map(|builtin| builtin.name)
            .collect();

        for (language, mapping) in [
            ("rust", &crate::language::RUST_TYPES),
            ("python", &crate::language::PYTHON_TYPES),
            ("typescript", &crate::language::TYPESCRIPT_TYPES),
        ] {
            let actual: std::collections::BTreeSet<_> = mapping
                .primitive_mappings
                .iter()
                .map(|entry| entry.builtin_type)
                .collect();
            assert_eq!(
                actual, expected,
                "{language} primitive mappings drifted from builtin type authority"
            );
        }
    }
}
