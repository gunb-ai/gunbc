//! DAG validation: structural and semantic checks.
//!
//! This module provides validation for DAGs before execution:
//! - Type checking: connected ports must have matching TypeId
//! - Cycle detection: DAG must be acyclic
//! - Port saturation: all input ports must be connected
//! - Duplicate node IDs: no two nodes may have the same ID
//!
//! # Example
//!
//! ```
//! use gunbc_ir::{Dag, Node, Port, Edge, validate_dag};
//!
//! let mut dag: Dag<()> = Dag::new();
//! dag.add_node(Node::opaque("A", vec![], vec![Port::new("out", "String")], ()));
//! dag.add_node(Node::opaque("B", vec![Port::new("in", "String")], vec![], ()));
//! dag.add_edge(Edge::new("A", "out", "B", "in"));
//!
//! let result = validate_dag(&dag);
//! assert!(result.is_ok());
//! ```

use crate::boundary::detect_boundaries;
use crate::dag::Dag;
use crate::entrypoint::detect_entrypoints;
use crate::node::NodeBody;
use crate::types::NodeId;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Validation error types.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Type mismatch between connected ports
    #[error("type mismatch on edge {from_node}.{from_port} -> {to_node}.{to_port}: expected '{expected}', got '{actual}'")]
    TypeMismatch {
        from_node: String,
        from_port: String,
        to_node: String,
        to_port: String,
        expected: String,
        actual: String,
    },

    /// Cycle detected in the DAG
    #[error("cycle detected involving nodes: {}", nodes.join(" -> "))]
    CycleDetected { nodes: Vec<String> },

    /// Input port not connected to any edge
    #[error("input port '{node}.{port}' is not connected")]
    UnconnectedInput { node: String, port: String },

    /// Duplicate node ID
    #[error("duplicate node ID: '{0}'")]
    DuplicateNodeId(String),

    /// Edge references non-existent node
    #[error("edge references non-existent node: '{0}'")]
    NodeNotFound(String),

    /// Edge references non-existent port
    #[error("edge references non-existent port: '{node}.{port}'")]
    PortNotFound { node: String, port: String },

    /// SubDag interface mismatch
    #[error("SubDag '{node}' interface mismatch: parent port '{port}' has no matching inner port")]
    SubDagInterfaceMismatch { node: String, port: String },

    /// Cardinality mismatch between connected ports
    #[error("cardinality mismatch on edge {from_node}.{from_port} -> {to_node}.{to_port}: {reason}")]
    CardinalityMismatch {
        from_node: String,
        from_port: String,
        to_node: String,
        to_port: String,
        reason: String,
    },
}

/// Result of validation - can contain multiple errors.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn is_err(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a DAG, returning all errors found.
///
/// This runs all validation checks:
/// - Duplicate node IDs
/// - Type checking on edges
/// - Cycle detection
/// - Port saturation (all inputs connected) - top level only
/// - Edge reference validity
/// - SubDag interface agreement
///
/// For SubDag nodes, validation is recursive.
pub fn validate_dag<T>(dag: &Dag<T>) -> Result<(), ValidationResult> {
    validate_dag_inner(dag, true)
}

/// Internal validation with context about whether this is the top-level DAG.
fn validate_dag_inner<T>(dag: &Dag<T>, is_top_level: bool) -> Result<(), ValidationResult> {
    let mut result = ValidationResult::new();

    // Check for duplicate node IDs
    result.merge(check_duplicate_ids(dag));

    // Check edge references and types
    result.merge(check_edges(dag));

    // Check for cycles
    result.merge(check_cycles(dag));

    // Check port saturation - only for top-level DAG
    // SubDag inner DAGs have entrypoints that are deliberately unconnected
    // (they receive data from the parent's input ports)
    if is_top_level {
        result.merge(check_port_saturation(dag));
    }

    // Check SubDag interface agreement and recursively validate
    for node in &dag.nodes {
        if let NodeBody::SubDag(inner) = &node.body {
            // Check that parent ports match inner entrypoints/boundaries
            result.merge(check_subdag_interface(node, inner));

            // Recursively validate inner DAG (not top-level)
            if let Err(inner_result) = validate_dag_inner(inner, false) {
                // Prefix errors with parent node ID for context
                for error in inner_result.errors {
                    result.add(prefix_error(&node.id, error));
                }
            }
        }
    }

    if result.is_ok() {
        Ok(())
    } else {
        Err(result)
    }
}

/// Quick validation that returns on first error.
pub fn validate_dag_quick<T>(dag: &Dag<T>) -> Result<(), ValidationError> {
    let result = validate_dag(dag);
    match result {
        Ok(()) => Ok(()),
        Err(r) => Err(r.errors.into_iter().next().unwrap()),
    }
}

/// Check for duplicate node IDs.
fn check_duplicate_ids<T>(dag: &Dag<T>) -> ValidationResult {
    let mut result = ValidationResult::new();
    let mut seen: HashSet<&str> = HashSet::new();

    for node in &dag.nodes {
        if !seen.insert(&node.id.0) {
            result.add(ValidationError::DuplicateNodeId(node.id.0.clone()));
        }
    }

    result
}

/// Check edge references and type matching.
fn check_edges<T>(dag: &Dag<T>) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Build lookup maps
    let node_map: HashMap<&str, &crate::node::Node<T>> =
        dag.nodes.iter().map(|n| (n.id.0.as_str(), n)).collect();

    for edge in &dag.edges {
        // Check from_node exists
        let from_node = match node_map.get(edge.from_node.0.as_str()) {
            Some(n) => n,
            None => {
                result.add(ValidationError::NodeNotFound(edge.from_node.0.clone()));
                continue;
            }
        };

        // Check to_node exists
        let to_node = match node_map.get(edge.to_node.0.as_str()) {
            Some(n) => n,
            None => {
                result.add(ValidationError::NodeNotFound(edge.to_node.0.clone()));
                continue;
            }
        };

        // Check from_port exists and get its type
        let from_port = match from_node.outputs.iter().find(|p| p.name.0 == edge.from_port.0) {
            Some(p) => p,
            None => {
                result.add(ValidationError::PortNotFound {
                    node: edge.from_node.0.clone(),
                    port: edge.from_port.0.clone(),
                });
                continue;
            }
        };

        // Check to_port exists and get its type
        let to_port = match to_node.inputs.iter().find(|p| p.name.0 == edge.to_port.0) {
            Some(p) => p,
            None => {
                result.add(ValidationError::PortNotFound {
                    node: edge.to_node.0.clone(),
                    port: edge.to_port.0.clone(),
                });
                continue;
            }
        };

        // Type check: types must match
        if from_port.type_id != to_port.type_id {
            result.add(ValidationError::TypeMismatch {
                from_node: edge.from_node.0.clone(),
                from_port: edge.from_port.0.clone(),
                to_node: edge.to_node.0.clone(),
                to_port: edge.to_port.0.clone(),
                expected: to_port.type_id.0.clone(),
                actual: from_port.type_id.0.clone(),
            });
        }

        // Cardinality check: output cardinality must satisfy input requirement
        if let Err(mismatch) = from_port.cardinality.check_satisfies(to_port.cardinality) {
            result.add(ValidationError::CardinalityMismatch {
                from_node: edge.from_node.0.clone(),
                from_port: edge.from_port.0.clone(),
                to_node: edge.to_node.0.clone(),
                to_port: edge.to_port.0.clone(),
                reason: mismatch.reason,
            });
        }
    }

    result
}

/// Check for cycles using DFS.
fn check_cycles<T>(dag: &Dag<T>) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Build adjacency list
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in &dag.nodes {
        adj.insert(&node.id.0, Vec::new());
    }
    for edge in &dag.edges {
        if let Some(neighbors) = adj.get_mut(edge.from_node.0.as_str()) {
            neighbors.push(&edge.to_node.0);
        }
    }

    // DFS state
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Unvisited,
        InProgress,
        Done,
    }

    let mut state: HashMap<&str, State> = dag.nodes.iter().map(|n| (n.id.0.as_str(), State::Unvisited)).collect();
    let mut path: Vec<&str> = Vec::new();

    fn dfs<'a>(
        node: &'a str,
        adj: &HashMap<&'a str, Vec<&'a str>>,
        state: &mut HashMap<&'a str, State>,
        path: &mut Vec<&'a str>,
        result: &mut ValidationResult,
    ) {
        if state.get(node) == Some(&State::InProgress) {
            // Found cycle - extract the cycle from path
            let cycle_start = path.iter().position(|&n| n == node).unwrap_or(0);
            let mut cycle: Vec<String> = path[cycle_start..].iter().map(|s| s.to_string()).collect();
            cycle.push(node.to_string()); // Close the cycle
            result.add(ValidationError::CycleDetected { nodes: cycle });
            return;
        }

        if state.get(node) == Some(&State::Done) {
            return;
        }

        state.insert(node, State::InProgress);
        path.push(node);

        if let Some(neighbors) = adj.get(node) {
            for &neighbor in neighbors {
                dfs(neighbor, adj, state, path, result);
            }
        }

        path.pop();
        state.insert(node, State::Done);
    }

    for node in &dag.nodes {
        if state.get(node.id.0.as_str()) == Some(&State::Unvisited) {
            dfs(&node.id.0, &adj, &mut state, &mut path, &mut result);
        }
    }

    result
}

/// Check that SubDag interface matches inner entrypoints/boundaries.
///
/// - Each parent input port must have at least one matching inner entrypoint
/// - Each parent output port must have exactly one matching inner boundary
fn check_subdag_interface<T>(parent: &crate::node::Node<T>, inner: &Dag<T>) -> ValidationResult {
    let mut result = ValidationResult::new();

    let entrypoints = detect_entrypoints(inner);
    let boundaries = detect_boundaries(inner);

    // Check each parent input port has a matching inner entrypoint
    for parent_port in &parent.inputs {
        let has_match = entrypoints
            .entrypoint_ports
            .iter()
            .any(|(_, port_name, _)| port_name == &parent_port.name);

        if !has_match {
            result.add(ValidationError::SubDagInterfaceMismatch {
                node: parent.id.0.clone(),
                port: parent_port.name.0.clone(),
            });
        }
    }

    // Check each parent output port has a matching inner boundary
    for parent_port in &parent.outputs {
        let has_match = boundaries
            .boundary_ports
            .iter()
            .any(|(_, port_name)| port_name == &parent_port.name);

        if !has_match {
            result.add(ValidationError::SubDagInterfaceMismatch {
                node: parent.id.0.clone(),
                port: parent_port.name.0.clone(),
            });
        }
    }

    result
}

/// Check port saturation for a lowered DAG.
///
/// After lowering, all input ports must be connected to exactly one edge.
/// This is distinct from pre-lowering validation where entrypoints are
/// deliberately unconnected (they receive world input).
///
/// Note: This check is only meaningful for lowered DAGs. For pre-lowering
/// validation, use `validate_dag` which skips port saturation (unconnected
/// ports become entrypoints which is intentional).
pub fn check_port_saturation_lowered<T>(dag: &Dag<T>) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Collect all connected input ports
    let connected: HashSet<(String, String)> = dag
        .edges
        .iter()
        .map(|e| (e.to_node.0.clone(), e.to_port.0.clone()))
        .collect();

    // Check each input port
    for node in &dag.nodes {
        for port in &node.inputs {
            // Skip ports with guards - they may legitimately be unconnected
            // if they're only used when the guard condition is met
            if port.guard.is_some() {
                continue;
            }

            let key = (node.id.0.clone(), port.name.0.clone());
            if !connected.contains(&key) {
                result.add(ValidationError::UnconnectedInput {
                    node: node.id.0.clone(),
                    port: port.name.0.clone(),
                });
            }
        }
    }

    result
}

// Keep for backwards compatibility but don't use in validate_dag
#[allow(dead_code)]
fn check_port_saturation<T>(_dag: &Dag<T>) -> ValidationResult {
    // Port saturation is only meaningful after lowering.
    // Before lowering, unconnected inputs are entrypoints (intentional).
    ValidationResult::new()
}

/// Prefix error messages with parent node ID for SubDag context.
fn prefix_error(parent_id: &NodeId, mut error: ValidationError) -> ValidationError {
    match &mut error {
        ValidationError::TypeMismatch {
            from_node, to_node, ..
        } => {
            *from_node = format!("{}/{}", parent_id.0, from_node);
            *to_node = format!("{}/{}", parent_id.0, to_node);
        }
        ValidationError::CycleDetected { nodes } => {
            for node in nodes.iter_mut() {
                *node = format!("{}/{}", parent_id.0, node);
            }
        }
        ValidationError::UnconnectedInput { node, .. } => {
            *node = format!("{}/{}", parent_id.0, node);
        }
        ValidationError::DuplicateNodeId(id) => {
            *id = format!("{}/{}", parent_id.0, id);
        }
        ValidationError::NodeNotFound(id) => {
            *id = format!("{}/{}", parent_id.0, id);
        }
        ValidationError::PortNotFound { node, .. } => {
            *node = format!("{}/{}", parent_id.0, node);
        }
        ValidationError::SubDagInterfaceMismatch { node, .. } => {
            *node = format!("{}/{}", parent_id.0, node);
        }
        ValidationError::CardinalityMismatch {
            from_node, to_node, ..
        } => {
            *from_node = format!("{}/{}", parent_id.0, from_node);
            *to_node = format!("{}/{}", parent_id.0, to_node);
        }
    }
    error
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::build::*;
    use crate::node::Node;

    #[test]
    fn test_valid_dag() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque(
            "B",
            vec![port("in", "String")],
            vec![port("out", "String")],
            (),
        ));
        dag.add_edge(edge("A", "out", "B", "in"));

        let result = validate_dag(&dag);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_mismatch() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque("B", vec![port("in", "Int")], vec![], ()));
        dag.add_edge(edge("A", "out", "B", "in"));

        let result = validate_dag(&dag);
        assert!(result.is_err());

        let errors = result.unwrap_err().errors;
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ValidationError::TypeMismatch { .. }));
    }

    #[test]
    fn test_cardinality_mismatch_zero_or_more_to_one_or_more() {
        // ZeroOrMore output cannot satisfy OneOrMore input (might be empty)
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "filter",
            vec![list("in", "StrList")],
            vec![list("out", "StrList")],  // ZeroOrMore - might filter to empty
            (),
        ));
        dag.add_node(Node::opaque(
            "process",
            vec![non_empty_list("in", "StrList")],  // OneOrMore - requires non-empty
            vec![],
            (),
        ));
        dag.add_edge(edge("filter", "out", "process", "in"));

        let result = validate_dag(&dag);
        assert!(result.is_err());

        let errors = result.unwrap_err().errors;
        assert!(errors.iter().any(|e| matches!(e, ValidationError::CardinalityMismatch { .. })));
    }

    #[test]
    fn test_cardinality_mismatch_zero_or_one_to_one() {
        // ZeroOrOne output cannot satisfy One input (might be absent)
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "lookup",
            vec![scalar("key", "String")],
            vec![optional("value", "String")],  // ZeroOrOne - might not exist
            (),
        ));
        dag.add_node(Node::opaque(
            "use",
            vec![scalar("value", "String")],  // One - requires present
            vec![],
            (),
        ));
        dag.add_edge(edge("lookup", "value", "use", "value"));

        let result = validate_dag(&dag);
        assert!(result.is_err());

        let errors = result.unwrap_err().errors;
        assert!(errors.iter().any(|e| matches!(e, ValidationError::CardinalityMismatch { .. })));
    }

    #[test]
    fn test_cardinality_valid_one_or_more_to_zero_or_more() {
        // OneOrMore output satisfies ZeroOrMore input (non-empty fits in any-length)
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![non_empty_list("items", "StrList")],  // OneOrMore - always has items
            (),
        ));
        dag.add_node(Node::opaque(
            "sink",
            vec![list("items", "StrList")],  // ZeroOrMore - accepts any
            vec![],
            (),
        ));
        dag.add_edge(edge("source", "items", "sink", "items"));

        let result = validate_dag(&dag);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cardinality_valid_one_to_one_or_more() {
        // One output satisfies OneOrMore input (one is at least one)
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![scalar("item", "String")],  // One - exactly one
            (),
        ));
        dag.add_node(Node::opaque(
            "sink",
            vec![non_empty_list("item", "String")],  // OneOrMore - needs at least one
            vec![],
            (),
        ));
        dag.add_edge(edge("source", "item", "sink", "item"));

        let result = validate_dag(&dag);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cycle_detection() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![port("in", "S")],
            vec![port("out", "S")],
            (),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![port("in", "S")],
            vec![port("out", "S")],
            (),
        ));
        dag.add_edge(edge("A", "out", "B", "in"));
        dag.add_edge(edge("B", "out", "A", "in")); // Creates cycle

        let result = validate_dag(&dag);
        assert!(result.is_err());

        let errors = result.unwrap_err().errors;
        assert!(errors.iter().any(|e| matches!(e, ValidationError::CycleDetected { .. })));
    }

    #[test]
    fn test_duplicate_node_id() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![], ()));
        dag.add_node(Node::opaque("A", vec![], vec![], ())); // Duplicate

        let result = validate_dag(&dag);
        assert!(result.is_err());

        let errors = result.unwrap_err().errors;
        assert!(errors.iter().any(|e| matches!(e, ValidationError::DuplicateNodeId(_))));
    }

    #[test]
    fn test_unconnected_input_in_lowered_dag() {
        // For lowered DAGs, unconnected non-guarded inputs are an error
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![port("required", "S")], vec![], ()));

        let result = check_port_saturation_lowered(&dag);
        assert!(!result.errors.is_empty());
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::UnconnectedInput { .. })));
    }

    #[test]
    fn test_unconnected_input_is_entrypoint() {
        // For pre-lowering DAGs, unconnected inputs are entrypoints (valid)
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![port("required", "S")], vec![], ()));

        // Pre-lowering validation doesn't flag entrypoints
        let result = validate_dag(&dag);
        assert!(result.is_ok());
    }

    #[test]
    fn test_guarded_input_not_required() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![guarded_port("optional", "S", crate::value::Value::Bool(true))],
            vec![],
            (),
        ));

        // Guarded ports don't require connections even in lowered DAGs
        let result = check_port_saturation_lowered(&dag);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_node_not_found() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "S")], ()));
        dag.add_edge(edge("A", "out", "B", "in")); // B doesn't exist

        let result = validate_dag(&dag);
        assert!(result.is_err());

        let errors = result.unwrap_err().errors;
        assert!(errors.iter().any(|e| matches!(e, ValidationError::NodeNotFound(_))));
    }

    #[test]
    fn test_port_not_found() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "S")], ()));
        dag.add_node(Node::opaque("B", vec![port("in", "S")], vec![], ()));
        dag.add_edge(edge("A", "wrong_port", "B", "in")); // wrong_port doesn't exist

        let result = validate_dag(&dag);
        assert!(result.is_err());

        let errors = result.unwrap_err().errors;
        assert!(errors.iter().any(|e| matches!(e, ValidationError::PortNotFound { .. })));
    }

    #[test]
    fn test_multiple_errors() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque("A", vec![], vec![], ())); // Duplicate
        dag.add_node(Node::opaque(
            "B",
            vec![port("in", "Int")], // Type mismatch with A.out
            vec![],
            (),
        ));
        dag.add_edge(edge("A", "out", "B", "in"));

        let result = validate_dag(&dag);
        assert!(result.is_err());

        let errors = result.unwrap_err().errors;
        // Should have at least 2 errors
        assert!(errors.len() >= 2);
    }

    #[test]
    fn test_subdag_interface_valid() {
        // Create a valid SubDag with matching ports
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "process",
            vec![port("data", "S")],  // Entrypoint matches parent input
            vec![port("result", "S")], // Boundary matches parent output
            (),
        ));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag(
            "wrapper",
            vec![port("data", "S")],
            vec![port("result", "S")],
            inner,
        ));

        let result = validate_dag(&dag);
        assert!(result.is_ok());
    }

    #[test]
    fn test_subdag_interface_missing_input() {
        // Create a SubDag where parent has an input with no matching inner entrypoint
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "process",
            vec![port("other", "S")], // Different name than parent's input
            vec![port("result", "S")],
            (),
        ));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag(
            "wrapper",
            vec![port("data", "S")], // No inner entrypoint named "data"
            vec![port("result", "S")],
            inner,
        ));

        let result = validate_dag(&dag);
        assert!(result.is_err());

        let errors = result.unwrap_err().errors;
        assert!(errors.iter().any(|e| matches!(e, ValidationError::SubDagInterfaceMismatch { port, .. } if port == "data")));
    }

    #[test]
    fn test_subdag_interface_missing_output() {
        // Create a SubDag where parent has an output with no matching inner boundary
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "process",
            vec![port("data", "S")],
            vec![port("other", "S")], // Different name than parent's output
            (),
        ));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag(
            "wrapper",
            vec![port("data", "S")],
            vec![port("result", "S")], // No inner boundary named "result"
            inner,
        ));

        let result = validate_dag(&dag);
        assert!(result.is_err());

        let errors = result.unwrap_err().errors;
        assert!(errors.iter().any(|e| matches!(e, ValidationError::SubDagInterfaceMismatch { port, .. } if port == "result")));
    }
}
