//! Type library: Helper functions that build `Dag<TypeOp>`.
//!
//! This module provides convenient functions to construct common type DAGs.
//! Types are just `Dag<TypeOp>`, so all DAG infrastructure (validation,
//! lowering, execution) works on them.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::type_lib;
//!
//! // Primitive types
//! let string_type = type_lib::string();
//! let int_type = type_lib::int();
//!
//! // Refined types (with validation predicates)
//! let url_type = type_lib::url();
//! let non_empty_string = type_lib::non_empty_string();
//!
//! // Container types
//! let optional_url = type_lib::optional(type_lib::url());
//! let list_of_strings = type_lib::list(type_lib::string());
//! ```

use crate::dag::{Dag, Edge, Port};
use crate::node::Node;
use crate::type_op::{Predicate, TypeOp, WrapperKind};
use crate::types::Cardinality;

/// URL pattern regex.
pub const URL_PATTERN: &str = r"^https?://[^\s/$.?#].[^\s]*$";

/// File path pattern regex (Unix or Windows style).
pub const FILE_PATH_PATTERN: &str = r"^([/~].*|[a-zA-Z]:.*)$";

/// Email pattern regex (simplified).
pub const EMAIL_PATTERN: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";

// =============================================================================
// Primitive Types
// =============================================================================

/// Create an identity type DAG (no validation, just passes through).
pub fn identity(type_name: &str) -> Dag<TypeOp> {
    let mut dag = Dag::new();

    dag.add_node(Node::opaque(
        "identity",
        vec![Port::scalar("in", type_name)],
        vec![Port::scalar("out", type_name)],
        TypeOp::Identity,
    ));

    dag
}

/// String type (identity).
pub fn string() -> Dag<TypeOp> {
    identity("String")
}

/// Boolean type (identity).
pub fn bool() -> Dag<TypeOp> {
    identity("Bool")
}

/// Integer type (identity).
pub fn int() -> Dag<TypeOp> {
    identity("Int")
}

/// Unit type (identity).
pub fn unit() -> Dag<TypeOp> {
    identity("Unit")
}

/// JSON type (identity).
pub fn json() -> Dag<TypeOp> {
    identity("Json")
}

// =============================================================================
// Refined Types (with validation predicates)
// =============================================================================

/// Create a refined type with validation predicates.
///
/// The type DAG chains validation nodes together:
/// ```text
/// input → validate_0 → validate_1 → ... → output
/// ```
pub fn refined(type_name: &str, predicates: Vec<Predicate>) -> Dag<TypeOp> {
    let mut dag = Dag::new();

    if predicates.is_empty() {
        return identity(type_name);
    }

    // Input node
    dag.add_node(Node::opaque(
        "input",
        vec![Port::scalar("in", type_name)],
        vec![Port::scalar("out", type_name)],
        TypeOp::Identity,
    ));

    let mut prev_node = "input".to_string();
    let prev_port = "out";

    // Chain validation nodes
    for (i, pred) in predicates.into_iter().enumerate() {
        let node_id = format!("validate_{}", i);

        dag.add_node(Node::opaque(
            node_id.as_str(),
            vec![Port::scalar("in", type_name)],
            vec![Port::scalar("out", type_name)],
            TypeOp::Validate(pred),
        ));

        dag.add_edge(Edge::new(prev_node.as_str(), prev_port, node_id.as_str(), "in"));

        prev_node = node_id;
    }

    dag
}

/// Non-empty string type.
pub fn non_empty_string() -> Dag<TypeOp> {
    refined("String", vec![Predicate::NonEmpty])
}

/// URL type (non-empty string matching URL pattern).
pub fn url() -> Dag<TypeOp> {
    refined(
        "String",
        vec![
            Predicate::NonEmpty,
            Predicate::Matches(URL_PATTERN.to_string()),
        ],
    )
}

/// File path type (non-empty string matching path pattern).
pub fn file_path() -> Dag<TypeOp> {
    refined(
        "String",
        vec![
            Predicate::NonEmpty,
            Predicate::Matches(FILE_PATH_PATTERN.to_string()),
        ],
    )
}

/// Email type (string matching email pattern).
pub fn email() -> Dag<TypeOp> {
    refined(
        "String",
        vec![
            Predicate::NonEmpty,
            Predicate::Matches(EMAIL_PATTERN.to_string()),
        ],
    )
}

/// Positive integer type.
pub fn positive_int() -> Dag<TypeOp> {
    refined("Int", vec![Predicate::InRange { min: 1, max: i64::MAX }])
}

/// Non-negative integer type.
pub fn non_negative_int() -> Dag<TypeOp> {
    refined("Int", vec![Predicate::InRange { min: 0, max: i64::MAX }])
}

// =============================================================================
// Container Types
// =============================================================================

/// Optional type — wraps an inner type with ZeroOrOne cardinality.
///
/// The inner type DAG is included as a SubDag.
pub fn optional(inner_type: Dag<TypeOp>) -> Dag<TypeOp> {
    let mut dag = Dag::new();

    // Input: optional value
    dag.add_node(Node::opaque(
        "input",
        vec![Port::optional("in", "Any")],
        vec![Port::optional("out", "Any")],
        TypeOp::Wrap(WrapperKind::Optional),
    ));

    // Inner type validation (as SubDag)
    dag.add_node(Node::subdag(
        "inner_type",
        inner_type,
    ));

    dag.add_edge(Edge::new("input", "out", "inner_type", "in"));

    dag
}

/// List type — wraps an inner type with ZeroOrMore cardinality.
///
/// The element type DAG is included as a SubDag.
pub fn list(element_type: Dag<TypeOp>) -> Dag<TypeOp> {
    let mut dag = Dag::new();

    // Input: list of values
    dag.add_node(Node::opaque(
        "input",
        vec![Port::list("in", "Any")],
        vec![Port::list("out", "Any")],
        TypeOp::Wrap(WrapperKind::List),
    ));

    // Element type validation (as SubDag, applied to each element)
    dag.add_node(Node::subdag(
        "element_type",
        element_type,
    ));

    dag.add_edge(Edge::new("input", "out", "element_type", "in"));

    dag
}

/// Non-empty list type — wraps an inner type with OneOrMore cardinality.
pub fn non_empty_list(element_type: Dag<TypeOp>) -> Dag<TypeOp> {
    let mut dag = Dag::new();

    // Input: non-empty list of values
    dag.add_node(Node::opaque(
        "input",
        vec![Port::non_empty_list("in", "Any")],
        vec![Port::non_empty_list("out", "Any")],
        TypeOp::Wrap(WrapperKind::NonEmptyList),
    ));

    // Non-empty check
    dag.add_node(Node::opaque(
        "check_non_empty",
        vec![Port::non_empty_list("in", "Any")],
        vec![Port::non_empty_list("out", "Any")],
        TypeOp::Validate(Predicate::NonEmpty),
    ));

    // Element type validation (as SubDag)
    dag.add_node(Node::subdag(
        "element_type",
        element_type,
    ));

    dag.add_edge(Edge::new("input", "out", "check_non_empty", "in"));
    dag.add_edge(Edge::new("check_non_empty", "out", "element_type", "in"));

    dag
}

/// Set type — wraps an inner type with ZeroOrMore cardinality and set (unique) semantics.
///
/// The element type DAG is included as a SubDag.
pub fn set(element_type: Dag<TypeOp>) -> Dag<TypeOp> {
    let mut dag = Dag::new();

    // Input: set of values
    dag.add_node(Node::opaque(
        "input",
        vec![Port::list("in", "Any")],
        vec![Port::list("out", "Any")],
        TypeOp::Wrap(WrapperKind::Set),
    ));

    // Element type validation (as SubDag, applied to each element)
    dag.add_node(Node::subdag(
        "element_type",
        element_type,
    ));

    dag.add_edge(Edge::new("input", "out", "element_type", "in"));

    dag
}

/// Non-empty set type — wraps an inner type with OneOrMore cardinality and set semantics.
pub fn non_empty_set(element_type: Dag<TypeOp>) -> Dag<TypeOp> {
    let mut dag = Dag::new();

    // Input: non-empty set of values
    dag.add_node(Node::opaque(
        "input",
        vec![Port::non_empty_list("in", "Any")],
        vec![Port::non_empty_list("out", "Any")],
        TypeOp::Wrap(WrapperKind::NonEmptySet),
    ));

    // Non-empty check
    dag.add_node(Node::opaque(
        "check_non_empty",
        vec![Port::non_empty_list("in", "Any")],
        vec![Port::non_empty_list("out", "Any")],
        TypeOp::Validate(Predicate::NonEmpty),
    ));

    // Element type validation (as SubDag)
    dag.add_node(Node::subdag(
        "element_type",
        element_type,
    ));

    dag.add_edge(Edge::new("input", "out", "check_non_empty", "in"));
    dag.add_edge(Edge::new("check_non_empty", "out", "element_type", "in"));

    dag
}

// =============================================================================
// Composite Type Helpers
// =============================================================================

/// Create an optional URL type.
pub fn optional_url() -> Dag<TypeOp> {
    optional(url())
}

/// Create a list of URLs type.
pub fn url_list() -> Dag<TypeOp> {
    list(url())
}

/// Create a list of file paths type.
pub fn file_path_list() -> Dag<TypeOp> {
    list(file_path())
}

/// Create a non-empty list of file paths type.
pub fn non_empty_file_path_list() -> Dag<TypeOp> {
    non_empty_list(file_path())
}

// =============================================================================
// Type DAG Utilities
// =============================================================================

/// Get the output cardinality of a type DAG.
///
/// This inspects the type DAG structure to determine cardinality:
/// - Optional<T> → ZeroOrOne
/// - List<T> / Set<T> → ZeroOrMore
/// - NonEmptyList<T> / NonEmptySet<T> → OneOrMore
/// - Everything else → One
pub fn infer_cardinality(type_dag: &Dag<TypeOp>) -> Cardinality {
    // Look for wrapper nodes to determine cardinality
    for node in &type_dag.nodes {
        if let crate::node::NodeBody::Opaque(TypeOp::Wrap(kind)) = &node.body {
            return match kind {
                WrapperKind::Optional => Cardinality::ZERO_OR_ONE,
                WrapperKind::List | WrapperKind::Set => Cardinality::ZERO_OR_MORE,
                WrapperKind::NonEmptyList | WrapperKind::NonEmptySet => Cardinality::ONE_OR_MORE,
            };
        }
    }

    // Default to One (scalar)
    Cardinality::ONE
}

/// Get the base type name from a type DAG.
pub fn base_type_name(type_dag: &Dag<TypeOp>) -> Option<String> {
    // Find the first Identity node and get its output type
    for node in &type_dag.nodes {
        if let crate::node::NodeBody::Opaque(TypeOp::Identity) = &node.body {
            if let Some(output) = node.outputs.first() {
                return Some(output.type_id.0.clone());
            }
        }
    }
    None
}

/// Get all predicates from a type DAG.
pub fn predicates(type_dag: &Dag<TypeOp>) -> Vec<Predicate> {
    type_dag
        .nodes
        .iter()
        .filter_map(|n| {
            if let crate::node::NodeBody::Opaque(TypeOp::Validate(pred)) = &n.body {
                Some(pred.clone())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        let string_type = string();
        let int_type = int();
        let bool_type = bool();

        assert_eq!(string_type.nodes.len(), 1);
        assert_eq!(int_type.nodes.len(), 1);
        assert_eq!(bool_type.nodes.len(), 1);
    }

    #[test]
    fn test_refined_types() {
        let url_type = url();
        let email_type = email();

        // URL has input + 2 validation nodes
        assert!(url_type.nodes.len() >= 2);
        assert!(email_type.nodes.len() >= 2);

        // Check predicates
        let url_preds = predicates(&url_type);
        assert!(url_preds.iter().any(|p| matches!(p, Predicate::NonEmpty)));
        assert!(url_preds.iter().any(|p| matches!(p, Predicate::Matches(_))));
    }

    #[test]
    fn test_container_types() {
        let optional_string = optional(string());
        let string_list = list(string());
        let non_empty_strings = non_empty_list(string());

        assert!(optional_string.nodes.len() >= 2);
        assert!(string_list.nodes.len() >= 2);
        assert!(non_empty_strings.nodes.len() >= 3);
    }

    #[test]
    fn test_cardinality_inference() {
        let string_type = string();
        let optional_type = optional(string());
        let list_type = list(string());
        let non_empty_type = non_empty_list(string());

        assert_eq!(infer_cardinality(&string_type), Cardinality::ONE);
        assert_eq!(infer_cardinality(&optional_type), Cardinality::ZERO_OR_ONE);
        assert_eq!(infer_cardinality(&list_type), Cardinality::ZERO_OR_MORE);
        assert_eq!(infer_cardinality(&non_empty_type), Cardinality::ONE_OR_MORE);
    }

    #[test]
    fn test_base_type_name() {
        let string_type = string();
        let url_type = url();

        assert_eq!(base_type_name(&string_type), Some("String".to_string()));
        assert_eq!(base_type_name(&url_type), Some("String".to_string()));
    }

    #[test]
    fn test_composite_types() {
        let opt_url = optional_url();
        let url_list_type = url_list();

        assert!(opt_url.nodes.len() >= 2);
        assert!(url_list_type.nodes.len() >= 2);

        assert_eq!(infer_cardinality(&opt_url), Cardinality::ZERO_OR_ONE);
        assert_eq!(infer_cardinality(&url_list_type), Cardinality::ZERO_OR_MORE);
    }
}
