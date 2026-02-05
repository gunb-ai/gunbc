//! DAG execution with boundary interception and simulation.
//!
//! # DryRun Interception
//!
//! DryRun mode intercepts **transport execution nodes** (nodes that consume
//! `TransportRequest` values), **environment nodes** (nodes that emit
//! resource outputs like `ToolHandle`, `FilesystemHandle`, `Timestamp`,
//! `AuthToken`, or `Platform`), and **tool consumer nodes** (nodes that
//! consume `ToolHandle`). Intercepted nodes require **explicit mocks for every
//! output port** — there is no default fallback.
//!
//! > "World I/O is performed only by transport executor nodes"
//! > "DryRun intercepts transport execution nodes, not boundary outputs"
//!
//! A node is considered a transport executor if:
//! - It has an input port with type `TransportRequest`
//!
//! A node is considered a tool environment node if:
//! - It has an output port with type `ToolHandle`
//!
//! A node is considered a resource environment node if:
//! - It has an output port with type `FilesystemHandle`, `Timestamp`, `AuthToken`, or `Platform`
//!
//! A node is considered a tool consumer node if:
//! - It has an input port with type `ToolHandle`
//!
//! Boundary detection (`BoundaryInfo`) is still used for signature inference
//! and workflow interface detection, but NOT for DryRun interception.

use crate::error::ExecError;
use crate::intercept::BoundaryMocks;
use crate::lower::lower;
use crate::progress::{DagSnapshot, OutputSummary, ProgressObserver};
use crate::topo::topo_sort;
use crate::Executable;
use gunbc_ir::{
    canonical_edge_order, detect_boundaries, BoundaryInfo, Cardinality, Dag, Node, NodeBody,
    NodeId, Value,
};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

/// Execution mode: real, dry-run, or simulate.
#[derive(Debug, Clone, Default)]
pub enum ExecutionMode {
    /// Execute all operations normally
    #[default]
    Real,
    /// Intercept boundary operations with mocks
    DryRun(BoundaryMocks),
    /// Simulate execution with timing and resource tracking
    Simulate(SimConfig),
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
        let marker = if self.was_intercepted {
            " [DRY-RUN]"
        } else {
            ""
        };
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
    execute_with_mode_and_inputs(dag, mode, None)
}

/// Execute a DAG with the specified execution mode and optional input mocks.
///
/// Input mocks are injected into entrypoint ports (inputs with no upstream edge).
pub fn execute_with_mode_and_inputs<T: Executable + Clone>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, ExecError> {
    // Lower sub-DAGs first
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {e}")))?;

    // Detect boundaries
    let boundaries = detect_boundaries(&flat);

    // Execute the flat DAG
    execute_flat(&flat, &boundaries, &mode, None, None, input_mocks)
}

/// Execute a DAG with CI context for workflow command emission.
///
/// When a CI context is provided, each node execution is wrapped in
/// collapsible groups, and errors/warnings are emitted as annotations.
/// The CI context auto-detects the provider (GitHub Actions, GitLab CI, etc.).
///
/// # Example
///
/// ```ignore
/// use gunbc_exec::{execute_with_ci, CiContext};
///
/// let mut ci = CiContext::detect();
/// let log = execute_with_ci(&dag, &mut ci)?;
/// ```
pub fn execute_with_ci<T: Executable + Clone>(
    dag: &Dag<T>,
    ci: &mut crate::CiContext,
) -> Result<ExecutionLog, ExecError> {
    execute_with_mode_and_ci(dag, ExecutionMode::Real, ci)
}

/// Execute a DAG with both execution mode and CI context.
pub fn execute_with_mode_and_ci<T: Executable + Clone>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    ci: &mut crate::CiContext,
) -> Result<ExecutionLog, ExecError> {
    // Lower sub-DAGs first
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {e}")))?;

    // Detect boundaries
    let boundaries = detect_boundaries(&flat);

    // Execute the flat DAG with CI context
    execute_flat(&flat, &boundaries, &mode, Some(ci), None, None)
}

/// Execute a DAG with a progress observer.
///
/// The progress observer receives callbacks at each execution stage
/// (node start, complete, fail, skip, intercept). This enables live
/// progress display, recording, or any other observation pattern.
///
/// # Example
///
/// ```ignore
/// use gunbc_exec::{execute_with_progress, progress::DagProgress};
///
/// let mut progress = None; // Will be initialized from snapshot
/// let log = execute_with_progress(&dag, &mut progress)?;
/// ```
pub fn execute_with_progress<T: Executable + Clone>(
    dag: &Dag<T>,
    observer: &mut dyn ProgressObserver,
) -> Result<ExecutionLog, ExecError> {
    execute_with_progress_and_mode(dag, ExecutionMode::Real, observer)
}

/// Execute a DAG with both execution mode and progress observer.
pub fn execute_with_progress_and_mode<T: Executable + Clone>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    observer: &mut dyn ProgressObserver,
) -> Result<ExecutionLog, ExecError> {
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {e}")))?;
    let boundaries = detect_boundaries(&flat);
    execute_flat(&flat, &boundaries, &mode, None, Some(observer), None)
}

/// Execute a DAG with both execution mode and progress observer plus input mocks.
pub fn execute_with_progress_and_mode_and_inputs<T: Executable + Clone>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    observer: &mut dyn ProgressObserver,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, ExecError> {
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {e}")))?;
    let boundaries = detect_boundaries(&flat);
    execute_flat(&flat, &boundaries, &mode, None, Some(observer), input_mocks)
}

/// Execute a DAG with execution mode, CI context, and progress observer.
pub fn execute_with_all<T: Executable + Clone>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    ci: &mut crate::CiContext,
    observer: &mut dyn ProgressObserver,
) -> Result<ExecutionLog, ExecError> {
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {e}")))?;
    let boundaries = detect_boundaries(&flat);
    execute_flat(&flat, &boundaries, &mode, Some(ci), Some(observer), None)
}

/// Execute a single node from a DAG.
///
/// This function allows running an individual DAG node for CI step visibility.
/// Each node can be executed as a separate CI step, with inputs provided from
/// the previous step's outputs (via environment variables or artifacts).
///
/// # Arguments
///
/// * `dag` - The full DAG (needed to find the node definition)
/// * `node_id` - The ID of the node to execute
/// * `inputs` - Pre-computed inputs for this node (typically from previous CI steps)
/// * `mode` - Execution mode (Real or DryRun)
///
/// # Returns
///
/// The outputs of the executed node, or an error if execution fails.
///
/// # Example
///
/// ```ignore
/// use gunbc_exec::{execute_single_node, ExecutionMode};
/// use std::collections::HashMap;
/// use gunbc_ir::Value;
///
/// let dag = build_ci_graph()?;
/// let inputs = HashMap::new();  // Or load from environment
/// let outputs = execute_single_node(&dag, "lint", inputs, ExecutionMode::Real)?;
/// ```
pub fn execute_single_node<T: Executable + Clone>(
    dag: &Dag<T>,
    node_id: &str,
    inputs: HashMap<String, Value>,
    mode: ExecutionMode,
) -> Result<HashMap<String, Value>, ExecError> {
    // Lower sub-DAGs first (in case the target node is inside a sub-DAG)
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {e}")))?;

    // Find the node
    let node = flat
        .nodes
        .iter()
        .find(|n| n.id.0 == node_id)
        .ok_or_else(|| ExecError::new(format!("node '{}' not found in DAG", node_id)))?;

    // Check if this is a transport execution node for interception
    let is_transport_executor = is_transport_execution_node(node);
    let is_tool_env = is_tool_env_node(node);
    let is_resource_env = is_resource_env_node(node);
    let is_tool_consumer = consumes_tool_handle(node);
    let should_intercept =
        (is_transport_executor || is_tool_env || is_resource_env || is_tool_consumer)
            && matches!(mode, ExecutionMode::DryRun(_) | ExecutionMode::Simulate(_));

    if should_intercept {
        // Intercept: use mock values for boundary outputs
        let mocks = match &mode {
            ExecutionMode::DryRun(m) => m,
            ExecutionMode::Simulate(config) => &config.boundary_mocks,
            _ => unreachable!(),
        };

        let outputs: HashMap<String, Value> = mock_intercept_outputs(node, mocks)?;
        return Ok(outputs);
    }

    // Execute the node
    match &node.body {
        NodeBody::Opaque(op) => op.execute(inputs),
        NodeBody::SubDag(_) => Err(ExecError::new(format!(
            "node '{}' is a SubDag — this should not happen after lowering",
            node_id
        ))),
    }
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

    // Execute with simulation tracking (no CI context in simulation)
    let log = execute_flat(
        &flat,
        &boundaries,
        &ExecutionMode::Simulate(config.clone()),
        None,
        None,
        None,
    )?;

    // Compute simulation metrics
    let timeline = compute_timeline(&order, &config);
    let total_time = timeline
        .iter()
        .map(|(_, start, dur)| *start + *dur)
        .max()
        .unwrap_or(Duration::ZERO);
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
    // For now, just return the topological order
    topo_sort(dag)
}

/// Execute a flat (fully lowered) DAG.
fn execute_flat<T: Executable>(
    dag: &Dag<T>,
    boundaries: &BoundaryInfo,
    mode: &ExecutionMode,
    ci: Option<&mut crate::CiContext>,
    observer: Option<&mut dyn ProgressObserver>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, ExecError> {
    let order = topo_sort(dag);
    let node_map: HashMap<&str, &Node<T>> =
        dag.nodes.iter().map(|n| (n.id.0.as_str(), n)).collect();

    let mut node_outputs: HashMap<String, HashMap<String, Value>> = HashMap::new();
    let mut entries = Vec::new();

    // Wrap CI context and observer in cells for mutable access in the loop
    let mut ci_ctx = ci;
    let mut obs = observer;

    // Build snapshot and fire on_dag_start
    let dag_start = Instant::now();
    if let Some(ref mut o) = obs {
        let snapshot = DagSnapshot::from_dag(dag, &order, boundaries);
        o.on_dag_start(&snapshot);
    }

    for node_id in &order {
        let node = node_map
            .get(node_id.0.as_str())
            .ok_or_else(|| ExecError::new(format!("node '{}' not found", node_id.0)))?;

        // Gather inputs from upstream edges (cardinality-aware).
        // Tool handles flow through edges like any other value.
        //
        // List ports (cardinality allows_many) collect fan-in values into
        // Value::List in canonical edge order. Scalar ports take a single
        // value (fan-in is rejected at build time).
        let mut inputs: HashMap<String, Value> = HashMap::new();
        let mut fan_in: HashMap<String, Vec<Value>> = HashMap::new();
        let mut scalar_sources: HashMap<String, String> = HashMap::new();

        // Build a lookup for list-typed input ports and their cardinalities
        let list_ports: HashMap<&str, Cardinality> = node
            .inputs
            .iter()
            .filter(|p| p.cardinality.is_list())
            .map(|p| (p.name.0.as_str(), p.cardinality))
            .collect();

        for edge in canonical_edge_order(&dag.edges) {
            if edge.to_node == *node_id {
                if let Some(upstream) = node_outputs.get(&edge.from_node.0) {
                    if let Some(val) = upstream.get(&edge.from_port.0) {
                        if list_ports.contains_key(edge.to_port.0.as_str()) {
                            let from_cardinality = dag
                                .get_node(&edge.from_node)
                                .and_then(|n| n.outputs.iter().find(|p| p.name == edge.from_port))
                                .map(|p| p.cardinality)
                                .unwrap_or(Cardinality::ONE);

                            // Optional/list outputs represent absence as Unit — skip those.
                            if matches!(val, Value::Unit) && from_cardinality.allows_empty() {
                                continue;
                            }

                            let bucket = fan_in.entry(edge.to_port.0.clone()).or_default();
                            if from_cardinality.is_list() {
                                if let Value::List(items) = val {
                                    bucket.extend(items.clone());
                                } else {
                                    bucket.push(val.clone());
                                }
                            } else {
                                bucket.push(val.clone());
                            }
                        } else {
                            if let Some(prev) = scalar_sources.get(&edge.to_port.0) {
                                let current = format!("{}.{}", edge.from_node.0, edge.from_port.0);
                                return Err(ExecError::new(format!(
                                    "scalar input '{}.{}' has multiple upstream edges: {} and {}",
                                    edge.to_node.0, edge.to_port.0, prev, current
                                )));
                            }
                            scalar_sources.insert(
                                edge.to_port.0.clone(),
                                format!("{}.{}", edge.from_node.0, edge.from_port.0),
                            );
                            inputs.insert(edge.to_port.0.clone(), val.clone());
                        }
                    }
                }
            }
        }

        // Wrap collected fan-in values as Value::List
        for (port_name, values) in fan_in {
            inputs.insert(port_name, Value::List(values));
        }

        // Inject input mocks for dangling input ports (DAG entry points).
        // Optional input mocks can be provided directly (real mode) or via
        // DryRun/Simulate boundary mocks.
        let mut inject_inputs = |mocks: &BoundaryMocks| {
            for port in &node.inputs {
                if !inputs.contains_key(&port.name.0) {
                    if let Some(mock_value) = mocks.get_input(&node.id.0, &port.name.0) {
                        inputs.insert(port.name.0.clone(), mock_value.clone());
                    }
                }
            }
        };

        // Prefer explicit input mocks if provided.
        if let Some(mocks) = input_mocks {
            inject_inputs(mocks);
        }

        if let ExecutionMode::DryRun(ref mocks)
        | ExecutionMode::Simulate(SimConfig {
            boundary_mocks: ref mocks,
            ..
        }) = mode
        {
            inject_inputs(mocks);
        }

        // Default list inputs to empty when allowed and still missing.
        // This makes list-typed ports explicit even with zero fan-in.
        for port in &node.inputs {
            if port.cardinality.is_list()
                && port.cardinality.allows_empty()
                && !inputs.contains_key(&port.name.0)
            {
                inputs.insert(port.name.0.clone(), Value::List(vec![]));
            }
        }

        // Check guards BEFORE emitting on_node_start — skipped nodes never
        // enter the "running" state. This prevents misleading transitions in
        // observers and avoids flicker in progress displays.
        let skip = should_skip_node(node, &inputs);

        // CI group and use_group are tracked at the loop iteration level
        // because end_group must be called after outputs are logged.
        let use_group = !skip && node_id.0 != "report";

        let (outputs, was_intercepted) = if skip {
            // Node is skipped — all outputs become Skipped
            let outputs: HashMap<String, Value> = node
                .outputs
                .iter()
                .map(|p| (p.name.0.clone(), Value::Skipped))
                .collect();
            if let Some(ref mut o) = obs {
                o.on_node_skipped(node_id);
            }
            (outputs, false)
        } else {
            // Start CI group for this node (skip for "report" so it's not collapsed)
            if use_group {
                if let Some(ref mut ci) = ci_ctx {
                    ci.start_group(&node_id.0, false);
                }
            }

            // Notify observer that node is starting (only for nodes that will execute)
            let node_start = Instant::now();
            if let Some(ref mut o) = obs {
                o.on_node_start(node_id);
            }

            // Check if this is a transport execution node (consumes TransportRequest),
            // a tool environment node (emits ToolHandle), or a tool consumer node
            // (consumes ToolHandle). These are intercepted in dry-run/simulate mode
            // because they perform I/O or would try to use mock tool paths.
            let is_transport_executor = is_transport_execution_node(node);
            let is_tool_env = is_tool_env_node(node);
            let is_resource_env = is_resource_env_node(node);
            let is_tool_consumer = consumes_tool_handle(node);
            let should_intercept =
                (is_transport_executor || is_tool_env || is_resource_env || is_tool_consumer)
                    && matches!(mode, ExecutionMode::DryRun(_) | ExecutionMode::Simulate(_));

            if should_intercept {
                // Intercept: use mock values for boundary outputs
                let mocks = match mode {
                    ExecutionMode::DryRun(ref m) => m,
                    ExecutionMode::Simulate(ref config) => &config.boundary_mocks,
                    _ => unreachable!(),
                };

                let outputs = mock_intercept_outputs(node, mocks)?;
                if let Some(ref mut o) = obs {
                    let summary = OutputSummary::from_outputs(&outputs, node_start.elapsed());
                    o.on_node_intercepted(node_id, summary);
                }
                (outputs, true)
            } else {
                // Execute normally
                match &node.body {
                    NodeBody::Opaque(op) => {
                        match op.execute(inputs) {
                            Ok(outputs) => {
                                if let Some(ref mut o) = obs {
                                    let summary =
                                        OutputSummary::from_outputs(&outputs, node_start.elapsed());
                                    o.on_node_complete(node_id, summary);
                                }
                                (outputs, false)
                            }
                            Err(e) => {
                                // Notify observer of failure
                                if let Some(ref mut o) = obs {
                                    o.on_node_failed(node_id, &e.to_string());
                                }
                                // Emit CI error annotation if context available
                                if let Some(ref mut ci) = ci_ctx {
                                    ci.error(&format!("Node '{}' failed: {}", node_id.0, e), None);
                                    if use_group {
                                        ci.end_group(); // Close the group before returning error
                                    }
                                }
                                // Notify observer of DAG completion (with failure)
                                if let Some(ref mut o) = obs {
                                    o.on_dag_complete(dag_start.elapsed());
                                }
                                return Err(e);
                            }
                        }
                    }
                    NodeBody::SubDag(_) => {
                        let err_msg = format!(
                            "node '{}' is a SubDag — DAG must be lowered before execution",
                            node_id.0
                        );
                        if let Some(ref mut o) = obs {
                            o.on_node_failed(node_id, &err_msg);
                            o.on_dag_complete(dag_start.elapsed());
                        }
                        if let Some(ref mut ci) = ci_ctx {
                            ci.error(&err_msg, None);
                            if use_group {
                                ci.end_group();
                            }
                        }
                        return Err(ExecError::new(err_msg));
                    }
                }
            }
        };

        // Mask any secret values in CI context so that CI runners
        // (GitHub Actions, GitLab CI) redact them from all output.
        if let Some(ref mut ci) = ci_ctx {
            for value in outputs.values() {
                if let Value::Secret(s) = value {
                    ci.mask(s.expose());
                }
            }
        }

        node_outputs.insert(node_id.0.clone(), outputs.clone());
        let entry = LogEntry {
            node_id: node_id.0.clone(),
            outputs,
            was_intercepted,
        };

        // Print node outputs inside the CI group so they appear in the
        // collapsible section, not after all groups have closed.
        if ci_ctx.is_some() {
            print_log_entry(&entry);
        }
        entries.push(entry);

        // End CI group for this node (skip for "report" since we didn't start one)
        if use_group {
            if let Some(ref mut ci) = ci_ctx {
                ci.end_group();
            }
        }
    }

    // Notify observer of successful DAG completion
    if let Some(ref mut o) = obs {
        o.on_dag_complete(dag_start.elapsed());
    }

    Ok(ExecutionLog { entries })
}

/// Print a log entry's outputs to stdout.
///
/// Used inside CI groups so that node outputs appear within the
/// collapsible section rather than in a flat summary after all groups.
///
/// Secret values are always redacted — they print as `***` regardless
/// of context. This is the last line of defense against credential leaks.
fn print_log_entry(entry: &LogEntry) {
    for (port, value) in &entry.outputs {
        match value {
            Value::Secret(_) => {
                // Always redact secrets — never print actual values
                println!("  {port}: ***");
            }
            Value::Str(s) => {
                if port.ends_with("stderr") || port.ends_with("stdout") {
                    if !s.is_empty() {
                        println!("  {port}: {s}");
                    }
                } else if s.contains('\n') {
                    // Multi-line values (reports, etc.) — print in full
                    println!("  {port}: {s}");
                } else if s.len() < 120 {
                    println!("  {port}: {s}");
                } else {
                    println!("  {port}: {}...", &s[..80]);
                }
            }
            Value::Int(i) => println!("  {port}: {i}"),
            Value::Bool(b) => println!("  {port}: {b}"),
            Value::List(list) => println!("  {port}: [{} items]", list.len()),
            Value::Set(set) => println!("  {port}: {{{} items}}", set.len()),
            Value::Map(map) => println!("  {port}: {{{} entries}}", map.len()),
            Value::Json(_) => println!("  {port}: <JSON>"),
            Value::Skipped => {} // Don't print skipped outputs
            _ => {}
        }
    }
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
    node.inputs
        .iter()
        .any(|port| port.type_id.0 == "TransportRequest")
}

/// Check if a node is a tool environment boundary.
///
/// Tool environment nodes emit `ToolHandle` outputs and are intercepted in DryRun.
fn is_tool_env_node<T>(node: &Node<T>) -> bool {
    node.outputs
        .iter()
        .any(|port| port.type_id.0 == "ToolHandle")
}

/// Check if a node is a non-tool resource environment boundary.
///
/// These emit resource values like FilesystemHandle, Timestamp, AuthToken, Platform.
fn is_resource_env_node<T>(node: &Node<T>) -> bool {
    node.outputs.iter().any(|port| {
        matches!(
            port.type_id.0.as_str(),
            "FilesystemHandle" | "Timestamp" | "AuthToken" | "Platform"
        )
    })
}

/// Check if a node consumes a ToolHandle input.
///
/// Nodes that consume ToolHandles (like CLI tool runners) should be intercepted
/// in DryRun mode because they would otherwise try to execute with a mock path.
fn consumes_tool_handle<T>(node: &Node<T>) -> bool {
    node.inputs
        .iter()
        .any(|port| port.type_id.0 == "ToolHandle")
}

/// Build mock outputs for a tool environment node.
/// Build mock outputs for any intercepted node.
fn mock_intercept_outputs<T>(
    node: &Node<T>,
    mocks: &BoundaryMocks,
) -> Result<HashMap<String, Value>, ExecError> {
    let mut outputs = HashMap::new();

    for port in &node.outputs {
        if !mocks.has_mock(&node.id, &port.name) {
            return Err(ExecError::new(format!(
                "missing mock for intercepted node '{}': output port '{}'",
                node.id.0, port.name.0
            )));
        }

        let mock = mocks.get_mock(&node.id, &port.name).ok_or_else(|| {
            ExecError::new(format!(
                "missing mock for intercepted node '{}': output port '{}'",
                node.id.0, port.name.0
            ))
        })?;
        outputs.insert(port.name.0.clone(), mock.value.clone());
    }

    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::build::*;
    use gunbc_ir::Edge;

    // Test operation: produces a fixed value, or passes through inputs if `pass_through` is set.
    #[derive(Debug, Clone)]
    struct TestOp {
        port: String,
        value: Value,
        pass_through: bool,
    }

    impl TestOp {
        fn produce(port: &str, value: Value) -> Self {
            Self {
                port: port.to_string(),
                value,
                pass_through: false,
            }
        }

        fn echo() -> Self {
            Self {
                port: String::new(),
                value: Value::Unit,
                pass_through: true,
            }
        }
    }

    impl Executable for TestOp {
        fn execute(
            &self,
            inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, ExecError> {
            if self.pass_through {
                return Ok(inputs);
            }
            let mut out = HashMap::new();
            out.insert(self.port.clone(), self.value.clone());
            Ok(out)
        }
    }

    // Backward-compat alias used in existing tests
    type Produce = TestOp;

    #[test]
    fn test_execute_simple_pipeline() {
        let mut dag: Dag<Produce> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![port("out", "String")],
            TestOp::produce("out", Value::Str("hello".to_string())),
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
            TestOp::produce("response", Value::Str("real-response".to_string())),
        ));

        // In dry-run mode, transport executor nodes should be intercepted
        let mut mocks = BoundaryMocks::new();
        mocks.set_value(
            "execute_transport",
            "response",
            Value::Str("mock-response".to_string()),
        );

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
            TestOp::produce("url", Value::Str("real-url".to_string())),
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
            TestOp::produce("request", Value::Str("prepared-request".to_string())),
        ));

        // Transport executor - consumes the request (will be intercepted)
        dag.add_node(Node::opaque(
            "execute",
            vec![port("request", "TransportRequest")], // This makes it a transport executor
            vec![port("response", "TransportResponse")],
            TestOp::produce("response", Value::Str("real-response".to_string())),
        ));
        dag.add_edge(edge("prepare", "request", "execute", "request"));

        let mut mocks = BoundaryMocks::new();
        mocks.set_value("execute", "response", Value::Str("mocked".to_string()));
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
            TestOp::produce("out", Value::Str("hello".to_string())),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![port("in", "String")],
            vec![port("out", "String")],
            TestOp::produce("out", Value::Str("world".to_string())),
        ));
        dag.add_edge(edge("A", "out", "B", "in"));

        // Configure simulation with timing
        let config = SimConfig::new()
            .with_timing("A", Duration::from_millis(100))
            .with_timing("B", Duration::from_millis(200))
            .with_mocks(BoundaryMocks::new());

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
            vec![port("request", "TransportRequest")], // Makes it a transport executor
            vec![port("result", "String")],
            TestOp::produce("result", Value::Str("real-value".to_string())),
        ));

        let mut mocks = BoundaryMocks::new();
        mocks.set_value(
            "transport_node",
            "result",
            Value::Str("simulated-value".to_string()),
        );

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
    fn test_fan_in_to_list_port_collects_values() {
        // Two producers feed into a single list port — values should be collected
        // into a Value::List in canonical edge order.
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![port("out", "String")],
            TestOp::produce("out", Value::Str("alpha".to_string())),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![],
            vec![port("out", "String")],
            TestOp::produce("out", Value::Str("beta".to_string())),
        ));
        dag.add_node(Node::opaque(
            "C",
            vec![list("items", "String")], // list port: cardinality ZeroOrMore
            vec![port("items", "String")], // echo: passes inputs through as outputs
            TestOp::echo(),
        ));
        // Two edges to the same list port, with explicit indices for ordering
        dag.add_edge(Edge::with_index("A", "out", "C", "items", 0));
        dag.add_edge(Edge::with_index("B", "out", "C", "items", 1));

        let log = execute(&dag).unwrap();

        let c_entry = log.get("C").unwrap();
        match c_entry.outputs.get("items") {
            Some(Value::List(items)) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], Value::Str("alpha".to_string()));
                assert_eq!(items[1], Value::Str("beta".to_string()));
            }
            other => panic!("expected Value::List, got {:?}", other),
        }
    }

    #[test]
    fn test_list_output_to_list_input_passes_through() {
        // A list output feeding a list input should not become a list-of-lists.
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![list("items", "String")],
            TestOp::produce(
                "items",
                Value::List(vec![
                    Value::Str("alpha".to_string()),
                    Value::Str("beta".to_string()),
                ]),
            ),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![list("items", "String")],
            vec![list("items", "String")],
            TestOp::echo(),
        ));
        dag.add_edge(edge("A", "items", "B", "items"));

        let log = execute(&dag).unwrap();

        let b_entry = log.get("B").unwrap();
        match b_entry.outputs.get("items") {
            Some(Value::List(items)) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], Value::Str("alpha".to_string()));
                assert_eq!(items[1], Value::Str("beta".to_string()));
            }
            other => panic!("expected Value::List, got {:?}", other),
        }
    }

    #[test]
    fn test_scalar_port_takes_single_value() {
        // A scalar port with one incoming edge should still work as before.
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![port("out", "String")],
            TestOp::produce("out", Value::Str("hello".to_string())),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![port("data", "String")],
            vec![port("data", "String")],
            TestOp::echo(),
        ));
        dag.add_edge(edge("A", "out", "B", "data"));

        let log = execute(&dag).unwrap();

        // B echoes its input — should receive the scalar value from A
        let b_entry = log.get("B").unwrap();
        assert_eq!(
            b_entry.outputs.get("data"),
            Some(&Value::Str("hello".to_string()))
        );
    }

    #[test]
    fn test_scalar_port_fan_in_errors() {
        // A scalar port with multiple incoming edges should fail.
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![port("out", "String")],
            TestOp::produce("out", Value::Str("alpha".to_string())),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![],
            vec![port("out", "String")],
            TestOp::produce("out", Value::Str("beta".to_string())),
        ));
        dag.add_node(Node::opaque(
            "C",
            vec![port("data", "String")],
            vec![port("data", "String")],
            TestOp::echo(),
        ));
        dag.add_edge(edge("A", "out", "C", "data"));
        dag.add_edge(edge("B", "out", "C", "data"));

        let err = execute(&dag).expect_err("expected scalar fan-in to error");
        assert!(err
            .to_string()
            .contains("scalar input 'C.data' has multiple upstream edges"));
    }

    #[test]
    fn test_list_port_zero_edges_defaults_to_empty_list() {
        // A list port with no incoming edges should default to an empty list.
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![list("items", "String")],
            vec![list("items", "String")],
            TestOp::echo(),
        ));

        let log = execute(&dag).unwrap();

        let a_entry = log.get("A").unwrap();
        match a_entry.outputs.get("items") {
            Some(Value::List(items)) => assert!(items.is_empty()),
            other => panic!("expected empty Value::List, got {:?}", other),
        }
    }

    #[test]
    fn test_optional_to_list_skips_unit() {
        // Optional output (Unit) feeding a list input should not insert Unit.
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![optional("item", "String")],
            TestOp::produce("item", Value::Unit),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![list("items", "String")],
            vec![list("items", "String")],
            TestOp::echo(),
        ));
        dag.add_edge(edge("A", "item", "B", "items"));

        let log = execute(&dag).unwrap();

        let b_entry = log.get("B").unwrap();
        match b_entry.outputs.get("items") {
            Some(Value::List(items)) => assert!(items.is_empty()),
            other => panic!("expected empty Value::List, got {:?}", other),
        }
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
                    .with_concurrency(4),
            );

        assert_eq!(
            config.node_duration(&NodeId::from("node1")),
            Duration::from_secs(1)
        );
        assert_eq!(
            config.node_duration(&NodeId::from("node2")),
            Duration::from_secs(2)
        );
        assert_eq!(
            config.node_duration(&NodeId::from("unknown")),
            Duration::ZERO
        );
        assert_eq!(config.random_seed, Some(42));
        assert_eq!(config.resources.max_memory, Some(1024 * 1024));
        assert_eq!(config.resources.max_cpu_ms, Some(5000));
        assert_eq!(config.resources.max_concurrency, Some(4));
    }
}
