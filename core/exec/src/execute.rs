//! DAG execution with boundary interception and simulation.
//!
//! # DryRun Interception
//!
//! DryRun mode intercepts **transport execution nodes** - nodes that consume
//! `TransportRequest` values. This is based on the design principle:
//!
//! > "World I/O is performed only by transport executor nodes"
//! > "DryRun intercepts transport execution nodes, not boundary outputs"
//!
//! A node is considered a transport executor if:
//! - It has an input port with type `TransportRequest`
//!
//! Boundary detection (`BoundaryInfo`) is still used for signature inference
//! and workflow interface detection, but NOT for DryRun interception.

use crate::error::ExecError;
use crate::intercept::BoundaryMocks;
use crate::lower::lower;
use crate::topo::topo_sort;
use crate::Executable;
use gunbc_ir::{detect_boundaries, BoundaryInfo, Dag, Node, NodeBody, NodeId, Value};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

/// Execution mode: real, dry-run, or simulate.
#[derive(Debug, Clone)]
pub enum ExecutionMode {
    /// Execute all operations normally
    Real,
    /// Intercept boundary operations with mocks
    DryRun(BoundaryMocks),
    /// Simulate execution with timing and resource tracking
    Simulate(SimConfig),
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::Real
    }
}

/// Configuration for simulation mode.
#[derive(Debug, Clone, Default)]
pub struct SimConfig {
    /// Simulated execution time for each node.
    /// If a node is not in the map, it defaults to zero.
    pub timing: HashMap<NodeId, Duration>,
    /// Resource budget (memory, CPU limits).
    pub resources: ResourceBudget,
    /// Random seed for deterministic simulation.
    /// If None, uses system randomness.
    pub random_seed: Option<u64>,
    /// Mock values for boundary nodes (like DryRun).
    pub boundary_mocks: BoundaryMocks,
}

impl SimConfig {
    /// Create a new empty simulation config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the timing for a specific node.
    pub fn with_timing(mut self, node_id: impl Into<NodeId>, duration: Duration) -> Self {
        self.timing.insert(node_id.into(), duration);
        self
    }

    /// Set the resource budget.
    pub fn with_resources(mut self, resources: ResourceBudget) -> Self {
        self.resources = resources;
        self
    }

    /// Set the random seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.random_seed = Some(seed);
        self
    }

    /// Set boundary mocks.
    pub fn with_mocks(mut self, mocks: BoundaryMocks) -> Self {
        self.boundary_mocks = mocks;
        self
    }

    /// Get the simulated duration for a node.
    pub fn node_duration(&self, node_id: &NodeId) -> Duration {
        self.timing.get(node_id).copied().unwrap_or(Duration::ZERO)
    }
}

/// Resource budget for simulation.
#[derive(Debug, Clone, Default)]
pub struct ResourceBudget {
    /// Maximum memory usage in bytes (None = unlimited).
    pub max_memory: Option<u64>,
    /// Maximum CPU time in milliseconds (None = unlimited).
    pub max_cpu_ms: Option<u64>,
    /// Maximum number of concurrent operations (None = unlimited).
    pub max_concurrency: Option<usize>,
}

impl ResourceBudget {
    /// Create a new unlimited resource budget.
    pub fn unlimited() -> Self {
        Self::default()
    }

    /// Set maximum memory.
    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.max_memory = Some(bytes);
        self
    }

    /// Set maximum CPU time.
    pub fn with_cpu(mut self, ms: u64) -> Self {
        self.max_cpu_ms = Some(ms);
        self
    }

    /// Set maximum concurrency.
    pub fn with_concurrency(mut self, n: usize) -> Self {
        self.max_concurrency = Some(n);
        self
    }
}

/// Result of a simulated execution.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// Total simulated execution time.
    pub total_time: Duration,
    /// The critical path (longest dependency chain).
    pub critical_path: Vec<NodeId>,
    /// Resource usage during simulation.
    pub resource_usage: ResourceUsage,
    /// Timeline of node executions (node_id, start_time, duration).
    pub timeline: Vec<(NodeId, Duration, Duration)>,
    /// The execution log (same as real execution).
    pub log: ExecutionLog,
}

/// Resource usage during simulation.
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// Peak memory usage in bytes.
    pub peak_memory: u64,
    /// Total CPU time in milliseconds.
    pub total_cpu_ms: u64,
    /// Maximum concurrent operations observed.
    pub max_concurrency: usize,
}

/// A single entry in the execution log.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
/// In simulate mode, timing and resource usage are tracked.
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

/// Simulate execution of a DAG with timing and resource tracking.
///
/// This is a convenience wrapper that returns a `SimulationResult` instead
/// of just an `ExecutionLog`.
pub fn simulate<T: Executable + Clone>(
    dag: &Dag<T>,
    config: SimConfig,
) -> Result<SimulationResult, ExecError> {
    // Lower sub-DAGs first
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {e}")))?;

    // Detect boundaries
    let boundaries = detect_boundaries(&flat);

    // Get topological order
    let order = topo_sort(&flat);

    // Execute with simulation tracking
    let log = execute_flat(&flat, &boundaries, &ExecutionMode::Simulate(config.clone()))?;

    // Compute simulation metrics
    let timeline = compute_timeline(&order, &config);
    let total_time = timeline.iter().map(|(_, start, dur)| *start + *dur).max().unwrap_or(Duration::ZERO);
    let critical_path = compute_critical_path(&flat, &config);
    let resource_usage = ResourceUsage::default(); // Simplified for now

    Ok(SimulationResult {
        total_time,
        critical_path,
        resource_usage,
        timeline,
        log,
    })
}

/// Compute the execution timeline (node_id, start_time, duration).
fn compute_timeline(order: &[NodeId], config: &SimConfig) -> Vec<(NodeId, Duration, Duration)> {
    let mut timeline = Vec::new();
    let mut current_time = Duration::ZERO;

    for node_id in order {
        let duration = config.node_duration(node_id);
        timeline.push((node_id.clone(), current_time, duration));
        current_time += duration;
    }

    timeline
}

/// Compute the critical path (longest dependency chain).
fn compute_critical_path<T>(dag: &Dag<T>, _config: &SimConfig) -> Vec<NodeId> {
    // Simple implementation: just return all nodes in topological order
    // A more sophisticated implementation would track actual dependencies
    let order = topo_sort(dag);
    
    // Find the path with the longest total duration
    // For now, just return the topological order
    order
}

/// Execute a flat (fully lowered) DAG.
fn execute_flat<T: Executable>(
    dag: &Dag<T>,
    _boundaries: &BoundaryInfo,  // Kept for future signature inference use
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
            // Check if this is a transport execution node (consumes TransportRequest)
            // Transport execution nodes are intercepted in dry-run/simulate mode
            // This follows the design principle: intercept where I/O happens, not boundaries
            let is_transport_executor = is_transport_execution_node(node);
            let should_intercept = is_transport_executor && matches!(mode, ExecutionMode::DryRun(_) | ExecutionMode::Simulate(_));

            if should_intercept {
                // Intercept: use mock values for boundary outputs
                let mocks = match mode {
                    ExecutionMode::DryRun(ref m) => m,
                    ExecutionMode::Simulate(ref config) => &config.boundary_mocks,
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

/// Check if a node is a transport execution node.
///
/// A transport execution node is one that consumes `TransportRequest` values.
/// These are the only nodes where actual I/O happens, and therefore the only
/// nodes that should be intercepted in DryRun mode.
///
/// This is a structural check based on port types, aligning with the design
/// principle: "impossibility by structure" - if a node doesn't consume a
/// TransportRequest, it can't perform transport I/O.
fn is_transport_execution_node<T>(node: &Node<T>) -> bool {
    node.inputs.iter().any(|port| {
        port.type_id.0 == "TransportRequest"
    })
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
    fn test_dry_run_intercepts_transport_executor() {
        // A transport executor node consumes TransportRequest
        let mut dag: Dag<Produce> = Dag::new();
        dag.add_node(Node::opaque(
            "execute_transport",
            // This input marks it as a transport executor - will be intercepted
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            Produce::new("response", Value::Str("real-response".to_string())),
        ));

        // In dry-run mode, transport executor nodes should be intercepted
        let mut mocks = BoundaryMocks::new();
        mocks.set_value("execute_transport", "response", Value::Str("mock-response".to_string()));

        let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

        assert_eq!(log.entries.len(), 1);
        assert!(log.entries[0].was_intercepted);
        match &log.entries[0].outputs.get("response") {
            Some(Value::Str(s)) => assert_eq!(s, "mock-response"),
            _ => panic!("expected mock response"),
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
    fn test_pure_node_not_intercepted() {
        // Pure nodes (no TransportRequest input) should never be intercepted
        // Only transport executor nodes should be intercepted
        let mut dag: Dag<Produce> = Dag::new();
        
        // Pure node - prepares a request but doesn't execute it
        dag.add_node(Node::opaque(
            "prepare",
            vec![port("content", "String")],
            vec![port("request", "TransportRequest")],
            Produce::new("request", Value::Str("prepared-request".to_string())),
        ));
        
        // Transport executor - consumes the request (will be intercepted)
        dag.add_node(Node::opaque(
            "execute",
            vec![port("request", "TransportRequest")],  // This makes it a transport executor
            vec![port("response", "TransportResponse")],
            Produce::new("response", Value::Str("real-response".to_string())),
        ));
        dag.add_edge(edge("prepare", "request", "execute", "request"));

        let mocks = BoundaryMocks::with_default(Value::Str("mocked".to_string()));
        let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

        // prepare is NOT a transport executor — should execute normally
        let prepare_entry = log.get("prepare").unwrap();
        assert!(!prepare_entry.was_intercepted);

        // execute IS a transport executor — should be intercepted
        let execute_entry = log.get("execute").unwrap();
        assert!(execute_entry.was_intercepted);
    }

    #[test]
    fn test_simulate_basic() {
        let mut dag: Dag<Produce> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![port("out", "String")],
            Produce::new("out", Value::Str("hello".to_string())),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![port("in", "String")],
            vec![port("out", "String")],
            Produce::new("out", Value::Str("world".to_string())),
        ));
        dag.add_edge(edge("A", "out", "B", "in"));

        // Configure simulation with timing
        let config = SimConfig::new()
            .with_timing("A", Duration::from_millis(100))
            .with_timing("B", Duration::from_millis(200))
            .with_mocks(BoundaryMocks::with_default(Value::Str("mocked".to_string())));

        let result = simulate(&dag, config).unwrap();

        // Check that simulation ran
        assert!(!result.log.entries.is_empty());
        
        // Check timeline
        assert_eq!(result.timeline.len(), 2);
        
        // Check total time is sum of node times (sequential execution)
        assert_eq!(result.total_time, Duration::from_millis(300));
    }

    #[test]
    fn test_simulate_with_mocks() {
        // Transport executor node (consumes TransportRequest) should be intercepted in simulation
        let mut dag: Dag<Produce> = Dag::new();
        dag.add_node(Node::opaque(
            "transport_node",
            vec![port("request", "TransportRequest")],  // Makes it a transport executor
            vec![port("result", "String")],
            Produce::new("result", Value::Str("real-value".to_string())),
        ));

        let mut mocks = BoundaryMocks::new();
        mocks.set_value("transport_node", "result", Value::Str("simulated-value".to_string()));

        let config = SimConfig::new().with_mocks(mocks);

        let result = simulate(&dag, config).unwrap();

        // Transport executor should be intercepted with mock value
        let entry = result.log.get("transport_node").unwrap();
        assert!(entry.was_intercepted);
        assert_eq!(
            entry.outputs.get("result"),
            Some(&Value::Str("simulated-value".to_string()))
        );
    }

    #[test]
    fn test_sim_config_builder() {
        let config = SimConfig::new()
            .with_timing("node1", Duration::from_secs(1))
            .with_timing("node2", Duration::from_secs(2))
            .with_seed(42)
            .with_resources(
                ResourceBudget::unlimited()
                    .with_memory(1024 * 1024)
                    .with_cpu(5000)
                    .with_concurrency(4)
            );

        assert_eq!(config.node_duration(&NodeId::from("node1")), Duration::from_secs(1));
        assert_eq!(config.node_duration(&NodeId::from("node2")), Duration::from_secs(2));
        assert_eq!(config.node_duration(&NodeId::from("unknown")), Duration::ZERO);
        assert_eq!(config.random_seed, Some(42));
        assert_eq!(config.resources.max_memory, Some(1024 * 1024));
        assert_eq!(config.resources.max_cpu_ms, Some(5000));
        assert_eq!(config.resources.max_concurrency, Some(4));
    }
}
