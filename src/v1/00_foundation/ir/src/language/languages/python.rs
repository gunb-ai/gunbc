//! Python SubDag: Python programming language definition.
//!
//! # Composes
//!
//! - TuringComplete (via category)
//! - TypeSystemMapping
//! - NamingConventions
//!
//! # Configuration
//!
//! - File extension: `.py`
//! - Comment prefix: `#`
//! - Types: str, int, float, bool, bytes, list[T], T | None, dict[K, V]
//! - Naming: snake_case functions, PascalCase types

use crate::dag::{Dag, Port};
use crate::language::traits::naming::LanguageNaming;
use crate::language::traits::type_system::TypeMapping;
use crate::language::LanguageOp;
use crate::language::NamingCase;
use crate::node::Node;

/// Python language static configuration.
pub struct PythonConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub file_extensions: &'static [&'static str],
    pub comment_prefix: &'static str,
}

/// Static Python configuration.
pub const PYTHON: PythonConfig = PythonConfig {
    id: "python",
    name: "Python",
    file_extensions: &[".py"],
    comment_prefix: "#",
};

/// Python type mappings.
pub const PYTHON_TYPES: TypeMapping = TypeMapping {
    string: "str",
    int: "int",
    float: "float",
    bool: "bool",
    bytes: "bytes",
    json: "Any",
    list_template: "list[{0}]",
    optional_template: "{0} | None",
    map_template: "dict[{0}, {1}]",
};

/// Python naming conventions.
pub const PYTHON_NAMING: LanguageNaming = LanguageNaming {
    type_case: NamingCase::PascalCase,
    function_case: NamingCase::SnakeCase,
    variable_case: NamingCase::SnakeCase,
    constant_case: NamingCase::ScreamingSnakeCase,
    module_case: NamingCase::SnakeCase,
};

/// Build the Python language SubDag node.
pub fn build_python_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    inner.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::scalar("name", "String"),
            Port::list("extensions", "List<String>"),
            Port::scalar("comment_prefix", "String"),
        ],
        LanguageOp::PythonConfig,
    ));

    Node::subdag("python", inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_python_subdag_is_subdag() {
        let node = build_python_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "python");
    }

    #[test]
    fn test_python_subdag_structure() {
        let node = build_python_subdag();

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
    fn test_python_config() {
        assert_eq!(PYTHON.id, "python");
        assert_eq!(PYTHON.file_extensions, &[".py"]);
        assert_eq!(PYTHON.comment_prefix, "#");
    }
}
