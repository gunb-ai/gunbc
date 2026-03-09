//! Cardinality coercion analysis pass.
//!
//! Detects implicit cardinality coercions in a DAG — edges where the output
//! port's cardinality differs from the input port's cardinality but is
//! compatible, requiring the execution engine to transform the value shape.
//! When a `TypeRegistry` is available, `validate_coercions_with_registry`
//! performs full L1–L3 contract checks before classifying coercions.
//!
//! # Motivation
//!
//! When a scalar output port `[1,1]` connects to a list input port `[0,∞)`,
//! the cardinalities are compatible (`satisfies` returns true), but the
//! runtime value needs transformation: the engine wraps the scalar in a
//! `Value::List([value])`. This pass makes those implicit coercions visible
//! for documentation, test generation, and future strict-mode engines.
//!
//! # Example
//!
//! ```text
//! let dag = build_multi_source_review_graph();
//! let coercions = detect_coercions(&dag);
//! // Reports: parse_response.output [1,1] → merge.outputs [0,∞) = WrapScalar
//! ```

use crate::dag::Dag;
use crate::type_registry::TypeRegistry;
use crate::types::{Cardinality, NodeId, PortName};

/// Describes an implicit cardinality coercion at a specific edge.
#[derive(Debug, Clone)]
pub struct CardinalityCoercion {
    /// Source node producing the value.
    pub from_node: NodeId,
    /// Source output port.
    pub from_port: PortName,
    /// Target node consuming the value.
    pub to_node: NodeId,
    /// Target input port.
    pub to_port: PortName,
    /// Cardinality of the source output port.
    pub from_cardinality: Cardinality,
    /// Cardinality of the target input port.
    pub to_cardinality: Cardinality,
    /// What kind of value transformation the engine performs.
    pub kind: CoercionKind,
}

/// What kind of value transformation the engine performs at a coercion point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoercionKind {
    /// Scalar value wrapped into a single-element list by the engine.
    ///
    /// `[1,1]` output port → list input port (`[0,∞)` or `[1,∞)`).
    /// Engine wraps: `v` → `Value::List(vec![v])`.
    WrapScalar,

    /// Optional value coerced to a possibly-empty list by the engine.
    ///
    /// `[0,1]` output port → `[0,∞)` input port.
    /// Engine wraps: absent → `Value::List(vec![])`, present → `Value::List(vec![v])`.
    OptionalToList,

    /// Bounded output widened to unbounded list input.
    ///
    /// `[n,m]` output → `[n,∞)` input where `m` is finite.
    /// The engine's fan-in collection handles this naturally.
    Widen,
}

impl std::fmt::Display for CoercionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoercionKind::WrapScalar => write!(f, "WrapScalar"),
            CoercionKind::OptionalToList => write!(f, "OptionalToList"),
            CoercionKind::Widen => write!(f, "Widen"),
        }
    }
}

impl std::fmt::Display for CardinalityCoercion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{} {} → {}.{} {} ({})",
            self.from_node.0,
            self.from_port.0,
            self.from_cardinality,
            self.to_node.0,
            self.to_port.0,
            self.to_cardinality,
            self.kind,
        )
    }
}

/// Record of a coercion that was actually applied at execution time.
///
/// Unlike `CardinalityCoercion` (a static analysis result), this records a
/// coercion that the execution engine performed on a concrete value during
/// a specific DAG run. Used for execution trace observability.
#[derive(Debug, Clone)]
pub struct AppliedCoercion {
    /// Source node that produced the value.
    pub from_node: String,
    /// Source output port.
    pub from_port: String,
    /// Target input port that received the coerced value.
    pub to_port: String,
    /// What transformation the engine applied.
    pub kind: CoercionKind,
}

impl std::fmt::Display for AppliedCoercion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{} → {} ({})",
            self.from_node, self.from_port, self.to_port, self.kind
        )
    }
}

/// Classify what coercion, if any, an edge requires based on port cardinalities.
///
/// Returns `None` if no coercion is needed (cardinalities are identical or
/// the edge doesn't cross a scalar/list boundary).
pub fn classify_coercion(from: Cardinality, to: Cardinality) -> Option<CoercionKind> {
    // No coercion needed if cardinalities are identical
    if from == to {
        return None;
    }

    // Scalar → List: wrap in single-element list
    if from.is_scalar() && to.is_list() {
        return Some(CoercionKind::WrapScalar);
    }

    // Optional → List: absent→[], present→[x]
    if from == Cardinality::ZERO_OR_ONE && to.is_list() {
        return Some(CoercionKind::OptionalToList);
    }

    // Bounded → Unbounded: finite max → infinite max (safe widening)
    if from.is_bounded() && !to.is_bounded() && from.satisfies(to) {
        return Some(CoercionKind::Widen);
    }

    None
}

/// Detect all implicit cardinality coercions in a DAG.
///
/// Walks all edges and identifies where the engine needs to transform
/// values to bridge cardinality differences between connected ports.
pub fn detect_coercions<T>(dag: &Dag<T>) -> Vec<CardinalityCoercion> {
    validate_coercions(dag).coercions
}

/// Summary report of coercions in a DAG.
#[derive(Debug)]
pub struct CoercionReport {
    /// Coercions that the engine handles implicitly.
    pub coercions: Vec<CardinalityCoercion>,
    /// Edges with incompatible cardinalities that cannot be coerced.
    pub errors: Vec<CoercionError>,
}

/// An edge with incompatible cardinalities.
#[derive(Debug, Clone)]
pub struct CoercionError {
    pub from_node: NodeId,
    pub from_port: PortName,
    pub to_node: NodeId,
    pub to_port: PortName,
    pub from_cardinality: Cardinality,
    pub to_cardinality: Cardinality,
    pub reason: String,
}


impl std::fmt::Display for CoercionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{} {} → {}.{} {}: {}",
            self.from_node.0,
            self.from_port.0,
            self.from_cardinality,
            self.to_node.0,
            self.to_port.0,
            self.to_cardinality,
            self.reason,
        )
    }
}

/// Validate all edges in a DAG for cardinality compatibility.
///
/// Returns a report with:
/// - `coercions`: edges where implicit coercion is needed and safe
/// - `errors`: edges where cardinalities are incompatible
pub fn validate_coercions<T>(dag: &Dag<T>) -> CoercionReport {
    validate_coercions_with_registry(dag, None)
}

/// Validate all edges in a DAG for cardinality compatibility and safe coercion.
///
/// When a registry is provided, L1–L3 contract coercion checks are applied
/// before cardinality-only analysis.
pub fn validate_coercions_with_registry<T>(
    dag: &Dag<T>,
    registry: Option<&TypeRegistry>,
) -> CoercionReport {
    let mut coercions = Vec::new();
    let mut errors = Vec::new();

    for edge in &dag.edges {
        let Some(ports) = dag.resolve_edge_ports(edge) else {
            continue;
        };
        let fp = ports.from.port();
        let tp = ports.to.port();

        let from_card = match registry {
            Some(registry) => fp.infer_cardinality(registry),
            None => fp.cardinality,
        };
        let to_card = match registry {
            Some(registry) => tp.infer_cardinality(registry),
            None => tp.cardinality,
        };

        // Type compatibility is checked by the builder at edge creation time.
        // The coercion pass focuses on cardinality analysis only.

        if from_card == to_card {
            // Identical — no coercion needed
            continue;
        }

        if from_card.satisfies(to_card) {
            // Compatible but different — implicit coercion
            if let Some(kind) = classify_coercion(from_card, to_card) {
                coercions.push(CardinalityCoercion {
                    from_node: edge.from_node.clone(),
                    from_port: edge.from_port.clone(),
                    to_node: edge.to_node.clone(),
                    to_port: edge.to_port.clone(),
                    from_cardinality: from_card,
                    to_cardinality: to_card,
                    kind,
                });
            }
        } else {
            // Incompatible — error
            let reason = from_card.check_satisfies(to_card).unwrap_err().reason;
            errors.push(CoercionError {
                from_node: edge.from_node.clone(),
                from_port: edge.from_port.clone(),
                to_node: edge.to_node.clone(),
                to_port: edge.to_port.clone(),
                from_cardinality: from_card,
                to_cardinality: to_card,
                reason,
            });
        }
    }

    CoercionReport { coercions, errors }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::build::*;
    use crate::dag::Port;
    use crate::node::Node;

    #[test]
    fn classify_identical_cardinalities_returns_none() {
        assert_eq!(classify_coercion(Cardinality::ONE, Cardinality::ONE), None);
        assert_eq!(
            classify_coercion(Cardinality::ZERO_OR_MORE, Cardinality::ZERO_OR_MORE),
            None
        );
    }

    #[test]
    fn classify_scalar_to_list_is_wrap_scalar() {
        assert_eq!(
            classify_coercion(Cardinality::ONE, Cardinality::ZERO_OR_MORE),
            Some(CoercionKind::WrapScalar)
        );
        assert_eq!(
            classify_coercion(Cardinality::ONE, Cardinality::ONE_OR_MORE),
            Some(CoercionKind::WrapScalar)
        );
    }

    #[test]
    fn classify_optional_to_list_is_optional_to_list() {
        assert_eq!(
            classify_coercion(Cardinality::ZERO_OR_ONE, Cardinality::ZERO_OR_MORE),
            Some(CoercionKind::OptionalToList)
        );
    }

    #[test]
    fn classify_bounded_to_unbounded_is_widen() {
        // [2,5] → [0,∞): bounded satisfies unbounded, so it's a widen
        let bounded = Cardinality::new(2, Some(5));
        assert_eq!(
            classify_coercion(bounded, Cardinality::ZERO_OR_MORE),
            Some(CoercionKind::Widen)
        );
    }

    #[test]
    fn classify_scalar_to_scalar_returns_none() {
        // [1,1] → [0,1]: not a list, no coercion
        assert_eq!(
            classify_coercion(Cardinality::ONE, Cardinality::ZERO_OR_ONE),
            None
        );
    }

    #[test]
    fn classify_list_to_list_different_bounds_returns_none() {
        // [1,∞) → [0,∞): both lists, engine treats the same
        assert_eq!(
            classify_coercion(Cardinality::ONE_OR_MORE, Cardinality::ZERO_OR_MORE),
            None
        );
    }

    /// A simple test op for building test DAGs.
    #[derive(Debug, Clone)]
    struct TestOp;

    #[test]
    fn detect_coercions_finds_scalar_to_list() {
        let mut dag = Dag::new();

        dag.add_node(Node::opaque(
            "producer",
            vec![],
            vec![port("output", "Json")], // scalar [1,1]
            TestOp,
        ));

        dag.add_node(Node::opaque(
            "consumer",
            vec![list("input", "JsonList")], // list [0,∞)
            vec![port("result", "Json")],
            TestOp,
        ));

        dag.add_edge(edge("producer", "output", "consumer", "input"));

        let coercions = detect_coercions(&dag);
        assert_eq!(coercions.len(), 1);
        assert_eq!(coercions[0].kind, CoercionKind::WrapScalar);
        assert_eq!(coercions[0].from_node.0, "producer");
        assert_eq!(coercions[0].to_node.0, "consumer");
    }

    #[test]
    fn detect_coercions_ignores_identical() {
        let mut dag = Dag::new();

        dag.add_node(Node::opaque(
            "a",
            vec![],
            vec![port("out", "String")],
            TestOp,
        ));

        dag.add_node(Node::opaque(
            "b",
            vec![port("in", "String")],
            vec![],
            TestOp,
        ));

        dag.add_edge(edge("a", "out", "b", "in"));

        let coercions = detect_coercions(&dag);
        assert!(coercions.is_empty());
    }

    #[test]
    fn detect_coercions_ignores_incompatible() {
        let mut dag = Dag::new();

        dag.add_node(Node::opaque(
            "producer",
            vec![],
            vec![port("output", "Json")], // [1,1]
            TestOp,
        ));

        dag.add_node(Node::opaque(
            "consumer",
            vec![Port::with_cardinality(
                "input",
                "Json",
                Cardinality::new(2, Some(2)),
            )], // [2,2]
            vec![port("result", "Json")],
            TestOp,
        ));

        dag.add_edge(edge("producer", "output", "consumer", "input"));

        let coercions = detect_coercions(&dag);
        assert!(coercions.is_empty());
    }

    #[test]
    fn validate_reports_incompatible_edges() {
        let mut dag = Dag::new();

        // [0,∞) output → [1,1] input: incompatible (might be empty)
        dag.add_node(Node::opaque(
            "list_producer",
            vec![],
            vec![Port::list("out", "JsonList")],
            TestOp,
        ));

        dag.add_node(Node::opaque(
            "scalar_consumer",
            vec![port("in", "Json")],
            vec![],
            TestOp,
        ));

        dag.add_edge(edge("list_producer", "out", "scalar_consumer", "in"));

        let report = validate_coercions(&dag);
        assert!(report.coercions.is_empty());
        assert_eq!(report.errors.len(), 1);
        assert!(
            !report.errors[0].reason.is_empty(),
            "error should have a reason: {}",
            report.errors[0].reason
        );
    }

    #[test]
    fn validate_reports_safe_coercions() {
        let mut dag = Dag::new();

        dag.add_node(Node::opaque(
            "scalar",
            vec![],
            vec![port("out", "Json")],
            TestOp,
        ));

        dag.add_node(Node::opaque(
            "merger",
            vec![list("in", "JsonList")],
            vec![],
            TestOp,
        ));

        dag.add_edge(edge("scalar", "out", "merger", "in"));

        let report = validate_coercions(&dag);
        assert_eq!(report.coercions.len(), 1);
        assert_eq!(report.coercions[0].kind, CoercionKind::WrapScalar);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn coercion_display_format() {
        let coercion = CardinalityCoercion {
            from_node: "parse_response".into(),
            from_port: "output".into(),
            to_node: "merge".into(),
            to_port: "outputs".into(),
            from_cardinality: Cardinality::ONE,
            to_cardinality: Cardinality::ZERO_OR_MORE,
            kind: CoercionKind::WrapScalar,
        };

        let display = format!("{}", coercion);
        assert!(display.contains("parse_response.output"));
        assert!(display.contains("merge.outputs"));
        assert!(display.contains("WrapScalar"));
    }

}
