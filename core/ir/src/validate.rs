//! SubDag interface validation.
//!
//! Validates that SubDag nodes' declared interfaces (input/output ports)
//! match the inner DAG's structural entrypoints and boundaries.
//!
//! This catches mismatches at build time rather than waiting for `lower()`
//! in the executor.

use crate::boundary::detect_boundaries;
use crate::dag::{Dag, Port};
use crate::entrypoint::detect_entrypoints;
use crate::node::{Node, NodeBody};
use crate::types::{NodeId, PortName, TypeId};
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
    /// Type mismatch between parent port and inner port.
    TypeMismatch {
        node: NodeId,
        port: PortName,
        direction: PortDirection,
        parent_type: TypeId,
        inner_type: TypeId,
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
            SubDagError::NoInnerEntrypoint { node, port, available } => {
                write!(
                    f,
                    "SubDag '{}': input port '{}' has no matching entrypoint in inner DAG (available: [{}])",
                    node, port,
                    available.iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join(", ")
                )
            }
            SubDagError::NoInnerBoundary { node, port, available } => {
                write!(
                    f,
                    "SubDag '{}': output port '{}' has no matching boundary in inner DAG (available: [{}])",
                    node, port,
                    available.iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join(", ")
                )
            }
            SubDagError::TypeMismatch { node, port, direction, parent_type, inner_type } => {
                write!(
                    f,
                    "SubDag '{}': {} port '{}' type mismatch: parent declares '{}', inner has '{}'",
                    node, direction, port, parent_type, inner_type
                )
            }
            SubDagError::Nested { parent, inner } => {
                write!(f, "in SubDag '{}': {}", parent, inner)
            }
        }
    }
}

impl std::error::Error for SubDagError {}

/// Validate all SubDag interfaces in a DAG, recursively.
///
/// For each SubDag node, checks that:
/// - Every parent input port has a matching entrypoint (by name) in the inner DAG
/// - Every parent output port has a matching boundary (by name) in the inner DAG
/// - Types match between parent ports and their inner counterparts
///
/// Returns all errors found (does not stop at the first).
pub fn validate_subdag_interfaces<T>(dag: &Dag<T>) -> Vec<SubDagError> {
    let mut errors = Vec::new();
    validate_dag_recursive(dag, &mut errors);
    errors
}

fn validate_dag_recursive<T>(dag: &Dag<T>, errors: &mut Vec<SubDagError>) {
    for node in &dag.nodes {
        if let NodeBody::SubDag(ref inner) = node.body {
            validate_single_subdag(node, inner, errors);
            // Recurse into the inner DAG
            let mut nested_errors = Vec::new();
            validate_dag_recursive(inner, &mut nested_errors);
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
                            errors,
                        );
                    }
                }
            }
        }
    }
}

fn check_type_match(
    node: &NodeId,
    port: &PortName,
    direction: PortDirection,
    parent_type: &TypeId,
    inner_type: &TypeId,
    errors: &mut Vec<SubDagError>,
) {
    if parent_type != inner_type {
        errors.push(SubDagError::TypeMismatch {
            node: node.clone(),
            port: port.clone(),
            direction,
            parent_type: parent_type.clone(),
            inner_type: inner_type.clone(),
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
    use crate::node::NodeBody;

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
        dag.add_node(Node::subdag(
            "wrapper",
            inner,
        ));

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
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        assert_eq!(errors.len(), 1);
        assert!(matches!(&errors[0], SubDagError::NoInnerEntrypoint { port, .. } if port.0 == "data"));
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
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        assert_eq!(errors.len(), 1);
        assert!(matches!(&errors[0], SubDagError::NoInnerBoundary { port, .. } if port.0 == "result"));
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
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            SubDagError::TypeMismatch { direction: PortDirection::Input, .. }
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
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            SubDagError::TypeMismatch { direction: PortDirection::Output, .. }
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
        dag.add_node(Node::subdag(
            "wrapper",
            inner,
        ));

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
        dag.add_node(Node::subdag(
            "wrapper",
            inner,
        ));

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
        };

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(bad_node);

        let errors = validate_subdag_interfaces(&dag);
        // Should catch the type mismatch on the second boundary
        assert_eq!(errors.len(), 1, "expected 1 error, got: {:?}", errors);
        assert!(matches!(
            &errors[0],
            SubDagError::TypeMismatch { direction: PortDirection::Output, .. }
        ));
    }
}
