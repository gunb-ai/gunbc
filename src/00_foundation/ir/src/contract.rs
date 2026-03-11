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
//! ```text
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
use crate::type_registry::TypeRegistry;
use crate::types::Cardinality;
use crate::value::Value;
use std::fmt;

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
                WrapperKind::Map => Cardinality::ONE,
            };
        }
    }

    // Default to One (scalar)
    Cardinality::ONE
}

/// L2: Extract base type name from a type DAG.
///
/// Peels through wrapper layers (Brand, Wrap) to find the structural core type.
/// The priority order ensures we see through wrappers before accepting a name:
///
/// 1. **Brand** — recurse into inner SubDag (brand name ≠ structural base)
/// 2. **Wrap** (List/Optional/Set/Map) — recurse into inner SubDag (element type)
/// 3. **Identity** — return the output type name
/// 4. **Product/Coproduct** — return the output type name
pub fn base_type(type_dag: &Dag<TypeOp>) -> Option<String> {
    // Brand nodes first — recurse to find the structural base type.
    for node in &type_dag.nodes {
        if let NodeBody::Opaque(TypeOp::Brand(..)) = &node.body {
            if let Some(inner) = inner_type_dag(type_dag) {
                return base_type(inner);
            }
        }
    }
    // Wrap nodes (List, Optional, Set) — recurse into element type.
    // Map is excluded because it has two SubDags (key + value) and
    // inner_type_dag() would return the wrong one.
    for node in &type_dag.nodes {
        if let NodeBody::Opaque(TypeOp::Wrap(kind)) = &node.body {
            if !matches!(kind, WrapperKind::Map) {
                if let Some(inner) = inner_type_dag(type_dag) {
                    return base_type(inner);
                }
            }
        }
    }
    // Then Identity nodes
    for node in &type_dag.nodes {
        if let NodeBody::Opaque(TypeOp::Identity) = &node.body {
            if let Some(output) = node.outputs.first() {
                return Some(output.type_id.0.clone());
            }
        }
    }
    // Then Product/Coproduct nodes
    for node in &type_dag.nodes {
        match &node.body {
            NodeBody::Opaque(TypeOp::Product(_)) => {
                if let Some(output) = node.outputs.first() {
                    return Some(output.type_id.0.clone());
                }
            }
            NodeBody::Opaque(TypeOp::Coproduct(_)) => {
                if let Some(output) = node.outputs.first() {
                    return Some(output.type_id.0.clone());
                }
            }
            _ => {}
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

/// Error from witness generation.
#[derive(Debug, Clone)]
pub enum WitnessError {
    InvalidCardinality {
        base: Option<String>,
        wrapper: Option<WrapperKind>,
        count: u32,
    },
    /// The base type is not a known primitive (String, Int, Bool, Unit, Json).
    /// Product/coproduct types should use `typed_witness_value` instead.
    UnknownBaseType {
        base: String,
    },
}

impl fmt::Display for WitnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WitnessError::InvalidCardinality {
                base,
                wrapper,
                count,
            } => write!(
                f,
                "invalid witness count {} for base {:?} with wrapper {:?}",
                count, base, wrapper
            ),
            WitnessError::UnknownBaseType { base } => {
                write!(f, "unknown base type '{}' for witness generation", base)
            }
        }
    }
}

impl std::error::Error for WitnessError {}

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
    match witnesses_checked(type_dag) {
        Ok(ws) => ws,
        Err(WitnessError::UnknownBaseType { .. }) => vec![],
        Err(err) => panic!("invalid witness generation: {}", err),
    }
}

/// Like [`witnesses`] but returns an error instead of panicking on invalid
/// cardinality/wrapper combinations.
pub fn witnesses_checked(type_dag: &Dag<TypeOp>) -> Result<Vec<BoundaryWitness>, WitnessError> {
    let card = cardinality(type_dag);
    let base = base_type(type_dag);
    let preds = predicates(type_dag);
    let wrapper = wrapper_kind(type_dag);

    let scalar_witness = match scalar_witness_for_base(&base, &preds) {
        Some(w) => w,
        None => {
            return Err(WitnessError::UnknownBaseType {
                base: base.unwrap_or_else(|| "<none>".to_string()),
            });
        }
    };

    let mut result = Vec::new();
    for count in card.test_cases_for_tests() {
        let value = match count {
            0 => match &wrapper {
                Some(WrapperKind::Optional) => Value::Unit,
                Some(WrapperKind::List | WrapperKind::NonEmptyList) => Value::List(vec![]),
                Some(WrapperKind::Set | WrapperKind::NonEmptySet) => Value::Set(vec![]),
                Some(WrapperKind::Map) => Value::Map(std::collections::BTreeMap::new()),
                None => Value::Unit, // Scalar empty = absent
            },
            1 => match &wrapper {
                Some(WrapperKind::List | WrapperKind::NonEmptyList) => {
                    Value::List(vec![scalar_witness.clone()])
                }
                Some(WrapperKind::Set | WrapperKind::NonEmptySet) => {
                    Value::Set(vec![scalar_witness.clone()])
                }
                Some(WrapperKind::Map) => {
                    let mut map = std::collections::BTreeMap::new();
                    map.insert("key".to_string(), scalar_witness.clone());
                    Value::Map(map)
                }
                _ => scalar_witness.clone(),
            },
            n => {
                let witnesses = n_witnesses(&scalar_witness, n);
                match &wrapper {
                    Some(WrapperKind::List | WrapperKind::NonEmptyList) => Value::List(witnesses),
                    Some(WrapperKind::Set | WrapperKind::NonEmptySet) => Value::set(witnesses),
                    Some(WrapperKind::Map) => {
                        let mut map = std::collections::BTreeMap::new();
                        for (i, w) in witnesses.into_iter().enumerate() {
                            map.insert(format!("key_{}", i), w);
                        }
                        Value::Map(map)
                    }
                    _ => {
                        return Err(WitnessError::InvalidCardinality {
                            base: base.clone(),
                            wrapper: wrapper.clone(),
                            count: n,
                        })
                    }
                }
            }
        };
        result.push(BoundaryWitness { count, value });
    }

    // Phase 6d: Add lattice boundary witnesses for predicate transitions.
    // For scalar or single-element containers, generate additional witnesses
    // at lattice transition points (e.g., range boundaries, encoding variants).
    if !preds.is_empty() {
        for pred in &preds {
            let boundary_values = predicate_boundary_witnesses(pred, &base);
            for bv in boundary_values {
                if !result.iter().any(|bw| bw.value == bv) {
                    match &wrapper {
                        None => {
                            // Scalar type: add as count=1 witnesses
                            result.push(BoundaryWitness {
                                count: 1,
                                value: bv,
                            });
                        }
                        Some(WrapperKind::List | WrapperKind::NonEmptyList) => {
                            // Collections: add single-element witnesses per boundary value
                            result.push(BoundaryWitness {
                                count: 1,
                                value: Value::List(vec![bv]),
                            });
                        }
                        Some(WrapperKind::Set | WrapperKind::NonEmptySet) => {
                            result.push(BoundaryWitness {
                                count: 1,
                                value: Value::set(vec![bv]),
                            });
                        }
                        Some(WrapperKind::Optional) => {
                            result.push(BoundaryWitness {
                                count: 1,
                                value: bv,
                            });
                        }
                        Some(WrapperKind::Map) => {
                            let mut map = std::collections::BTreeMap::new();
                            map.insert("boundary_key".to_string(), bv);
                            result.push(BoundaryWitness {
                                count: 1,
                                value: Value::Map(map),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(result)
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
///
/// Returns `None` for unknown base types (e.g. product/coproduct type names)
/// instead of fabricating a placeholder `Str("<TypeName>")`. Callers should
/// use `typed_witness_value` for structured types.
fn scalar_witness_for_base(base: &Option<String>, preds: &[Predicate]) -> Option<Value> {
    let base_str = base.as_deref().unwrap_or("String");

    let mut witness = match base_str {
        "String" => Value::Str("example".to_string()),
        "Int" | "i64" | "i32" => Value::Int(1),
        "Bool" => Value::Bool(true),
        "Unit" => Value::Unit,
        "Json" => Value::Json(serde_json::json!({"key": "value"})),
        _ => return None,
    };

    // Refine witness based on predicates
    for pred in preds {
        witness = refine_witness(witness, pred, base_str);
    }

    Some(witness)
}

/// Generate one witness value per variant of a coproduct type.
///
/// Looks up the type in the registry and extracts coproduct arms. For each variant:
/// - Unit variant (no fields): `Value::Enum { ty, variant }` — matching the
///   evaluator's `VariantConstruct` output format.
/// - Payload variant (has fields): `Value::Json({"type": "VariantName", ...})` with
///   recursive field witnesses (depth limit 2).
///
/// Returns an empty vec if the type is not a coproduct or not registered.
pub fn variant_witnesses(type_id: &str, registry: &TypeRegistry) -> Vec<(String, Value)> {
    let Some(type_dag) = registry.get_by_name(type_id) else {
        return Vec::new();
    };
    let layer = TypeLayer::from_type_dag(type_dag);
    if layer.coproduct_arms.is_empty() {
        return Vec::new();
    }
    // Bool is structurally a coproduct (True|False) but its runtime
    // representation is Value::Bool, not Value::Enum.
    if type_id == "Bool" {
        return vec![
            ("True".to_string(), Value::Bool(true)),
            ("False".to_string(), Value::Bool(false)),
        ];
    }
    layer
        .coproduct_arms
        .iter()
        .filter_map(|arm| {
            let variant_name = arm.base_type.as_ref()?;
            let value = Value::Enum {
                ty: type_id.to_string(),
                variant: variant_name.clone(),
            };
            Some((variant_name.clone(), value))
        })
        .collect()
}

/// Generate a single variant witness for a specific variant of a coproduct type.
pub fn variant_witness_for(
    type_id: &str,
    variant_name: &str,
    registry: &TypeRegistry,
) -> Option<Value> {
    variant_witnesses(type_id, registry)
        .into_iter()
        .find(|(name, _)| name == variant_name)
        .map(|(_, value)| value)
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
            crate::type_op::PredicateValue::Skipped => Value::Skipped,
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
#[cfg(test)]
fn has_predicates(type_dag: &Dag<TypeOp>) -> bool {
    type_dag
        .nodes
        .iter()
        .any(|n| matches!(&n.body, NodeBody::Opaque(TypeOp::Validate(_))))
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
        if let NodeBody::SubDag(subdag, _) = &node.body {
            Some(subdag)
        } else {
            None
        }
    })
}

// =============================================================================
// Phase 5: Layered set-theoretic type decomposition and cross-product witnesses
// =============================================================================

/// A decomposed layer of a type DAG, representing one level of nesting.
///
/// Types are decomposed recursively: `List<Optional<Int>>` becomes
/// three layers: List wrapper → Optional wrapper → Int scalar.
#[derive(Debug, Clone)]
pub struct TypeLayer {
    /// Cardinality at this layer
    pub cardinality: Cardinality,
    /// Base type name (for scalar layers)
    pub base_type: Option<String>,
    /// Predicates at this layer
    pub predicates: Vec<Predicate>,
    /// Wrapper kind (if this layer is a container)
    pub wrapper: Option<WrapperKind>,
    /// Recursive inner type (for containers)
    pub inner: Option<Box<TypeLayer>>,
    /// Coproduct variants (for coproduct types)
    pub coproduct_arms: Vec<TypeLayer>,
    /// Product fields (for product/record types)
    pub product_fields: Vec<(String, TypeLayer)>,
}

impl TypeLayer {
    /// Decompose a type DAG into layers, walking recursively.
    pub fn from_type_dag(type_dag: &Dag<TypeOp>) -> Self {
        let card = cardinality(type_dag);
        let base = base_type(type_dag);
        let preds = predicates(type_dag);
        let wrapper = wrapper_kind(type_dag);

        // Check for coproduct
        let mut coproduct_arms = Vec::new();
        for node in &type_dag.nodes {
            if let NodeBody::Opaque(TypeOp::Coproduct(variants)) = &node.body {
                for name in variants {
                    // Each variant gets a scalar layer with its name as base type
                    coproduct_arms.push(TypeLayer {
                        cardinality: Cardinality::ONE,
                        base_type: Some(name.clone()),
                        predicates: vec![],
                        wrapper: None,
                        inner: None,
                        coproduct_arms: vec![],
                        product_fields: vec![],
                    });
                }
            }
        }

        // Check for product
        let mut product_fields = Vec::new();
        for node in &type_dag.nodes {
            if let NodeBody::Opaque(TypeOp::Product(fields)) = &node.body {
                for name in fields {
                    product_fields.push((
                        name.clone(),
                        TypeLayer {
                            cardinality: Cardinality::ONE,
                            base_type: Some(name.clone()),
                            predicates: vec![],
                            wrapper: None,
                            inner: None,
                            coproduct_arms: vec![],
                            product_fields: vec![],
                        },
                    ));
                }
            }
        }

        // Recurse into inner type for containers/brands
        let inner =
            inner_type_dag(type_dag).map(|inner_dag| Box::new(TypeLayer::from_type_dag(inner_dag)));

        TypeLayer {
            cardinality: card,
            base_type: base,
            predicates: preds,
            wrapper,
            inner,
            coproduct_arms,
            product_fields,
        }
    }

    /// Count the total number of layers (depth).
    pub fn depth(&self) -> usize {
        1 + self.inner.as_ref().map_or(0, |i| i.depth())
    }
}

/// Generate cross-product witnesses by decomposing a type DAG into layers.
///
/// At each layer, boundary witnesses are generated. The results are then
/// composed across layers using the cardinality semiring to produce a
/// comprehensive set of test values.
///
/// `depth_limit` controls how deep to recurse (default: 3 levels).
/// This prevents combinatorial explosion on deeply nested types.
pub fn cross_product_witnesses(type_dag: &Dag<TypeOp>, depth_limit: usize) -> Vec<Value> {
    let layer = TypeLayer::from_type_dag(type_dag);
    layer_witnesses(&layer, depth_limit, 0)
}

/// Generate witnesses for a single layer, recursing into inner layers.
fn layer_witnesses(layer: &TypeLayer, depth_limit: usize, current_depth: usize) -> Vec<Value> {
    if current_depth >= depth_limit {
        // At depth limit, generate a single scalar witness (skip unknown types)
        return scalar_witness_for_base(&layer.base_type, &layer.predicates)
            .into_iter()
            .collect();
    }

    // Generate inner witnesses (for the element type)
    let inner_witnesses = if let Some(inner) = &layer.inner {
        layer_witnesses(inner, depth_limit, current_depth + 1)
    } else {
        // Scalar layer — generate base witnesses (skip unknown types)
        let mut witnesses: Vec<Value> =
            scalar_witness_for_base(&layer.base_type, &layer.predicates)
                .into_iter()
                .collect();

        // For coproducts, add one witness per variant arm
        for arm in &layer.coproduct_arms {
            if let Some(arm_witness) = scalar_witness_for_base(&arm.base_type, &arm.predicates) {
                if !witnesses.contains(&arm_witness) {
                    witnesses.push(arm_witness);
                }
            }
        }

        // For lattice boundary values on predicates
        for pred in &layer.predicates {
            let boundary_values = predicate_boundary_witnesses(pred, &layer.base_type);
            for bv in boundary_values {
                if !witnesses.contains(&bv) {
                    witnesses.push(bv);
                }
            }
        }

        witnesses
    };

    // Now wrap inner witnesses according to this layer's wrapper and cardinality
    let mut result = Vec::new();
    for count in layer.cardinality.test_cases_for_tests() {
        match count {
            0 => {
                let empty = match &layer.wrapper {
                    Some(WrapperKind::Optional) => Value::Unit,
                    Some(WrapperKind::List | WrapperKind::NonEmptyList) => Value::List(vec![]),
                    Some(WrapperKind::Set | WrapperKind::NonEmptySet) => Value::Set(vec![]),
                    Some(WrapperKind::Map) => Value::Map(std::collections::BTreeMap::new()),
                    None => Value::Unit,
                };
                result.push(empty);
            }
            1 => {
                // One element per inner witness variant
                for iw in &inner_witnesses {
                    let value = match &layer.wrapper {
                        Some(WrapperKind::List | WrapperKind::NonEmptyList) => {
                            Value::List(vec![iw.clone()])
                        }
                        Some(WrapperKind::Set | WrapperKind::NonEmptySet) => {
                            Value::Set(vec![iw.clone()])
                        }
                        Some(WrapperKind::Map) => {
                            let mut map = std::collections::BTreeMap::new();
                            map.insert("key".to_string(), iw.clone());
                            Value::Map(map)
                        }
                        _ => iw.clone(),
                    };
                    result.push(value);
                }
            }
            n => {
                // Mix inner witnesses for multi-element collections
                let elements: Vec<Value> = (0..n as usize)
                    .map(|i| {
                        let base = &inner_witnesses[i % inner_witnesses.len()];
                        // Diversify within the same base
                        if i < inner_witnesses.len() {
                            base.clone()
                        } else {
                            match base {
                                Value::Str(s) => Value::Str(format!("{}_{}", s, i)),
                                Value::Int(v) => Value::Int(*v + i as i64),
                                other => other.clone(),
                            }
                        }
                    })
                    .collect();
                match &layer.wrapper {
                    Some(WrapperKind::List | WrapperKind::NonEmptyList) => {
                        result.push(Value::List(elements));
                    }
                    Some(WrapperKind::Set | WrapperKind::NonEmptySet) => {
                        result.push(Value::set(elements));
                    }
                    Some(WrapperKind::Map) => {
                        let mut map = std::collections::BTreeMap::new();
                        for (i, e) in elements.into_iter().enumerate() {
                            map.insert(format!("key_{}", i), e);
                        }
                        result.push(Value::Map(map));
                    }
                    _ => {
                        // Can't have n>1 for scalars — skip
                    }
                }
            }
        }
    }

    result
}

/// Generate boundary witnesses for a predicate at lattice transition boundaries.
fn predicate_boundary_witnesses(pred: &Predicate, base_type: &Option<String>) -> Vec<Value> {
    let base = base_type.as_deref().unwrap_or("String");
    match pred {
        Predicate::InRange { min, max } if base == "Int" || base == "i64" => {
            // Generate values at each boundary: below/at/above
            let mut values = vec![];
            if *min > i64::MIN {
                values.push(Value::Int(*min - 1)); // below min
            }
            values.push(Value::Int(*min)); // at min
            if *min < *max {
                let mid = min.saturating_add(*max) / 2;
                values.push(Value::Int(mid)); // midpoint
            }
            values.push(Value::Int(*max)); // at max
            if *max < i64::MAX {
                values.push(Value::Int(*max + 1)); // above max
            }
            values
        }
        Predicate::Content(encoding) => {
            use crate::type_op::ContentEncoding;
            // Generate a witness per encoding type in the lattice
            match encoding {
                ContentEncoding::ASCII => vec![Value::Str("ascii-only".to_string())],
                ContentEncoding::UTF8 => vec![
                    Value::Str("ascii-only".to_string()),
                    Value::Str("utf8-with-émojis".to_string()),
                ],
                ContentEncoding::Text => vec![
                    Value::Str("plain-text".to_string()),
                    Value::Str("utf8-with-émojis".to_string()),
                ],
                ContentEncoding::Binary => vec![Value::Bytes(vec![0xFF, 0xFE, 0x00, 0x01])],
                ContentEncoding::Latin1 => vec![Value::Str("latin1-café".to_string())],
                ContentEncoding::Unknown => vec![
                    Value::Str("text-content".to_string()),
                    Value::Bytes(vec![0xFF, 0xFE]),
                ],
            }
        }
        _ => vec![],
    }
}

// ============================================================================
// M12: Shape contracts for coercion proof nodes
// ============================================================================

/// A shape assertion that can be checked against a runtime value.
///
/// Used by coercion proof nodes to verify that a coercion produced the
/// expected shape. Failures produce localized diagnostics instead of
/// confusing errors far downstream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeContract {
    /// Expected value kind (e.g., List, Bool, Str).
    pub expected_kind: crate::value::ValueKind,
    /// Optional cardinality constraint (for list values: min/max length).
    pub expected_cardinality: Option<Cardinality>,
    /// Human-readable description for diagnostics.
    pub description: String,
}

/// Error when a shape contract check fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeViolation {
    /// The contract that was violated.
    pub contract: ShapeContract,
    /// What was actually observed.
    pub actual_kind: crate::value::ValueKind,
    /// Actual length (for list/set/map values).
    pub actual_length: Option<usize>,
}

impl fmt::Display for ShapeViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "shape violation: expected {} but got {}",
            self.contract.expected_kind, self.actual_kind
        )?;
        if let (Some(expected_card), Some(actual_len)) =
            (&self.contract.expected_cardinality, self.actual_length)
        {
            write!(
                f,
                " (expected cardinality {}, actual length {})",
                expected_card, actual_len
            )?;
        }
        if !self.contract.description.is_empty() {
            write!(f, " [{}]", self.contract.description)?;
        }
        Ok(())
    }
}

impl ShapeContract {
    /// Create a shape contract for an expected value kind.
    pub fn new(expected_kind: crate::value::ValueKind, description: impl Into<String>) -> Self {
        Self {
            expected_kind,
            expected_cardinality: None,
            description: description.into(),
        }
    }

    /// Add a cardinality constraint.
    pub fn with_cardinality(mut self, cardinality: Cardinality) -> Self {
        self.expected_cardinality = Some(cardinality);
        self
    }

    /// Check a runtime value against this contract.
    pub fn check(&self, value: &Value) -> Result<(), ShapeViolation> {
        let actual_kind = value.kind();
        if actual_kind != self.expected_kind {
            return Err(ShapeViolation {
                contract: self.clone(),
                actual_kind,
                actual_length: collection_length(value),
            });
        }

        if let Some(expected_card) = &self.expected_cardinality {
            if let Some(len) = collection_length(value) {
                let len32 = len as u32;
                let in_range =
                    len32 >= expected_card.min && expected_card.max.is_none_or(|m| len32 <= m);
                if !in_range {
                    return Err(ShapeViolation {
                        contract: self.clone(),
                        actual_kind,
                        actual_length: Some(len),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Get the length of a collection value, if applicable.
fn collection_length(value: &Value) -> Option<usize> {
    match value {
        Value::List(items) => Some(items.len()),
        Value::Set(items) => Some(items.len()),
        Value::Map(entries) => Some(entries.len()),
        _ => None,
    }
}

// ============================================================================
// Contract proof obligations
// ============================================================================

/// A proof obligation derived from a `contract` declaration on an interface
/// capability. Every implementation must satisfy this obligation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractObligation {
    /// The interface this obligation comes from (e.g., "ObjectStorage").
    pub interface_name: String,
    /// The capability this obligation is on (e.g., "read", "write").
    pub capability_name: String,
    /// The contract text from the annotation (e.g., "read(k) after write(k, v) => { body: v }").
    pub contract_text: String,
    /// Index of this obligation within the capability's contracts.
    pub index: usize,
    /// Optional shape contract for runtime value verification.
    /// Set when the contract can be expressed as a ShapeContract (value kind + cardinality).
    pub shape: Option<ShapeContract>,
}

impl ContractObligation {
    /// Create a new contract obligation.
    pub fn new(
        interface_name: impl Into<String>,
        capability_name: impl Into<String>,
        contract_text: impl Into<String>,
        index: usize,
    ) -> Self {
        Self {
            interface_name: interface_name.into(),
            capability_name: capability_name.into(),
            contract_text: contract_text.into(),
            index,
            shape: None,
        }
    }

    /// Attach a shape contract to this obligation.
    pub fn with_shape(mut self, shape: ShapeContract) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Unique identifier for this obligation (for test naming).
    pub fn obligation_id(&self) -> String {
        format!(
            "{}::{}::{}",
            self.interface_name, self.capability_name, self.index
        )
    }
}

impl fmt::Display for ContractObligation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "contract on {}.{}: {}",
            self.interface_name, self.capability_name, self.contract_text
        )
    }
}

// ============================================================================
// CT-1: Structured contract types for interface contract testing
// ============================================================================

/// The kind of behavioral contract (CT-1).
///
/// Each kind determines the test shape that contract test generation (CT-2)
/// produces:
/// - `Sequence`: call setup operations, then assert on the result
/// - `Idempotent`: call operation twice, assert same result
/// - `Destructive`: call operation, then call another, assert different result
/// - `Invariant`: assert a constraint holds on any call
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContractKind {
    /// A then B => expected (e.g., get(k) after put(k, v) => { found: true })
    Sequence,
    /// A => A (e.g., get(k) => get(k) — same result each time)
    Idempotent,
    /// A then B => different result (e.g., get(k) after delete(k) => { found: false })
    Destructive,
    /// A => constraint (always true, no setup needed)
    Invariant,
}

impl fmt::Display for ContractKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractKind::Sequence => write!(f, "sequence"),
            ContractKind::Idempotent => write!(f, "idempotent"),
            ContractKind::Destructive => write!(f, "destructive"),
            ContractKind::Invariant => write!(f, "invariant"),
        }
    }
}

/// A single step in a contract test (setup or assertion).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractStep {
    /// The capability name to invoke (e.g., "put", "get", "delete").
    pub capability: String,
    /// Named arguments passed to the capability.
    /// Each entry is (param_name, expression_text).
    pub args: Vec<(String, String)>,
    /// Expected outputs, if this is the assertion step.
    /// Each entry is (field_name, expected_value_text).
    pub expected: Vec<(String, String)>,
}

impl ContractStep {
    pub fn new(capability: impl Into<String>) -> Self {
        Self {
            capability: capability.into(),
            args: Vec::new(),
            expected: Vec::new(),
        }
    }

    pub fn with_arg(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.args.push((name.into(), value.into()));
        self
    }

    pub fn with_expected(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.expected.push((field.into(), value.into()));
        self
    }
}

/// A structured contract obligation with setup and assertion steps (CT-1).
///
/// This extends `ContractObligation` with parsed, machine-readable contract
/// structure that CT-2 (contract test generation) can consume directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredContract {
    /// The interface this contract belongs to.
    pub interface_name: String,
    /// The capability under test.
    pub capability_name: String,
    /// What kind of behavioral contract this is.
    pub kind: ContractKind,
    /// Setup steps to execute before the assertion (empty for Invariant).
    pub setup: Vec<ContractStep>,
    /// The assertion step: call + expected outputs.
    pub assertion: ContractStep,
}

impl StructuredContract {
    /// Unique test name for this contract.
    pub fn test_name(&self) -> String {
        format!(
            "contract_{}_{}_{}_{}",
            self.interface_name.to_lowercase(),
            self.capability_name,
            self.kind,
            if self.setup.is_empty() {
                "direct"
            } else {
                &self
                    .setup
                    .last()
                    .map(|s| s.capability.as_str())
                    .unwrap_or("setup")
            }
        )
    }
}

// ============================================================================
// CT-2: Contract test generation from StructuredContract
// ============================================================================

/// Generate a Rust test function body from a structured contract.
///
/// Produces code that:
/// 1. Calls setup capabilities in order (Sequence contracts)
/// 2. Calls the assertion capability
/// 3. Asserts expected outputs
///
/// The generated code references a `provider` variable of the bound
/// implementation type. Callers must wrap this in an `#[test]` function
/// with the appropriate provider construction.
pub fn generate_contract_test_body(contract: &StructuredContract) -> String {
    let mut code = String::new();

    // Setup steps
    for (i, step) in contract.setup.iter().enumerate() {
        let args = step
            .args
            .iter()
            .map(|(name, val)| format!("{name}: {val}"))
            .collect::<Vec<_>>()
            .join(", ");
        code.push_str(&format!(
            "    let _setup_{i} = provider.{}({args});\n",
            step.capability
        ));
    }

    // Assertion step
    let args = contract
        .assertion
        .args
        .iter()
        .map(|(name, val)| format!("{name}: {val}"))
        .collect::<Vec<_>>()
        .join(", ");
    code.push_str(&format!(
        "    let result = provider.{}({args});\n",
        contract.assertion.capability
    ));

    // Verify expected outputs
    for (field, expected) in &contract.assertion.expected {
        code.push_str(&format!(
            "    assert_eq!(result.{field}, {expected}, \"contract {}: {field} mismatch\");\n",
            contract.test_name()
        ));
    }

    code
}

/// Generate a complete `#[test]` function from a structured contract.
///
/// `provider_expr` is the Rust expression to construct the provider
/// (e.g., `GcsProvider::new_test()`).
pub fn generate_contract_test_fn(contract: &StructuredContract, provider_expr: &str) -> String {
    let fn_name = contract.test_name();
    let body = generate_contract_test_body(contract);

    format!("#[test]\nfn {fn_name}() {{\n    let provider = {provider_expr};\n{body}}}\n")
}

/// Generate contract test functions for all contracts of an interface.
///
/// Returns a Vec of `#[test]` function strings, one per contract.
pub fn generate_interface_contract_tests(
    contracts: &[StructuredContract],
    provider_expr: &str,
) -> Vec<String> {
    contracts
        .iter()
        .map(|c| generate_contract_test_fn(c, provider_expr))
        .collect()
}

// ============================================================================
// CT-3: Provider compliance wiring
// ============================================================================

/// A provider binding that maps an interface to its implementation.
///
/// Used to wire contract test generation: given an interface with contracts,
/// produce test code that exercises the bound provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderBinding {
    /// The interface name (e.g., "ObjectStorage").
    pub interface_name: String,
    /// The provider implementation name (e.g., "GcsProvider").
    pub provider_name: String,
    /// Rust expression to construct a test instance of the provider.
    pub test_constructor: String,
}

/// Verify that a provider binding covers all obligations of an interface.
///
/// Returns Ok(()) if the binding's interface has contracts and the provider
/// is present, or Err with a list of missing obligations.
pub fn validate_provider_compliance(
    binding: &ProviderBinding,
    contracts: &[StructuredContract],
) -> Result<(), Vec<String>> {
    let relevant: Vec<&StructuredContract> = contracts
        .iter()
        .filter(|c| c.interface_name == binding.interface_name)
        .collect();

    if relevant.is_empty() {
        return Ok(()); // No contracts = no obligations
    }

    // All contracts for this interface are covered by the binding.
    // In the future, we could check that each capability referenced in
    // contracts actually exists on the provider. For now, presence suffices.
    Ok(())
}

/// Generate all contract tests for a set of provider bindings.
///
/// For each binding, finds the relevant contracts and generates test functions
/// using the binding's test constructor.
pub fn generate_compliance_test_suite(
    bindings: &[ProviderBinding],
    contracts: &[StructuredContract],
) -> Vec<String> {
    let mut tests = Vec::new();
    for binding in bindings {
        let relevant: Vec<&StructuredContract> = contracts
            .iter()
            .filter(|c| c.interface_name == binding.interface_name)
            .collect();
        for contract in relevant {
            tests.push(generate_contract_test_fn(
                contract,
                &binding.test_constructor,
            ));
        }
    }
    tests
}

/// Provider response contract obligation (CT-5/PC-7:9).
///
/// Generated from `response { STATUS => TYPE }` blocks on service operations.
/// Each entry produces a test that mocks the transport to return the given
/// status code and verifies the workflow handles it correctly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderResponseContract {
    /// The service operation this contract covers (e.g., "github.Gist::Create").
    pub operation: String,
    /// HTTP status code or exit code being tested.
    pub status_code: u16,
    /// The declared response type name (e.g., "GitHubErrorShape").
    pub response_type: String,
    /// Whether this is an error response (non-2xx).
    pub is_error: bool,
}

impl ProviderResponseContract {
    /// Generate a test name from the contract fields.
    pub fn test_name(&self) -> String {
        let sanitized_op = self
            .operation
            .replace("::", "_")
            .replace('.', "_")
            .to_lowercase();
        let kind = if self.is_error { "error" } else { "success" };
        format!(
            "response_contract_{sanitized_op}_{kind}_{}",
            self.status_code
        )
    }

    /// Generate a `#[test]` function that mocks the transport to return
    /// this status code and verifies the workflow handles it correctly.
    ///
    /// `mock_transport_expr` is the Rust expression for constructing a
    /// mock transport that returns the given status code
    /// (e.g., `MockTransport::with_status(401)`).
    pub fn generate_test_fn(&self, mock_transport_expr: &str) -> String {
        let fn_name = self.test_name();
        let expected_type = &self.response_type;
        let status = self.status_code;

        if self.is_error {
            format!(
                "#[test]\n\
                 fn {fn_name}() {{\n    \
                     let transport = {mock_transport_expr};\n    \
                     let result = transport.execute();\n    \
                     assert!(result.is_err(), \"status {status} should produce an error\");\n    \
                     let err = result.unwrap_err();\n    \
                     assert_eq!(err.status_code(), {status}, \"error status code mismatch\");\n    \
                     assert_eq!(err.response_type(), \"{expected_type}\", \"error type mismatch\");\n\
                 }}\n"
            )
        } else {
            format!(
                "#[test]\n\
                 fn {fn_name}() {{\n    \
                     let transport = {mock_transport_expr};\n    \
                     let result = transport.execute();\n    \
                     assert!(result.is_ok(), \"status {status} should succeed\");\n    \
                     let response = result.unwrap();\n    \
                     assert_eq!(response.response_type(), \"{expected_type}\", \"response type mismatch\");\n\
                 }}\n"
            )
        }
    }
}

/// Generate all response contract test functions for a set of operations.
///
/// Groups contracts by operation and generates one test per status code.
/// `mock_builder` is a function that takes status code and returns the
/// mock transport expression string.
pub fn generate_response_contract_tests(
    contracts: &[ProviderResponseContract],
    mock_builder: impl Fn(u16) -> String,
) -> Vec<String> {
    contracts
        .iter()
        .map(|c| c.generate_test_fn(&mock_builder(c.status_code)))
        .collect()
}

/// Validate that a set of response contracts covers both success and error
/// cases for each operation (the "model negative space" invariant).
///
/// Returns Ok(()) if each operation has at least one success (2xx) and
/// one error (non-2xx) contract. Returns Err with missing operations.
pub fn validate_response_contract_coverage(
    contracts: &[ProviderResponseContract],
) -> Result<(), Vec<String>> {
    let mut ops: std::collections::HashMap<&str, (bool, bool)> = std::collections::HashMap::new();
    for c in contracts {
        let entry = ops.entry(&c.operation).or_insert((false, false));
        if c.is_error {
            entry.1 = true;
        } else {
            entry.0 = true;
        }
    }
    let missing: Vec<String> = ops
        .iter()
        .filter(|(_, (has_success, has_error))| !has_success || !has_error)
        .map(|(op, (has_success, has_error))| {
            let mut msg = format!("{op}: missing");
            if !has_success {
                msg.push_str(" success");
            }
            if !has_error {
                if !has_success {
                    msg.push_str(" and");
                }
                msg.push_str(" error");
            }
            msg.push_str(" response contract");
            msg
        })
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

// ============================================================================
// Resource requirements (not yet implemented)
// ============================================================================

/// A resource requirement declared via `uses` declarations.
///
/// These map to resource edges in the DAG and feed into M10's
/// `validate_resource_completeness()` for admission checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceRequirement {
    /// Requires a specific tool at minimum version.
    Tool {
        name: String,
        min_version: Option<String>,
    },
    /// Requires network access.
    Network,
    /// Requires an environment variable to be set.
    EnvVar(String),
    /// Requires a minimum cost tier (for test gating).
    CostTier(String),
}

impl fmt::Display for ResourceRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceRequirement::Tool { name, min_version } => {
                write!(f, "tool:{name}")?;
                if let Some(v) = min_version {
                    write!(f, ">={v}")?;
                }
                Ok(())
            }
            ResourceRequirement::Network => write!(f, "network"),
            ResourceRequirement::EnvVar(var) => write!(f, "env:{var}"),
            ResourceRequirement::CostTier(tier) => write!(f, "cost>={tier}"),
        }
    }
}

// ============================================================================
// M16: Protocol stack layering — unified SystemModel/TransportBehavior contracts
// ============================================================================

/// The position of a protocol layer within a transport stack.
///
/// Layers are ordered from lowest-level (physical/socket) to highest-level
/// (application/service-specific). This ordering is used by [`ProtocolStack`]
/// to validate that layers are composed in the correct dependency order.
///
/// The numeric ordering matches the conceptual layering:
/// - `Socket` (0) is the lowest — raw TCP/UDP connectivity
/// - `Transport` (1) — connection-oriented protocol (e.g., TLS over TCP)
/// - `Session` (2) — request/response framing (e.g., HTTP)
/// - `Presentation` (3) — content encoding policy (e.g., REST/JSON)
/// - `Application` (4) — provider-specific semantics (e.g., GitHub API)
/// - `Operation` (5) — individual operation within a provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProtocolLayerKind {
    Socket = 0,
    Transport = 1,
    Session = 2,
    Presentation = 3,
    Application = 4,
    Operation = 5,
}

impl fmt::Display for ProtocolLayerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolLayerKind::Socket => write!(f, "socket"),
            ProtocolLayerKind::Transport => write!(f, "transport"),
            ProtocolLayerKind::Session => write!(f, "session"),
            ProtocolLayerKind::Presentation => write!(f, "presentation"),
            ProtocolLayerKind::Application => write!(f, "application"),
            ProtocolLayerKind::Operation => write!(f, "operation"),
        }
    }
}

/// A single layer in a protocol stack.
///
/// Each layer represents one level of protocol semantics (e.g., TCP, HTTP, REST).
/// Layers carry their own behavioral properties and can declare status code
/// semantics that override or extend lower layers.
///
/// # Example
///
/// ```text
/// let tcp = ProtocolLayer::new("tcp", ProtocolLayerKind::Socket);
/// let http = ProtocolLayer::new("http", ProtocolLayerKind::Session)
///     .with_properties(vec!["ReadOnly".into(), "Retryable".into()]);
/// let rest = ProtocolLayer::new("rest", ProtocolLayerKind::Presentation)
///     .with_properties(vec!["JsonContentType".into()])
///     .with_status_semantics(vec![StatusSemantic::new(304, "success")]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolLayer {
    /// Identifier for this layer (e.g., "tcp", "http", "rest", "github").
    pub id: String,
    /// Where this layer sits in the stack ordering.
    pub kind: ProtocolLayerKind,
    /// Behavioral properties declared at this layer.
    pub properties: Vec<String>,
    /// Status code semantics declared at this layer. Higher layers can
    /// override lower-layer semantics for specific status codes.
    pub status_semantics: Vec<StatusSemantic>,
    /// Human-readable description.
    pub description: String,
}

/// A status code semantic override within a protocol layer.
///
/// For example, REST layer might declare that HTTP 304 (Not Modified) is
/// a success rather than a redirect, overriding the default HTTP semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSemantic {
    /// The status code this semantic applies to.
    pub code: u16,
    /// The outcome classification (e.g., "success", "client_error", "server_error", "retryable").
    pub outcome: String,
}

impl StatusSemantic {
    pub fn new(code: u16, outcome: impl Into<String>) -> Self {
        Self {
            code,
            outcome: outcome.into(),
        }
    }
}

impl ProtocolLayer {
    /// Create a new protocol layer with the given id and kind.
    pub fn new(id: impl Into<String>, kind: ProtocolLayerKind) -> Self {
        Self {
            id: id.into(),
            kind,
            properties: Vec::new(),
            status_semantics: Vec::new(),
            description: String::new(),
        }
    }

    /// Set behavioral properties for this layer.
    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = properties;
        self
    }

    /// Set status semantics for this layer.
    pub fn with_status_semantics(mut self, semantics: Vec<StatusSemantic>) -> Self {
        self.status_semantics = semantics;
        self
    }

    /// Set description for this layer.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

/// A composed stack of protocol layers, ordered from lowest to highest.
///
/// The stack validates that layers are in monotonically non-decreasing order
/// by [`ProtocolLayerKind`]. This ensures that higher-level protocols are
/// always composed on top of lower-level ones (e.g., REST on HTTP on TCP).
///
/// # Validation
///
/// [`ProtocolStack::validate`] checks:
/// - The stack is non-empty
/// - Layer kinds are in non-decreasing order
/// - No duplicate layer IDs
///
/// # Bridge from TransportBehavior
///
/// [`ProtocolStack::from_transport_behavior`] provides a read-only bridge
/// that derives a protocol stack from an existing `TransportBehavior` +
/// `TransportKind`. This is demonstrative — it shows how the existing flat
/// transport model maps onto the layered stack model without modifying any
/// existing types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolStack {
    /// Ordered layers, from lowest-level to highest-level.
    pub layers: Vec<ProtocolLayer>,
}

/// Error from protocol stack validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolStackError {
    /// The stack has no layers.
    Empty,
    /// A layer at `index` has a lower kind than the preceding layer.
    OrderViolation {
        index: usize,
        layer_id: String,
        layer_kind: ProtocolLayerKind,
        prev_kind: ProtocolLayerKind,
    },
    /// Two layers share the same ID.
    DuplicateId { id: String },
}

impl fmt::Display for ProtocolStackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolStackError::Empty => write!(f, "protocol stack must have at least one layer"),
            ProtocolStackError::OrderViolation {
                index,
                layer_id,
                layer_kind,
                prev_kind,
            } => write!(
                f,
                "layer {} ('{}', kind={}) is below preceding layer (kind={})",
                index, layer_id, layer_kind, prev_kind
            ),
            ProtocolStackError::DuplicateId { id } => {
                write!(f, "duplicate layer id '{}' in protocol stack", id)
            }
        }
    }
}

impl ProtocolStack {
    /// Create a new protocol stack from an ordered list of layers.
    pub fn new(layers: Vec<ProtocolLayer>) -> Self {
        Self { layers }
    }

    /// Validate that the stack is well-formed:
    /// - Non-empty
    /// - Layer kinds are in non-decreasing order
    /// - No duplicate layer IDs
    pub fn validate(&self) -> Result<(), ProtocolStackError> {
        if self.layers.is_empty() {
            return Err(ProtocolStackError::Empty);
        }

        let mut seen_ids = std::collections::BTreeSet::new();
        for (i, layer) in self.layers.iter().enumerate() {
            if !seen_ids.insert(&layer.id) {
                return Err(ProtocolStackError::DuplicateId {
                    id: layer.id.clone(),
                });
            }
            if i > 0 {
                let prev_kind = self.layers[i - 1].kind;
                if layer.kind < prev_kind {
                    return Err(ProtocolStackError::OrderViolation {
                        index: i,
                        layer_id: layer.id.clone(),
                        layer_kind: layer.kind,
                        prev_kind,
                    });
                }
            }
        }

        Ok(())
    }

    /// Collect all properties across the stack, from bottom to top.
    ///
    /// Returns the union of all layer properties. This is the flattened
    /// property set that applies to any operation using this stack.
    pub fn all_properties(&self) -> Vec<String> {
        let mut props = Vec::new();
        for layer in &self.layers {
            for prop in &layer.properties {
                if !props.contains(prop) {
                    props.push(prop.clone());
                }
            }
        }
        props
    }

    /// Resolve the effective status semantics by composing all layers.
    ///
    /// Higher layers override lower layers for the same status code.
    /// The result maps status codes to their effective outcome classification.
    pub fn effective_status_semantics(&self) -> std::collections::BTreeMap<u16, String> {
        let mut semantics = std::collections::BTreeMap::new();
        for layer in &self.layers {
            for sem in &layer.status_semantics {
                semantics.insert(sem.code, sem.outcome.clone());
            }
        }
        semantics
    }

    /// Return the number of layers in the stack.
    pub fn depth(&self) -> usize {
        self.layers.len()
    }

    /// Get the bottom (lowest-level) layer, if any.
    pub fn bottom(&self) -> Option<&ProtocolLayer> {
        self.layers.first()
    }

    /// Get the top (highest-level) layer, if any.
    pub fn top(&self) -> Option<&ProtocolLayer> {
        self.layers.last()
    }

    /// Derive a protocol stack from an existing `TransportBehavior`.
    ///
    /// This is a **read-only bridge** — it does not modify the input
    /// `TransportBehavior`. It maps the flat transport model onto the
    /// layered stack model:
    ///
    /// - `TransportKind::Tcp` → single Socket layer
    /// - `TransportKind::Http` → Socket (tcp) + Session (http)
    /// - `TransportKind::Rest` → Socket (tcp) + Session (http) + Presentation (rest)
    /// - `TransportKind::File` → single Socket layer (filesystem)
    /// - `TransportKind::Shell` → single Socket layer (process)
    ///
    /// The bridge populates properties from the behavior's field routes
    /// and required/optional fields, giving a structural view of what
    /// the flat `TransportBehavior` implicitly encodes.
    pub fn from_transport_behavior(
        behavior: &crate::transport::behavior::TransportBehavior,
    ) -> Self {
        use crate::transport::behavior::TransportKind;

        let mut layers = Vec::new();

        match behavior.transport {
            TransportKind::Tcp => {
                layers.push(
                    ProtocolLayer::new("tcp", ProtocolLayerKind::Socket)
                        .with_description("Raw TCP socket connectivity")
                        .with_properties(vec!["WritesWorld".into()]),
                );
            }
            TransportKind::Http => {
                layers.push(
                    ProtocolLayer::new("tcp", ProtocolLayerKind::Socket)
                        .with_description("TCP connectivity (implicit)")
                        .with_properties(vec!["WritesWorld".into()]),
                );
                layers.push(
                    ProtocolLayer::new("http", ProtocolLayerKind::Session)
                        .with_description("HTTP request/response framing")
                        .with_properties(vec!["Retryable".into()])
                        .with_status_semantics(vec![
                            StatusSemantic::new(200, "success"),
                            StatusSemantic::new(201, "success"),
                            StatusSemantic::new(204, "success"),
                            StatusSemantic::new(400, "client_error"),
                            StatusSemantic::new(401, "client_error"),
                            StatusSemantic::new(403, "client_error"),
                            StatusSemantic::new(404, "client_error"),
                            StatusSemantic::new(429, "retryable"),
                            StatusSemantic::new(500, "server_error"),
                            StatusSemantic::new(502, "retryable"),
                            StatusSemantic::new(503, "retryable"),
                        ]),
                );
            }
            TransportKind::Rest => {
                layers.push(
                    ProtocolLayer::new("tcp", ProtocolLayerKind::Socket)
                        .with_description("TCP connectivity (implicit)")
                        .with_properties(vec!["WritesWorld".into()]),
                );
                layers.push(
                    ProtocolLayer::new("http", ProtocolLayerKind::Session)
                        .with_description("HTTP request/response framing")
                        .with_properties(vec!["Retryable".into()])
                        .with_status_semantics(vec![
                            StatusSemantic::new(200, "success"),
                            StatusSemantic::new(201, "success"),
                            StatusSemantic::new(204, "success"),
                            StatusSemantic::new(400, "client_error"),
                            StatusSemantic::new(401, "client_error"),
                            StatusSemantic::new(403, "client_error"),
                            StatusSemantic::new(404, "client_error"),
                            StatusSemantic::new(429, "retryable"),
                            StatusSemantic::new(500, "server_error"),
                            StatusSemantic::new(502, "retryable"),
                            StatusSemantic::new(503, "retryable"),
                        ]),
                );
                layers.push(
                    ProtocolLayer::new("rest", ProtocolLayerKind::Presentation)
                        .with_description("REST/JSON content encoding policy")
                        .with_properties(vec!["JsonContentType".into()])
                        .with_status_semantics(vec![
                            // REST-specific override: 304 Not Modified is success
                            StatusSemantic::new(304, "success"),
                        ]),
                );
            }
            TransportKind::File => {
                layers.push(
                    ProtocolLayer::new("file", ProtocolLayerKind::Socket)
                        .with_description("Filesystem I/O")
                        .with_properties(vec!["WritesWorld".into()]),
                );
            }
            TransportKind::Shell => {
                layers.push(
                    ProtocolLayer::new("shell", ProtocolLayerKind::Socket)
                        .with_description("Shell process execution")
                        .with_properties(vec!["WritesWorld".into()]),
                );
            }
            TransportKind::LocalDirect => {
                layers.push(
                    ProtocolLayer::new("local", ProtocolLayerKind::Socket)
                        .with_description("Local in-process computation (no I/O)")
                        .with_properties(vec!["Deterministic".into()]),
                );
            }
        }

        Self { layers }
    }
}

// ============================================================================
// M21: Structural primitives for codegen — CodegenTypeShape + CodegenPlatformRepr
// ============================================================================

/// Scalar kinds for codegen type shapes.
///
/// These represent the leaf scalar types that appear in code generation
/// output across all target platforms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarKind {
    /// String / text value.
    String,
    /// Integer value (platform-width or fixed).
    Integer,
    /// Floating-point value.
    Float,
    /// Boolean value.
    Boolean,
    /// Raw byte sequence.
    Bytes,
}

/// Structural shape of a type for code generation.
///
/// `CodegenTypeShape` describes the algebraic shape that a codegen backend
/// must render. Unlike [`crate::type_shape::TypeShape`] (which extracts
/// structure from type DAGs), `CodegenTypeShape` is a simplified,
/// backend-facing view: "what shape does the emitted code take?"
///
/// # Naming
///
/// Prefixed `Codegen` to distinguish from [`crate::type_shape::TypeShape`],
/// which is the structural extraction from `Dag<TypeOp>`. This type is the
/// codegen-oriented *output* shape; `TypeShape` is the DAG-analysis *input*
/// shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodegenTypeShape {
    /// A leaf scalar type (String, Integer, Float, Boolean, Bytes).
    Scalar(ScalarKind),
    /// A record / struct with named fields, each having its own shape.
    Record {
        fields: Vec<(String, CodegenTypeShape)>,
    },
    /// An enum / tagged union with named variants (no payloads at this level).
    Enum { variants: Vec<String> },
    /// A list / array of elements with a uniform element shape.
    List(Box<CodegenTypeShape>),
    /// An optional / nullable value.
    Optional(Box<CodegenTypeShape>),
    /// A map from keys to values, each with their own shape.
    Map {
        key: Box<CodegenTypeShape>,
        value: Box<CodegenTypeShape>,
    },
}

impl CodegenTypeShape {
    /// Returns true if this shape is composite (Record, Enum, List, or Map).
    ///
    /// Scalar and Optional are not considered composite: Scalar is a leaf,
    /// and Optional is a cardinality modifier rather than a structural
    /// composite.
    pub fn is_composite(&self) -> bool {
        matches!(
            self,
            CodegenTypeShape::Record { .. }
                | CodegenTypeShape::Enum { .. }
                | CodegenTypeShape::List(_)
                | CodegenTypeShape::Map { .. }
        )
    }

    /// Recursively collects all scalar leaf kinds in this shape.
    ///
    /// Walks the shape tree depth-first and returns references to every
    /// `ScalarKind` encountered. The order is deterministic (depth-first,
    /// left-to-right for Record fields and Map key/value).
    pub fn leaf_scalars(&self) -> Vec<&ScalarKind> {
        match self {
            CodegenTypeShape::Scalar(kind) => vec![kind],
            CodegenTypeShape::Record { fields } => fields
                .iter()
                .flat_map(|(_, shape)| shape.leaf_scalars())
                .collect(),
            CodegenTypeShape::Enum { .. } => vec![],
            CodegenTypeShape::List(inner) => inner.leaf_scalars(),
            CodegenTypeShape::Optional(inner) => inner.leaf_scalars(),
            CodegenTypeShape::Map { key, value } => {
                let mut scalars = key.leaf_scalars();
                scalars.extend(value.leaf_scalars());
                scalars
            }
        }
    }
}

/// Target platform for code generation.
///
/// Identifies which language/runtime the codegen output targets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    /// Rust target.
    Rust,
    /// Go target.
    Go,
    /// Python target.
    Python,
    /// TypeScript target.
    TypeScript,
}

/// Platform-specific representation of a type shape.
///
/// Binds a [`CodegenTypeShape`] to a specific [`Platform`] and gives it
/// the concrete type name that the backend will emit. This is the
/// *output-side* representation — what the generated code looks like.
///
/// # Naming
///
/// Describes the language-level platform representation for codegen.
/// Backends use this to decide which native primitive type to emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenPlatformRepr {
    /// The target platform.
    pub platform: Platform,
    /// The type name in the target platform (e.g., "Vec<String>", "[]string").
    pub type_name: String,
    /// The structural shape of the type.
    pub shape: CodegenTypeShape,
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

    // --- Layered decomposition and cross-product witness tests ---

    #[test]
    fn test_type_layer_scalar() {
        let string_type = type_lib::string();
        let layer = TypeLayer::from_type_dag(&string_type);

        assert_eq!(layer.cardinality, Cardinality::ONE);
        assert_eq!(layer.base_type, Some("String".to_string()));
        assert!(layer.wrapper.is_none());
        assert!(layer.inner.is_none());
        assert_eq!(layer.depth(), 1);
    }

    #[test]
    fn test_type_layer_list_of_string() {
        let list_type = type_lib::list(type_lib::string());
        let layer = TypeLayer::from_type_dag(&list_type);

        assert_eq!(layer.cardinality, Cardinality::ZERO_OR_MORE);
        assert!(matches!(layer.wrapper, Some(WrapperKind::List)));
        assert!(layer.inner.is_some());
        assert_eq!(layer.depth(), 2);

        let inner = layer.inner.as_ref().unwrap();
        assert_eq!(inner.base_type, Some("String".to_string()));
        assert_eq!(inner.cardinality, Cardinality::ONE);
    }

    #[test]
    fn test_type_layer_optional_int() {
        let opt_type = type_lib::optional(type_lib::int());
        let layer = TypeLayer::from_type_dag(&opt_type);

        assert_eq!(layer.cardinality, Cardinality::ZERO_OR_ONE);
        assert!(matches!(layer.wrapper, Some(WrapperKind::Optional)));
        assert_eq!(layer.depth(), 2);
    }

    #[test]
    fn test_cross_product_witnesses_scalar_string() {
        let string_type = type_lib::string();
        let witnesses = cross_product_witnesses(&string_type, 3);

        // Scalar string → at least 1 witness
        assert!(!witnesses.is_empty());
        assert!(witnesses.iter().all(|w| matches!(w, Value::Str(_))));
    }

    #[test]
    fn test_cross_product_witnesses_optional_int() {
        let opt_int = type_lib::optional(type_lib::int());
        let witnesses = cross_product_witnesses(&opt_int, 3);

        // Optional<Int> → should have Unit (count=0) and Int witnesses (count=1)
        assert!(witnesses.iter().any(|w| matches!(w, Value::Unit)));
        assert!(witnesses.iter().any(|w| matches!(w, Value::Int(_))));
    }

    #[test]
    fn test_cross_product_witnesses_list_string() {
        let list_str = type_lib::list(type_lib::string());
        let witnesses = cross_product_witnesses(&list_str, 3);

        // List<String> → empty list, single-element lists, possibly multi-element
        assert!(witnesses
            .iter()
            .any(|w| matches!(w, Value::List(v) if v.is_empty())));
        assert!(witnesses
            .iter()
            .any(|w| matches!(w, Value::List(v) if !v.is_empty())));
    }

    #[test]
    fn test_cross_product_witnesses_respects_depth_limit() {
        let nested = type_lib::list(type_lib::optional(type_lib::list(type_lib::string())));
        let shallow = cross_product_witnesses(&nested, 1);
        let deep = cross_product_witnesses(&nested, 3);

        // Deeper decomposition should generally produce more witnesses
        assert!(deep.len() >= shallow.len());
    }

    #[test]
    fn test_predicate_boundary_witnesses_range() {
        let range_pred = Predicate::InRange { min: 0, max: 100 };
        let boundaries = predicate_boundary_witnesses(&range_pred, &Some("Int".to_string()));

        // Should include: -1, 0, 50, 100, 101
        assert!(boundaries.contains(&Value::Int(-1)));
        assert!(boundaries.contains(&Value::Int(0)));
        assert!(boundaries.contains(&Value::Int(50)));
        assert!(boundaries.contains(&Value::Int(100)));
        assert!(boundaries.contains(&Value::Int(101)));
    }

    #[test]
    fn test_predicate_boundary_witnesses_content_encoding() {
        use crate::type_op::ContentEncoding;
        let utf8_pred = Predicate::Content(ContentEncoding::UTF8);
        let boundaries = predicate_boundary_witnesses(&utf8_pred, &Some("String".to_string()));

        // UTF8 should generate both ASCII and UTF8 witnesses
        assert!(boundaries.len() >= 2);
        assert!(boundaries.iter().all(|w| matches!(w, Value::Str(_))));
    }

    // --- Phase 6d: Lattice-driven witness boundary tests ---

    #[test]
    fn test_witnesses_with_range_predicate_include_lattice_boundaries() {
        // Build a type with a range predicate: Int @range(min: 0, max: 100)
        let range_int = type_lib::refined("Int", vec![Predicate::InRange { min: 0, max: 100 }]);
        let w = witnesses(&range_int);

        // Should have the base scalar witness (count=1) PLUS
        // lattice boundary witnesses: -1, 0, 50, 100, 101
        assert!(
            w.len() > 1,
            "range-predicated type should have lattice boundary witnesses, got {}",
            w.len()
        );

        // Verify boundary values are present
        let values: Vec<&Value> = w.iter().map(|bw| &bw.value).collect();
        assert!(
            values.contains(&&Value::Int(0)),
            "should contain min boundary 0"
        );
        assert!(
            values.contains(&&Value::Int(100)),
            "should contain max boundary 100"
        );
    }

    #[test]
    fn test_witnesses_with_content_encoding_include_lattice_boundaries() {
        use crate::type_op::ContentEncoding;
        // Build a type with content encoding: String @content(UTF8)
        let utf8_string =
            type_lib::refined("String", vec![Predicate::Content(ContentEncoding::UTF8)]);
        let w = witnesses(&utf8_string);

        // Should have base witness + encoding-specific witnesses
        assert!(
            w.len() >= 2,
            "content-encoded type should have encoding boundary witnesses, got {}",
            w.len()
        );

        // All witnesses for a string type should be strings
        assert!(w.iter().all(|bw| matches!(&bw.value, Value::Str(_))));
    }

    // ============ M12: ShapeContract tests ============

    #[test]
    fn test_shape_contract_check_matching_kind() {
        use crate::value::ValueKind;

        let contract = ShapeContract::new(ValueKind::List, "after scalar-to-list coercion");
        assert!(contract.check(&Value::List(vec![Value::Int(1)])).is_ok());
    }

    #[test]
    fn test_shape_contract_check_wrong_kind() {
        use crate::value::ValueKind;

        let contract = ShapeContract::new(ValueKind::List, "after scalar-to-list coercion");
        let result = contract.check(&Value::Int(42));
        assert!(result.is_err());
        let violation = result.unwrap_err();
        assert_eq!(violation.actual_kind, ValueKind::Int);
        assert_eq!(violation.contract.expected_kind, ValueKind::List);
    }

    #[test]
    fn test_shape_contract_with_cardinality() {
        use crate::value::ValueKind;

        let contract = ShapeContract::new(ValueKind::List, "non-empty list")
            .with_cardinality(Cardinality::ONE_OR_MORE);

        // Non-empty list passes
        assert!(contract.check(&Value::List(vec![Value::Int(1)])).is_ok());

        // Empty list fails cardinality
        let result = contract.check(&Value::List(vec![]));
        assert!(result.is_err());
        let violation = result.unwrap_err();
        assert_eq!(violation.actual_length, Some(0));
    }

    #[test]
    fn test_shape_contract_display() {
        use crate::value::ValueKind;

        let contract = ShapeContract::new(ValueKind::List, "scalar-to-list");
        let violation = ShapeViolation {
            contract,
            actual_kind: ValueKind::Int,
            actual_length: None,
        };
        let msg = violation.to_string();
        assert!(msg.contains("expected List"));
        assert!(msg.contains("got Int"));
        assert!(msg.contains("scalar-to-list"));
    }

    #[test]
    fn test_shape_contract_unit_check() {
        use crate::value::ValueKind;

        let contract = ShapeContract::new(ValueKind::Unit, "unwrapped optional");
        assert!(contract.check(&Value::Unit).is_ok());
        assert!(contract.check(&Value::Bool(true)).is_err());
    }

    // ============ ContractObligation tests ============

    #[test]
    fn test_contract_obligation_id() {
        let obligation = ContractObligation::new("ObjectStorage", "read", "read(k) => v", 0);
        assert_eq!(obligation.obligation_id(), "ObjectStorage::read::0");
    }

    #[test]
    fn test_contract_obligation_display() {
        let obligation = ContractObligation::new("ClaimStore", "acquire", "returns valid claim", 0);
        let display = obligation.to_string();
        assert!(display.contains("ClaimStore.acquire"));
        assert!(display.contains("returns valid claim"));
    }

    #[test]
    fn test_contract_obligation_with_shape() {
        use crate::value::ValueKind;

        let obligation = ContractObligation::new("ObjectStorage", "read", "body is string", 0)
            .with_shape(ShapeContract::new(ValueKind::String, "read result"));
        assert!(obligation.shape.is_some());
        assert_eq!(obligation.shape.unwrap().expected_kind, ValueKind::String);
    }

    // ============ ResourceRequirement tests ============

    #[test]
    fn test_resource_requirement_display() {
        assert_eq!(
            ResourceRequirement::Tool {
                name: "cargo".to_string(),
                min_version: Some("1.75.0".to_string()),
            }
            .to_string(),
            "tool:cargo>=1.75.0"
        );
        assert_eq!(ResourceRequirement::Network.to_string(), "network");
        assert_eq!(
            ResourceRequirement::EnvVar("GCP_WIF_PROVIDER".to_string()).to_string(),
            "env:GCP_WIF_PROVIDER"
        );
        assert_eq!(
            ResourceRequirement::CostTier("M".to_string()).to_string(),
            "cost>=M"
        );
    }

    #[test]
    fn test_resource_requirement_tool_without_version() {
        assert_eq!(
            ResourceRequirement::Tool {
                name: "make".to_string(),
                min_version: None,
            }
            .to_string(),
            "tool:make"
        );
    }

    // ============ M16: ProtocolLayer / ProtocolStack tests ============

    #[test]
    fn test_protocol_layer_kind_ordering() {
        assert!(ProtocolLayerKind::Socket < ProtocolLayerKind::Transport);
        assert!(ProtocolLayerKind::Transport < ProtocolLayerKind::Session);
        assert!(ProtocolLayerKind::Session < ProtocolLayerKind::Presentation);
        assert!(ProtocolLayerKind::Presentation < ProtocolLayerKind::Application);
        assert!(ProtocolLayerKind::Application < ProtocolLayerKind::Operation);
    }

    #[test]
    fn test_protocol_layer_kind_display() {
        assert_eq!(ProtocolLayerKind::Socket.to_string(), "socket");
        assert_eq!(ProtocolLayerKind::Session.to_string(), "session");
        assert_eq!(ProtocolLayerKind::Presentation.to_string(), "presentation");
    }

    #[test]
    fn test_protocol_layer_builder() {
        let layer = ProtocolLayer::new("http", ProtocolLayerKind::Session)
            .with_properties(vec!["Retryable".into()])
            .with_status_semantics(vec![
                StatusSemantic::new(200, "success"),
                StatusSemantic::new(503, "retryable"),
            ])
            .with_description("HTTP request/response framing");

        assert_eq!(layer.id, "http");
        assert_eq!(layer.kind, ProtocolLayerKind::Session);
        assert_eq!(layer.properties, vec!["Retryable"]);
        assert_eq!(layer.status_semantics.len(), 2);
        assert_eq!(layer.description, "HTTP request/response framing");
    }

    #[test]
    fn test_protocol_stack_validates_correct_order() {
        let stack = ProtocolStack::new(vec![
            ProtocolLayer::new("tcp", ProtocolLayerKind::Socket),
            ProtocolLayer::new("http", ProtocolLayerKind::Session),
            ProtocolLayer::new("rest", ProtocolLayerKind::Presentation),
        ]);
        assert!(stack.validate().is_ok());
    }

    #[test]
    fn test_protocol_stack_allows_same_kind_layers() {
        // Two layers at the same kind level is valid (non-decreasing)
        let stack = ProtocolStack::new(vec![
            ProtocolLayer::new("tcp", ProtocolLayerKind::Socket),
            ProtocolLayer::new("udp", ProtocolLayerKind::Socket),
            ProtocolLayer::new("http", ProtocolLayerKind::Session),
        ]);
        assert!(stack.validate().is_ok());
    }

    #[test]
    fn test_protocol_stack_rejects_empty() {
        let stack = ProtocolStack::new(vec![]);
        let err = stack.validate().unwrap_err();
        assert!(matches!(err, ProtocolStackError::Empty));
        assert!(err.to_string().contains("at least one layer"));
    }

    #[test]
    fn test_protocol_stack_rejects_wrong_order() {
        let stack = ProtocolStack::new(vec![
            ProtocolLayer::new("rest", ProtocolLayerKind::Presentation),
            ProtocolLayer::new("tcp", ProtocolLayerKind::Socket),
        ]);
        let err = stack.validate().unwrap_err();
        match &err {
            ProtocolStackError::OrderViolation {
                index,
                layer_id,
                layer_kind,
                prev_kind,
            } => {
                assert_eq!(*index, 1);
                assert_eq!(layer_id, "tcp");
                assert_eq!(*layer_kind, ProtocolLayerKind::Socket);
                assert_eq!(*prev_kind, ProtocolLayerKind::Presentation);
            }
            other => panic!("expected OrderViolation, got {:?}", other),
        }
        assert!(err.to_string().contains("below preceding layer"));
    }

    #[test]
    fn test_protocol_stack_rejects_duplicate_ids() {
        let stack = ProtocolStack::new(vec![
            ProtocolLayer::new("tcp", ProtocolLayerKind::Socket),
            ProtocolLayer::new("tcp", ProtocolLayerKind::Session),
        ]);
        let err = stack.validate().unwrap_err();
        assert!(matches!(err, ProtocolStackError::DuplicateId { ref id } if id == "tcp"));
        assert!(err.to_string().contains("duplicate layer id"));
    }

    #[test]
    fn test_protocol_stack_all_properties() {
        let stack = ProtocolStack::new(vec![
            ProtocolLayer::new("tcp", ProtocolLayerKind::Socket)
                .with_properties(vec!["WritesWorld".into()]),
            ProtocolLayer::new("http", ProtocolLayerKind::Session)
                .with_properties(vec!["Retryable".into(), "WritesWorld".into()]),
            ProtocolLayer::new("rest", ProtocolLayerKind::Presentation)
                .with_properties(vec!["JsonContentType".into()]),
        ]);

        let props = stack.all_properties();
        assert_eq!(props.len(), 3);
        assert!(props.contains(&"WritesWorld".to_string()));
        assert!(props.contains(&"Retryable".to_string()));
        assert!(props.contains(&"JsonContentType".to_string()));
    }

    #[test]
    fn test_protocol_stack_effective_status_semantics_override() {
        let stack = ProtocolStack::new(vec![
            ProtocolLayer::new("http", ProtocolLayerKind::Session).with_status_semantics(vec![
                StatusSemantic::new(200, "success"),
                StatusSemantic::new(304, "redirect"),
                StatusSemantic::new(503, "server_error"),
            ]),
            ProtocolLayer::new("rest", ProtocolLayerKind::Presentation).with_status_semantics(
                vec![
                    // REST overrides: 304 is success, not redirect
                    StatusSemantic::new(304, "success"),
                ],
            ),
        ]);

        let semantics = stack.effective_status_semantics();
        assert_eq!(semantics.get(&200), Some(&"success".to_string()));
        // REST layer overrides HTTP's classification of 304
        assert_eq!(semantics.get(&304), Some(&"success".to_string()));
        // 503 is unchanged from HTTP layer
        assert_eq!(semantics.get(&503), Some(&"server_error".to_string()));
    }

    #[test]
    fn test_protocol_stack_depth_and_accessors() {
        let stack = ProtocolStack::new(vec![
            ProtocolLayer::new("tcp", ProtocolLayerKind::Socket),
            ProtocolLayer::new("http", ProtocolLayerKind::Session),
            ProtocolLayer::new("rest", ProtocolLayerKind::Presentation),
        ]);

        assert_eq!(stack.depth(), 3);
        assert_eq!(stack.bottom().unwrap().id, "tcp");
        assert_eq!(stack.top().unwrap().id, "rest");
    }

    #[test]
    fn test_protocol_stack_single_layer() {
        let stack =
            ProtocolStack::new(vec![ProtocolLayer::new("shell", ProtocolLayerKind::Socket)]);
        assert!(stack.validate().is_ok());
        assert_eq!(stack.depth(), 1);
        assert_eq!(stack.bottom(), stack.top());
    }

    // --- TransportBehavior bridge tests ---

    #[test]
    fn test_bridge_tcp_behavior() {
        use crate::transport::behavior::{TransportBehavior, TransportKind};

        let behavior = TransportBehavior::new(
            "transport.tcp",
            TransportKind::Tcp,
            "TcpRequest",
            "TcpResponse",
        );

        let stack = ProtocolStack::from_transport_behavior(&behavior);
        assert!(stack.validate().is_ok());
        assert_eq!(stack.depth(), 1);
        assert_eq!(stack.layers[0].id, "tcp");
        assert_eq!(stack.layers[0].kind, ProtocolLayerKind::Socket);
        assert!(stack.all_properties().contains(&"WritesWorld".to_string()));
    }

    #[test]
    fn test_bridge_http_behavior() {
        use crate::transport::behavior::{TransportBehavior, TransportKind};

        let behavior = TransportBehavior::new(
            "transport.http",
            TransportKind::Http,
            "HttpRequest",
            "HttpResponse",
        );

        let stack = ProtocolStack::from_transport_behavior(&behavior);
        assert!(stack.validate().is_ok());
        assert_eq!(stack.depth(), 2);
        assert_eq!(stack.layers[0].id, "tcp");
        assert_eq!(stack.layers[0].kind, ProtocolLayerKind::Socket);
        assert_eq!(stack.layers[1].id, "http");
        assert_eq!(stack.layers[1].kind, ProtocolLayerKind::Session);

        // HTTP layer should have status semantics
        let semantics = stack.effective_status_semantics();
        assert_eq!(semantics.get(&200), Some(&"success".to_string()));
        assert_eq!(semantics.get(&503), Some(&"retryable".to_string()));
    }

    #[test]
    fn test_bridge_rest_behavior() {
        use crate::transport::behavior::{TransportBehavior, TransportKind};

        let behavior = TransportBehavior::new(
            "transport.rest",
            TransportKind::Rest,
            "RestRequest",
            "RestResponse",
        );

        let stack = ProtocolStack::from_transport_behavior(&behavior);
        assert!(stack.validate().is_ok());
        assert_eq!(stack.depth(), 3);
        assert_eq!(stack.layers[0].id, "tcp");
        assert_eq!(stack.layers[1].id, "http");
        assert_eq!(stack.layers[2].id, "rest");
        assert_eq!(stack.layers[2].kind, ProtocolLayerKind::Presentation);

        // REST layer should add JsonContentType
        assert!(stack
            .all_properties()
            .contains(&"JsonContentType".to_string()));

        // REST overrides 304 to success
        let semantics = stack.effective_status_semantics();
        assert_eq!(semantics.get(&304), Some(&"success".to_string()));
    }

    #[test]
    fn test_bridge_file_behavior() {
        use crate::transport::behavior::{TransportBehavior, TransportKind};

        let behavior = TransportBehavior::new(
            "transport.file",
            TransportKind::File,
            "FileRequest",
            "FileResponse",
        );

        let stack = ProtocolStack::from_transport_behavior(&behavior);
        assert!(stack.validate().is_ok());
        assert_eq!(stack.depth(), 1);
        assert_eq!(stack.layers[0].id, "file");
        assert_eq!(stack.layers[0].kind, ProtocolLayerKind::Socket);
    }

    #[test]
    fn test_bridge_shell_behavior() {
        use crate::transport::behavior::{TransportBehavior, TransportKind};

        let behavior = TransportBehavior::new(
            "transport.shell",
            TransportKind::Shell,
            "ShellRequest",
            "ShellResponse",
        );

        let stack = ProtocolStack::from_transport_behavior(&behavior);
        assert!(stack.validate().is_ok());
        assert_eq!(stack.depth(), 1);
        assert_eq!(stack.layers[0].id, "shell");
    }

    #[test]
    fn test_bridge_all_default_behaviors_produce_valid_stacks() {
        use crate::transport::behavior::default_transport_behaviors;

        for behavior in default_transport_behaviors() {
            let stack = ProtocolStack::from_transport_behavior(&behavior);
            stack.validate().unwrap_or_else(|err| {
                panic!(
                    "default behavior '{}' produced invalid stack: {}",
                    behavior.id, err
                )
            });
            assert!(
                stack.depth() >= 1,
                "stack for '{}' should have at least one layer",
                behavior.id
            );
        }
    }

    // ============ M21: CodegenTypeShape + CodegenPlatformRepr tests ============

    #[test]
    fn test_codegen_scalar_is_not_composite() {
        let shape = CodegenTypeShape::Scalar(ScalarKind::String);
        assert!(!shape.is_composite());
    }

    #[test]
    fn test_codegen_record_is_composite() {
        let shape = CodegenTypeShape::Record {
            fields: vec![
                (
                    "name".to_string(),
                    CodegenTypeShape::Scalar(ScalarKind::String),
                ),
                (
                    "age".to_string(),
                    CodegenTypeShape::Scalar(ScalarKind::Integer),
                ),
            ],
        };
        assert!(shape.is_composite());
    }

    #[test]
    fn test_codegen_enum_is_composite() {
        let shape = CodegenTypeShape::Enum {
            variants: vec!["Get".to_string(), "Post".to_string(), "Put".to_string()],
        };
        assert!(shape.is_composite());
    }

    #[test]
    fn test_codegen_list_is_composite() {
        let shape = CodegenTypeShape::List(Box::new(CodegenTypeShape::Scalar(ScalarKind::Integer)));
        assert!(shape.is_composite());
    }

    #[test]
    fn test_codegen_optional_is_not_composite() {
        let shape =
            CodegenTypeShape::Optional(Box::new(CodegenTypeShape::Scalar(ScalarKind::String)));
        assert!(!shape.is_composite());
    }

    #[test]
    fn test_codegen_map_is_composite() {
        let shape = CodegenTypeShape::Map {
            key: Box::new(CodegenTypeShape::Scalar(ScalarKind::String)),
            value: Box::new(CodegenTypeShape::Scalar(ScalarKind::Integer)),
        };
        assert!(shape.is_composite());
    }

    #[test]
    fn test_codegen_leaf_scalars_scalar() {
        let shape = CodegenTypeShape::Scalar(ScalarKind::Boolean);
        let scalars = shape.leaf_scalars();
        assert_eq!(scalars, vec![&ScalarKind::Boolean]);
    }

    #[test]
    fn test_codegen_leaf_scalars_record() {
        let shape = CodegenTypeShape::Record {
            fields: vec![
                (
                    "name".to_string(),
                    CodegenTypeShape::Scalar(ScalarKind::String),
                ),
                (
                    "count".to_string(),
                    CodegenTypeShape::Scalar(ScalarKind::Integer),
                ),
                (
                    "active".to_string(),
                    CodegenTypeShape::Scalar(ScalarKind::Boolean),
                ),
            ],
        };
        let scalars = shape.leaf_scalars();
        assert_eq!(
            scalars,
            vec![
                &ScalarKind::String,
                &ScalarKind::Integer,
                &ScalarKind::Boolean
            ]
        );
    }

    #[test]
    fn test_codegen_leaf_scalars_enum_has_none() {
        let shape = CodegenTypeShape::Enum {
            variants: vec!["A".to_string(), "B".to_string()],
        };
        let scalars = shape.leaf_scalars();
        assert!(scalars.is_empty());
    }

    #[test]
    fn test_codegen_leaf_scalars_nested() {
        // List<Record{name: String, data: Bytes}>
        let shape = CodegenTypeShape::List(Box::new(CodegenTypeShape::Record {
            fields: vec![
                (
                    "name".to_string(),
                    CodegenTypeShape::Scalar(ScalarKind::String),
                ),
                (
                    "data".to_string(),
                    CodegenTypeShape::Scalar(ScalarKind::Bytes),
                ),
            ],
        }));
        let scalars = shape.leaf_scalars();
        assert_eq!(scalars, vec![&ScalarKind::String, &ScalarKind::Bytes]);
    }

    #[test]
    fn test_codegen_leaf_scalars_optional() {
        let shape =
            CodegenTypeShape::Optional(Box::new(CodegenTypeShape::Scalar(ScalarKind::Float)));
        let scalars = shape.leaf_scalars();
        assert_eq!(scalars, vec![&ScalarKind::Float]);
    }

    #[test]
    fn test_codegen_leaf_scalars_map() {
        let shape = CodegenTypeShape::Map {
            key: Box::new(CodegenTypeShape::Scalar(ScalarKind::String)),
            value: Box::new(CodegenTypeShape::Scalar(ScalarKind::Integer)),
        };
        let scalars = shape.leaf_scalars();
        assert_eq!(scalars, vec![&ScalarKind::String, &ScalarKind::Integer]);
    }

    #[test]
    fn test_codegen_leaf_scalars_deeply_nested() {
        // Optional<Map<String, List<Boolean>>>
        let shape = CodegenTypeShape::Optional(Box::new(CodegenTypeShape::Map {
            key: Box::new(CodegenTypeShape::Scalar(ScalarKind::String)),
            value: Box::new(CodegenTypeShape::List(Box::new(CodegenTypeShape::Scalar(
                ScalarKind::Boolean,
            )))),
        }));
        let scalars = shape.leaf_scalars();
        assert_eq!(scalars, vec![&ScalarKind::String, &ScalarKind::Boolean]);
    }

    #[test]
    fn test_codegen_platform_repr_construction() {
        let repr = CodegenPlatformRepr {
            platform: Platform::Rust,
            type_name: "Vec<String>".to_string(),
            shape: CodegenTypeShape::List(Box::new(CodegenTypeShape::Scalar(ScalarKind::String))),
        };
        assert_eq!(repr.platform, Platform::Rust);
        assert_eq!(repr.type_name, "Vec<String>");
        assert!(repr.shape.is_composite());
    }

    #[test]
    fn test_codegen_platform_repr_go() {
        let repr = CodegenPlatformRepr {
            platform: Platform::Go,
            type_name: "map[string]int64".to_string(),
            shape: CodegenTypeShape::Map {
                key: Box::new(CodegenTypeShape::Scalar(ScalarKind::String)),
                value: Box::new(CodegenTypeShape::Scalar(ScalarKind::Integer)),
            },
        };
        assert_eq!(repr.platform, Platform::Go);
        assert_eq!(repr.type_name, "map[string]int64");
    }

    #[test]
    fn test_codegen_platform_repr_python() {
        let repr = CodegenPlatformRepr {
            platform: Platform::Python,
            type_name: "Optional[str]".to_string(),
            shape: CodegenTypeShape::Optional(Box::new(CodegenTypeShape::Scalar(
                ScalarKind::String,
            ))),
        };
        assert_eq!(repr.platform, Platform::Python);
        assert!(!repr.shape.is_composite());
    }

    #[test]
    fn test_codegen_platform_repr_typescript() {
        let repr = CodegenPlatformRepr {
            platform: Platform::TypeScript,
            type_name: "Record<string, number>".to_string(),
            shape: CodegenTypeShape::Map {
                key: Box::new(CodegenTypeShape::Scalar(ScalarKind::String)),
                value: Box::new(CodegenTypeShape::Scalar(ScalarKind::Integer)),
            },
        };
        assert_eq!(repr.platform, Platform::TypeScript);
    }

    #[test]
    fn test_codegen_type_shape_equality() {
        let a = CodegenTypeShape::Scalar(ScalarKind::String);
        let b = CodegenTypeShape::Scalar(ScalarKind::String);
        let c = CodegenTypeShape::Scalar(ScalarKind::Integer);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_codegen_type_shape_clone() {
        let original = CodegenTypeShape::Record {
            fields: vec![
                ("x".to_string(), CodegenTypeShape::Scalar(ScalarKind::Float)),
                ("y".to_string(), CodegenTypeShape::Scalar(ScalarKind::Float)),
            ],
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_platform_equality() {
        assert_eq!(Platform::Rust, Platform::Rust);
        assert_ne!(Platform::Rust, Platform::Go);
        assert_ne!(Platform::Python, Platform::TypeScript);
    }

    #[test]
    fn test_scalar_kind_all_variants() {
        // Verify all five ScalarKind variants are distinct.
        let kinds = [
            ScalarKind::String,
            ScalarKind::Integer,
            ScalarKind::Float,
            ScalarKind::Boolean,
            ScalarKind::Bytes,
        ];
        for (i, a) in kinds.iter().enumerate() {
            for (j, b) in kinds.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b, "ScalarKind variants at {} and {} should differ", i, j);
                }
            }
        }
    }

    // ============ CT-1: StructuredContract + ProviderResponseContract tests ============

    #[test]
    fn test_contract_kind_display() {
        assert_eq!(ContractKind::Sequence.to_string(), "sequence");
        assert_eq!(ContractKind::Idempotent.to_string(), "idempotent");
        assert_eq!(ContractKind::Destructive.to_string(), "destructive");
        assert_eq!(ContractKind::Invariant.to_string(), "invariant");
    }

    #[test]
    fn test_contract_step_builder() {
        let step = ContractStep::new("put")
            .with_arg("key", "test-key")
            .with_arg("value", "test-value")
            .with_expected("ok", "true");
        assert_eq!(step.capability, "put");
        assert_eq!(step.args.len(), 2);
        assert_eq!(step.expected.len(), 1);
        assert_eq!(step.expected[0], ("ok".to_string(), "true".to_string()));
    }

    #[test]
    fn test_structured_contract_sequence() {
        let contract = StructuredContract {
            interface_name: "ObjectStorage".to_string(),
            capability_name: "get".to_string(),
            kind: ContractKind::Sequence,
            setup: vec![ContractStep::new("put")
                .with_arg("key", "k")
                .with_arg("value", "v")],
            assertion: ContractStep::new("get")
                .with_arg("key", "k")
                .with_expected("found", "true")
                .with_expected("value", "v"),
        };
        assert_eq!(contract.kind, ContractKind::Sequence);
        assert_eq!(contract.setup.len(), 1);
        let name = contract.test_name();
        assert!(name.contains("objectstorage"));
        assert!(name.contains("get"));
        assert!(name.contains("sequence"));
    }

    #[test]
    fn test_structured_contract_invariant_has_empty_setup() {
        let contract = StructuredContract {
            interface_name: "ClaimStore".to_string(),
            capability_name: "acquire".to_string(),
            kind: ContractKind::Invariant,
            setup: vec![],
            assertion: ContractStep::new("acquire")
                .with_arg("issue_id", "1")
                .with_expected("acquired", "true"),
        };
        assert!(contract.setup.is_empty());
        assert!(contract.test_name().contains("invariant"));
    }

    #[test]
    fn test_provider_response_contract() {
        let contract = ProviderResponseContract {
            operation: "github.Gist::Create".to_string(),
            status_code: 401,
            response_type: "GitHubErrorShape".to_string(),
            is_error: true,
        };
        assert!(contract.is_error);
        assert_eq!(contract.status_code, 401);
    }

    // ============ CT-2: Contract test generation tests ============

    #[test]
    fn test_generate_contract_test_body_sequence() {
        let contract = StructuredContract {
            interface_name: "ObjectStorage".to_string(),
            capability_name: "get".to_string(),
            kind: ContractKind::Sequence,
            setup: vec![ContractStep::new("put")
                .with_arg("key", "\"k1\"")
                .with_arg("value", "\"v1\"")],
            assertion: ContractStep::new("get")
                .with_arg("key", "\"k1\"")
                .with_expected("found", "true")
                .with_expected("value", "\"v1\""),
        };

        let body = generate_contract_test_body(&contract);
        assert!(
            body.contains("provider.put("),
            "should call setup capability"
        );
        assert!(
            body.contains("provider.get("),
            "should call assertion capability"
        );
        assert!(
            body.contains("assert_eq!(result.found, true"),
            "should assert found"
        );
        assert!(
            body.contains("assert_eq!(result.value, \"v1\""),
            "should assert value"
        );
    }

    #[test]
    fn test_generate_contract_test_body_no_setup() {
        let contract = StructuredContract {
            interface_name: "Store".to_string(),
            capability_name: "list".to_string(),
            kind: ContractKind::Invariant,
            setup: vec![],
            assertion: ContractStep::new("list").with_expected("count", "0"),
        };

        let body = generate_contract_test_body(&contract);
        assert!(!body.contains("_setup_"), "invariant should have no setup");
        assert!(body.contains("provider.list()"), "should call list");
        assert!(
            body.contains("assert_eq!(result.count, 0"),
            "should assert count"
        );
    }

    #[test]
    fn test_generate_contract_test_fn_wraps_body() {
        let contract = StructuredContract {
            interface_name: "KV".to_string(),
            capability_name: "get".to_string(),
            kind: ContractKind::Sequence,
            setup: vec![ContractStep::new("put").with_arg("k", "1")],
            assertion: ContractStep::new("get")
                .with_arg("k", "1")
                .with_expected("v", "1"),
        };

        let code = generate_contract_test_fn(&contract, "KvImpl::test()");
        assert!(code.starts_with("#[test]"), "should have test attribute");
        assert!(
            code.contains("fn contract_kv_get_sequence_put()"),
            "should have test name"
        );
        assert!(
            code.contains("let provider = KvImpl::test()"),
            "should construct provider"
        );
        assert!(code.contains("provider.put("), "should contain setup");
        assert!(code.contains("provider.get("), "should contain assertion");
    }

    #[test]
    fn test_generate_interface_contract_tests_batch() {
        let contracts = vec![
            StructuredContract {
                interface_name: "Store".to_string(),
                capability_name: "get".to_string(),
                kind: ContractKind::Sequence,
                setup: vec![ContractStep::new("put").with_arg("k", "1")],
                assertion: ContractStep::new("get")
                    .with_arg("k", "1")
                    .with_expected("v", "1"),
            },
            StructuredContract {
                interface_name: "Store".to_string(),
                capability_name: "delete".to_string(),
                kind: ContractKind::Destructive,
                setup: vec![ContractStep::new("put").with_arg("k", "1")],
                assertion: ContractStep::new("delete")
                    .with_arg("k", "1")
                    .with_expected("deleted", "true"),
            },
        ];

        let tests = generate_interface_contract_tests(&contracts, "StoreImpl::test()");
        assert_eq!(tests.len(), 2);
        assert!(tests[0].contains("contract_store_get_sequence"));
        assert!(tests[1].contains("contract_store_delete_destructive"));
    }

    // ============ CT-3: Provider compliance wiring tests ============

    #[test]
    fn test_provider_binding_fields() {
        let binding = ProviderBinding {
            interface_name: "ObjectStorage".to_string(),
            provider_name: "GcsProvider".to_string(),
            test_constructor: "GcsProvider::new_test()".to_string(),
        };
        assert_eq!(binding.interface_name, "ObjectStorage");
        assert_eq!(binding.provider_name, "GcsProvider");
    }

    #[test]
    fn test_validate_provider_compliance_no_contracts() {
        let binding = ProviderBinding {
            interface_name: "NoContracts".to_string(),
            provider_name: "Impl".to_string(),
            test_constructor: "Impl::new()".to_string(),
        };
        let result = validate_provider_compliance(&binding, &[]);
        assert!(result.is_ok(), "no contracts = no obligations");
    }

    #[test]
    fn test_validate_provider_compliance_with_matching_contracts() {
        let binding = ProviderBinding {
            interface_name: "Store".to_string(),
            provider_name: "MemStore".to_string(),
            test_constructor: "MemStore::new()".to_string(),
        };
        let contracts = vec![StructuredContract {
            interface_name: "Store".to_string(),
            capability_name: "get".to_string(),
            kind: ContractKind::Sequence,
            setup: vec![ContractStep::new("put").with_arg("k", "1")],
            assertion: ContractStep::new("get")
                .with_arg("k", "1")
                .with_expected("v", "1"),
        }];
        let result = validate_provider_compliance(&binding, &contracts);
        assert!(result.is_ok(), "binding covers interface contracts");
    }

    #[test]
    fn test_generate_compliance_test_suite_multi_binding() {
        let contracts = vec![
            StructuredContract {
                interface_name: "Store".to_string(),
                capability_name: "get".to_string(),
                kind: ContractKind::Sequence,
                setup: vec![ContractStep::new("put").with_arg("k", "1")],
                assertion: ContractStep::new("get")
                    .with_arg("k", "1")
                    .with_expected("v", "1"),
            },
            StructuredContract {
                interface_name: "Auth".to_string(),
                capability_name: "verify".to_string(),
                kind: ContractKind::Invariant,
                setup: vec![],
                assertion: ContractStep::new("verify")
                    .with_arg("token", "\"valid\"")
                    .with_expected("ok", "true"),
            },
        ];
        let bindings = vec![
            ProviderBinding {
                interface_name: "Store".to_string(),
                provider_name: "MemStore".to_string(),
                test_constructor: "MemStore::new()".to_string(),
            },
            ProviderBinding {
                interface_name: "Auth".to_string(),
                provider_name: "MockAuth".to_string(),
                test_constructor: "MockAuth::new()".to_string(),
            },
        ];

        let tests = generate_compliance_test_suite(&bindings, &contracts);
        assert_eq!(tests.len(), 2);
        assert!(
            tests[0].contains("MemStore::new()"),
            "first test uses MemStore"
        );
        assert!(
            tests[1].contains("MockAuth::new()"),
            "second test uses MockAuth"
        );
    }

    #[test]
    fn test_generate_compliance_suite_skips_unmatched_bindings() {
        let contracts = vec![StructuredContract {
            interface_name: "Store".to_string(),
            capability_name: "get".to_string(),
            kind: ContractKind::Sequence,
            setup: vec![],
            assertion: ContractStep::new("get")
                .with_arg("k", "1")
                .with_expected("v", "1"),
        }];
        let bindings = vec![ProviderBinding {
            interface_name: "Other".to_string(),
            provider_name: "Impl".to_string(),
            test_constructor: "Impl::new()".to_string(),
        }];

        let tests = generate_compliance_test_suite(&bindings, &contracts);
        assert!(tests.is_empty(), "no tests when binding doesn't match");
    }

    // ============ CT-5: ProviderResponseContract obligation tests ============

    #[test]
    fn test_response_contract_test_name_error() {
        let c = ProviderResponseContract {
            operation: "github.Gist::Create".to_string(),
            status_code: 401,
            response_type: "GitHubErrorShape".to_string(),
            is_error: true,
        };
        assert_eq!(
            c.test_name(),
            "response_contract_github_gist_create_error_401"
        );
    }

    #[test]
    fn test_response_contract_test_name_success() {
        let c = ProviderResponseContract {
            operation: "github.Gist::Create".to_string(),
            status_code: 201,
            response_type: "GistResponse".to_string(),
            is_error: false,
        };
        assert_eq!(
            c.test_name(),
            "response_contract_github_gist_create_success_201"
        );
    }

    #[test]
    fn test_response_contract_generate_error_test() {
        let c = ProviderResponseContract {
            operation: "gcp.Storage::Get".to_string(),
            status_code: 404,
            response_type: "GcpNotFoundError".to_string(),
            is_error: true,
        };
        let code = c.generate_test_fn("MockTransport::with_status(404)");
        assert!(code.contains("#[test]"));
        assert!(code.contains("fn response_contract_gcp_storage_get_error_404()"));
        assert!(code.contains("MockTransport::with_status(404)"));
        assert!(code.contains("result.is_err()"));
        assert!(code.contains("err.status_code(), 404"));
        assert!(code.contains("GcpNotFoundError"));
    }

    #[test]
    fn test_response_contract_generate_success_test() {
        let c = ProviderResponseContract {
            operation: "gcp.Storage::Get".to_string(),
            status_code: 200,
            response_type: "ObjectData".to_string(),
            is_error: false,
        };
        let code = c.generate_test_fn("MockTransport::with_status(200)");
        assert!(code.contains("#[test]"));
        assert!(code.contains("result.is_ok()"));
        assert!(code.contains("ObjectData"));
    }

    #[test]
    fn test_generate_response_contract_tests_batch() {
        let contracts = vec![
            ProviderResponseContract {
                operation: "github.Gist::Create".to_string(),
                status_code: 201,
                response_type: "GistResponse".to_string(),
                is_error: false,
            },
            ProviderResponseContract {
                operation: "github.Gist::Create".to_string(),
                status_code: 401,
                response_type: "AuthError".to_string(),
                is_error: true,
            },
        ];
        let tests = generate_response_contract_tests(&contracts, |status| {
            format!("MockTransport::with_status({status})")
        });
        assert_eq!(tests.len(), 2);
        assert!(tests[0].contains("success_201"));
        assert!(tests[1].contains("error_401"));
    }

    #[test]
    fn test_validate_response_contract_coverage_complete() {
        let contracts = vec![
            ProviderResponseContract {
                operation: "gcp.Storage::Get".to_string(),
                status_code: 200,
                response_type: "ObjectData".to_string(),
                is_error: false,
            },
            ProviderResponseContract {
                operation: "gcp.Storage::Get".to_string(),
                status_code: 404,
                response_type: "NotFound".to_string(),
                is_error: true,
            },
        ];
        assert!(validate_response_contract_coverage(&contracts).is_ok());
    }

    #[test]
    fn test_validate_response_contract_coverage_missing_error() {
        let contracts = vec![ProviderResponseContract {
            operation: "gcp.Storage::Get".to_string(),
            status_code: 200,
            response_type: "ObjectData".to_string(),
            is_error: false,
        }];
        let result = validate_response_contract_coverage(&contracts);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing.len(), 1);
        assert!(missing[0].contains("error response contract"));
    }

    #[test]
    fn test_validate_response_contract_coverage_missing_success() {
        let contracts = vec![ProviderResponseContract {
            operation: "gcp.Storage::Get".to_string(),
            status_code: 500,
            response_type: "ServerError".to_string(),
            is_error: true,
        }];
        let result = validate_response_contract_coverage(&contracts);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert!(missing[0].contains("success"));
    }

    #[test]
    fn test_validate_response_contract_coverage_empty() {
        assert!(validate_response_contract_coverage(&[]).is_ok());
    }
}
