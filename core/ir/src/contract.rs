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
use crate::value::Value;

/// L1: Extract cardinality from a type DAG.
///
/// Cardinality is determined by the wrapper kind:
/// - `Optional<T>` → `ZeroOrOne`
/// - `List<T>` / `Set<T>` → `ZeroOrMore`
/// - `NonEmptyList<T>` / `NonEmptySet<T>` → `OneOrMore`
/// - Everything else → `One`
pub fn cardinality(type_dag: &Dag<TypeOp>) -> Cardinality {
    // Look for wrapper nodes to determine cardinality
    for node in &type_dag.nodes {
        if let NodeBody::Opaque(TypeOp::Wrap(kind)) = &node.body {
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

/// L4: Generate boundary witness values for a type.
///
/// Witnesses are example values that satisfy the type's constraints,
/// generated from the contract levels:
///
/// 1. **Base type** determines the shape (String, Int, Bool, etc.)
/// 2. **Predicates** refine the base witness (NonEmpty, InRange, etc.)
/// 3. **Cardinality** generates boundary values from the interval
///
/// Returns one witness per boundary value the cardinality accepts.
/// For a `List<String>` (cardinality `[0,∞)`), this produces witnesses
/// at counts 0 and 1 (the in-range boundary values).
pub fn witnesses(type_dag: &Dag<TypeOp>) -> Vec<BoundaryWitness> {
    let card = cardinality(type_dag);
    let base = base_type(type_dag);
    let preds = predicates(type_dag);
    let wrapper = wrapper_kind(type_dag);

    let scalar_witness = scalar_witness_for_base(&base, &preds);

    card.test_cases_for_tests()
        .into_iter()
        .map(|count| {
            let value = match count {
                0 => match &wrapper {
                    Some(WrapperKind::Optional) => Value::Unit,
                    Some(WrapperKind::List | WrapperKind::NonEmptyList) => Value::List(vec![]),
                    Some(WrapperKind::Set | WrapperKind::NonEmptySet) => Value::Set(vec![]),
                    None => Value::Unit, // Scalar empty = absent
                },
                1 => match &wrapper {
                    Some(WrapperKind::List | WrapperKind::NonEmptyList) => {
                        Value::List(vec![scalar_witness.clone()])
                    }
                    Some(WrapperKind::Set | WrapperKind::NonEmptySet) => {
                        Value::Set(vec![scalar_witness.clone()])
                    }
                    _ => scalar_witness.clone(),
                },
                n => {
                    let witnesses = n_witnesses(&scalar_witness, n);
                    match &wrapper {
                        Some(WrapperKind::List | WrapperKind::NonEmptyList) => {
                            Value::List(witnesses)
                        }
                        Some(WrapperKind::Set | WrapperKind::NonEmptySet) => Value::set(witnesses),
                        _ => Value::List(witnesses), // fallback
                    }
                }
            };
            BoundaryWitness { count, value }
        })
        .collect()
}

/// A boundary witness: a boundary value count paired with an example value.
#[derive(Debug, Clone)]
pub struct BoundaryWitness {
    /// Number of elements at this boundary.
    pub count: u32,
    /// An example value satisfying the type contract at this boundary.
    pub value: Value,
}

/// Generate a scalar witness for a base type, refined by predicates.
fn scalar_witness_for_base(base: &Option<String>, preds: &[Predicate]) -> Value {
    let base_str = base.as_deref().unwrap_or("String");

    let mut witness = match base_str {
        "String" => Value::Str("example".to_string()),
        "Int" | "i64" | "i32" => Value::Int(1),
        "Bool" => Value::Bool(true),
        "Unit" => Value::Unit,
        "Json" => Value::Json(serde_json::json!({"key": "value"})),
        _ => Value::Str(format!("<{}>", base_str)),
    };

    // Refine witness based on predicates
    for pred in preds {
        witness = refine_witness(witness, pred, base_str);
    }

    witness
}

/// Refine a witness value based on a predicate constraint.
fn refine_witness(witness: Value, pred: &Predicate, base: &str) -> Value {
    match pred {
        Predicate::NonEmpty => {
            // Ensure the witness is non-empty (it already should be from base generation)
            witness
        }
        Predicate::InRange { min, max } => {
            if base == "Int" || base == "i64" || base == "i32" {
                // Pick mid-point of range
                let mid = min.saturating_add(*max) / 2;
                Value::Int(mid)
            } else {
                witness
            }
        }
        Predicate::Equals(pval) => match pval {
            crate::type_op::PredicateValue::Bool(b) => Value::Bool(*b),
            crate::type_op::PredicateValue::Int(i) => Value::Int(*i),
            crate::type_op::PredicateValue::Str(s) => Value::Str(s.clone()),
        },
        Predicate::Matches(pattern) => {
            // For well-known patterns, generate a matching example
            if pattern.contains("http") {
                Value::Str("https://example.com".to_string())
            } else if pattern.contains("@") {
                Value::Str("user@example.com".to_string())
            } else if pattern.contains("[/~]") || pattern.contains("path") {
                Value::Str("/tmp/example".to_string())
            } else {
                // Can't reliably generate from arbitrary regex; keep base witness
                witness
            }
        }
        // Composite predicates — apply recursively
        Predicate::And(preds) => {
            let mut w = witness;
            for p in preds {
                w = refine_witness(w, p, base);
            }
            w
        }
        Predicate::Or(preds) => {
            // Satisfy the first alternative
            if let Some(first) = preds.first() {
                refine_witness(witness, first, base)
            } else {
                witness
            }
        }
        _ => witness,
    }
}

/// Generate `n` distinct witness values based on a scalar witness.
fn n_witnesses(scalar: &Value, n: u32) -> Vec<Value> {
    (0..n)
        .map(|i| match scalar {
            Value::Str(s) => Value::Str(format!("{}_{}", s, i + 1)),
            Value::Int(v) => Value::Int(*v + i as i64),
            Value::Bool(_) => Value::Bool(i % 2 == 0),
            other => other.clone(),
        })
        .collect()
}

/// Check if a type DAG has any validation predicates.
pub fn has_predicates(type_dag: &Dag<TypeOp>) -> bool {
    type_dag
        .nodes
        .iter()
        .any(|n| matches!(&n.body, NodeBody::Opaque(TypeOp::Validate(_))))
}

/// Check if a type is a container type (Optional, List, NonEmptyList, Set, NonEmptySet).
pub fn is_container(type_dag: &Dag<TypeOp>) -> bool {
    type_dag
        .nodes
        .iter()
        .any(|n| matches!(&n.body, NodeBody::Opaque(TypeOp::Wrap(_))))
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

fn inner_type_dag(type_dag: &Dag<TypeOp>) -> Option<&Dag<TypeOp>> {
    type_dag.nodes.iter().find_map(|node| {
        if let NodeBody::SubDag(subdag) = &node.body {
            Some(subdag)
        } else {
            None
        }
    })
}

fn wrap_predicate_for_container(pred: Predicate) -> Predicate {
    match pred {
        Predicate::All(_) | Predicate::Any(_) => pred,
        other => Predicate::All(Box::new(other)),
    }
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
        let wrapper = wrapper_kind(type_dag);
        let is_container = is_container(type_dag);
        let mut base = base_type(type_dag);
        let mut preds = predicates(type_dag);

        if wrapper.is_some() {
            if let Some(inner) = inner_type_dag(type_dag) {
                let inner_contract = TypeContract::from_type_dag(inner);
                base = inner_contract.base_type;
                preds.extend(
                    inner_contract
                        .predicates
                        .into_iter()
                        .map(wrap_predicate_for_container),
                );
            }
        }

        Self {
            cardinality: cardinality(type_dag),
            base_type: base,
            predicates: preds,
            is_container,
            wrapper_kind: wrapper,
        }
    }

    /// Check whether this contract can safely coerce to a target contract.
    ///
    /// A coercion is safe only if it is a widening on all three levels:
    /// - L1: Cardinality containment
    /// - L2: Base type upcast
    /// - L3: Predicate entailment (source predicates cover target predicates)
    pub fn can_safely_coerce_to(&self, target: &TypeContract) -> CoercionResult {
        self.can_safely_coerce_to_with(target, base_type_upcasts_to)
    }

    /// Check whether this contract can safely coerce to a target contract,
    /// using a caller-provided base type lattice.
    pub fn can_safely_coerce_to_with<F>(
        &self,
        target: &TypeContract,
        base_upcasts_to: F,
    ) -> CoercionResult
    where
        F: Fn(&str, &str) -> bool,
    {
        if let Err(mismatch) = self.cardinality.check_satisfies(target.cardinality) {
            return CoercionResult::err(format!(
                "cardinality {} does not satisfy {} ({})",
                mismatch.output, mismatch.input, mismatch.reason
            ));
        }

        match (&self.base_type, &target.base_type) {
            (Some(from), Some(to)) if !base_upcasts_to(from, to) => {
                return CoercionResult::err(format!(
                    "base type '{}' cannot upcast to '{}'",
                    from, to
                ));
            }
            (None, Some(to)) => {
                return CoercionResult::err(format!(
                    "unknown source base type cannot prove compatibility with '{}'",
                    to
                ));
            }
            _ => {}
        }

        for tp in &target.predicates {
            if !self.predicates.iter().any(|sp| sp.entails(tp)) {
                return CoercionResult::err(format!(
                    "source predicates do not entail target predicate {:?}",
                    tp
                ));
            }
        }

        CoercionResult::Ok
    }
}

/// Result of a coercion check between two contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoercionResult {
    /// Coercion is safe — values flow directly or via a deducible upcast.
    Ok,
    /// Coercion is not possible.
    Err(String),
}

impl CoercionResult {
    pub fn err(msg: impl Into<String>) -> Self {
        CoercionResult::Err(msg.into())
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, CoercionResult::Ok)
    }
}

/// Can `from` safely upcast to `to` in the base type lattice?
fn base_type_upcasts_to(from: &str, to: &str) -> bool {
    if from == to {
        return true;
    }

    match (from, to) {
        // Everything upcasts to Json (top of lattice)
        (_, "Json") => true,
        // Url is a refinement of String (when represented as base types)
        ("Url", "String") => true,
        _ => false,
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
        assert_eq!(
            wrapper_kind(&non_empty_type),
            Some(WrapperKind::NonEmptyList)
        );
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

    #[test]
    fn test_contract_coercion() {
        let url_type = type_lib::url();
        let string_type = type_lib::string();
        let int_type = type_lib::int();
        let json_type = type_lib::json();

        let url_contract = TypeContract::from_type_dag(&url_type);
        let string_contract = TypeContract::from_type_dag(&string_type);
        let int_contract = TypeContract::from_type_dag(&int_type);
        let json_contract = TypeContract::from_type_dag(&json_type);

        assert!(url_contract.can_safely_coerce_to(&string_contract).is_ok());
        assert!(!string_contract
            .can_safely_coerce_to(&url_contract)
            .is_ok());
        assert!(int_contract.can_safely_coerce_to(&json_contract).is_ok());
    }

    #[test]
    fn test_type_contract_container_inner_predicates() {
        let url_list = type_lib::list(type_lib::url());
        let contract = TypeContract::from_type_dag(&url_list);

        assert_eq!(contract.cardinality, Cardinality::ZERO_OR_MORE);
        assert_eq!(contract.base_type, Some("String".to_string()));
        assert!(contract
            .predicates
            .iter()
            .any(|p| matches!(p, Predicate::All(_))));
    }

    // --- Witness generation tests ---

    #[test]
    fn test_witnesses_scalar_string() {
        let string_type = type_lib::string();
        let w = witnesses(&string_type);

        // Scalar (cardinality ONE) → exactly one witness (count=1)
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].count, 1);
        assert!(matches!(&w[0].value, Value::Str(_)));
    }

    #[test]
    fn test_witnesses_scalar_int() {
        let int_type = type_lib::int();
        let w = witnesses(&int_type);

        assert_eq!(w.len(), 1);
        assert_eq!(w[0].count, 1);
        assert!(matches!(&w[0].value, Value::Int(_)));
    }

    #[test]
    fn test_witnesses_optional() {
        let opt_type = type_lib::optional(type_lib::string());
        let w = witnesses(&opt_type);

        // Optional (cardinality [0,1]) → count=0 + count=1
        assert_eq!(w.len(), 2);
        assert_eq!(w[0].count, 0);
        assert_eq!(w[0].value, Value::Unit);
        assert_eq!(w[1].count, 1);
    }

    #[test]
    fn test_witnesses_list() {
        let list_type = type_lib::list(type_lib::string());
        let w = witnesses(&list_type);

        // List (cardinality [0,∞)) → count=0, count=1
        assert_eq!(w.len(), 2);
        assert_eq!(w[0].count, 0);
        assert_eq!(w[0].value, Value::List(vec![]));
        assert_eq!(w[1].count, 1);
        assert!(matches!(&w[1].value, Value::List(v) if v.len() == 1));
    }

    #[test]
    fn test_witnesses_non_empty_list() {
        let ne_list = type_lib::non_empty_list(type_lib::string());
        let w = witnesses(&ne_list);

        // NonEmptyList (cardinality [1,∞)) → count=1, count=2
        assert_eq!(w.len(), 2);
        assert_eq!(w[0].count, 1);
        assert_eq!(w[1].count, 2);
    }

    #[test]
    fn test_witnesses_url_has_predicate_refinement() {
        let url_type = type_lib::url();
        let w = witnesses(&url_type);

        assert_eq!(w.len(), 1); // scalar
                                // URL has Matches predicate with "http" — should produce URL-like witness
        if let Value::Str(s) = &w[0].value {
            assert!(s.contains("http"), "URL witness should contain http: {}", s);
        } else {
            panic!("expected string witness for URL type");
        }
    }

    #[test]
    fn test_witnesses_set_type() {
        let set_type = type_lib::set(type_lib::string());
        let w = witnesses(&set_type);

        // Set (cardinality [0,∞)) → count=0, count=1
        assert_eq!(w.len(), 2);
        assert_eq!(w[0].count, 0);
        assert!(matches!(&w[0].value, Value::Set(v) if v.is_empty()));
        assert_eq!(w[1].count, 1);
        assert!(matches!(&w[1].value, Value::Set(v) if v.len() == 1));
    }

    #[test]
    fn test_witnesses_set_deduplicates_many() {
        // Value::set() must deduplicate identical elements to uphold the
        // set uniqueness invariant. many_witnesses() returns duplicates
        // for non-String/Int/Bool scalars (the `other` fallback branch).
        let duplicates = vec![Value::Unit, Value::Unit];
        let deduped = Value::set(duplicates);
        assert!(
            matches!(&deduped, Value::Set(v) if v.len() == 1),
            "Value::set() should deduplicate identical Unit values"
        );

        let json_dups = vec![
            Value::Json(serde_json::json!({"key": "value"})),
            Value::Json(serde_json::json!({"key": "value"})),
        ];
        let deduped_json = Value::set(json_dups);
        assert!(
            matches!(&deduped_json, Value::Set(v) if v.len() == 1),
            "Value::set() should deduplicate identical Json values"
        );

        // Distinct values should be preserved
        let distinct = vec![Value::Int(1), Value::Int(2)];
        let kept = Value::set(distinct);
        assert!(
            matches!(&kept, Value::Set(v) if v.len() == 2),
            "Value::set() should preserve distinct values"
        );
    }
}
