//! DAG validation: SubDag interfaces and resource wiring.
//!
//! Validates that SubDag nodes' declared interfaces (input/output ports)
//! match the inner DAG's structural entrypoints and boundaries.
//!
//! This catches mismatches at build time rather than waiting for `lower()`
//! in the executor.
//!
//! Note: Operation overlap detection was removed; replaced by C22
//! (Deductive Redundancy Elimination using idempotency fingerprints).

use crate::boundary::detect_boundaries;
use crate::dag::{Dag, Port};
use crate::entrypoint::detect_entrypoints;
use crate::node::{Node, NodeBody};
use crate::type_registry::TypeRegistry;
use crate::types::{NodeId, PortName, SemanticCarrierKind, TypeId};
use std::fmt;

/// Error from SubDag interface validation.
#[derive(Debug, Clone)]
pub enum SubDagError {
    /// Parent input port has no matching entrypoint in the inner DAG.
    NoInnerEntrypoint {
        node: NodeId,
        port: PortName,
        available: Vec<PortName>,
    },
    /// Parent output port has no matching boundary in the inner DAG.
    NoInnerBoundary {
        node: NodeId,
        port: PortName,
        available: Vec<PortName>,
    },
    /// Inner entrypoint has no matching parent input port.
    UnexposedEntrypoint {
        node: NodeId,
        inner_port: PortName,
        parent_inputs: Vec<PortName>,
    },
    /// Inner boundary has no matching parent output port.
    UnexposedBoundary {
        node: NodeId,
        inner_port: PortName,
        parent_outputs: Vec<PortName>,
    },
    /// Type mismatch between parent port and inner port.
    TypeMismatch {
        node: NodeId,
        port: PortName,
        direction: PortDirection,
        parent_type: TypeId,
        inner_type: TypeId,
    },
    /// Semantic carrier mismatch between parent and inner ports.
    SemanticCarrierMismatch {
        node: NodeId,
        port: PortName,
        direction: PortDirection,
        parent_type: TypeId,
        inner_type: TypeId,
        parent_kind: SemanticCarrierKind,
        inner_kind: SemanticCarrierKind,
    },
    /// Port uses an invalid type expression.
    InvalidTypeExpression {
        node: NodeId,
        port: PortName,
        direction: PortDirection,
        type_id: TypeId,
        error: crate::type_registry::TypeExprError,
        source: TypeExprSource,
    },
    /// Nested SubDag failed validation.
    Nested {
        parent: NodeId,
        inner: Box<SubDagError>,
    },
}

/// Whether the mismatched port is an input or output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}

/// Whether the invalid type expression came from the parent or inner port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeExprSource {
    Parent,
    Inner,
}

impl fmt::Display for PortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortDirection::Input => write!(f, "input"),
            PortDirection::Output => write!(f, "output"),
        }
    }
}

impl fmt::Display for SubDagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubDagError::NoInnerEntrypoint {
                node,
                port,
                available,
            } => {
                write!(
                    f,
                    "SubDag '{}': input port '{}' has no matching entrypoint in inner DAG (available: [{}])",
                    node, port,
                    available.iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join(", ")
                )
            }
            SubDagError::NoInnerBoundary {
                node,
                port,
                available,
            } => {
                write!(
                    f,
                    "SubDag '{}': output port '{}' has no matching boundary in inner DAG (available: [{}])",
                    node, port,
                    available.iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join(", ")
                )
            }
            SubDagError::UnexposedEntrypoint {
                node,
                inner_port,
                parent_inputs,
            } => {
                write!(
                    f,
                    "SubDag '{}': inner entrypoint '{}' has no matching parent input port (parent inputs: [{}])",
                    node, inner_port,
                    parent_inputs.iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join(", ")
                )
            }
            SubDagError::UnexposedBoundary {
                node,
                inner_port,
                parent_outputs,
            } => {
                write!(
                    f,
                    "SubDag '{}': inner boundary '{}' has no matching parent output port (parent outputs: [{}])",
                    node, inner_port,
                    parent_outputs.iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join(", ")
                )
            }
            SubDagError::TypeMismatch {
                node,
                port,
                direction,
                parent_type,
                inner_type,
            } => {
                write!(
                    f,
                    "SubDag '{}': {} port '{}' type mismatch: parent declares '{}', inner has '{}'",
                    node, direction, port, parent_type, inner_type
                )
            }
            SubDagError::SemanticCarrierMismatch {
                node,
                port,
                direction,
                parent_type,
                inner_type,
                parent_kind,
                inner_kind,
            } => {
                write!(
                    f,
                    "SubDag '{}': {} port '{}' semantic carrier mismatch: parent '{}' ({:?}), inner '{}' ({:?})",
                    node, direction, port, parent_type, parent_kind, inner_type, inner_kind
                )
            }
            SubDagError::InvalidTypeExpression {
                node,
                port,
                direction,
                type_id,
                error,
                source,
            } => {
                write!(
                    f,
                    "invalid type expression on {} {} port '{}:{}' ({}): {}",
                    match source {
                        TypeExprSource::Parent => "parent",
                        TypeExprSource::Inner => "inner",
                    },
                    direction,
                    node,
                    port,
                    type_id,
                    error
                )
            }
            SubDagError::Nested { parent, inner } => {
                write!(f, "in SubDag '{}': {}", parent, inner)
            }
        }
    }
}

impl std::error::Error for SubDagError {}

/// Unwired resource input (res:* entrypoint) discovered in a DAG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnwiredResource {
    pub node: NodeId,
    pub port: PortName,
}

impl fmt::Display for UnwiredResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unwired resource input: {}:{}", self.node, self.port)
    }
}

/// Validate all SubDag interfaces in a DAG, recursively.
///
/// For each SubDag node, checks that:
/// - Every parent input port has a matching entrypoint (by name) in the inner DAG
/// - Every parent output port has a matching boundary (by name) in the inner DAG
/// - Types match between parent ports and their inner counterparts
///
/// Returns all errors found (does not stop at the first).
pub fn validate_subdag_interfaces<T>(dag: &Dag<T>) -> Vec<SubDagError> {
    let registry = TypeRegistry::with_core_types();
    let mut errors = Vec::new();
    validate_dag_recursive(dag, &mut errors, &registry);
    errors
}

/// Validate that all `res:*` input ports are wired.
///
/// Returns a list of unwired resource inputs (unconnected entrypoints).
pub fn validate_resource_wiring<T>(dag: &Dag<T>) -> Vec<UnwiredResource> {
    detect_entrypoints(dag)
        .entrypoint_ports
        .iter()
        .filter(|(_, port_name, _)| port_name.is_resource())
        .map(|(node_id, port_name, _)| UnwiredResource {
            node: node_id.clone(),
            port: port_name.clone(),
        })
        .collect()
}

/// Validate resource wiring recursively through SubDag nodes.
///
/// At each level, checks for unwired `res:*` entrypoints. For SubDag nodes,
/// recursively walks inner DAGs to ensure inner `res:*` entrypoints are
/// properly exposed as parent input ports (which auto-inference ensures).
///
/// Returns unwired resources found at any nesting level.
pub fn validate_resource_wiring_recursive<T>(dag: &Dag<T>) -> Vec<UnwiredResource> {
    let mut unwired = Vec::new();
    validate_resource_wiring_recursive_impl(dag, &std::collections::HashSet::new(), &mut unwired);
    unwired
}

fn validate_resource_wiring_recursive_impl<T>(
    dag: &Dag<T>,
    inherited_resources: &std::collections::HashSet<String>,
    unwired: &mut Vec<UnwiredResource>,
) {
    // Check unwired resources at this level, but suppress ports that are
    // already exposed by ancestor SubDag inputs.
    unwired.extend(
        validate_resource_wiring(dag)
            .into_iter()
            .filter(|u| !inherited_resources.contains(&u.port.0)),
    );

    // Recurse into SubDag nodes, carrying forward any resource inputs
    // exposed on the SubDag wrapper.
    for node in &dag.nodes {
        if let NodeBody::SubDag(ref inner) = node.body {
            let mut next_inherited = inherited_resources.clone();
            for port in &node.inputs {
                if port.name.is_resource() {
                    next_inherited.insert(port.name.0.clone());
                }
            }
            validate_resource_wiring_recursive_impl(inner, &next_inherited, unwired);
        }
    }
}

fn validate_dag_recursive<T>(dag: &Dag<T>, errors: &mut Vec<SubDagError>, registry: &TypeRegistry) {
    for node in &dag.nodes {
        if let NodeBody::SubDag(ref inner) = node.body {
            validate_single_subdag(node, inner, registry, errors);
            // Recurse into the inner DAG
            let mut nested_errors = Vec::new();
            validate_dag_recursive(inner, &mut nested_errors, registry);
            for err in nested_errors {
                errors.push(SubDagError::Nested {
                    parent: node.id.clone(),
                    inner: Box::new(err),
                });
            }
        }
    }
}

fn validate_single_subdag<T>(
    parent_node: &Node<T>,
    inner_dag: &Dag<T>,
    registry: &TypeRegistry,
    errors: &mut Vec<SubDagError>,
) {
    let entrypoints = detect_entrypoints(inner_dag);
    let boundaries = detect_boundaries(inner_dag);

    // Validate input ports -> entrypoints
    for parent_port in &parent_node.inputs {
        let matching: Vec<_> = entrypoints
            .entrypoint_ports
            .iter()
            .filter(|(_, name, _)| name == &parent_port.name)
            .collect();

        if matching.is_empty() {
            let available: Vec<PortName> = entrypoints
                .entrypoint_ports
                .iter()
                .map(|(_, name, _)| name.clone())
                .collect();
            errors.push(SubDagError::NoInnerEntrypoint {
                node: parent_node.id.clone(),
                port: parent_port.name.clone(),
                available,
            });
        } else {
            // Check type compatibility for each match
            for (_, _, inner_type) in &matching {
                check_type_match(
                    &parent_node.id,
                    &parent_port.name,
                    PortDirection::Input,
                    &parent_port.type_id,
                    inner_type,
                    registry,
                    errors,
                );
            }
        }
    }

    // Validate output ports -> boundaries
    // Check ALL matching boundaries (not just the first) to catch type
    // mismatches on duplicate boundary names.
    for parent_port in &parent_node.outputs {
        let matching: Vec<_> = boundaries
            .boundary_ports
            .iter()
            .filter(|(_, name)| name == &parent_port.name)
            .collect();

        if matching.is_empty() {
            let available: Vec<PortName> = boundaries
                .boundary_ports
                .iter()
                .map(|(_, name)| name.clone())
                .collect();
            errors.push(SubDagError::NoInnerBoundary {
                node: parent_node.id.clone(),
                port: parent_port.name.clone(),
                available,
            });
        } else {
            // Type-check every matching boundary port
            for (inner_node_id, inner_port_name) in &matching {
                if let Some(inner_node) = inner_dag.get_node(inner_node_id) {
                    if let Some(inner_port) = find_output_port(inner_node, inner_port_name) {
                        check_type_match(
                            &parent_node.id,
                            &parent_port.name,
                            PortDirection::Output,
                            &parent_port.type_id,
                            &inner_port.type_id,
                            registry,
                            errors,
                        );
                    }
                }
            }
        }
    }

    // Inverse check: inner entrypoints not exposed on parent
    let mut seen_entrypoint_names = std::collections::HashSet::new();
    for (_, name, _) in &entrypoints.entrypoint_ports {
        if seen_entrypoint_names.insert(name.clone())
            && !parent_node.inputs.iter().any(|p| p.name == *name)
        {
            errors.push(SubDagError::UnexposedEntrypoint {
                node: parent_node.id.clone(),
                inner_port: name.clone(),
                parent_inputs: parent_node.inputs.iter().map(|p| p.name.clone()).collect(),
            });
        }
    }

    // Inverse check: inner boundaries not exposed on parent
    let mut seen_boundary_names = std::collections::HashSet::new();
    for (_, name) in &boundaries.boundary_ports {
        if seen_boundary_names.insert(name.clone())
            && !parent_node.outputs.iter().any(|p| p.name == *name)
        {
            errors.push(SubDagError::UnexposedBoundary {
                node: parent_node.id.clone(),
                inner_port: name.clone(),
                parent_outputs: parent_node.outputs.iter().map(|p| p.name.clone()).collect(),
            });
        }
    }
}

fn check_type_match(
    node: &NodeId,
    port: &PortName,
    direction: PortDirection,
    parent_type: &TypeId,
    inner_type: &TypeId,
    registry: &TypeRegistry,
    errors: &mut Vec<SubDagError>,
) {
    if let Err(error) = registry.validate_type_expr(parent_type) {
        errors.push(SubDagError::InvalidTypeExpression {
            node: node.clone(),
            port: port.clone(),
            direction,
            type_id: parent_type.clone(),
            error,
            source: TypeExprSource::Parent,
        });
        return;
    }

    if let Err(error) = registry.validate_type_expr(inner_type) {
        errors.push(SubDagError::InvalidTypeExpression {
            node: node.clone(),
            port: port.clone(),
            direction,
            type_id: inner_type.clone(),
            error,
            source: TypeExprSource::Inner,
        });
        return;
    }

    let (flow_from, flow_to) = match direction {
        PortDirection::Input => (parent_type, inner_type),
        PortDirection::Output => (inner_type, parent_type),
    };

    if !registry.is_compatible(flow_from, flow_to) {
        errors.push(SubDagError::TypeMismatch {
            node: node.clone(),
            port: port.clone(),
            direction,
            parent_type: parent_type.clone(),
            inner_type: inner_type.clone(),
        });
        return;
    }

    if !registry.is_compatible_strict_semantic(flow_from, flow_to) {
        errors.push(SubDagError::SemanticCarrierMismatch {
            node: node.clone(),
            port: port.clone(),
            direction,
            parent_type: parent_type.clone(),
            inner_type: inner_type.clone(),
            parent_kind: parent_type.semantic_carrier_kind(),
            inner_kind: inner_type.semantic_carrier_kind(),
        });
    }
}

fn find_output_port<'a, T>(node: &'a Node<T>, port_name: &PortName) -> Option<&'a Port> {
    node.outputs.iter().find(|p| &p.name == port_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::{build::*, Dag, Edge};
    use crate::node::{NodeBody, NodeKind};

    #[test]
    fn test_valid_subdag_passes() {
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![port("data", "String")],
            vec![port("result", "String")],
            (),
        ));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag("wrapper", inner));

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_inferred_subdag_always_valid() {
        // With auto-inference, Node::subdag always produces matching ports.
        // Verify the inferred ports match the inner DAG.
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![port("config", "String")],
            vec![port("result", "String")],
            (),
        ));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag("wrapper", inner));

        // Auto-inferred: input="config", output="result"
        let wrapper = &dag.nodes[0];
        assert!(wrapper.inputs.iter().any(|p| p.name.0 == "config"));
        assert!(wrapper.outputs.iter().any(|p| p.name.0 == "result"));

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_missing_entrypoint_manual_construction() {
        // Manually construct a SubDag node with wrong ports to test validation
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![port("config", "String")],
            vec![port("result", "String")],
            (),
        ));

        // Manually create node with wrong input port name
        let bad_node = Node {
            id: NodeId::new("wrapper"),
            inputs: vec![port("data", "String")], // "data" != "config"
            outputs: vec![port("result", "String")],
            body: NodeBody::SubDag(inner),
            examples: Vec::new(),
            log_detail: None,
            kind: NodeKind::Pure,
            operation_key: None,
            transport_class: None,
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        // 2 errors: parent "data" not in inner + inner "config" not on parent
        assert_eq!(errors.len(), 2, "expected 2 errors, got: {:?}", errors);
        assert!(errors
            .iter()
            .any(|e| matches!(e, SubDagError::NoInnerEntrypoint { port, .. } if port.0 == "data")));
        assert!(errors.iter().any(|e| matches!(e, SubDagError::UnexposedEntrypoint { inner_port, .. } if inner_port.0 == "config")));
    }

    #[test]
    fn test_missing_boundary_manual_construction() {
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![port("data", "String")],
            vec![port("output", "String")],
            (),
        ));

        // Manually create node with wrong output port name
        let bad_node = Node {
            id: NodeId::new("wrapper"),
            inputs: vec![port("data", "String")],
            outputs: vec![port("result", "String")], // "result" != "output"
            body: NodeBody::SubDag(inner),
            examples: Vec::new(),
            log_detail: None,
            kind: NodeKind::Pure,
            operation_key: None,
            transport_class: None,
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        // 2 errors: parent "result" not in inner + inner "output" not on parent
        assert_eq!(errors.len(), 2, "expected 2 errors, got: {:?}", errors);
        assert!(errors
            .iter()
            .any(|e| matches!(e, SubDagError::NoInnerBoundary { port, .. } if port.0 == "result")));
        assert!(errors.iter().any(|e| matches!(e, SubDagError::UnexposedBoundary { inner_port, .. } if inner_port.0 == "output")));
    }

    #[test]
    fn test_type_mismatch_on_input_manual_construction() {
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![port("data", "Int")],
            vec![port("result", "String")],
            (),
        ));

        // Manually declare input as String but inner has Int
        let bad_node = Node {
            id: NodeId::new("wrapper"),
            inputs: vec![port("data", "String")],
            outputs: vec![port("result", "String")],
            body: NodeBody::SubDag(inner),
            examples: Vec::new(),
            log_detail: None,
            kind: NodeKind::Pure,
            operation_key: None,
            transport_class: None,
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            SubDagError::TypeMismatch {
                direction: PortDirection::Input,
                ..
            }
        ));
    }

    #[test]
    fn test_type_mismatch_on_output_manual_construction() {
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![port("data", "String")],
            vec![port("result", "Int")],
            (),
        ));

        // Manually declare output as String but inner has Int
        let bad_node = Node {
            id: NodeId::new("wrapper"),
            inputs: vec![port("data", "String")],
            outputs: vec![port("result", "String")],
            body: NodeBody::SubDag(inner),
            examples: Vec::new(),
            log_detail: None,
            kind: NodeKind::Pure,
            operation_key: None,
            transport_class: None,
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            SubDagError::TypeMismatch {
                direction: PortDirection::Output,
                ..
            }
        ));
    }

    #[test]
    fn test_semantic_carrier_mismatch_on_input_manual_construction() {
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![port("auth", "Any")],
            vec![port("result", "String")],
            (),
        ));

        // Parent -> inner is structurally compatible (Credential -> Any) but
        // semantically unsafe in strict mode.
        let bad_node = Node {
            id: NodeId::new("wrapper"),
            inputs: vec![port("auth", "Credential")],
            outputs: vec![port("result", "String")],
            body: NodeBody::SubDag(inner),
            examples: Vec::new(),
            log_detail: None,
            kind: NodeKind::Pure,
            operation_key: None,
            transport_class: None,
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            SubDagError::SemanticCarrierMismatch {
                direction: PortDirection::Input,
                ..
            }
        ));
    }

    #[test]
    fn test_nested_subdag_validation() {
        // With auto-inference, nested SubDags also have matching ports.
        // Verify recursive validation passes for well-formed nested SubDags.
        let mut deep_inner: Dag<()> = Dag::new();
        deep_inner.add_node(Node::opaque(
            "deep",
            vec![port("x", "String")],
            vec![port("y", "String")],
            (),
        ));

        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::subdag("nested", deep_inner));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag("wrapper", inner));

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_fanout_entrypoint_valid() {
        // Two inner nodes with the same input port name (fan-out case)
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "a",
            vec![port("data", "String")],
            vec![port("out1", "String")],
            (),
        ));
        inner.add_node(Node::opaque(
            "b",
            vec![port("data", "String")],
            vec![port("out2", "String")],
            (),
        ));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag("wrapper", inner));

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_connected_ports_not_entrypoints() {
        // Inner edge connects "a.out" -> "b.in", so "b.in" is NOT an entrypoint
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "a",
            vec![port("data", "String")],
            vec![port("mid", "String")],
            (),
        ));
        inner.add_node(Node::opaque(
            "b",
            vec![port("mid", "String")],
            vec![port("result", "String")],
            (),
        ));
        inner.add_edge(Edge::new("a", "mid", "b", "mid"));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag("wrapper", inner));

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_opaque_nodes_skipped() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "plain",
            vec![port("in", "String")],
            vec![port("out", "String")],
            (),
        ));

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_multiple_errors_manual_construction() {
        let inner: Dag<()> = Dag::new(); // empty inner — no entrypoints or boundaries

        // Manually construct with ports that don't exist in inner DAG
        let bad_node = Node {
            id: NodeId::new("broken"),
            inputs: vec![port("in1", "String"), port("in2", "Int")],
            outputs: vec![port("out", "String")],
            body: NodeBody::SubDag(inner),
            examples: Vec::new(),
            log_detail: None,
            kind: NodeKind::Pure,
            operation_key: None,
            transport_class: None,
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        // Should have 3 errors: 2 missing entrypoints + 1 missing boundary
        assert_eq!(errors.len(), 3, "expected 3 errors, got: {:?}", errors);
    }

    #[test]
    fn test_empty_inner_inferred_has_no_ports() {
        // With auto-inference, an empty inner DAG produces no ports
        let inner: Dag<()> = Dag::new();

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag("empty", inner));

        let node = &dag.nodes[0];
        assert!(node.inputs.is_empty());
        assert!(node.outputs.is_empty());

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_unexposed_entrypoint_detected() {
        // Inner DAG has entrypoints that the parent doesn't expose
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![port("data", "String"), port("config", "String")],
            vec![port("result", "String")],
            (),
        ));

        // Parent only exposes "data", missing "config"
        let bad_node = Node {
            id: NodeId::new("wrapper"),
            inputs: vec![port("data", "String")],
            outputs: vec![port("result", "String")],
            body: NodeBody::SubDag(inner),
            examples: Vec::new(),
            log_detail: None,
            kind: NodeKind::Pure,
            operation_key: None,
            transport_class: None,
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        assert_eq!(errors.len(), 1, "expected 1 error, got: {:?}", errors);
        assert!(matches!(
            &errors[0],
            SubDagError::UnexposedEntrypoint { inner_port, .. } if inner_port.0 == "config"
        ));
    }

    #[test]
    fn test_unexposed_boundary_detected() {
        // Inner DAG has boundaries that the parent doesn't expose
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![port("data", "String")],
            vec![port("result", "String"), port("log", "String")],
            (),
        ));

        // Parent only exposes "result", missing "log"
        let bad_node = Node {
            id: NodeId::new("wrapper"),
            inputs: vec![port("data", "String")],
            outputs: vec![port("result", "String")],
            body: NodeBody::SubDag(inner),
            examples: Vec::new(),
            log_detail: None,
            kind: NodeKind::Pure,
            operation_key: None,
            transport_class: None,
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        assert_eq!(errors.len(), 1, "expected 1 error, got: {:?}", errors);
        assert!(matches!(
            &errors[0],
            SubDagError::UnexposedBoundary { inner_port, .. } if inner_port.0 == "log"
        ));
    }

    #[test]
    fn test_duplicate_boundary_type_mismatch_caught() {
        // Two inner nodes have the same boundary name but different types.
        // Validation should catch the mismatched one.
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "a",
            vec![port("data", "String")],
            vec![port("result", "String")], // matches parent
            (),
        ));
        inner.add_node(Node::opaque(
            "b",
            vec![port("data", "String")],
            vec![port("result", "Int")], // type mismatch with parent
            (),
        ));

        // Manually construct: parent declares result: String
        let bad_node = Node {
            id: NodeId::new("wrapper"),
            inputs: vec![port("data", "String")],
            outputs: vec![port("result", "String")],
            body: NodeBody::SubDag(inner),
            examples: Vec::new(),
            log_detail: None,
            kind: NodeKind::Pure,
            operation_key: None,
            transport_class: None,
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        // Should catch the type mismatch on the second boundary
        assert_eq!(errors.len(), 1, "expected 1 error, got: {:?}", errors);
        assert!(matches!(
            &errors[0],
            SubDagError::TypeMismatch {
                direction: PortDirection::Output,
                ..
            }
        ));
    }

    // ============ validate_resource_wiring_recursive() tests ============

    #[test]
    fn test_recursive_resource_wiring_top_level() {
        // Top-level DAG with an unwired res:* port
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "node_a",
            vec![resource("platform", "Platform", AccessMode::Read)],
            vec![port("out", "String")],
            (),
        ));

        let unwired = validate_resource_wiring_recursive(&dag);
        assert_eq!(unwired.len(), 1);
        assert_eq!(unwired[0].port.0, "res:platform");
    }

    #[test]
    fn test_recursive_resource_wiring_nested_subdag() {
        // Inner DAG has an unwired res:* port
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![
                port("data", "String"),
                resource("file", "FilesystemHandle", AccessMode::Read),
            ],
            vec![port("result", "String")],
            (),
        ));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag("wrapper", inner));

        // The SubDag auto-infers the res:file port, so wrapper has it as input.
        // That means wrapper's res:file is an entrypoint on the outer DAG.
        let unwired = validate_resource_wiring_recursive(&dag);
        // Only the outer-level entrypoint should be reported — the inner DAG's
        // res:file is already exposed via auto-inference on the parent, so it's
        // deduplicated to avoid double-reporting.
        assert_eq!(unwired.len(), 1);
        assert_eq!(unwired[0].port.0, "res:file");
    }

    #[test]
    fn test_recursive_resource_wiring_deep_nested_subdag_dedupes_ancestor_resource() {
        // Deepest DAG has a boundary-style node requiring res:file.
        let mut deepest: Dag<()> = Dag::new();
        deepest.add_node(Node::opaque(
            "execute",
            vec![
                port("request", "TransportRequest"),
                resource("file", "FilesystemHandle", AccessMode::Read),
            ],
            vec![port("response", "TransportResponse")],
            (),
        ));

        // Middle DAG wraps deepest as "body" and leaves res:file as entrypoint.
        let mut middle: Dag<()> = Dag::new();
        middle.add_node(Node::subdag("body", deepest));

        // Top DAG wraps middle. Auto-inference bubbles res:file up to wrapper.
        let mut top: Dag<()> = Dag::new();
        top.add_node(Node::subdag("wrapper", middle));

        let unwired = validate_resource_wiring_recursive(&top);
        // Only the outermost wrapper res:file should be reported once.
        assert_eq!(unwired.len(), 1, "unexpected unwired: {:?}", unwired);
        assert_eq!(unwired[0].port.0, "res:file");
    }

    #[test]
    fn test_recursive_wiring_clean_when_no_resources() {
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![port("data", "String")],
            vec![port("result", "String")],
            (),
        ));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag("wrapper", inner));

        let unwired = validate_resource_wiring_recursive(&dag);
        // data is not a resource port, so no unwired resources
        let resource_unwired: Vec<_> = unwired
            .iter()
            .filter(|u| u.port.is_resource())
            .collect();
        assert!(resource_unwired.is_empty());
    }

    // NOTE: validate_no_operation_overlap tests removed.
    // Replaced by C22: Deductive Redundancy Elimination using idempotency fingerprints.
}
