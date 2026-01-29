//! DAG execution with boundary interception.

use crate::error::ExecError;
use crate::intercept::BoundaryMocks;
use crate::lower::lower;
use crate::topo::topo_sort;
use crate::Executable;
use gunbc_ir::{detect_boundaries, BoundaryInfo, Dag, Node, NodeBody, Value};
use std::collections::HashMap;
use std::fmt;

/// Execution mode: real or dry-run.
#[derive(Debug, Clone)]
pub enum ExecutionMode {
    /// Execute all operations normally
    Real,
    /// Intercept boundary operations with mocks
    DryRun(BoundaryMocks),
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::Real
    }
}

/// A single entry in the execution log.
#[derive(Debug)]
pub struct LogEntry {
    pub node_id: String,
    pub outputs: HashMap<String, Value>,
    pub was_intercepted: bool,
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = if self.was_intercepted { " [DRY-RUN]" } else { "" };
        write!(f, "[{}]{}", self.node_id, marker)?;
        for (k, v) in &self.outputs {
            write!(f, " {k}={v}")?;
        }
        Ok(())
    }
}

/// Full execution log.
#[derive(Debug)]
pub struct ExecutionLog {
    pub entries: Vec<LogEntry>,
}

impl ExecutionLog {
    /// Get the entry for a specific node.
    pub fn get(&self, node_id: &str) -> Option<&LogEntry> {
        self.entries.iter().find(|e| e.node_id == node_id)
    }

    /// Check if any node was intercepted (dry-run).
    pub fn has_intercepted(&self) -> bool {
        self.entries.iter().any(|e| e.was_intercepted)
    }
}

impl fmt::Display for ExecutionLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for entry in &self.entries {
            writeln!(f, "{entry}")?;
        }
        Ok(())
    }
}

/// Execute a DAG in real mode.
pub fn execute<T: Executable + Clone>(dag: &Dag<T>) -> Result<ExecutionLog, ExecError> {
    execute_with_mode(dag, ExecutionMode::Real)
}

/// Execute a DAG with the specified execution mode.
///
/// In dry-run mode, boundary nodes have their outputs replaced with mock values.
pub fn execute_with_mode<T: Executable + Clone>(
    dag: &Dag<T>,
    mode: ExecutionMode,
) -> Result<ExecutionLog, ExecError> {
    // Lower sub-DAGs first
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {e}")))?;

    // Detect boundaries
    let boundaries = detect_boundaries(&flat);

    // Execute the flat DAG
    execute_flat(&flat, &boundaries, &mode)
}

/// Execute a flat (fully lowered) DAG.
fn execute_flat<T: Executable>(
    dag: &Dag<T>,
    boundaries: &BoundaryInfo,
    mode: &ExecutionMode,
) -> Result<ExecutionLog, ExecError> {
    let order = topo_sort(dag);
    let node_map: HashMap<&str, &Node<T>> = dag.nodes.iter().map(|n| (n.id.0.as_str(), n)).collect();

    let mut node_outputs: HashMap<String, HashMap<String, Value>> = HashMap::new();
    let mut entries = Vec::new();

    for node_id in &order {
        let node = node_map
            .get(node_id.0.as_str())
            .ok_or_else(|| ExecError::new(format!("node '{}' not found", node_id.0)))?;

        // Gather inputs from upstream edges
        let mut inputs: HashMap<String, Value> = HashMap::new();
        for edge in &dag.edges {
            if edge.to_node == *node_id {
                if let Some(upstream) = node_outputs.get(&edge.from_node.0) {
                    if let Some(val) = upstream.get(&edge.from_port.0) {
                        inputs.insert(edge.to_port.0.clone(), val.clone());
                    }
                }
            }
        }

        // Check guards
        let skip = should_skip_node(node, &inputs);

        let (outputs, was_intercepted) = if skip {
            // Node is skipped — all outputs become Skipped
            let outputs: HashMap<String, Value> = node
                .outputs
                .iter()
                .map(|p| (p.name.0.clone(), Value::Skipped))
                .collect();
            (outputs, false)
        } else {
            // Check if this is a boundary node in dry-run mode
            let is_boundary = boundaries.is_boundary_node(node_id);
            let should_intercept = is_boundary && matches!(mode, ExecutionMode::DryRun(_));

            if should_intercept {
                // Intercept: use mock values for boundary outputs
                let mocks = match mode {
                    ExecutionMode::DryRun(ref m) => m,
                    _ => unreachable!(),
                };

                let outputs: HashMap<String, Value> = node
                    .outputs
                    .iter()
                    .map(|p| {
                        let mock = mocks.get_mock(node_id, &p.name);
                        (p.name.0.clone(), mock.value.clone())
                    })
                    .collect();
                (outputs, true)
            } else {
                // Execute normally
                match &node.body {
                    NodeBody::Opaque(op) => {
                        let outputs = op.execute(inputs)?;
                        (outputs, false)
                    }
                    NodeBody::SubDag(_) => {
                        return Err(ExecError::new(format!(
                            "node '{}' is a SubDag — DAG must be lowered before execution",
                            node_id.0
                        )));
                    }
                }
            }
        };

        node_outputs.insert(node_id.0.clone(), outputs.clone());
        entries.push(LogEntry {
            node_id: node_id.0.clone(),
            outputs,
            was_intercepted,
        });
    }

    Ok(ExecutionLog { entries })
}

/// Check whether a node should be skipped based on guard predicates.
fn should_skip_node<T>(node: &Node<T>, inputs: &HashMap<String, Value>) -> bool {
    for port in &node.inputs {
        if port.has_guard() {
            if let Some(value) = inputs.get(&port.name.0) {
                if !port.check_guard(value) {
                    return true;
                }
            } else {
                // Missing input value — skip the node
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::build::*;

    // Operation that produces a specific value on a named port
    #[derive(Debug, Clone)]
    struct Produce {
        port: String,
        value: Value,
    }

    impl Produce {
        fn new(port: &str, value: Value) -> Self {
            Self { port: port.to_string(), value }
        }
    }

    impl Executable for Produce {
        fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
            let mut out = HashMap::new();
            out.insert(self.port.clone(), self.value.clone());
            Ok(out)
        }
    }

    #[test]
    fn test_execute_simple_pipeline() {
        let mut dag: Dag<Produce> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![port("out", "String")],
            Produce::new("out", Value::Str("hello".to_string())),
        ));

        let log = execute(&dag).unwrap();
        
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].node_id, "A");
        match &log.entries[0].outputs.get("out") {
            Some(Value::Str(s)) => assert_eq!(s, "hello"),
            _ => panic!("expected string output"),
        }
    }

    #[test]
    fn test_dry_run_intercepts_boundary() {
        let mut dag: Dag<Produce> = Dag::new();
        dag.add_node(Node::opaque(
            "create_gist",
            vec![],
            vec![port("url", "String")],
            Produce::new("url", Value::Str("real-url".to_string())),
        ));

        // In dry-run mode, the boundary should be intercepted
        let mut mocks = BoundaryMocks::new();
        mocks.set_value("create_gist", "url", Value::Str("mock-url".to_string()));

        let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

        assert_eq!(log.entries.len(), 1);
        assert!(log.entries[0].was_intercepted);
        match &log.entries[0].outputs.get("url") {
            Some(Value::Str(s)) => assert_eq!(s, "mock-url"),
            _ => panic!("expected mock url"),
        }
    }

    #[test]
    fn test_real_mode_executes_boundary() {
        let mut dag: Dag<Produce> = Dag::new();
        dag.add_node(Node::opaque(
            "create_gist",
            vec![],
            vec![port("url", "String")],
            Produce::new("url", Value::Str("real-url".to_string())),
        ));

        let log = execute(&dag).unwrap();

        assert_eq!(log.entries.len(), 1);
        assert!(!log.entries[0].was_intercepted);
        match &log.entries[0].outputs.get("url") {
            Some(Value::Str(s)) => assert_eq!(s, "real-url"),
            _ => panic!("expected real url"),
        }
    }

    #[test]
    fn test_non_boundary_not_intercepted() {
        // A -> B pipeline: A is not a boundary (connected to B), B is a boundary
        let mut dag: Dag<Produce> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![port("out", "S")],
            Produce::new("out", Value::Str("from-A".to_string())),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![port("in", "S")],
            vec![port("out", "S")],
            Produce::new("out", Value::Str("from-B".to_string())),
        ));
        dag.add_edge(edge("A", "out", "B", "in"));

        let mocks = BoundaryMocks::with_default(Value::Str("mocked".to_string()));
        let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

        // A is not a boundary — should execute normally
        let a_entry = log.get("A").unwrap();
        assert!(!a_entry.was_intercepted);

        // B is a boundary — should be intercepted
        let b_entry = log.get("B").unwrap();
        assert!(b_entry.was_intercepted);
    }
}
