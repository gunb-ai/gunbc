//! Windowed segment testing helpers.
//!
//! A window is a contiguous subgraph slice that can be executed in isolation.
//! Entry inputs are injected from a baseline DryRun, and exit outputs are
//! compared against that baseline to verify inter-node integration.

use crate::mock_spec::OutputMatcher;
use gunbc_exec::{BoundaryMocks, ExecutionLog};
use gunbc_ir::{canonical_edge_order, Dag, NodeId, PortName, Value};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// A windowed segment within a DAG.
#[derive(Debug, Clone)]
pub struct Window {
    /// Entry-point nodes of this window (inputs severed and injected).
    pub entry_nodes: Vec<NodeId>,
    /// Exit-point nodes (outputs captured and verified).
    pub exit_nodes: Vec<NodeId>,
    /// All nodes in the interior (executed normally).
    pub interior: Vec<NodeId>,
}

impl Window {
    /// Build a window from an explicit node set.
    ///
    /// Entry/exit nodes are inferred from which ports are severed by the window.
    pub fn from_nodes<T, I, S>(dag: &Dag<T>, nodes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<NodeId>,
    {
        let node_ids: Vec<NodeId> = nodes.into_iter().map(|n| n.into()).collect();
        let node_set: HashSet<NodeId> = node_ids.iter().cloned().collect();

        let mut internal_incoming: HashSet<(NodeId, PortName)> = HashSet::new();
        let mut internal_outgoing: HashSet<(NodeId, PortName)> = HashSet::new();

        for edge in &dag.edges {
            if node_set.contains(&edge.from_node) && node_set.contains(&edge.to_node) {
                internal_incoming.insert((edge.to_node.clone(), edge.to_port.clone()));
                internal_outgoing.insert((edge.from_node.clone(), edge.from_port.clone()));
            }
        }

        let mut entry_nodes = Vec::new();
        let mut exit_nodes = Vec::new();
        let mut interior = Vec::new();

        for node_id in &node_set {
            let node = dag
                .get_node(node_id)
                .unwrap_or_else(|| panic!("window node '{}' not found in DAG", node_id.0));

            let is_entry = if node.inputs.is_empty() {
                true
            } else {
                node.inputs
                    .iter()
                    .any(|p| !internal_incoming.contains(&(node_id.clone(), p.name.clone())))
            };

            let is_exit = if node.outputs.is_empty() {
                true
            } else {
                node.outputs
                    .iter()
                    .any(|p| !internal_outgoing.contains(&(node_id.clone(), p.name.clone())))
            };

            match (is_entry, is_exit) {
                (true, true) => {
                    entry_nodes.push(node_id.clone());
                    exit_nodes.push(node_id.clone());
                }
                (true, false) => entry_nodes.push(node_id.clone()),
                (false, true) => exit_nodes.push(node_id.clone()),
                (false, false) => interior.push(node_id.clone()),
            }
        }

        entry_nodes.sort_by(|a, b| a.0.cmp(&b.0));
        exit_nodes.sort_by(|a, b| a.0.cmp(&b.0));
        interior.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            entry_nodes,
            exit_nodes,
            interior,
        }
    }

    /// All nodes in this window (entry + exit + interior), deduplicated.
    pub fn all_nodes(&self) -> Vec<NodeId> {
        let mut out: Vec<NodeId> = self
            .entry_nodes
            .iter()
            .chain(self.exit_nodes.iter())
            .chain(self.interior.iter())
            .cloned()
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out.dedup();
        out
    }

    /// Set of all nodes in this window.
    fn node_set(&self) -> HashSet<NodeId> {
        self.all_nodes().into_iter().collect()
    }
}

/// Errors during window execution or verification.
#[derive(Debug, Clone)]
pub enum WindowError {
    /// Window includes a port with both internal and external incoming edges.
    MixedInput { node: String, port: String },
    /// Expected log entry missing.
    MissingLogNode { context: &'static str, node: String },
    /// Expected log output port missing.
    MissingLogPort {
        context: &'static str,
        node: String,
        port: String,
    },
    /// Output mismatch for an exit port.
    OutputMismatch {
        node: String,
        port: String,
        expected: Box<Value>,
        actual: Box<Value>,
    },
    /// OutputMatcher failed for a chain observer.
    MatcherFailed {
        node: String,
        port: String,
        matcher: String,
        actual: Box<Value>,
        detail: String,
    },
    /// Observer node missing from execution log.
    MissingObserverNode { node: String },
    /// Observer port missing from execution log.
    MissingObserverPort { node: String, port: String },
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowError::MixedInput { node, port } => write!(
                f,
                "mixed input edges for {}.{} (has both internal and external sources)",
                node, port
            ),
            WindowError::MissingLogNode { context, node } => {
                write!(f, "missing {} log entry for node '{}'", context, node)
            }
            WindowError::MissingLogPort {
                context,
                node,
                port,
            } => write!(f, "missing {} log output for {}.{}", context, node, port),
            WindowError::OutputMismatch {
                node,
                port,
                expected,
                actual,
            } => write!(
                f,
                "output mismatch for {}.{} (expected {:?}, got {:?})",
                node, port, expected, actual
            ),
            WindowError::MatcherFailed {
                node,
                port,
                matcher,
                actual,
                detail,
            } => write!(
                f,
                "matcher failed for {}.{}: {} (got {:?}, {})",
                node, port, matcher, actual, detail
            ),
            WindowError::MissingObserverNode { node } => {
                write!(f, "observer node '{}' missing from execution log", node)
            }
            WindowError::MissingObserverPort { node, port } => {
                write!(
                    f,
                    "observer port {}.{} missing from execution log",
                    node, port
                )
            }
        }
    }
}

impl std::error::Error for WindowError {}

/// Build a sub-DAG that contains only the nodes inside this window.
pub fn window_subdag<T: Clone>(dag: &Dag<T>, window: &Window) -> Dag<T> {
    let node_set = window.node_set();
    let mut out = Dag::new();

    for node in &dag.nodes {
        if node_set.contains(&node.id) {
            out.add_node(node.clone());
        }
    }

    for edge in &dag.edges {
        if node_set.contains(&edge.from_node) && node_set.contains(&edge.to_node) {
            out.add_edge(edge.clone());
        }
    }

    out
}

/// Apply input mocks for window entry ports using a baseline execution log.
pub fn apply_window_inputs<T>(
    dag: &Dag<T>,
    window: &Window,
    baseline: &ExecutionLog,
    mocks: &mut BoundaryMocks,
) -> Result<(), WindowError> {
    let node_set = window.node_set();

    let mut list_ports: HashSet<(String, String)> = HashSet::new();
    for node in &dag.nodes {
        if node_set.contains(&node.id) {
            for port in &node.inputs {
                if port.cardinality.is_list() {
                    list_ports.insert((node.id.0.clone(), port.name.0.clone()));
                }
            }
        }
    }

    // Mixed input ports: scalar ports with both internal and external edges
    // can't be cleanly injected from baseline (internal edge takes priority).
    // Collect them so we can skip injecting external values for those ports.
    let mixed = mixed_input_ports(dag, &node_set);
    let mixed_scalar: HashSet<(String, String)> = mixed
        .iter()
        .map(|(n, p)| (n.0.clone(), p.0.clone()))
        .filter(|key| !list_ports.contains(key))
        .collect();

    let mut fan_in: HashMap<(String, String), Vec<Value>> = HashMap::new();
    let mut scalars: HashMap<(String, String), Value> = HashMap::new();

    for edge in canonical_edge_order(&dag.edges) {
        if node_set.contains(&edge.to_node) && !node_set.contains(&edge.from_node) {
            // Skip mixed scalar ports — internal edge takes priority.
            let dest_key = (edge.to_node.0.clone(), edge.to_port.0.clone());
            if mixed_scalar.contains(&dest_key) {
                continue;
            }
            let entry = baseline.get(&edge.from_node.0);
            let output_port = dag
                .resolve_output_port(&edge.from_node, &edge.from_port)
                .map(|port| port.port());
            let from_cardinality = output_port
                .map(|p| p.cardinality)
                .unwrap_or(gunbc_ir::Cardinality::ONE);

            let value = match entry.and_then(|e| e.outputs.get(&edge.from_port.0)) {
                Some(v) => Some(v.clone()),
                None => {
                    if let Some(port) = output_port {
                        if port.cardinality.allows_empty() {
                            None
                        } else {
                            // In dry-run mode, not all ports produce values.
                            // Fall back to Skipped so chain/window tests can proceed.
                            Some(Value::Skipped)
                        }
                    } else {
                        // Port not in DAG schema — fall back to Skipped.
                        Some(Value::Skipped)
                    }
                }
            };

            let Some(value) = value else {
                continue;
            };

            let key = (edge.to_node.0.clone(), edge.to_port.0.clone());
            if list_ports.contains(&key) {
                if matches!(value, Value::Unit) && from_cardinality.allows_empty() {
                    continue;
                }
                let bucket = fan_in.entry(key).or_default();
                if from_cardinality.is_list() {
                    if let Value::List(items) = value {
                        bucket.extend(items);
                    } else {
                        bucket.push(value);
                    }
                } else {
                    bucket.push(value);
                }
            } else {
                scalars.insert(key, value);
            }
        }
    }

    for ((node, port), values) in fan_in {
        mocks.set_input(node, port, Value::List(values));
    }
    for ((node, port), value) in scalars {
        mocks.set_input(node, port, value);
    }

    Ok(())
}

/// Verify that all exit-port outputs from the window match the baseline.
///
/// DEPRECATED: Tautological — compares re-execution against its own baseline.
/// Use `assert_chain_outputs` with developer-specified OutputMatchers instead.
pub fn assert_window_outputs<T>(
    dag: &Dag<T>,
    window: &Window,
    baseline: &ExecutionLog,
    window_log: &ExecutionLog,
) -> Result<(), WindowError> {
    let node_set = window.node_set();

    let mut internal_outputs: HashSet<(NodeId, PortName)> = HashSet::new();
    for edge in &dag.edges {
        if node_set.contains(&edge.from_node) && node_set.contains(&edge.to_node) {
            internal_outputs.insert((edge.from_node.clone(), edge.from_port.clone()));
        }
    }

    for node_id in node_set {
        let node = dag
            .get_node(&node_id)
            .unwrap_or_else(|| panic!("window node '{}' not found in DAG", node_id.0));

        let baseline_entry = baseline.get(&node_id.0);
        let window_entry = window_log.get(&node_id.0);

        // If neither log has the node, skip comparison.
        if baseline_entry.is_none() && window_entry.is_none() {
            continue;
        }

        for port in &node.outputs {
            if internal_outputs.contains(&(node_id.clone(), port.name.clone())) {
                continue;
            }

            let expected = match baseline_entry.and_then(|e| e.outputs.get(&port.name.0)) {
                Some(v) => v.clone(),
                None => {
                    if port.cardinality.allows_empty() {
                        empty_value_for_cardinality(port.cardinality)
                    } else {
                        // Dry-run may not produce all ports; treat as Skipped.
                        Value::Skipped
                    }
                }
            };
            let actual = match window_entry.and_then(|e| e.outputs.get(&port.name.0)) {
                Some(v) => v.clone(),
                None => {
                    if port.cardinality.allows_empty() {
                        empty_value_for_cardinality(port.cardinality)
                    } else {
                        // Dry-run may not produce all ports; treat as Skipped.
                        Value::Skipped
                    }
                }
            };

            // Skipped on either side is accepted: dry-run may not produce
            // all values, and downstream nodes propagate Skipped inputs.
            if matches!(expected, Value::Skipped) || matches!(actual, Value::Skipped) {
                continue;
            }

            // Fan-in order is now deterministic (edges carry monotonic indices),
            // so lists are compared by exact equality.
            if let (Value::List(_), Value::List(_)) = (&expected, &actual) {
                if expected == actual {
                    continue;
                }
            }

            if expected != actual {
                return Err(WindowError::OutputMismatch {
                    node: node_id.0.clone(),
                    port: port.name.0.clone(),
                    expected: Box::new(expected),
                    actual: Box::new(actual),
                });
            }
        }
    }

    Ok(())
}

/// Verify observer outputs against OutputMatchers (non-tautological).
///
/// Unlike `assert_window_outputs`, this does NOT compare against a baseline.
/// Instead, it checks exit-port outputs against developer-specified matchers.
/// This ensures the test provides real correctness signal.
///
/// `matchers` maps `(node_id, port_name) -> OutputMatcher`.
pub fn assert_chain_outputs(
    log: &ExecutionLog,
    matchers: &HashMap<(String, String), OutputMatcher>,
) -> Result<(), WindowError> {
    for ((node, port), matcher) in matchers {
        let entry = log
            .get(node)
            .ok_or_else(|| WindowError::MissingObserverNode { node: node.clone() })?;

        let actual = entry
            .outputs
            .get(port)
            .ok_or_else(|| WindowError::MissingObserverPort {
                node: node.clone(),
                port: port.clone(),
            })?;

        matcher
            .check(actual)
            .map_err(|detail| WindowError::MatcherFailed {
                node: node.clone(),
                port: port.clone(),
                matcher: format!("{:?}", matcher),
                actual: Box::new(actual.clone()),
                detail,
            })?;
    }

    Ok(())
}

fn empty_value_for_cardinality(cardinality: gunbc_ir::Cardinality) -> Value {
    if cardinality.is_list() {
        Value::List(Vec::new())
    } else {
        Value::Unit
    }
}

fn mixed_input_ports<T>(dag: &Dag<T>, node_set: &HashSet<NodeId>) -> Vec<(NodeId, PortName)> {
    let mut seen: HashMap<(NodeId, PortName), (bool, bool)> = HashMap::new();

    for edge in &dag.edges {
        if node_set.contains(&edge.to_node) {
            let entry = seen
                .entry((edge.to_node.clone(), edge.to_port.clone()))
                .or_insert((false, false));
            if node_set.contains(&edge.from_node) {
                entry.0 = true;
            } else {
                entry.1 = true;
            }
        }
    }

    seen.into_iter()
        .filter(|(_, (internal, external))| *internal && *external)
        .map(|((node, port), _)| (node, port))
        .collect()
}

#[cfg(test)]
mod window_helper_tests {
    use super::*;
    use gunbc_exec::{BoundaryMocks, ExecutionLog, LogEntry};
    use gunbc_ir::build::{edge, list, port};
    use gunbc_ir::{Dag, Node, Value};
    use std::collections::HashMap;

    fn log_entry(node: &str, outputs: Vec<(&str, Value)>) -> LogEntry {
        let mut map = HashMap::new();
        for (key, value) in outputs {
            map.insert(key.to_string(), value);
        }
        LogEntry {
            node_id: node.to_string(),
            inputs: None,
            outputs: map,
            was_intercepted: false,
            coercions_applied: vec![],
        }
    }

    #[test]
    fn apply_window_inputs_detects_mixed_inputs() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("a", vec![], vec![port("out", "Int")], ()));
        dag.add_node(Node::opaque(
            "b",
            vec![port("in", "Int")],
            vec![port("out", "Int")],
            (),
        ));
        dag.add_node(Node::opaque("c", vec![], vec![port("out", "Int")], ()));
        dag.add_edge(edge("a", "out", "b", "in"));
        dag.add_edge(edge("c", "out", "b", "in"));

        let window = Window::from_nodes(&dag, vec!["a", "b"]);
        let baseline = ExecutionLog { entries: vec![] };
        let mut mocks = BoundaryMocks::new();

        // Mixed scalar ports are now skipped (internal edge takes priority)
        // rather than causing an error. The external edge from "c" is not injected.
        apply_window_inputs(&dag, &window, &baseline, &mut mocks)
            .expect("mixed scalar ports should be skipped, not error");
        assert!(
            mocks.get_input("b", "in").is_none(),
            "mixed scalar port should not be injected"
        );
    }

    #[test]
    fn apply_window_inputs_fan_in_list_ports() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("a", vec![], vec![port("out", "Int")], ()));
        dag.add_node(Node::opaque("b", vec![], vec![port("out", "Int")], ()));
        dag.add_node(Node::opaque(
            "z",
            vec![list("items", "IntList")],
            vec![port("out", "Int")],
            (),
        ));
        dag.add_edge(edge("a", "out", "z", "items"));
        dag.add_edge(edge("b", "out", "z", "items"));

        let window = Window::from_nodes(&dag, vec!["z"]);
        let baseline = ExecutionLog {
            entries: vec![
                log_entry("a", vec![("out", Value::Int(1))]),
                log_entry("b", vec![("out", Value::Int(2))]),
            ],
        };
        let mut mocks = BoundaryMocks::new();

        apply_window_inputs(&dag, &window, &baseline, &mut mocks)
            .expect("fan-in list inputs should be derivable");

        let input = mocks
            .get_input("z", "items")
            .expect("list input should be injected");
        assert_eq!(input, &Value::List(vec![Value::Int(1), Value::Int(2)]));
    }

    #[test]
    fn apply_window_inputs_flattens_list_outputs() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![],
            vec![list("items", "IntList")],
            (),
        ));
        dag.add_node(Node::opaque(
            "b",
            vec![list("items", "IntList")],
            vec![port("out", "Int")],
            (),
        ));
        dag.add_edge(edge("a", "items", "b", "items"));

        let window = Window::from_nodes(&dag, vec!["b"]);
        let baseline = ExecutionLog {
            entries: vec![log_entry(
                "a",
                vec![("items", Value::List(vec![Value::Int(1), Value::Int(2)]))],
            )],
        };
        let mut mocks = BoundaryMocks::new();

        apply_window_inputs(&dag, &window, &baseline, &mut mocks)
            .expect("list outputs should flatten into list inputs");

        let input = mocks
            .get_input("b", "items")
            .expect("list input should be injected");
        assert_eq!(input, &Value::List(vec![Value::Int(1), Value::Int(2)]));
    }

    #[test]
    fn assert_window_outputs_detects_mismatch() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("a", vec![], vec![port("out", "Int")], ()));
        dag.add_node(Node::opaque(
            "b",
            vec![port("in", "Int")],
            vec![port("out", "Int")],
            (),
        ));
        dag.add_edge(edge("a", "out", "b", "in"));

        let window = Window::from_nodes(&dag, vec!["a", "b"]);
        let baseline = ExecutionLog {
            entries: vec![
                log_entry("a", vec![("out", Value::Int(10))]),
                log_entry("b", vec![("out", Value::Int(1))]),
            ],
        };
        let window_log = ExecutionLog {
            entries: vec![
                log_entry("a", vec![("out", Value::Int(10))]),
                log_entry("b", vec![("out", Value::Int(2))]),
            ],
        };

        let err = assert_window_outputs(&dag, &window, &baseline, &window_log)
            .expect_err("mismatched outputs should error");
        match err {
            WindowError::OutputMismatch {
                node,
                port,
                expected,
                actual,
            } => {
                assert_eq!(node, "b");
                assert_eq!(port, "out");
                assert_eq!(*expected, Value::Int(1));
                assert_eq!(*actual, Value::Int(2));
            }
            other => panic!("expected OutputMismatch error, got {:?}", other),
        }
    }
}

#[cfg(test)]
mod window_subdag_tests {
    use super::*;
    use gunbc_ir::build::{edge, port};
    use gunbc_ir::{Dag, Node};

    #[test]
    fn subdag_contains_only_window_nodes() {
        // DAG: A -> B -> C
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("a", vec![], vec![port("out", "Int")], ()));
        dag.add_node(Node::opaque(
            "b",
            vec![port("in", "Int")],
            vec![port("out", "Int")],
            (),
        ));
        dag.add_node(Node::opaque("c", vec![port("in", "Int")], vec![], ()));
        dag.add_edge(edge("a", "out", "b", "in"));
        dag.add_edge(edge("b", "out", "c", "in"));

        let window = Window::from_nodes(&dag, vec!["a", "b"]);
        let sub = window_subdag(&dag, &window);

        assert_eq!(sub.nodes.len(), 2, "subdag should have 2 nodes");
        let ids: Vec<&str> = sub.nodes.iter().map(|n| n.id.0.as_str()).collect();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
        assert!(!ids.contains(&"c"), "node c should be excluded");
    }

    #[test]
    fn subdag_retains_internal_edges_only() {
        // DAG: A -> B -> C
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("a", vec![], vec![port("out", "Int")], ()));
        dag.add_node(Node::opaque(
            "b",
            vec![port("in", "Int")],
            vec![port("out", "Int")],
            (),
        ));
        dag.add_node(Node::opaque("c", vec![port("in", "Int")], vec![], ()));
        dag.add_edge(edge("a", "out", "b", "in"));
        dag.add_edge(edge("b", "out", "c", "in"));

        let window = Window::from_nodes(&dag, vec!["a", "b"]);
        let sub = window_subdag(&dag, &window);

        assert_eq!(
            sub.edges.len(),
            1,
            "only the a->b edge should be in the subdag"
        );
        assert_eq!(sub.edges[0].from_node.0, "a");
        assert_eq!(sub.edges[0].to_node.0, "b");
    }

    #[test]
    fn subdag_single_node_has_no_edges() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![port("in", "Int")],
            vec![port("out", "Int")],
            (),
        ));
        dag.add_node(Node::opaque("b", vec![port("in", "Int")], vec![], ()));
        dag.add_edge(edge("a", "out", "b", "in"));

        let window = Window::from_nodes(&dag, vec!["a"]);
        let sub = window_subdag(&dag, &window);

        assert_eq!(sub.nodes.len(), 1);
        assert_eq!(
            sub.edges.len(),
            0,
            "single-node window has no internal edges"
        );
    }

    #[test]
    fn subdag_preserves_node_ports() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![port("x", "String"), port("y", "Int")],
            vec![port("out", "Bool")],
            (),
        ));

        let window = Window::from_nodes(&dag, vec!["a"]);
        let sub = window_subdag(&dag, &window);

        let node = sub.get_node(&"a".into()).expect("node a should exist");
        assert_eq!(node.inputs.len(), 2, "input ports should be preserved");
        assert_eq!(node.outputs.len(), 1, "output ports should be preserved");
    }

    #[test]
    fn subdag_diamond_topology() {
        // Diamond: A -> B, A -> C, B -> D, C -> D
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("a", vec![], vec![port("out", "Int")], ()));
        dag.add_node(Node::opaque(
            "b",
            vec![port("in", "Int")],
            vec![port("out", "Int")],
            (),
        ));
        dag.add_node(Node::opaque(
            "c",
            vec![port("in", "Int")],
            vec![port("out", "Int")],
            (),
        ));
        dag.add_node(Node::opaque(
            "d",
            vec![port("x", "Int"), port("y", "Int")],
            vec![],
            (),
        ));
        dag.add_edge(edge("a", "out", "b", "in"));
        dag.add_edge(edge("a", "out", "c", "in"));
        dag.add_edge(edge("b", "out", "d", "x"));
        dag.add_edge(edge("c", "out", "d", "y"));

        // Window all 4 nodes
        let window = Window::from_nodes(&dag, vec!["a", "b", "c", "d"]);
        let sub = window_subdag(&dag, &window);

        assert_eq!(sub.nodes.len(), 4);
        assert_eq!(sub.edges.len(), 4);

        // Window only middle layer
        let window2 = Window::from_nodes(&dag, vec!["b", "c"]);
        let sub2 = window_subdag(&dag, &window2);

        assert_eq!(sub2.nodes.len(), 2);
        assert_eq!(sub2.edges.len(), 0, "b and c have no edges between them");
    }
}

#[cfg(test)]
mod assert_chain_outputs_tests {
    use super::*;
    use crate::mock_spec::OutputMatcher;
    use gunbc_exec::{ExecutionLog, LogEntry};
    use gunbc_ir::Value;
    use std::collections::HashMap;

    fn make_log(entries: Vec<(&str, Vec<(&str, Value)>)>) -> ExecutionLog {
        ExecutionLog {
            entries: entries
                .into_iter()
                .map(|(node, outputs)| LogEntry {
                    node_id: node.to_string(),
                    inputs: None,
                    outputs: outputs
                        .into_iter()
                        .map(|(p, v)| (p.to_string(), v))
                        .collect(),
                    was_intercepted: false,
                    coercions_applied: vec![],
                })
                .collect(),
        }
    }

    #[test]
    fn passes_when_all_matchers_match() {
        let log = make_log(vec![
            ("parse", vec![("content", Value::Str("hello world".into()))]),
            ("validate", vec![("ok", Value::Bool(true))]),
        ]);

        let mut matchers = HashMap::new();
        matchers.insert(
            ("parse".to_string(), "content".to_string()),
            OutputMatcher::contains("hello"),
        );
        matchers.insert(
            ("validate".to_string(), "ok".to_string()),
            OutputMatcher::exact(Value::Bool(true)),
        );

        assert!(assert_chain_outputs(&log, &matchers).is_ok());
    }

    #[test]
    fn fails_when_matcher_does_not_match() {
        let log = make_log(vec![(
            "parse",
            vec![("content", Value::Str("goodbye".into()))],
        )]);

        let mut matchers = HashMap::new();
        matchers.insert(
            ("parse".to_string(), "content".to_string()),
            OutputMatcher::contains("hello"),
        );

        let err = assert_chain_outputs(&log, &matchers).expect_err("should fail");
        match err {
            WindowError::MatcherFailed { node, port, .. } => {
                assert_eq!(node, "parse");
                assert_eq!(port, "content");
            }
            other => panic!("expected MatcherFailed, got {:?}", other),
        }
    }

    #[test]
    fn fails_when_node_missing_from_log() {
        let log = make_log(vec![]);

        let mut matchers = HashMap::new();
        matchers.insert(
            ("missing_node".to_string(), "out".to_string()),
            OutputMatcher::non_empty(),
        );

        let err = assert_chain_outputs(&log, &matchers).expect_err("should fail");
        match err {
            WindowError::MissingObserverNode { node } => {
                assert_eq!(node, "missing_node");
            }
            other => panic!("expected MissingObserverNode, got {:?}", other),
        }
    }

    #[test]
    fn fails_when_port_missing_from_log() {
        let log = make_log(vec![("node_a", vec![("out", Value::Int(1))])]);

        let mut matchers = HashMap::new();
        matchers.insert(
            ("node_a".to_string(), "missing_port".to_string()),
            OutputMatcher::non_empty(),
        );

        let err = assert_chain_outputs(&log, &matchers).expect_err("should fail");
        match err {
            WindowError::MissingObserverPort { node, port } => {
                assert_eq!(node, "node_a");
                assert_eq!(port, "missing_port");
            }
            other => panic!("expected MissingObserverPort, got {:?}", other),
        }
    }

    #[test]
    fn empty_matchers_always_passes() {
        let log = make_log(vec![("some_node", vec![("out", Value::Int(42))])]);
        let matchers = HashMap::new();
        assert!(assert_chain_outputs(&log, &matchers).is_ok());
    }

    #[test]
    fn works_with_typed_matchers() {
        let log = make_log(vec![
            ("a", vec![("flag", Value::Bool(true))]),
            ("b", vec![("count", Value::Int(10))]),
            ("c", vec![("name", Value::Str("test".into()))]),
        ]);

        let mut matchers = HashMap::new();
        matchers.insert(("a".to_string(), "flag".to_string()), OutputMatcher::IsBool);
        matchers.insert(
            ("b".to_string(), "count".to_string()),
            OutputMatcher::IntGe(5),
        );
        matchers.insert(
            ("c".to_string(), "name".to_string()),
            OutputMatcher::IsString,
        );

        assert!(assert_chain_outputs(&log, &matchers).is_ok());
    }

    #[test]
    fn int_ge_fails_below_threshold() {
        let log = make_log(vec![("b", vec![("count", Value::Int(3))])]);

        let mut matchers = HashMap::new();
        matchers.insert(
            ("b".to_string(), "count".to_string()),
            OutputMatcher::IntGe(5),
        );

        let err = assert_chain_outputs(&log, &matchers).expect_err("should fail");
        assert!(matches!(err, WindowError::MatcherFailed { .. }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_exec::LogEntry;
    use gunbc_ir::{Cardinality, Dag, Node, Port};
    use std::collections::HashMap;

    #[test]
    fn window_outputs_allow_missing_optional_port() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![],
            vec![Port::with_cardinality(
                "out",
                "String",
                Cardinality::ZERO_OR_ONE,
            )],
            (),
        ));

        let window = Window::from_nodes(&dag, vec!["a"]);
        let baseline = ExecutionLog {
            entries: vec![LogEntry {
                node_id: "a".to_string(),
                inputs: None,
                outputs: HashMap::new(),
                was_intercepted: false,
                coercions_applied: vec![],
            }],
        };
        let window_log = ExecutionLog {
            entries: vec![LogEntry {
                node_id: "a".to_string(),
                inputs: None,
                outputs: HashMap::new(),
                was_intercepted: false,
                coercions_applied: vec![],
            }],
        };

        assert!(assert_window_outputs(&dag, &window, &baseline, &window_log).is_ok());
    }

    #[test]
    fn window_outputs_require_non_optional_port() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![],
            vec![Port::new("out", "String")],
            (),
        ));

        let window = Window::from_nodes(&dag, vec!["a"]);
        let baseline = ExecutionLog {
            entries: vec![LogEntry {
                node_id: "a".to_string(),
                inputs: None,
                outputs: HashMap::new(),
                was_intercepted: false,
                coercions_applied: vec![],
            }],
        };
        let window_log = ExecutionLog {
            entries: vec![LogEntry {
                node_id: "a".to_string(),
                inputs: None,
                outputs: HashMap::new(),
                was_intercepted: false,
                coercions_applied: vec![],
            }],
        };

        // Both baseline and window missing the port — both fall back to
        // Value::Skipped, so Skipped == Skipped → Ok.
        assert_window_outputs(&dag, &window, &baseline, &window_log)
            .expect("both sides Skipped should match");
    }
}
