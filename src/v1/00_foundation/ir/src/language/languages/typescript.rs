//! TypeScript SubDag: TypeScript/JavaScript programming language definition.
//!
//! # Composes
//!
//! - TuringComplete (via category)
//! - TypeSystemMapping
//! - NamingConventions
//!
//! # Configuration
//!
//! - File extensions: `.ts`, `.tsx`
//! - Comment prefix: `//`
//! - Types: string, number, boolean, Uint8Array, T[], T | undefined, Record<K, V>
//! - Naming: camelCase functions, PascalCase types, kebab-case modules

use crate::dag::{Dag, Port};
use crate::language::traits::naming::LanguageNaming;
use crate::language::traits::type_system::{PrimitiveTypeMapping, TypeMapping};
use crate::language::LanguageOp;
use crate::language::NamingCase;
use crate::node::Node;

/// TypeScript language static configuration.
pub struct TypeScriptConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub file_extensions: &'static [&'static str],
    pub comment_prefix: &'static str,
    pub block_comment_open: &'static str,
    pub block_comment_close: &'static str,
}

/// Static TypeScript configuration.
pub const TYPESCRIPT: TypeScriptConfig = TypeScriptConfig {
    id: "typescript",
    name: "TypeScript",
    file_extensions: &[".ts", ".tsx"],
    comment_prefix: "//",
    block_comment_open: "/*",
    block_comment_close: "*/",
};

/// TypeScript type mappings (shared with JavaScript).
const TYPESCRIPT_PRIMITIVE_MAPPINGS: &[PrimitiveTypeMapping] = &[
    PrimitiveTypeMapping {
        builtin_type: "String",
        concrete_type: "string",
    },
    PrimitiveTypeMapping {
        builtin_type: "Int",
        concrete_type: "number",
    },
    PrimitiveTypeMapping {
        builtin_type: "Float",
        concrete_type: "number",
    },
    PrimitiveTypeMapping {
        builtin_type: "Bool",
        concrete_type: "boolean",
    },
    PrimitiveTypeMapping {
        builtin_type: "Bytes",
        concrete_type: "Uint8Array",
    },
    PrimitiveTypeMapping {
        builtin_type: "Json",
        concrete_type: "unknown",
    },
];

pub const TYPESCRIPT_TYPES: TypeMapping = TypeMapping {
    primitive_mappings: TYPESCRIPT_PRIMITIVE_MAPPINGS,
    list_template: "{0}[]",
    optional_template: "{0} | undefined",
    map_template: "Record<{0}, {1}>",
};

/// TypeScript naming conventions (shared with JavaScript).
pub const TYPESCRIPT_NAMING: LanguageNaming = LanguageNaming {
    type_case: NamingCase::PascalCase,
    function_case: NamingCase::CamelCase,
    variable_case: NamingCase::CamelCase,
    constant_case: NamingCase::ScreamingSnakeCase,
    module_case: NamingCase::KebabCase,
};

/// Build the TypeScript language SubDag node.
pub fn build_typescript_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

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
        ],
        LanguageOp::TypeScriptConfig,
    ));

    Node::subdag("typescript", inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_typescript_subdag_is_subdag() {
        let node = build_typescript_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "typescript");
    }

    #[test]
    fn test_typescript_subdag_structure() {
        let node = build_typescript_subdag();

        match &node.body {
            NodeBody::SubDag(dag, _) => {
                assert_eq!(dag.nodes.len(), 1);
                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_ids.contains(&"config"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_typescript_config() {
        assert_eq!(TYPESCRIPT.id, "typescript");
        assert_eq!(TYPESCRIPT.file_extensions, &[".ts", ".tsx"]);
        assert_eq!(TYPESCRIPT.comment_prefix, "//");
    }
}
