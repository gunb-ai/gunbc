//! Windowed segment testing helpers.
//!
//! A window is a contiguous subgraph slice that can be executed in isolation.
//! Entry inputs are injected from a baseline DryRun, and exit outputs are
//! compared against that baseline to verify inter-node integration.

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
    let mixed = mixed_input_ports(dag, &node_set);
    if let Some((node, port)) = mixed.first() {
        return Err(WindowError::MixedInput {
            node: node.0.clone(),
            port: port.0.clone(),
        });
    }

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

    let mut fan_in: HashMap<(String, String), Vec<Value>> = HashMap::new();
    let mut scalars: HashMap<(String, String), Value> = HashMap::new();

    for edge in canonical_edge_order(&dag.edges) {
        if node_set.contains(&edge.to_node) && !node_set.contains(&edge.from_node) {
            let entry =
                baseline
                    .get(&edge.from_node.0)
                    .ok_or_else(|| WindowError::MissingLogNode {
                        context: "baseline",
                        node: edge.from_node.0.clone(),
                    })?;
            let output_port = dag
                .get_node(&edge.from_node)
                .and_then(|n| n.outputs.iter().find(|p| p.name == edge.from_port));
            let from_cardinality = output_port
                .map(|p| p.cardinality)
                .unwrap_or(gunbc_ir::Cardinality::ONE);

            let value = match entry.outputs.get(&edge.from_port.0) {
                Some(v) => Some(v.clone()),
                None => {
                    if let Some(port) = output_port {
                        if port.cardinality.allows_empty() {
                            None
                        } else {
                            return Err(WindowError::MissingLogPort {
                                context: "baseline",
                                node: edge.from_node.0.clone(),
                                port: edge.from_port.0.clone(),
                            });
                        }
                    } else {
                        return Err(WindowError::MissingLogPort {
                            context: "baseline",
                            node: edge.from_node.0.clone(),
                            port: edge.from_port.0.clone(),
                        });
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

        let baseline_entry =
            baseline
                .get(&node_id.0)
                .ok_or_else(|| WindowError::MissingLogNode {
                    context: "baseline",
                    node: node_id.0.clone(),
                })?;
        let window_entry =
            window_log
                .get(&node_id.0)
                .ok_or_else(|| WindowError::MissingLogNode {
                    context: "window",
                    node: node_id.0.clone(),
                })?;

        for port in &node.outputs {
            if internal_outputs.contains(&(node_id.clone(), port.name.clone())) {
                continue;
            }

            let expected = match baseline_entry.outputs.get(&port.name.0) {
                Some(v) => v.clone(),
                None => {
                    if port.cardinality.allows_empty() {
                        empty_value_for_cardinality(port.cardinality)
                    } else {
                        return Err(WindowError::MissingLogPort {
                            context: "baseline",
                            node: node_id.0.clone(),
                            port: port.name.0.clone(),
                        });
                    }
                }
            };
            let actual = match window_entry.outputs.get(&port.name.0) {
                Some(v) => v.clone(),
                None => {
                    if port.cardinality.allows_empty() {
                        empty_value_for_cardinality(port.cardinality)
                    } else {
                        return Err(WindowError::MissingLogPort {
                            context: "window",
                            node: node_id.0.clone(),
                            port: port.name.0.clone(),
                        });
                    }
                }
            };

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
            outputs: map,
            was_intercepted: false,
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

        let err = apply_window_inputs(&dag, &window, &baseline, &mut mocks)
            .expect_err("mixed input ports should error");
        match err {
            WindowError::MixedInput { node, port } => {
                assert_eq!(node, "b");
                assert_eq!(port, "in");
            }
            other => panic!("expected MixedInput error, got {:?}", other),
        }
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
        dag.add_node(Node::opaque("a", vec![], vec![list("items", "IntList")], ()));
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
                outputs: HashMap::new(),
                was_intercepted: false,
            }],
        };
        let window_log = ExecutionLog {
            entries: vec![LogEntry {
                node_id: "a".to_string(),
                outputs: HashMap::new(),
                was_intercepted: false,
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
                outputs: HashMap::new(),
                was_intercepted: false,
            }],
        };
        let window_log = ExecutionLog {
            entries: vec![LogEntry {
                node_id: "a".to_string(),
                outputs: HashMap::new(),
                was_intercepted: false,
            }],
        };

        let err = assert_window_outputs(&dag, &window, &baseline, &window_log)
            .expect_err("missing required output should error");
        match err {
            WindowError::MissingLogPort {
                context,
                node,
                port,
            } => {
                assert_eq!(context, "baseline");
                assert_eq!(node, "a");
                assert_eq!(port, "out");
            }
            other => panic!("unexpected error: {}", other),
        }
    }
}
