//! Type operations for type DAGs.
//!
//! Types are DAGs (`Dag<TypeOp>`) that describe how values are validated and transformed.
//! This unifies types with workflows — same infrastructure, same composition rules.
//!
//! # Philosophy: Types as Causal Chains
//!
//! A type like `Url` is a causal chain:
//! ```text
//! String (raw) → [NonEmpty check] → [URL pattern check] → Url (validated)
//! ```
//!
//! This is not an analogy — type validation IS a causal chain.
//! Using `Dag<TypeOp>` makes this explicit and reuses all DAG infrastructure.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::type_lib;
//!
//! // Types are just Dag<TypeOp>
//! let url_type = type_lib::url();
//! let optional_url = type_lib::optional(url_type);
//!
//! // Cardinality emerges from structure
//! // optional_url output port has ZeroOrOne cardinality
//! ```

use serde::{Deserialize, Serialize};

/// Operations in a type DAG.
///
/// TypeOp is the operation type for type DAGs, analogous to how
/// `GistOp`, `DepsOp`, etc. are operation types for workflow DAGs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeOp {
    /// Identity operation — pass value through unchanged.
    /// Used as the "base" node in type DAGs.
    Identity,

    /// Validation predicate — checks a condition on the value.
    /// If validation fails, the type DAG produces an error.
    Validate(Predicate),

    /// Transformation — coerces value from one base type to another.
    /// Used for type conversions (e.g., String → Int parsing).
    Transform(Coercion),

    /// Wrapper operation — wraps a value in a container type.
    /// Used for Optional<T>, List<T>, etc.
    Wrap(WrapperKind),

    /// Unwrap operation — extracts value from a container type.
    Unwrap(WrapperKind),
}

/// Predicates that can be validated against values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Predicate {
    /// Value must be non-empty (non-empty string, non-empty list, etc.)
    NonEmpty,

    /// String must match a regex pattern.
    Matches(String),

    /// Numeric value must be in range [min, max].
    InRange { min: i64, max: i64 },

    /// All elements of a collection must satisfy a predicate.
    All(Box<Predicate>),

    /// Any element of a collection must satisfy a predicate.
    Any(Box<Predicate>),

    /// Value must equal a specific value.
    Equals(PredicateValue),

    /// Logical AND of multiple predicates.
    And(Vec<Predicate>),

    /// Logical OR of multiple predicates.
    Or(Vec<Predicate>),

    /// Logical NOT of a predicate.
    Not(Box<Predicate>),

    /// Custom named predicate (resolved at runtime).
    Custom(String),
}

/// Simple values that can appear in predicates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PredicateValue {
    Bool(bool),
    Int(i64),
    Str(String),
}

impl Predicate {
    /// Check whether this predicate entails another (i.e., is at least as strict).
    ///
    /// This is intentionally conservative: only provable entailments return true.
    pub fn entails(&self, other: &Predicate) -> bool {
        if self == other {
            return true;
        }

        match (self, other) {
            (Predicate::And(_), Predicate::And(targets)) => {
                targets.iter().all(|t| self.entails(t))
            }
            (Predicate::And(preds), target) => preds.iter().any(|p| p.entails(target)),
            (source, Predicate::And(targets)) => targets.iter().all(|t| source.entails(t)),

            (Predicate::Or(preds), target) => preds.iter().all(|p| p.entails(target)),
            (source, Predicate::Or(targets)) => targets.iter().any(|t| source.entails(t)),

            (Predicate::All(a), Predicate::All(b)) => a.entails(b),
            (Predicate::Any(a), Predicate::Any(b)) => a.entails(b),
            (Predicate::Not(a), Predicate::Not(b)) => a.entails(b),

            (
                Predicate::InRange { min, max },
                Predicate::InRange {
                    min: target_min,
                    max: target_max,
                },
            ) => min >= target_min && max <= target_max,
            (Predicate::Equals(PredicateValue::Int(v)), Predicate::InRange { min, max }) => {
                v >= min && v <= max
            }
            (Predicate::Equals(a), Predicate::Equals(b)) => a == b,

            _ => false,
        }
    }
}

/// Coercion between base types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coercion {
    /// Source base type.
    pub from: BaseType,
    /// Target base type.
    pub to: BaseType,
}

impl Coercion {
    /// Create a new coercion.
    pub fn new(from: BaseType, to: BaseType) -> Self {
        Self { from, to }
    }
}

/// Base types — the fundamental shapes of data.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseType {
    /// Unit type (no data).
    Unit,
    /// Boolean.
    Bool,
    /// Integer.
    Int,
    /// String.
    String,
    /// JSON value (dynamic).
    Json,
    /// List of elements.
    List(Box<BaseType>),
    /// Optional value (may be absent).
    Option(Box<BaseType>),
    /// Map from keys to values.
    Map(Box<BaseType>, Box<BaseType>),
    /// Named/opaque type (user-defined or external).
    Named(String),
}

impl BaseType {
    /// Create a list type.
    pub fn list(element: BaseType) -> Self {
        BaseType::List(Box::new(element))
    }

    /// Create an optional type.
    pub fn option(inner: BaseType) -> Self {
        BaseType::Option(Box::new(inner))
    }

    /// Create a map type.
    pub fn map(key: BaseType, value: BaseType) -> Self {
        BaseType::Map(Box::new(key), Box::new(value))
    }

    /// Create a named type.
    pub fn named(name: impl Into<String>) -> Self {
        BaseType::Named(name.into())
    }
}

/// Wrapper kinds for container types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WrapperKind {
    /// Optional wrapper (T → Option<T>).
    Optional,
    /// List wrapper (T → List<T>).
    List,
    /// Non-empty list wrapper (T → NonEmptyList<T>).
    NonEmptyList,
    /// Set wrapper (T → Set<T>) — unordered, unique elements.
    Set,
    /// Non-empty set wrapper (T → NonEmptySet<T>).
    NonEmptySet,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_op_variants() {
        let identity = TypeOp::Identity;
        let validate = TypeOp::Validate(Predicate::NonEmpty);
        let transform = TypeOp::Transform(Coercion::new(BaseType::String, BaseType::Int));
        let wrap = TypeOp::Wrap(WrapperKind::Optional);

        assert_eq!(identity, TypeOp::Identity);
        assert!(matches!(validate, TypeOp::Validate(Predicate::NonEmpty)));
        assert!(matches!(transform, TypeOp::Transform(_)));
        assert!(matches!(wrap, TypeOp::Wrap(WrapperKind::Optional)));
    }

    #[test]
    fn test_predicate_composition() {
        let non_empty = Predicate::NonEmpty;
        let matches_url = Predicate::Matches(r"https?://.*".to_string());
        let combined = Predicate::And(vec![non_empty.clone(), matches_url.clone()]);

        assert!(matches!(combined, Predicate::And(_)));
    }

    #[test]
    fn test_predicate_entails() {
        let non_empty = Predicate::NonEmpty;
        let matches_url = Predicate::Matches(r"https?://.*".to_string());
        let combined = Predicate::And(vec![non_empty.clone(), matches_url.clone()]);

        assert!(combined.entails(&non_empty));
        assert!(!non_empty.entails(&combined));

        let narrow = Predicate::InRange { min: 1, max: 5 };
        let wide = Predicate::InRange { min: 0, max: 10 };
        assert!(narrow.entails(&wide));
        assert!(!wide.entails(&narrow));
    }

    #[test]
    fn test_base_type_construction() {
        let string_type = BaseType::String;
        let list_of_strings = BaseType::list(BaseType::String);
        let optional_int = BaseType::option(BaseType::Int);
        let map_type = BaseType::map(BaseType::String, BaseType::Json);

        assert_eq!(string_type, BaseType::String);
        assert!(matches!(list_of_strings, BaseType::List(_)));
        assert!(matches!(optional_int, BaseType::Option(_)));
        assert!(matches!(map_type, BaseType::Map(_, _)));
    }

    #[test]
    fn test_coercion() {
        let string_to_int = Coercion::new(BaseType::String, BaseType::Int);
        assert_eq!(string_to_int.from, BaseType::String);
        assert_eq!(string_to_int.to, BaseType::Int);
    }
}
