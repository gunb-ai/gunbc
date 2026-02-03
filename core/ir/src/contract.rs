//! Contract tower: Extract contract levels from type DAGs.
//!
//! The contract tower **emerges** from the `Dag<TypeOp>` structure.
//! These are just queries on a regular `Dag<TypeOp>` — no new abstraction needed.
//!
//! # Contract Levels
//!
//! | Level | Name | What It Describes |
//! |-------|------|-------------------|
//! | L1 | Cardinality | How many values (One, ZeroOrOne, etc.) |
//! | L2 | Base Type | The shape of data (String, Int, etc.) |
//! | L3 | Predicates | Validation constraints (NonEmpty, Matches, etc.) |
//! | L4 | Witnesses | Example valid values |
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::{contract, type_lib};
//!
//! let url_type = type_lib::url();
//!
//! // Extract contract levels
//! let card = contract::cardinality(&url_type);      // One
//! let base = contract::base_type(&url_type);        // "String"
//! let preds = contract::predicates(&url_type);      // [NonEmpty, Matches(URL_PATTERN)]
//! ```

use crate::dag::Dag;
use crate::node::NodeBody;
use crate::type_op::{Predicate, TypeOp, WrapperKind};
use crate::types::Cardinality;

/// L1: Extract cardinality from a type DAG.
///
/// Cardinality is determined by the wrapper kind:
/// - `Optional<T>` → `ZeroOrOne`
/// - `List<T>` → `ZeroOrMore`
/// - `NonEmptyList<T>` → `OneOrMore`
/// - Everything else → `One`
pub fn cardinality(type_dag: &Dag<TypeOp>) -> Cardinality {
    // Look for wrapper nodes to determine cardinality
    for node in &type_dag.nodes {
        if let NodeBody::Opaque(TypeOp::Wrap(kind)) = &node.body {
            return match kind {
                WrapperKind::Optional => Cardinality::ZERO_OR_ONE,
                WrapperKind::List => Cardinality::ZERO_OR_MORE,
                WrapperKind::NonEmptyList => Cardinality::ONE_OR_MORE,
            };
        }
    }

    // Default to One (scalar)
    Cardinality::ONE
}

/// L2: Extract base type name from a type DAG.
///
/// The base type is found by looking at the first Identity node's output type.
pub fn base_type(type_dag: &Dag<TypeOp>) -> Option<String> {
    // Find the first Identity node and get its output type
    for node in &type_dag.nodes {
        if let NodeBody::Opaque(TypeOp::Identity) = &node.body {
            if let Some(output) = node.outputs.first() {
                return Some(output.type_id.0.clone());
            }
        }
    }
    None
}

/// L3: Extract all predicates from a type DAG.
///
/// Collects all `Validate(predicate)` operations in the DAG.
pub fn predicates(type_dag: &Dag<TypeOp>) -> Vec<Predicate> {
    type_dag
        .nodes
        .iter()
        .filter_map(|n| {
            if let NodeBody::Opaque(TypeOp::Validate(pred)) = &n.body {
                Some(pred.clone())
            } else {
                None
            }
        })
        .collect()
}

/// L4: Generate witness values for a type.
///
/// Witnesses are example values that satisfy the type's constraints.
/// This is useful for property-based testing and documentation.
///
/// Currently returns empty — full implementation would use predicate
/// analysis to generate valid values.
pub fn witnesses(_type_dag: &Dag<TypeOp>) -> Vec<crate::value::Value> {
    // Future: Generate witnesses from predicate constraints
    // - NonEmpty → generate non-empty value
    // - InRange { min, max } → generate value in range
    // - Matches(pattern) → generate string matching pattern
    vec![]
}

/// Check if a type DAG has any validation predicates.
pub fn has_predicates(type_dag: &Dag<TypeOp>) -> bool {
    type_dag.nodes.iter().any(|n| {
        matches!(&n.body, NodeBody::Opaque(TypeOp::Validate(_)))
    })
}

/// Check if a type is a container type (Optional, List, NonEmptyList).
pub fn is_container(type_dag: &Dag<TypeOp>) -> bool {
    type_dag.nodes.iter().any(|n| {
        matches!(&n.body, NodeBody::Opaque(TypeOp::Wrap(_)))
    })
}

/// Get the wrapper kind if this is a container type.
pub fn wrapper_kind(type_dag: &Dag<TypeOp>) -> Option<WrapperKind> {
    for node in &type_dag.nodes {
        if let NodeBody::Opaque(TypeOp::Wrap(kind)) = &node.body {
            return Some(kind.clone());
        }
    }
    None
}

/// Full contract summary for a type.
#[derive(Debug, Clone)]
pub struct TypeContract {
    /// L1: Cardinality
    pub cardinality: Cardinality,
    /// L2: Base type name
    pub base_type: Option<String>,
    /// L3: Predicates
    pub predicates: Vec<Predicate>,
    /// Whether this is a container type
    pub is_container: bool,
    /// Wrapper kind (if container)
    pub wrapper_kind: Option<WrapperKind>,
}

impl TypeContract {
    /// Extract full contract from a type DAG.
    pub fn from_type_dag(type_dag: &Dag<TypeOp>) -> Self {
        Self {
            cardinality: cardinality(type_dag),
            base_type: base_type(type_dag),
            predicates: predicates(type_dag),
            is_container: is_container(type_dag),
            wrapper_kind: wrapper_kind(type_dag),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_lib;

    #[test]
    fn test_cardinality_extraction() {
        let string_type = type_lib::string();
        let optional_type = type_lib::optional(type_lib::string());
        let list_type = type_lib::list(type_lib::string());
        let non_empty_type = type_lib::non_empty_list(type_lib::string());

        assert_eq!(cardinality(&string_type), Cardinality::ONE);
        assert_eq!(cardinality(&optional_type), Cardinality::ZERO_OR_ONE);
        assert_eq!(cardinality(&list_type), Cardinality::ZERO_OR_MORE);
        assert_eq!(cardinality(&non_empty_type), Cardinality::ONE_OR_MORE);
    }

    #[test]
    fn test_base_type_extraction() {
        let string_type = type_lib::string();
        let url_type = type_lib::url();
        let int_type = type_lib::int();

        assert_eq!(base_type(&string_type), Some("String".to_string()));
        assert_eq!(base_type(&url_type), Some("String".to_string()));
        assert_eq!(base_type(&int_type), Some("Int".to_string()));
    }

    #[test]
    fn test_predicates_extraction() {
        let string_type = type_lib::string();
        let url_type = type_lib::url();

        assert!(predicates(&string_type).is_empty());
        
        let url_preds = predicates(&url_type);
        assert!(!url_preds.is_empty());
        assert!(url_preds.iter().any(|p| matches!(p, Predicate::NonEmpty)));
    }

    #[test]
    fn test_has_predicates() {
        let string_type = type_lib::string();
        let url_type = type_lib::url();

        assert!(!has_predicates(&string_type));
        assert!(has_predicates(&url_type));
    }

    #[test]
    fn test_is_container() {
        let string_type = type_lib::string();
        let optional_type = type_lib::optional(type_lib::string());
        let list_type = type_lib::list(type_lib::string());

        assert!(!is_container(&string_type));
        assert!(is_container(&optional_type));
        assert!(is_container(&list_type));
    }

    #[test]
    fn test_wrapper_kind() {
        let string_type = type_lib::string();
        let optional_type = type_lib::optional(type_lib::string());
        let list_type = type_lib::list(type_lib::string());
        let non_empty_type = type_lib::non_empty_list(type_lib::string());

        assert_eq!(wrapper_kind(&string_type), None);
        assert_eq!(wrapper_kind(&optional_type), Some(WrapperKind::Optional));
        assert_eq!(wrapper_kind(&list_type), Some(WrapperKind::List));
        assert_eq!(wrapper_kind(&non_empty_type), Some(WrapperKind::NonEmptyList));
    }

    #[test]
    fn test_type_contract() {
        let url_type = type_lib::url();
        let contract = TypeContract::from_type_dag(&url_type);

        assert_eq!(contract.cardinality, Cardinality::ONE);
        assert_eq!(contract.base_type, Some("String".to_string()));
        assert!(!contract.predicates.is_empty());
        assert!(!contract.is_container);
        assert_eq!(contract.wrapper_kind, None);
    }
}
