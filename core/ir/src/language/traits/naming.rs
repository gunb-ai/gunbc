//! NamingConventions SubDag: Convert between naming cases.
//!
//! # I/O Contract
//!
//! Inputs:
//! - `name`: String - Input name to convert
//! - `source_case`: String - Source naming case (optional, auto-detected)
//! - `target_case`: String - Target naming case
//!
//! Outputs:
//! - `converted`: String - Name in target case

// Allow dead_code for infrastructure APIs provided for future use
#![allow(dead_code)]

use crate::dag::{Dag, Port};
use crate::language::{LanguageOp, NamingCase};
use crate::node::Node;

/// Naming conventions for a language.
#[derive(Debug, Clone)]
pub struct LanguageNaming {
    pub type_case: NamingCase,
    pub function_case: NamingCase,
    pub variable_case: NamingCase,
    pub constant_case: NamingCase,
    pub module_case: NamingCase,
}

/// Rust naming conventions.
pub const RUST_NAMING: LanguageNaming = LanguageNaming {
    type_case: NamingCase::PascalCase,
    function_case: NamingCase::SnakeCase,
    variable_case: NamingCase::SnakeCase,
    constant_case: NamingCase::ScreamingSnakeCase,
    module_case: NamingCase::SnakeCase,
};

/// Python naming conventions.
pub const PYTHON_NAMING: LanguageNaming = LanguageNaming {
    type_case: NamingCase::PascalCase,
    function_case: NamingCase::SnakeCase,
    variable_case: NamingCase::SnakeCase,
    constant_case: NamingCase::ScreamingSnakeCase,
    module_case: NamingCase::SnakeCase,
};

/// TypeScript/JavaScript naming conventions.
pub const TYPESCRIPT_NAMING: LanguageNaming = LanguageNaming {
    type_case: NamingCase::PascalCase,
    function_case: NamingCase::CamelCase,
    variable_case: NamingCase::CamelCase,
    constant_case: NamingCase::ScreamingSnakeCase,
    module_case: NamingCase::KebabCase,
};

/// Build the NamingConventions SubDag node.
///
/// This SubDag converts names between different case conventions.
///
/// # Example
///
/// ```ignore
/// let naming_node = build_naming_conventions_subdag();
/// // Execute with name = "my_function", target_case = "PascalCase"
/// // → converted = "MyFunction"
/// ```
pub fn build_naming_conventions_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // Convert case node
    inner.add_node(Node::opaque(
        "convert_case",
        vec![
            Port::scalar("name", "String"),
            Port::scalar("target_case", "String"),
        ],
        vec![Port::scalar("converted", "String")],
        LanguageOp::ConvertCase,
    ));

    // Create the SubDag node with interface
    Node::subdag("naming", inner)
}

/// Get the naming conventions for a language.
pub fn naming_for_language(language: &str) -> Option<&'static LanguageNaming> {
    match language {
        "rust" => Some(&RUST_NAMING),
        "python" => Some(&PYTHON_NAMING),
        "typescript" | "javascript" => Some(&TYPESCRIPT_NAMING),
        _ => None,
    }
}

/// Convert a name to match a language's convention for a specific context.
pub fn convert_for_language(name: &str, language: &str, context: &str) -> Option<String> {
    let naming = naming_for_language(language)?;

    let case = match context {
        "type" => naming.type_case,
        "function" => naming.function_case,
        "variable" => naming.variable_case,
        "constant" => naming.constant_case,
        "module" => naming.module_case,
        _ => return None,
    };

    Some(case.apply(name))
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_naming_subdag_is_subdag() {
        let node = build_naming_conventions_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "naming");
    }

    #[test]
    fn test_naming_subdag_interface() {
        let node = build_naming_conventions_subdag();

        // Check inputs (inferred from inner DAG entrypoints)
        assert_eq!(node.inputs.len(), 2);
        assert!(node.inputs.iter().any(|p| p.name.0 == "name"));
        assert!(node.inputs.iter().any(|p| p.name.0 == "target_case"));

        // Check outputs (inferred from inner DAG boundaries)
        assert_eq!(node.outputs.len(), 1);
        assert!(node.outputs.iter().any(|p| p.name.0 == "converted"));
    }

    #[test]
    fn test_convert_for_language_rust() {
        assert_eq!(
            convert_for_language("my_function", "rust", "type"),
            Some("MyFunction".to_string())
        );
        assert_eq!(
            convert_for_language("MyType", "rust", "function"),
            Some("my_type".to_string())
        );
        assert_eq!(
            convert_for_language("maxValue", "rust", "constant"),
            Some("MAX_VALUE".to_string())
        );
    }

    #[test]
    fn test_convert_for_language_typescript() {
        assert_eq!(
            convert_for_language("my_function", "typescript", "function"),
            Some("myFunction".to_string())
        );
        assert_eq!(
            convert_for_language("my_component", "typescript", "module"),
            Some("my-component".to_string())
        );
    }

    #[test]
    fn test_naming_for_language() {
        let rust = naming_for_language("rust").unwrap();
        assert_eq!(rust.type_case, NamingCase::PascalCase);
        assert_eq!(rust.function_case, NamingCase::SnakeCase);

        let ts = naming_for_language("typescript").unwrap();
        assert_eq!(ts.function_case, NamingCase::CamelCase);
    }
}
