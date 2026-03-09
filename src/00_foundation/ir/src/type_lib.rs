//! Type library: Helper functions that build `Dag<TypeOp>`.
//!
//! This module provides convenient functions to construct common type DAGs.
//! Types are just `Dag<TypeOp>`, so all DAG infrastructure (validation,
//! lowering, execution) works on them.
//!
//! # Example
//!
//! ```text
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
use crate::type_op::{ContentEncoding, Predicate, TypeOp, WrapperKind};
use crate::types::Cardinality;

const URL_PATTERN: &str = r"^https?://[^\s/$.?#].[^\s]*$";

const FILE_PATH_PATTERN: &str = r"^([/~].*|[a-zA-Z]:.*)$";

const EMAIL_PATTERN: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";

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

/// Bytes type (identity).
pub fn bytes() -> Dag<TypeOp> {
    identity("Bytes")
}

/// Float type (identity).
pub fn float() -> Dag<TypeOp> {
    identity("Float")
}

/// Secret type (identity — redacted string).
pub fn secret() -> Dag<TypeOp> {
    identity("Secret")
}

// =============================================================================
// Branded / Product / Coproduct Types
// =============================================================================

/// Branded (nominal) type — a named wrapper around an inner type.
///
/// The brand ensures nominal distinctness: `TextFilePath` is not `FilePath`
/// even though structurally identical, unless the brand allows coercion.
/// The inner type is embedded as a SubDag child.
pub fn branded(name: &str, inner_type: Dag<TypeOp>) -> Dag<TypeOp> {
    let mut dag = Dag::new();

    dag.add_node(Node::opaque(
        "brand",
        vec![Port::scalar("in", name)],
        vec![Port::scalar("out", name)],
        TypeOp::Brand(name.to_string()),
    ));

    dag.add_node(Node::subdag("inner_type", inner_type));

    dag.add_edge(Edge::new("brand", "out", "inner_type", "in"));

    dag
}

/// Product type — a record with named typed fields.
///
/// Field types are embedded as SubDag children named `field_{name}`.
/// Uses string-based type names, wrapping each in an `identity()` DAG.
/// Prefer `product_resolved` when resolved type DAGs are available.
pub fn product(name: &str, fields: Vec<(&str, &str)>) -> Dag<TypeOp> {
    let resolved: Vec<(&str, Dag<TypeOp>)> = fields
        .into_iter()
        .map(|(n, t)| (n, identity(t)))
        .collect();
    product_resolved(name, resolved)
}

/// Product type with resolved field type DAGs.
///
/// Each field's type DAG is embedded as a SubDag child named `field_{name}`,
/// enabling structural recursion through record boundaries.
pub fn product_resolved(name: &str, fields: Vec<(&str, Dag<TypeOp>)>) -> Dag<TypeOp> {
    let mut dag = Dag::new();

    let field_names: Vec<String> = fields.iter().map(|(n, _)| n.to_string()).collect();

    dag.add_node(Node::opaque(
        "product",
        vec![Port::scalar("in", name)],
        vec![Port::scalar("out", name)],
        TypeOp::Product(field_names),
    ));

    for (field_name, field_dag) in fields {
        let child_id = format!("field_{field_name}");
        dag.add_node(Node::subdag(child_id.as_str(), field_dag));
        dag.add_edge(Edge::new("product", "out", child_id.as_str(), "in"));
    }

    dag
}

/// Coproduct type — a tagged union of named typed variants.
///
/// Uses string-based type names, wrapping each in an `identity()` DAG.
/// Prefer `coproduct_resolved` when resolved type DAGs are available.
pub fn coproduct(name: &str, variants: Vec<(&str, &str)>) -> Dag<TypeOp> {
    let resolved: Vec<(&str, Dag<TypeOp>)> = variants
        .into_iter()
        .map(|(n, t)| (n, identity(t)))
        .collect();
    coproduct_resolved(name, resolved)
}

/// Coproduct type with resolved variant type DAGs.
///
/// Each variant's type DAG is embedded as a SubDag child named `variant_{name}`,
/// enabling structural recursion through union boundaries.
pub fn coproduct_resolved(name: &str, variants: Vec<(&str, Dag<TypeOp>)>) -> Dag<TypeOp> {
    let mut dag = Dag::new();

    let variant_names: Vec<String> = variants.iter().map(|(n, _)| n.to_string()).collect();

    dag.add_node(Node::opaque(
        "coproduct",
        vec![Port::scalar("in", name)],
        vec![Port::scalar("out", name)],
        TypeOp::Coproduct(variant_names),
    ));

    for (variant_name, variant_dag) in variants {
        let child_id = format!("variant_{variant_name}");
        dag.add_node(Node::subdag(child_id.as_str(), variant_dag));
        dag.add_edge(Edge::new("coproduct", "out", child_id.as_str(), "in"));
    }

    dag
}

/// Content-refined type — a type with a `@content` encoding predicate.
///
/// e.g., `content_refined("String", ContentEncoding::UTF8)` → String @content(UTF8)
fn content_refined(type_name: &str, encoding: ContentEncoding) -> Dag<TypeOp> {
    refined(type_name, vec![Predicate::Content(encoding)])
}

/// TextFilePath — branded FilePath with @content(Text) predicate.
pub fn text_file_path() -> Dag<TypeOp> {
    branded(
        "TextFilePath",
        content_refined("FilePath", ContentEncoding::Text),
    )
}

/// BinaryFilePath — branded FilePath with @content(Binary) predicate.
pub fn binary_file_path() -> Dag<TypeOp> {
    branded(
        "BinaryFilePath",
        content_refined("FilePath", ContentEncoding::Binary),
    )
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

        dag.add_edge(Edge::new(
            prev_node.as_str(),
            prev_port,
            node_id.as_str(),
            "in",
        ));

        prev_node = node_id;
    }

    dag
}

/// Create a refined type that inherits structure from a resolved base DAG.
///
/// The base type's DAG is embedded as a SubDag so that `derive_platform_properties`
/// (which recurses into SubDags) can discover the base type's predicates
/// (width, signed, domain, etc.). New predicates are chained after the base.
///
/// ```text
/// base_type(SubDag) → validate_0 → validate_1 → ... → output
/// ```
pub fn refined_with_base(
    type_name: &str,
    base_dag: Dag<TypeOp>,
    predicates: Vec<Predicate>,
) -> Dag<TypeOp> {
    let mut dag = Dag::new();

    // Embed the base type as a SubDag so structural predicates are inherited
    dag.add_node(Node::subdag("base_type", base_dag));

    if predicates.is_empty() {
        return dag;
    }

    let mut prev_node = "base_type".to_string();
    let prev_port = "out";

    // Chain new validation nodes after the base
    for (i, pred) in predicates.into_iter().enumerate() {
        let node_id = format!("validate_{}", i);

        dag.add_node(Node::opaque(
            node_id.as_str(),
            vec![Port::scalar("in", type_name)],
            vec![Port::scalar("out", type_name)],
            TypeOp::Validate(pred),
        ));

        dag.add_edge(Edge::new(
            prev_node.as_str(),
            prev_port,
            node_id.as_str(),
            "in",
        ));

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
    refined(
        "Int",
        vec![Predicate::InRange {
            min: 1,
            max: i64::MAX,
        }],
    )
}

/// Non-negative integer type.
pub fn non_negative_int() -> Dag<TypeOp> {
    refined(
        "Int",
        vec![Predicate::InRange {
            min: 0,
            max: i64::MAX,
        }],
    )
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
    dag.add_node(Node::subdag("inner_type", inner_type));

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
    dag.add_node(Node::subdag("element_type", element_type));

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
    dag.add_node(Node::subdag("element_type", element_type));

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
    dag.add_node(Node::subdag("element_type", element_type));

    dag.add_edge(Edge::new("input", "out", "element_type", "in"));

    dag
}

/// Non-empty set type — wraps an inner type with OneOrMore cardinality.
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
    dag.add_node(Node::subdag("element_type", element_type));

    dag.add_edge(Edge::new("input", "out", "check_non_empty", "in"));
    dag.add_edge(Edge::new("check_non_empty", "out", "element_type", "in"));

    dag
}

/// Map type with typed keys and values.
///
/// Both key and value type DAGs are included as SubDags, preserving full
/// structural information for both. Key and value are named `"key_type"`
/// and `"value_type"` respectively, matching the SubDag naming convention
/// used by Product fields (`field_{name}`) and Coproduct variants.
pub fn map(key_type: Dag<TypeOp>, value_type: Dag<TypeOp>) -> Dag<TypeOp> {
    let mut dag = Dag::new();

    dag.add_node(Node::opaque(
        "input",
        vec![Port::scalar("in", "Map")],
        vec![Port::scalar("out", "Map")],
        TypeOp::Wrap(WrapperKind::Map),
    ));

    dag.add_node(Node::subdag("key_type", key_type));
    dag.add_node(Node::subdag("value_type", value_type));

    dag.add_edge(Edge::new("input", "out", "value_type", "in"));

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

// Query functions delegate to contract module (single source of truth).

/// Get the output cardinality of a type DAG.
///
/// Delegates to [`crate::contract::cardinality`].
pub fn infer_cardinality(type_dag: &Dag<TypeOp>) -> Cardinality {
    crate::contract::cardinality(type_dag)
}

/// Get the base type name from a type DAG.
///
/// Delegates to [`crate::contract::base_type`].
#[cfg(test)]
fn base_type_name(type_dag: &Dag<TypeOp>) -> Option<String> {
    crate::contract::base_type(type_dag)
}

/// Get all predicates from a type DAG.
///
/// Delegates to [`crate::contract::predicates`].
#[cfg(test)]
fn predicates(type_dag: &Dag<TypeOp>) -> Vec<Predicate> {
    crate::contract::predicates(type_dag)
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
        let non_empty_set = non_empty_set(string());
        let string_map = map(string(), string());

        assert!(optional_string.nodes.len() >= 2);
        assert!(string_list.nodes.len() >= 2);
        assert!(non_empty_strings.nodes.len() >= 3);
        assert!(non_empty_set.nodes.len() >= 3);
        assert!(string_map.nodes.len() >= 3);
    }

    #[test]
    fn test_cardinality_inference() {
        let string_type = string();
        let optional_type = optional(string());
        let list_type = list(string());
        let non_empty_type = non_empty_list(string());
        let non_empty_set_type = non_empty_set(string());
        let map_type = map(string(), string());

        assert_eq!(infer_cardinality(&string_type), Cardinality::ONE);
        assert_eq!(infer_cardinality(&optional_type), Cardinality::ZERO_OR_ONE);
        assert_eq!(infer_cardinality(&list_type), Cardinality::ZERO_OR_MORE);
        assert_eq!(infer_cardinality(&non_empty_type), Cardinality::ONE_OR_MORE);
        assert_eq!(
            infer_cardinality(&non_empty_set_type),
            Cardinality::ONE_OR_MORE
        );
        assert_eq!(infer_cardinality(&map_type), Cardinality::ONE);
    }

    #[test]
    fn test_map_type_has_value_subdag() {
        use crate::contract::wrapper_kind;

        let int_map = map(string(), int());

        // Map DAG should have input node + key_type SubDag + value_type SubDag
        assert_eq!(int_map.nodes.len(), 3);
        assert_eq!(wrapper_kind(&int_map), Some(WrapperKind::Map));

        // Cardinality should be ONE (maps are scalar containers)
        assert_eq!(infer_cardinality(&int_map), Cardinality::ONE);
    }

    #[test]
    fn test_refined_with_base_inherits_predicates() {
        // Simulate: Word32 has Width(32), then Float32 = Word32 where domain, arithmetic
        let word32 = refined("Word32", vec![Predicate::Width(32)]);
        let float32 = refined_with_base(
            "Word32",
            word32,
            vec![
                Predicate::Domain("ieee754_binary32".to_string()),
                Predicate::Arithmetic,
            ],
        );

        // The base DAG is embedded as a SubDag, so derive_platform_properties
        // (which recurses into SubDags) should find Width(32)
        let props = crate::contract::predicates(&float32);
        // The new predicates are at the top level
        assert!(props.iter().any(|p| matches!(p, Predicate::Domain(d) if d == "ieee754_binary32")));
        assert!(props.iter().any(|p| matches!(p, Predicate::Arithmetic)));
        // Width(32) is inside the SubDag — not visible via flat predicate scan,
        // but derive_platform_properties recurses into SubDags.
        // Verify the SubDag is present.
        use crate::node::NodeBody;
        let has_subdag = float32.nodes.iter().any(|n| matches!(&n.body, NodeBody::SubDag(_, _)));
        assert!(has_subdag, "base DAG should be embedded as SubDag");
    }

    #[test]
    fn test_refined_with_base_empty_predicates_returns_base() {
        let base = refined("Int", vec![Predicate::Width(64), Predicate::Signed(None)]);
        let alias = refined_with_base("Int", base.clone(), vec![]);
        // With empty predicates, just returns the base as a SubDag
        use crate::node::NodeBody;
        let has_subdag = alias.nodes.iter().any(|n| matches!(&n.body, NodeBody::SubDag(_, _)));
        assert!(has_subdag);
    }

    #[test]
    fn test_branded_wraps_inner() {
        let inner = refined("String", vec![Predicate::NonEmpty]);
        let branded_type = branded("PathSegment", inner);

        use crate::node::NodeBody;
        use crate::type_op::TypeOp;
        let has_brand = branded_type.nodes.iter().any(|n| {
            matches!(&n.body, NodeBody::Opaque(TypeOp::Brand(name)) if name == "PathSegment")
        });
        assert!(has_brand, "branded type should have Brand node");
        let has_subdag = branded_type.nodes.iter().any(|n| matches!(&n.body, NodeBody::SubDag(_, _)));
        assert!(has_subdag, "branded type should embed inner as SubDag");
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
