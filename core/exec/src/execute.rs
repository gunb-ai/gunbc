//! DAG execution with boundary interception and simulation.
//!
//! # DryRun Interception
//!
//! DryRun mode intercepts **transport execution nodes** (nodes that consume
//! `TransportRequest` values), **environment nodes** (nodes that emit
//! resource outputs like `ToolHandle`, `FilesystemHandle`, `NetworkHandle`, `Timestamp`,
//! `Credential`, or `Platform`), **tool consumer nodes** (nodes that
//! consume `ToolHandle`), and **nodes with explicit mocks for all outputs**.
//! Intercepted nodes require **explicit mocks for every output port** — there
//! is no default fallback.
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
//! - It has an output port with type `FilesystemHandle`, `NetworkHandle`, `Timestamp`, `Credential`, or `Platform`
//!
//! A node is considered a tool consumer node if:
//! - It has an input port with type `ToolHandle`
//!
//! Boundary detection (`BoundaryInfo`) is still used for signature inference
//! and workflow interface detection, but NOT for DryRun interception.

use crate::error::{ExecError, IntoExecResult};
use crate::intercept::BoundaryMocks;
use crate::lower::{lower, LoopInfo};
use crate::progress::{DagSnapshot, OutputSummary, ProgressObserver};
use crate::topo::topo_sort;
use crate::Executable;
use gunbc_ir::{
    canonical_edge_order, detect_boundaries, detect_entrypoints, BoundaryInfo, Cardinality, Dag,
    Node, NodeBody, NodeId, Value,
};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::mpsc;
use std::thread;
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
pub fn execute<T: Executable + Clone + Send>(dag: &Dag<T>) -> Result<ExecutionLog, ExecError> {
    execute_with_mode(dag, ExecutionMode::Real)
}

/// Execute a DAG with the specified execution mode.
///
/// In dry-run mode, boundary nodes have their outputs replaced with mock values.
/// In simulate mode, timing and resource usage are tracked.
pub fn execute_with_mode<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
) -> Result<ExecutionLog, ExecError> {
    execute_with_mode_and_inputs(dag, mode, None)
}

/// Execute a DAG with the specified execution mode and optional input mocks.
///
/// Input mocks are injected into entrypoint ports (inputs with no upstream edge).
/// Mock keys using original SubDag IDs are automatically remapped to the
/// lowered inner entrypoint IDs.
pub fn execute_with_mode_and_inputs<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, ExecError> {
    // Lower sub-DAGs first
    let lowered = lower(dag).exec_context("lowering failed")?;

    // Remap input mock keys from original SubDag IDs to lowered inner IDs
    let remapped_mocks = input_mocks.map(|mocks| remap_input_mocks(mocks, &lowered.input_remaps));
    let effective_mocks = remapped_mocks.as_ref().or(input_mocks);

    // Remap DryRun/Simulate mode input mocks too
    let effective_mode = remap_mode_inputs(mode, &lowered.input_remaps);

    // Detect boundaries
    let boundaries = detect_boundaries(&lowered.dag);

    // Execute the flat DAG
    execute_flat(
        &lowered.dag,
        &boundaries,
        &effective_mode,
        None,
        None,
        effective_mocks,
        &lowered.loops,
    )
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
pub fn execute_with_ci<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    ci: &mut crate::CiContext,
) -> Result<ExecutionLog, ExecError> {
    execute_with_mode_and_ci(dag, ExecutionMode::Real, ci)
}

/// Execute a DAG with both execution mode and CI context.
pub fn execute_with_mode_and_ci<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    ci: &mut crate::CiContext,
) -> Result<ExecutionLog, ExecError> {
    execute_with_mode_and_ci_and_inputs(dag, mode, ci, None)
}

/// Execute a DAG with execution mode, CI context, and optional input mocks.
///
/// Input mocks are injected into entrypoint ports after lowering/remapping,
/// matching the behavior of [`execute_with_mode_and_inputs`].
pub fn execute_with_mode_and_ci_and_inputs<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    ci: &mut crate::CiContext,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, ExecError> {
    // Lower sub-DAGs first
    let lowered = lower(dag).exec_context("lowering failed")?;

    // Remap input mock keys from original SubDag IDs to lowered inner IDs
    let remapped_mocks = input_mocks.map(|mocks| remap_input_mocks(mocks, &lowered.input_remaps));
    let effective_mocks = remapped_mocks.as_ref().or(input_mocks);

    // Remap DryRun/Simulate mode input mocks too
    let effective_mode = remap_mode_inputs(mode, &lowered.input_remaps);

    // Detect boundaries
    let boundaries = detect_boundaries(&lowered.dag);

    // Execute the flat DAG with CI context
    execute_flat(
        &lowered.dag,
        &boundaries,
        &effective_mode,
        Some(ci),
        None,
        effective_mocks,
        &lowered.loops,
    )
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
pub fn execute_with_progress<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    observer: &mut dyn ProgressObserver,
) -> Result<ExecutionLog, ExecError> {
    execute_with_progress_and_mode(dag, ExecutionMode::Real, observer)
}

/// Execute a DAG with both execution mode and progress observer.
pub fn execute_with_progress_and_mode<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    observer: &mut dyn ProgressObserver,
) -> Result<ExecutionLog, ExecError> {
    let lowered = lower(dag).exec_context("lowering failed")?;
    let boundaries = detect_boundaries(&lowered.dag);
    execute_flat(
        &lowered.dag,
        &boundaries,
        &mode,
        None,
        Some(observer),
        None,
        &lowered.loops,
    )
}

/// Execute a DAG with both execution mode and progress observer plus input mocks.
pub fn execute_with_progress_and_mode_and_inputs<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    observer: &mut dyn ProgressObserver,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, ExecError> {
    let lowered = lower(dag).exec_context("lowering failed")?;
    let remapped_mocks = input_mocks.map(|mocks| remap_input_mocks(mocks, &lowered.input_remaps));
    let effective_mocks = remapped_mocks.as_ref().or(input_mocks);
    let effective_mode = remap_mode_inputs(mode, &lowered.input_remaps);
    let boundaries = detect_boundaries(&lowered.dag);
    execute_flat(
        &lowered.dag,
        &boundaries,
        &effective_mode,
        None,
        Some(observer),
        effective_mocks,
        &lowered.loops,
    )
}

/// Execute a DAG with execution mode, CI context, and progress observer.
pub fn execute_with_all<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    ci: &mut crate::CiContext,
    observer: &mut dyn ProgressObserver,
) -> Result<ExecutionLog, ExecError> {
    let lowered = lower(dag).exec_context("lowering failed")?;
    let boundaries = detect_boundaries(&lowered.dag);
    execute_flat(
        &lowered.dag,
        &boundaries,
        &mode,
        Some(ci),
        Some(observer),
        None,
        &lowered.loops,
    )
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
pub fn execute_single_node<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    node_id: &str,
    inputs: HashMap<String, Value>,
    mode: ExecutionMode,
) -> Result<HashMap<String, Value>, ExecError> {
    // Lower sub-DAGs first (in case the target node is inside a sub-DAG)
    let lowered = lower(dag).exec_context("lowering failed")?;

    // Find the node
    let node = lowered
        .dag
        .nodes
        .iter()
        .find(|n| n.id.0 == node_id)
        .ok_or_else(|| ExecError::new(format!("node '{}' not found in DAG", node_id)))?;

    // Check if this is a transport execution node for interception
    let is_transport_executor = is_transport_execution_node(node);
    let is_tool_env = is_tool_env_node(node);
    let is_resource_env = is_resource_env_node(node);
    let is_tool_consumer = consumes_tool_handle(node);
    let has_full_mock = match &mode {
        ExecutionMode::DryRun(m) => has_full_mock_for_node(node, m),
        ExecutionMode::Simulate(config) => has_full_mock_for_node(node, &config.boundary_mocks),
        _ => false,
    };
    let should_intercept = (is_transport_executor
        || is_tool_env
        || is_resource_env
        || is_tool_consumer
        || has_full_mock)
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
pub fn simulate<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    config: SimConfig,
) -> Result<SimulationResult, ExecError> {
    // Lower sub-DAGs first
    let lowered = lower(dag).exec_context("lowering failed")?;

    // Detect boundaries
    let boundaries = detect_boundaries(&lowered.dag);

    // Get topological order
    let order = topo_sort(&lowered.dag);

    // Execute with simulation tracking (no CI context in simulation)
    let log = execute_flat(
        &lowered.dag,
        &boundaries,
        &ExecutionMode::Simulate(config.clone()),
        None,
        None,
        None,
        &lowered.loops,
    )?;

    // Compute simulation metrics
    let timeline = compute_timeline(&order, &config);
    let total_time = timeline
        .iter()
        .map(|(_, start, dur)| *start + *dur)
        .max()
        .unwrap_or(Duration::ZERO);
    let critical_path = compute_critical_path(&lowered.dag, &config);
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

/// Remap input mock keys from original SubDag IDs to lowered inner IDs.
///
/// When a user sets an input mock for a SubDag node (e.g., `("my_loop", "items")`),
/// the lowered DAG has no `"my_loop"` node — instead it has `"my_loop/unpack"`.
/// This function creates a new `BoundaryMocks` with keys remapped using the
/// lowering's `input_remaps` table.
fn remap_input_mocks(
    mocks: &BoundaryMocks,
    input_remaps: &HashMap<(String, String), Vec<(String, String)>>,
) -> BoundaryMocks {
    let mut result = mocks.clone();
    for ((node_id, port_name), value) in mocks.iter_inputs() {
        let key = (node_id.clone(), port_name.clone());
        if let Some(targets) = input_remaps.get(&key) {
            for (inner_id, inner_port) in targets {
                result.set_input(inner_id.clone(), inner_port.clone(), value.clone());
            }
        }
    }
    result
}

/// Remap input mocks embedded in DryRun/Simulate execution modes.
fn remap_mode_inputs(
    mode: ExecutionMode,
    input_remaps: &HashMap<(String, String), Vec<(String, String)>>,
) -> ExecutionMode {
    if input_remaps.is_empty() {
        return mode;
    }
    match mode {
        ExecutionMode::DryRun(mocks) => {
            ExecutionMode::DryRun(remap_input_mocks(&mocks, input_remaps))
        }
        other => other,
    }
}

/// Execute a flat (fully lowered) DAG.
fn execute_flat<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    boundaries: &BoundaryInfo,
    mode: &ExecutionMode,
    ci: Option<&mut crate::CiContext>,
    observer: Option<&mut dyn ProgressObserver>,
    input_mocks: Option<&BoundaryMocks>,
    loops: &[LoopInfo<T>],
) -> Result<ExecutionLog, ExecError> {
    if ci.is_none() {
        return execute_flat_parallel(dag, boundaries, mode, observer, input_mocks, loops);
    }

    execute_flat_sequential(dag, boundaries, mode, ci, observer, input_mocks, loops)
}

/// Execute a flat DAG sequentially.
fn execute_flat_sequential<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    boundaries: &BoundaryInfo,
    mode: &ExecutionMode,
    ci: Option<&mut crate::CiContext>,
    observer: Option<&mut dyn ProgressObserver>,
    input_mocks: Option<&BoundaryMocks>,
    loops: &[LoopInfo<T>],
) -> Result<ExecutionLog, ExecError> {
    let order = topo_sort(dag);
    let node_map: HashMap<&str, &Node<T>> =
        dag.nodes.iter().map(|n| (n.id.0.as_str(), n)).collect();

    let mut node_outputs: HashMap<String, HashMap<String, Value>> = HashMap::new();
    let mut entries = Vec::new();

    // Precompute canonical edge order and group by destination node.
    let ordered_edges = canonical_edge_order(&dag.edges);
    let mut edges_by_to_node: HashMap<NodeId, Vec<&gunbc_ir::Edge>> = HashMap::new();
    for edge in ordered_edges {
        edges_by_to_node
            .entry(edge.to_node.clone())
            .or_default()
            .push(edge);
    }

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

        if let Some(edges) = edges_by_to_node.get(node_id) {
            for &edge in edges {
                if let Some(upstream) = node_outputs.get(&edge.from_node.0) {
                    if let Some(val) = upstream.get(&edge.from_port.0) {
                        if list_ports.contains_key(edge.to_port.0.as_str()) {
                            let from_cardinality = dag
                                .get_node(&edge.from_node)
                                .and_then(|n| n.outputs.iter().find(|p| p.name == edge.from_port))
                                .map(|p| p.cardinality)
                                .unwrap_or(Cardinality::ONE);

                            if let Some(elements) = collect_fan_in(val, from_cardinality) {
                                let bucket = fan_in.entry(edge.to_port.0.clone()).or_default();
                                bucket.extend(elements);
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
            let has_full_mock = match mode {
                ExecutionMode::DryRun(ref m) => has_full_mock_for_node(node, m),
                ExecutionMode::Simulate(ref config) => {
                    has_full_mock_for_node(node, &config.boundary_mocks)
                }
                _ => false,
            };
            let should_intercept = (is_transport_executor
                || is_tool_env
                || is_resource_env
                || is_tool_consumer
                || has_full_mock)
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

        // Loop body execution: if this node is a loop unpack, execute the body
        // template once per element and replace the element output with results.
        if let Some(loop_info) = loops.iter().find(|l| l.unpack_id == *node_id) {
            let body_entries = execute_loop_body(loop_info, &node_outputs, mode)?;

            // Collect body result values to replace unpack's element output.
            let results: Vec<Value> = body_entries
                .iter()
                .filter_map(|e| {
                    // The last node in each iteration's body produces "result".
                    e.outputs.get("result").cloned()
                })
                .collect();

            // Replace the element port output with body results so
            // the unpack→pack edge carries transformed values to pack.
            if let Some(unpack_out) = node_outputs.get_mut(&loop_info.unpack_id.0) {
                unpack_out.insert(loop_info.element_port.clone(), Value::List(results));
            }

            entries.extend(body_entries);
        }
    }

    // Notify observer of successful DAG completion
    if let Some(ref mut o) = obs {
        o.on_dag_complete(dag_start.elapsed());
    }

    Ok(ExecutionLog { entries })
}

/// Parse max node concurrency from `GUNBC_EXEC_MAX_CONCURRENCY`.
///
/// Defaults to unbounded so all ready nodes can run immediately.
fn execution_max_concurrency() -> usize {
    std::env::var("GUNBC_EXEC_MAX_CONCURRENCY")
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(usize::MAX)
}

fn should_intercept_for_mode<T>(node: &Node<T>, mode: &ExecutionMode) -> bool {
    let is_transport_executor = is_transport_execution_node(node);
    let is_tool_env = is_tool_env_node(node);
    let is_resource_env = is_resource_env_node(node);
    let is_tool_consumer = consumes_tool_handle(node);
    let has_full_mock = match mode {
        ExecutionMode::DryRun(mocks) => has_full_mock_for_node(node, mocks),
        ExecutionMode::Simulate(config) => has_full_mock_for_node(node, &config.boundary_mocks),
        ExecutionMode::Real => false,
    };

    (is_transport_executor || is_tool_env || is_resource_env || is_tool_consumer || has_full_mock)
        && matches!(mode, ExecutionMode::DryRun(_) | ExecutionMode::Simulate(_))
}

/// Collect a single upstream value into a fan-in bucket for a list port.
///
/// Given a value produced by an upstream node and the source port's
/// cardinality, returns the elements to add to the fan-in bucket, or
/// `None` if the value should be skipped (absent optional, Skipped sentinel).
///
/// This is the core coercion logic — it handles:
/// - **WrapScalar**: scalar value → single-element vec
/// - **OptionalToList**: Unit from empty-allowing port → None (skipped)
/// - **Widen**: list value from list port → flattened elements
fn collect_fan_in(val: &Value, from_cardinality: Cardinality) -> Option<Vec<Value>> {
    // Optional/list outputs represent absence as Unit — skip those.
    if matches!(val, Value::Unit) && from_cardinality.allows_empty() {
        return None;
    }
    // Skipped outputs should not become list elements.
    if matches!(val, Value::Skipped) {
        return None;
    }

    if from_cardinality.is_list() {
        // List → list: flatten elements (Widen coercion)
        if let Value::List(items) = val {
            Some(items.clone())
        } else {
            Some(vec![val.clone()])
        }
    } else {
        // Scalar → list: wrap as single element (WrapScalar coercion)
        Some(vec![val.clone()])
    }
}

fn build_node_inputs<T>(
    dag: &Dag<T>,
    node: &Node<T>,
    node_id: &NodeId,
    edges_by_to_node: &HashMap<NodeId, Vec<&gunbc_ir::Edge>>,
    node_outputs: &HashMap<String, HashMap<String, Value>>,
    mode: &ExecutionMode,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<HashMap<String, Value>, ExecError> {
    // Gather inputs from upstream edges (cardinality-aware).
    let mut inputs: HashMap<String, Value> = HashMap::new();
    let mut fan_in: HashMap<String, Vec<Value>> = HashMap::new();
    let mut scalar_sources: HashMap<String, String> = HashMap::new();

    let list_ports: HashMap<&str, Cardinality> = node
        .inputs
        .iter()
        .filter(|p| p.cardinality.is_list())
        .map(|p| (p.name.0.as_str(), p.cardinality))
        .collect();

    if let Some(edges) = edges_by_to_node.get(node_id) {
        for &edge in edges {
            if let Some(upstream) = node_outputs.get(&edge.from_node.0) {
                if let Some(val) = upstream.get(&edge.from_port.0) {
                    if list_ports.contains_key(edge.to_port.0.as_str()) {
                        let from_cardinality = dag
                            .get_node(&edge.from_node)
                            .and_then(|n| n.outputs.iter().find(|p| p.name == edge.from_port))
                            .map(|p| p.cardinality)
                            .unwrap_or(Cardinality::ONE);

                        if let Some(elements) = collect_fan_in(val, from_cardinality) {
                            let bucket = fan_in.entry(edge.to_port.0.clone()).or_default();
                            bucket.extend(elements);
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

    for (port_name, values) in fan_in {
        inputs.insert(port_name, Value::List(values));
    }

    let mut inject_inputs = |mocks: &BoundaryMocks| {
        for port in &node.inputs {
            if !inputs.contains_key(&port.name.0) {
                if let Some(mock_value) = mocks.get_input(&node.id.0, &port.name.0) {
                    inputs.insert(port.name.0.clone(), mock_value.clone());
                }
            }
        }
    };

    if let Some(mocks) = input_mocks {
        inject_inputs(mocks);
    }

    if let ExecutionMode::DryRun(mocks)
    | ExecutionMode::Simulate(SimConfig {
        boundary_mocks: mocks,
        ..
    }) = mode
    {
        inject_inputs(mocks);
    }

    for port in &node.inputs {
        if port.cardinality.is_list()
            && port.cardinality.allows_empty()
            && !inputs.contains_key(&port.name.0)
        {
            inputs.insert(port.name.0.clone(), Value::List(vec![]));
        }
    }

    Ok(inputs)
}

struct ParallelSchedulerState<'a, T> {
    // Immutable lookup tables
    node_index: HashMap<NodeId, usize>,
    loops_by_unpack: HashMap<NodeId, &'a LoopInfo<T>>,
    dependents: HashMap<NodeId, Vec<NodeId>>,

    // Mutable scheduling state
    node_outputs: HashMap<String, HashMap<String, Value>>,
    node_entries: Vec<Option<LogEntry>>,
    loop_entries: Vec<Vec<LogEntry>>,
    remaining_deps: HashMap<NodeId, usize>,
    ready: Vec<NodeId>,
    completed: usize,
}

fn finalize_node_parallel<T: Executable + Clone + Send>(
    node_id: &NodeId,
    outputs: HashMap<String, Value>,
    was_intercepted: bool,
    mode: &ExecutionMode,
    state: &mut ParallelSchedulerState<'_, T>,
) -> Result<(), ExecError> {
    let idx = *state.node_index.get(node_id).ok_or_else(|| {
        ExecError::new(format!(
            "node '{}' missing from topological order",
            node_id.0
        ))
    })?;

    state
        .node_outputs
        .insert(node_id.0.clone(), outputs.clone());
    state.node_entries[idx] = Some(LogEntry {
        node_id: node_id.0.clone(),
        outputs,
        was_intercepted,
    });

    if let Some(loop_info) = state.loops_by_unpack.get(node_id) {
        let body_entries = execute_loop_body(loop_info, &state.node_outputs, mode)?;

        // Replace the unpack element output with transformed body results.
        let results: Vec<Value> = body_entries
            .iter()
            .filter_map(|entry| entry.outputs.get("result").cloned())
            .collect();
        if let Some(unpack_out) = state.node_outputs.get_mut(&loop_info.unpack_id.0) {
            unpack_out.insert(loop_info.element_port.clone(), Value::List(results));
        }

        state.loop_entries[idx].extend(body_entries);
    }

    state.completed += 1;
    if let Some(children) = state.dependents.get(node_id) {
        for child in children {
            let rem = state.remaining_deps.get_mut(child).ok_or_else(|| {
                ExecError::new(format!("node '{}' missing dependency counter", child.0))
            })?;
            if *rem == 0 {
                return Err(ExecError::new(format!(
                    "node '{}' became ready more than once",
                    child.0
                )));
            }
            *rem -= 1;
            if *rem == 0 {
                state.ready.push(child.clone());
            }
        }
    }

    Ok(())
}

fn execute_flat_parallel<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    boundaries: &BoundaryInfo,
    mode: &ExecutionMode,
    observer: Option<&mut dyn ProgressObserver>,
    input_mocks: Option<&BoundaryMocks>,
    loops: &[LoopInfo<T>],
) -> Result<ExecutionLog, ExecError> {
    struct NodeExecutionResult {
        node_id: NodeId,
        started_at: Instant,
        result: Result<HashMap<String, Value>, ExecError>,
    }

    let order = topo_sort(dag);
    let node_map: HashMap<&str, &Node<T>> =
        dag.nodes.iter().map(|n| (n.id.0.as_str(), n)).collect();
    let node_index: HashMap<NodeId, usize> = order
        .iter()
        .enumerate()
        .map(|(idx, id)| (id.clone(), idx))
        .collect();
    let loops_by_unpack: HashMap<NodeId, &LoopInfo<T>> = loops
        .iter()
        .map(|loop_info| (loop_info.unpack_id.clone(), loop_info))
        .collect();

    // Precompute canonical edge order and group by destination node.
    let ordered_edges = canonical_edge_order(&dag.edges);
    let mut edges_by_to_node: HashMap<NodeId, Vec<&gunbc_ir::Edge>> = HashMap::new();
    for edge in ordered_edges {
        edges_by_to_node
            .entry(edge.to_node.clone())
            .or_default()
            .push(edge);
    }

    // Scheduling graph built from unique node dependencies.
    let mut dependencies: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
    let mut dependents_set: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
    for node in &dag.nodes {
        dependencies.entry(node.id.clone()).or_default();
        dependents_set.entry(node.id.clone()).or_default();
    }
    for edge in &dag.edges {
        dependencies
            .entry(edge.to_node.clone())
            .or_default()
            .insert(edge.from_node.clone());
        dependents_set
            .entry(edge.from_node.clone())
            .or_default()
            .insert(edge.to_node.clone());
    }

    let remaining_deps: HashMap<NodeId, usize> = dependencies
        .into_iter()
        .map(|(node_id, parents)| (node_id, parents.len()))
        .collect();
    let mut dependents: HashMap<NodeId, Vec<NodeId>> = dependents_set
        .into_iter()
        .map(|(node_id, children)| {
            let mut sorted: Vec<NodeId> = children.into_iter().collect();
            sorted.sort_by_key(|id| node_index.get(id).copied().unwrap_or(usize::MAX));
            (node_id, sorted)
        })
        .collect();
    for node in &dag.nodes {
        dependents.entry(node.id.clone()).or_default();
    }

    let ready: Vec<NodeId> = order
        .iter()
        .filter(|id| remaining_deps.get(*id).copied().unwrap_or(0) == 0)
        .cloned()
        .collect();

    let mut state = ParallelSchedulerState {
        node_index,
        loops_by_unpack,
        dependents,
        node_outputs: HashMap::new(),
        node_entries: vec![None; order.len()],
        loop_entries: (0..order.len()).map(|_| Vec::new()).collect(),
        remaining_deps,
        ready,
        completed: 0,
    };

    let max_concurrency = execution_max_concurrency();
    let mut in_flight = 0usize;
    let mut obs = observer;
    let dag_start = Instant::now();
    if let Some(ref mut o) = obs {
        let snapshot = DagSnapshot::from_dag(dag, &order, boundaries);
        o.on_dag_start(&snapshot);
    }

    let (tx, rx) = mpsc::channel::<NodeExecutionResult>();
    let scoped_result = thread::scope(|scope| -> Result<(), ExecError> {
        while state.completed < order.len() {
            let node_index = &state.node_index;
            state
                .ready
                .sort_by_key(|id| node_index.get(id).copied().unwrap_or(usize::MAX));
            while !state.ready.is_empty() && in_flight < max_concurrency {
                let node_id = state.ready.remove(0);
                let node = node_map
                    .get(node_id.0.as_str())
                    .ok_or_else(|| ExecError::new(format!("node '{}' not found", node_id.0)))?;

                let inputs = build_node_inputs(
                    dag,
                    node,
                    &node_id,
                    &edges_by_to_node,
                    &state.node_outputs,
                    mode,
                    input_mocks,
                )?;

                if should_skip_node(node, &inputs) {
                    let outputs: HashMap<String, Value> = node
                        .outputs
                        .iter()
                        .map(|port| (port.name.0.clone(), Value::Skipped))
                        .collect();
                    if let Some(ref mut o) = obs {
                        o.on_node_skipped(&node_id);
                    }
                    finalize_node_parallel(&node_id, outputs, false, mode, &mut state)?;
                    continue;
                }

                let node_start = Instant::now();
                if let Some(ref mut o) = obs {
                    o.on_node_start(&node_id);
                }

                if should_intercept_for_mode(node, mode) {
                    let mocks = match mode {
                        ExecutionMode::DryRun(mocks) => mocks,
                        ExecutionMode::Simulate(config) => &config.boundary_mocks,
                        ExecutionMode::Real => unreachable!(),
                    };
                    let outputs = match mock_intercept_outputs(node, mocks) {
                        Ok(outputs) => outputs,
                        Err(e) => {
                            if let Some(ref mut o) = obs {
                                o.on_node_failed(&node_id, &e.to_string());
                            }
                            return Err(e);
                        }
                    };
                    if let Some(ref mut o) = obs {
                        let summary = OutputSummary::from_outputs(&outputs, node_start.elapsed());
                        o.on_node_intercepted(&node_id, summary);
                    }
                    finalize_node_parallel(&node_id, outputs, true, mode, &mut state)?;
                    continue;
                }

                match &node.body {
                    NodeBody::Opaque(op) => {
                        let op = op.clone();
                        let node_id_clone = node_id.clone();
                        let tx = tx.clone();
                        scope.spawn(move || {
                            let result = op.execute(inputs);
                            let _ = tx.send(NodeExecutionResult {
                                node_id: node_id_clone,
                                started_at: node_start,
                                result,
                            });
                        });
                        in_flight += 1;
                    }
                    NodeBody::SubDag(_) => {
                        let err = ExecError::new(format!(
                            "node '{}' is a SubDag — DAG must be lowered before execution",
                            node_id.0
                        ));
                        if let Some(ref mut o) = obs {
                            o.on_node_failed(&node_id, &err.to_string());
                        }
                        return Err(err);
                    }
                }
            }

            if state.completed >= order.len() {
                break;
            }

            if in_flight == 0 {
                return Err(ExecError::new(
                    "execution stalled: no ready nodes and no running tasks",
                ));
            }

            let completed_node = rx
                .recv()
                .map_err(|_| ExecError::new("execution worker channel closed unexpectedly"))?;
            in_flight = in_flight.saturating_sub(1);
            match completed_node.result {
                Ok(outputs) => {
                    if let Some(ref mut o) = obs {
                        let summary = OutputSummary::from_outputs(
                            &outputs,
                            completed_node.started_at.elapsed(),
                        );
                        o.on_node_complete(&completed_node.node_id, summary);
                    }
                    finalize_node_parallel(
                        &completed_node.node_id,
                        outputs,
                        false,
                        mode,
                        &mut state,
                    )?
                }
                Err(e) => {
                    if let Some(ref mut o) = obs {
                        o.on_node_failed(&completed_node.node_id, &e.to_string());
                    }
                    return Err(e);
                }
            }
        }

        Ok(())
    });

    match scoped_result {
        Ok(()) => {
            if let Some(ref mut o) = obs {
                o.on_dag_complete(dag_start.elapsed());
            }
        }
        Err(e) => {
            if let Some(ref mut o) = obs {
                o.on_dag_complete(dag_start.elapsed());
            }
            return Err(e);
        }
    }

    let mut entries = Vec::new();
    for (idx, node_id) in order.iter().enumerate() {
        let entry = state.node_entries[idx].take().ok_or_else(|| {
            ExecError::new(format!(
                "node '{}' did not produce an execution log entry",
                node_id.0
            ))
        })?;
        entries.push(entry);
        entries.append(&mut state.loop_entries[idx]);
    }

    Ok(ExecutionLog { entries })
}

/// Add default mocks for transport nodes in a loop body DAG that don't
/// already have explicit mocks. This lets DryRun mode intercept body-internal
/// transport nodes without requiring graph_mock to reference their IDs
/// (which aren't visible at the outer DAG level).
fn auto_mock_body_transport<T>(body_dag: &Dag<T>, existing: &BoundaryMocks) -> BoundaryMocks {
    use gunbc_ir::transport::{ShellResponse, TransportResponse};

    let mut augmented = existing.clone();
    for node in &body_dag.nodes {
        if is_transport_execution_node(node) {
            // Only add default mocks for outputs that don't already have one
            for port in &node.outputs {
                if !existing.has_mock(&node.id, &port.name) {
                    let default_response =
                        Value::Response(TransportResponse::Shell(ShellResponse::ok("")));
                    augmented.set_value(&node.id.0, &port.name.0, default_response);
                }
            }
        }
    }
    augmented
}

/// Execute a loop body template once per element from the unpack node.
///
/// For each element in the unpack's list output, the body template DAG is
/// lowered, injected with the element value as an input mock, and executed.
/// Each iteration's nodes get prefixed with `{unpack_id}/body_{i}/` for
/// unique identification in the execution log.
fn execute_loop_body<T: Executable + Clone + Send>(
    loop_info: &LoopInfo<T>,
    node_outputs: &HashMap<String, HashMap<String, Value>>,
    mode: &ExecutionMode,
) -> Result<Vec<LogEntry>, ExecError> {
    // Get the element list from the unpack outputs
    let unpack_outputs = node_outputs.get(&loop_info.unpack_id.0).ok_or_else(|| {
        ExecError::new(format!(
            "loop body: unpack '{}' has no outputs",
            loop_info.unpack_id.0
        ))
    })?;

    let elements = match unpack_outputs.get(&loop_info.element_port) {
        Some(Value::List(list)) => list.clone(),
        Some(Value::Skipped) | None => vec![],
        Some(other) => vec![other.clone()],
    };

    // Collect extra input values from the unpack's inputs (these were wired
    // through the unpack node at build time and are available in its inputs).
    // We look them up from node_outputs of upstream nodes feeding the unpack.
    // For simplicity, we search all stored outputs for values that the body needs.
    let body_entrypoints = detect_entrypoints(&loop_info.body_dag);
    let mut extra_inputs: HashMap<String, Value> = HashMap::new();
    for (_, port_name, _) in &body_entrypoints.entrypoint_ports {
        if port_name.0 == loop_info.element_port {
            continue;
        }
        // Check if the unpack node received this as an input
        if let Some(val) = unpack_outputs.get(&port_name.0) {
            extra_inputs.insert(port_name.0.clone(), val.clone());
        }
    }

    // In DryRun/Simulate mode, auto-mock transport nodes in the body DAG
    // that don't already have explicit mocks. This lets graph_mock skip
    // body-internal node IDs (which aren't visible at the outer DAG level).
    let body_mode = match mode {
        ExecutionMode::DryRun(ref mocks) => {
            let augmented = auto_mock_body_transport(&loop_info.body_dag, mocks);
            ExecutionMode::DryRun(augmented)
        }
        ExecutionMode::Simulate(ref config) => {
            let augmented = auto_mock_body_transport(&loop_info.body_dag, &config.boundary_mocks);
            ExecutionMode::Simulate(SimConfig {
                boundary_mocks: augmented,
                ..config.clone()
            })
        }
        _ => mode.clone(),
    };

    let mut all_entries = Vec::new();

    for (i, element) in elements.iter().enumerate() {
        // Build input mocks for this iteration: the element port gets the single element
        let mut iter_mocks = BoundaryMocks::new();

        // Find the body's entrypoint node for the element port
        for (node_id, port_name, _) in &body_entrypoints.entrypoint_ports {
            if port_name.0 == loop_info.element_port {
                iter_mocks.set_input(&node_id.0, &port_name.0, element.clone());
            }
            // Inject extra inputs
            if let Some(val) = extra_inputs.get(&port_name.0) {
                iter_mocks.set_input(&node_id.0, &port_name.0, val.clone());
            }
        }

        // Lower and execute the body template for this iteration
        let lowered_body =
            crate::lower::lower(&loop_info.body_dag).map_err(|e| ExecError::new(e.to_string()))?;
        let body_boundaries = detect_boundaries(&lowered_body.dag);

        // Execute in the body mode (with auto-mocked transport nodes if DryRun)
        let body_log = execute_flat(
            &lowered_body.dag,
            &body_boundaries,
            &body_mode,
            None,
            None,
            Some(&iter_mocks),
            &lowered_body.loops,
        )?;

        // Prefix iteration entries for unique identification in the log
        let prefix = format!("{}/body_{}", loop_info.unpack_id.0, i);
        for entry in body_log.entries {
            all_entries.push(LogEntry {
                node_id: format!("{}/{}", prefix, entry.node_id),
                outputs: entry.outputs,
                was_intercepted: entry.was_intercepted,
            });
        }
    }

    Ok(all_entries)
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
        crate::display::print_value(port, value);
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

/// Check if explicit mocks cover all output ports for a node.
///
/// This allows DryRun/Simulate to intercept nodes that are not transport/tool
/// nodes but still have full mock coverage (e.g., self-acquiring CLI tool nodes).
fn has_full_mock_for_node<T>(node: &Node<T>, mocks: &BoundaryMocks) -> bool {
    if node.outputs.is_empty() {
        return false;
    }

    node.outputs
        .iter()
        .all(|port| mocks.has_mock(&node.id, &port.name))
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
/// These emit resource values like FilesystemHandle, Timestamp, Credential, Platform.
fn is_resource_env_node<T>(node: &Node<T>) -> bool {
    node.outputs.iter().any(|port| {
        matches!(
            port.type_id.0.as_str(),
            "FilesystemHandle"
                | "NetworkHandle"
                | "Timestamp"
                | "Credential"
                | "Platform"
                | "CloudSecretConfig"
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
        let (value, used_fallback) = mock.next_value_with_status();
        if used_fallback && mock.is_strict() {
            return Err(ExecError::new(format!(
                "boundary mock sequence exhausted for node '{}': output port '{}'",
                node.id.0, port.name.0
            )));
        }
        outputs.insert(port.name.0.clone(), value);
    }

    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::build::*;
    use gunbc_ir::Edge;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

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
    fn test_execute_runs_ready_nodes_in_parallel() {
        if execution_max_concurrency() == 1 {
            return;
        }

        #[derive(Debug, Clone)]
        struct BlockingOp {
            port: String,
            value: Value,
            sleep_ms: u64,
            active: Arc<AtomicUsize>,
            peak: Arc<AtomicUsize>,
        }

        impl BlockingOp {
            fn new(
                port: &str,
                value: Value,
                sleep_ms: u64,
                active: Arc<AtomicUsize>,
                peak: Arc<AtomicUsize>,
            ) -> Self {
                Self {
                    port: port.to_string(),
                    value,
                    sleep_ms,
                    active,
                    peak,
                }
            }
        }

        impl Executable for BlockingOp {
            fn execute(
                &self,
                _inputs: HashMap<String, Value>,
            ) -> Result<HashMap<String, Value>, ExecError> {
                let current = self.active.fetch_add(1, Ordering::SeqCst) + 1;
                loop {
                    let observed = self.peak.load(Ordering::SeqCst);
                    if current <= observed {
                        break;
                    }
                    if self
                        .peak
                        .compare_exchange(observed, current, Ordering::SeqCst, Ordering::SeqCst)
                        .is_ok()
                    {
                        break;
                    }
                }
                std::thread::sleep(Duration::from_millis(self.sleep_ms));
                self.active.fetch_sub(1, Ordering::SeqCst);

                let mut out = HashMap::new();
                out.insert(self.port.clone(), self.value.clone());
                Ok(out)
            }
        }

        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));

        let mut dag: Dag<BlockingOp> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![port("a", "Int")],
            BlockingOp::new("a", Value::Int(1), 50, active.clone(), peak.clone()),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![],
            vec![port("b", "Int")],
            BlockingOp::new("b", Value::Int(2), 50, active.clone(), peak.clone()),
        ));
        dag.add_node(Node::opaque(
            "C",
            vec![port("a", "Int"), port("b", "Int")],
            vec![port("out", "Int")],
            BlockingOp::new("out", Value::Int(3), 0, active.clone(), peak.clone()),
        ));
        dag.add_edge(edge("A", "a", "C", "a"));
        dag.add_edge(edge("B", "b", "C", "b"));

        let log = execute(&dag).unwrap();
        assert_eq!(log.entries.len(), 3);
        assert!(
            peak.load(Ordering::SeqCst) >= 2,
            "expected at least 2 concurrent nodes, saw {}",
            peak.load(Ordering::SeqCst)
        );
    }

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
            vec![list("items", "StringList")], // list port: cardinality ZeroOrMore
            vec![list("items", "StringList")], // echo: passes inputs through as outputs (list→list)
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
            vec![list("items", "StringList")],
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
            vec![list("items", "StringList")],
            vec![list("items", "StringList")],
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
            vec![list("items", "StringList")],
            vec![list("items", "StringList")],
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
            vec![optional("item", "OptionalString")],
            TestOp::produce("item", Value::Unit),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![list("items", "StringList")],
            vec![list("items", "StringList")],
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
    fn test_optional_to_list_skips_skipped() {
        // Skipped output feeding a list input should not insert Skipped.
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![optional("item", "OptionalString")],
            TestOp::produce("item", Value::Skipped),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![list("items", "StringList")],
            vec![list("items", "StringList")],
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

    // =========================================================================
    // collect_fan_in unit tests
    //
    // These test the extracted fan-in function directly, mapping 1:1 to the
    // CoercionKind variants in coerce.rs.
    // =========================================================================

    #[test]
    fn fan_in_wraps_scalar() {
        // WrapScalar: scalar [1,1] value → single-element vec
        let val = Value::Str("hello".into());
        let elements = collect_fan_in(&val, Cardinality::ONE).unwrap();
        assert_eq!(elements, vec![Value::Str("hello".into())]);
    }

    #[test]
    fn fan_in_skips_absent_optional() {
        // OptionalToList (absent): Unit from [0,1] port → None (skipped)
        assert!(collect_fan_in(&Value::Unit, Cardinality::ZERO_OR_ONE).is_none());
    }

    #[test]
    fn fan_in_wraps_present_optional() {
        // OptionalToList (present): real value from [0,1] port → single-element vec
        let val = Value::Str("present".into());
        let elements = collect_fan_in(&val, Cardinality::ZERO_OR_ONE).unwrap();
        assert_eq!(elements, vec![Value::Str("present".into())]);
    }

    #[test]
    fn fan_in_flattens_list() {
        // Widen: list [2,5] value → flattened elements
        let val = Value::List(vec![
            Value::Str("a".into()),
            Value::Str("b".into()),
            Value::Str("c".into()),
        ]);
        let elements = collect_fan_in(&val, Cardinality::new(2, Some(5))).unwrap();
        assert_eq!(
            elements,
            vec![
                Value::Str("a".into()),
                Value::Str("b".into()),
                Value::Str("c".into()),
            ]
        );
    }

    #[test]
    fn fan_in_skips_skipped_value() {
        // Skipped sentinel is never collected regardless of cardinality
        assert!(collect_fan_in(&Value::Skipped, Cardinality::ONE).is_none());
        assert!(collect_fan_in(&Value::Skipped, Cardinality::ZERO_OR_ONE).is_none());
        assert!(collect_fan_in(&Value::Skipped, Cardinality::ZERO_OR_MORE).is_none());
    }

    #[test]
    fn fan_in_unit_from_required_port_is_kept() {
        // Unit from a required [1,1] port is NOT skipped — only empty-allowing
        // ports treat Unit as absence.
        let elements = collect_fan_in(&Value::Unit, Cardinality::ONE).unwrap();
        assert_eq!(elements, vec![Value::Unit]);
    }

    #[test]
    fn fan_in_skips_unit_from_empty_list() {
        // Unit from a [0,∞) port is skipped (allows_empty = true)
        assert!(collect_fan_in(&Value::Unit, Cardinality::ZERO_OR_MORE).is_none());
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

    #[test]
    fn test_loop_body_executes_per_element() {
        use gunbc_ir::patterns::{LoopBuilder, PatternOp};

        // Build a body DAG with a single transform node that appends "_processed"
        #[derive(Debug, Clone)]
        enum TestLoopOp {
            Pattern(PatternOp),
            AppendSuffix,
        }

        impl From<PatternOp> for TestLoopOp {
            fn from(op: PatternOp) -> Self {
                TestLoopOp::Pattern(op)
            }
        }

        impl Executable for TestLoopOp {
            fn execute(
                &self,
                inputs: HashMap<String, Value>,
            ) -> Result<HashMap<String, Value>, ExecError> {
                match self {
                    TestLoopOp::Pattern(op) => op.execute(inputs),
                    TestLoopOp::AppendSuffix => {
                        let element = inputs
                            .get("element")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        let mut out = HashMap::new();
                        out.insert(
                            "result".to_string(),
                            Value::Str(format!("{}_processed", element)),
                        );
                        Ok(out)
                    }
                }
            }
        }

        // Body DAG: single node that takes "element" and outputs "result"
        let mut body_dag: Dag<TestLoopOp> = Dag::new();
        body_dag.add_node(Node::opaque(
            "transform",
            vec![port("element", "String")],
            vec![port("result", "String")],
            TestLoopOp::AppendSuffix,
        ));

        // Build the loop node
        let loop_node: Node<TestLoopOp> = LoopBuilder::new("test_loop")
            .with_input("items", "String", Cardinality::ZERO_OR_MORE)
            .with_element("element", "String")
            .with_body(body_dag)
            .with_output("results", "String")
            .build();

        // Build a DAG: producer → loop → consumer
        let mut dag: Dag<TestLoopOp> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![list("items", "StringList")],
            TestLoopOp::Pattern(PatternOp::LoopUnpack {
                // Repurpose as a producer that outputs a list
                input_port: "unused".to_string(),
                element_port: "items".to_string(),
            }),
        ));

        // Actually, let's use a simpler approach: use input mocks
        let mut dag: Dag<TestLoopOp> = Dag::new();
        dag.add_node(loop_node);

        // Use input mocks to inject the list (set_input for DAG entry injection)
        let mut mocks = BoundaryMocks::new();
        mocks.set_input(
            "test_loop",
            "items",
            Value::List(vec![
                Value::Str("alpha".to_string()),
                Value::Str("beta".to_string()),
                Value::Str("gamma".to_string()),
            ]),
        );

        let log = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&mocks)).unwrap();

        // Find the pack node's output
        let pack_entry = log
            .entries
            .iter()
            .find(|e| e.node_id.ends_with("/pack"))
            .expect("should have a pack node entry");

        match pack_entry.outputs.get("results") {
            Some(Value::List(items)) => {
                assert_eq!(items.len(), 3, "should have 3 processed items");
                assert_eq!(items[0], Value::Str("alpha_processed".to_string()));
                assert_eq!(items[1], Value::Str("beta_processed".to_string()));
                assert_eq!(items[2], Value::Str("gamma_processed".to_string()));
            }
            other => panic!("expected Value::List, got {:?}", other),
        }

        // Verify iteration count
        match pack_entry.outputs.get("iterations") {
            Some(Value::Int(n)) => assert_eq!(*n, 3),
            other => panic!("expected iterations=3, got {:?}", other),
        }
    }

    #[test]
    fn test_loop_empty_list_produces_empty_output() {
        use gunbc_ir::patterns::{LoopBuilder, PatternOp};

        #[derive(Debug, Clone)]
        enum TestLoopOp {
            Pattern(PatternOp),
            Identity,
        }

        impl From<PatternOp> for TestLoopOp {
            fn from(op: PatternOp) -> Self {
                TestLoopOp::Pattern(op)
            }
        }

        impl Executable for TestLoopOp {
            fn execute(
                &self,
                inputs: HashMap<String, Value>,
            ) -> Result<HashMap<String, Value>, ExecError> {
                match self {
                    TestLoopOp::Pattern(op) => op.execute(inputs),
                    TestLoopOp::Identity => {
                        let mut out = HashMap::new();
                        if let Some(v) = inputs.get("element") {
                            out.insert("result".to_string(), v.clone());
                        }
                        Ok(out)
                    }
                }
            }
        }

        let mut body_dag: Dag<TestLoopOp> = Dag::new();
        body_dag.add_node(Node::opaque(
            "passthrough",
            vec![port("element", "String")],
            vec![port("result", "String")],
            TestLoopOp::Identity,
        ));

        let loop_node: Node<TestLoopOp> = LoopBuilder::new("empty_loop")
            .with_input("items", "String", Cardinality::ZERO_OR_MORE)
            .with_element("element", "String")
            .with_body(body_dag)
            .with_output("results", "String")
            .build();

        let mut dag: Dag<TestLoopOp> = Dag::new();
        dag.add_node(loop_node);

        // Inject empty list (set_input for DAG entry injection)
        let mut mocks = BoundaryMocks::new();
        mocks.set_input("empty_loop", "items", Value::List(vec![]));

        let log = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&mocks)).unwrap();

        let pack_entry = log
            .entries
            .iter()
            .find(|e| e.node_id.ends_with("/pack"))
            .expect("should have a pack node entry");

        match pack_entry.outputs.get("results") {
            Some(Value::List(items)) => assert!(items.is_empty()),
            other => panic!("expected empty Value::List, got {:?}", other),
        }
    }

    #[test]
    fn test_loop_resource_input_flows_to_body_iterations() {
        use gunbc_ir::patterns::{LoopBuilder, PatternOp, ResourceInput};

        #[derive(Debug, Clone)]
        enum TestLoopOp {
            Pattern(PatternOp),
            ConcatToken,
        }

        impl From<PatternOp> for TestLoopOp {
            fn from(op: PatternOp) -> Self {
                TestLoopOp::Pattern(op)
            }
        }

        impl Executable for TestLoopOp {
            fn execute(
                &self,
                inputs: HashMap<String, Value>,
            ) -> Result<HashMap<String, Value>, ExecError> {
                match self {
                    TestLoopOp::Pattern(op) => op.execute(inputs),
                    TestLoopOp::ConcatToken => {
                        let element = inputs
                            .get("element")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ExecError::new("missing element"))?;
                        let token = inputs
                            .get("res:token")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ExecError::new("missing res:token"))?;

                        let mut out = HashMap::new();
                        out.insert(
                            "result".to_string(),
                            Value::Str(format!("{}@{}", element, token)),
                        );
                        Ok(out)
                    }
                }
            }
        }

        let mut body_dag: Dag<TestLoopOp> = Dag::new();
        body_dag.add_node(Node::opaque(
            "transform",
            vec![port("element", "String"), port("res:token", "String")],
            vec![port("result", "String")],
            TestLoopOp::ConcatToken,
        ));

        let loop_node: Node<TestLoopOp> = LoopBuilder::new("token_loop")
            .with_input("items", "String", Cardinality::ZERO_OR_MORE)
            .with_element("element", "String")
            .with_resource_input(ResourceInput::new("res:token", "String"))
            .with_body(body_dag)
            .with_output("results", "String")
            .build();

        let mut dag: Dag<TestLoopOp> = Dag::new();
        dag.add_node(loop_node);

        let mut mocks = BoundaryMocks::new();
        mocks.set_input(
            "token_loop",
            "items",
            Value::List(vec![
                Value::Str("alpha".to_string()),
                Value::Str("beta".to_string()),
            ]),
        );
        mocks.set_input("token_loop", "res:token", Value::Str("t".to_string()));

        let log = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&mocks))
            .expect("loop execution should succeed with resource input");

        let pack_entry = log
            .entries
            .iter()
            .find(|e| e.node_id.ends_with("/pack"))
            .expect("should have a pack node entry");

        match pack_entry.outputs.get("results") {
            Some(Value::List(items)) => {
                assert_eq!(
                    items,
                    &vec![
                        Value::Str("alpha@t".to_string()),
                        Value::Str("beta@t".to_string()),
                    ]
                );
            }
            other => panic!("expected Value::List, got {:?}", other),
        }
    }

    // =========================================================================
    // execute_with_mode_and_inputs unit tests
    // =========================================================================

    #[test]
    fn test_input_mocks_inject_into_entrypoint() {
        // Node with no upstream edges receives value from input mocks
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "echo",
            vec![port("data", "String")],
            vec![port("data", "String")],
            TestOp::echo(),
        ));

        let mut mocks = BoundaryMocks::new();
        mocks.set_input("echo", "data", Value::Str("injected".into()));

        let log = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&mocks)).unwrap();

        let entry = log.get("echo").unwrap();
        assert_eq!(
            entry.outputs.get("data"),
            Some(&Value::Str("injected".into())),
            "input mock should be injected into entrypoint"
        );
    }

    #[test]
    fn test_input_mocks_with_dry_run_mode() {
        // Combine input mocks with DryRun boundary interception
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "prepare",
            vec![port("arg", "String")],
            vec![port("request", "TransportRequest")],
            TestOp::produce("request", Value::Str("built-request".into())),
        ));
        dag.add_node(Node::opaque(
            "execute_http",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            TestOp::produce("response", Value::Str("real-response".into())),
        ));
        dag.add_edge(edge("prepare", "request", "execute_http", "request"));

        // DryRun mocks intercept the transport executor
        let mut dry_mocks = BoundaryMocks::new();
        dry_mocks.set_value(
            "execute_http",
            "response",
            Value::Str("mock-response".into()),
        );

        // Input mocks inject the entrypoint arg
        let mut input_mocks = BoundaryMocks::new();
        input_mocks.set_input("prepare", "arg", Value::Str("injected-arg".into()));

        let log = execute_with_mode_and_inputs(
            &dag,
            ExecutionMode::DryRun(dry_mocks),
            Some(&input_mocks),
        )
        .unwrap();

        // prepare should run normally with the injected input
        let prepare = log.get("prepare").unwrap();
        assert!(!prepare.was_intercepted);

        // execute_http should be intercepted
        let exec = log.get("execute_http").unwrap();
        assert!(exec.was_intercepted);
        assert_eq!(
            exec.outputs.get("response"),
            Some(&Value::Str("mock-response".into()))
        );
    }

    #[test]
    fn test_input_mocks_per_port_on_non_root_node() {
        // Node B has two inputs: x (wired from A) and y (unwired entrypoint).
        // Input mock injects B.y; B.x should come from A's output.
        // This verifies per-port entrypoint injection, not per-node.
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![port("out", "String")],
            TestOp::produce("out", Value::Str("from-A".into())),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![port("x", "String"), port("y", "String")],
            vec![port("x", "String"), port("y", "String")],
            TestOp::echo(), // echoes all inputs as outputs
        ));
        dag.add_edge(edge("A", "out", "B", "x"));

        // Provide input mock for the unwired entrypoint port B.y
        let mut input_mocks = BoundaryMocks::new();
        input_mocks.set_input("B", "y", Value::Str("from-mock".into()));

        let log =
            execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&input_mocks)).unwrap();

        let b = log.get("B").unwrap();
        assert_eq!(
            b.outputs.get("x"),
            Some(&Value::Str("from-A".into())),
            "wired port B.x should receive value from upstream A"
        );
        assert_eq!(
            b.outputs.get("y"),
            Some(&Value::Str("from-mock".into())),
            "unwired entrypoint port B.y should receive value from input mock"
        );
    }

    #[test]
    fn test_input_mocks_none_works() {
        // Passing None for input_mocks should work the same as execute_with_mode
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![],
            vec![port("out", "String")],
            TestOp::produce("out", Value::Str("hello".into())),
        ));

        let log = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, None).unwrap();
        assert_eq!(log.entries.len(), 1);
        assert_eq!(
            log.entries[0].outputs.get("out"),
            Some(&Value::Str("hello".into()))
        );
    }

    #[test]
    fn test_remap_input_mocks_preserves_non_subdag() {
        // remap_input_mocks should keep original entries alongside remapped ones.
        // We test this indirectly via execute_with_mode_and_inputs on a flat DAG.
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![port("x", "String")],
            vec![port("x", "String")],
            TestOp::echo(),
        ));
        dag.add_node(Node::opaque(
            "b",
            vec![port("y", "String")],
            vec![port("y", "String")],
            TestOp::echo(),
        ));

        let mut input = BoundaryMocks::new();
        input.set_input("a", "x", Value::Str("alpha".into()));
        input.set_input("b", "y", Value::Str("beta".into()));

        let log = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&input)).unwrap();

        assert_eq!(
            log.get("a").unwrap().outputs.get("x"),
            Some(&Value::Str("alpha".into()))
        );
        assert_eq!(
            log.get("b").unwrap().outputs.get("y"),
            Some(&Value::Str("beta".into()))
        );
    }

    // =========================================================================
    // remap_input_mocks unit tests
    // =========================================================================

    #[test]
    fn test_remap_input_mocks_with_remaps() {
        let mut mocks = BoundaryMocks::new();
        mocks.set_input("subdag", "port_a", Value::Str("value".into()));

        let mut remaps: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
        remaps.insert(
            ("subdag".to_string(), "port_a".to_string()),
            vec![("subdag/inner_entry".to_string(), "inner_port".to_string())],
        );

        let result = remap_input_mocks(&mocks, &remaps);

        // Original key should still exist
        assert_eq!(
            result.get_input("subdag", "port_a"),
            Some(&Value::Str("value".into()))
        );
        // Remapped key should also exist
        assert_eq!(
            result.get_input("subdag/inner_entry", "inner_port"),
            Some(&Value::Str("value".into()))
        );
    }

    #[test]
    fn test_remap_input_mocks_empty_remaps() {
        let mut mocks = BoundaryMocks::new();
        mocks.set_input("node", "port", Value::Int(42));

        let remaps: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();

        let result = remap_input_mocks(&mocks, &remaps);
        assert_eq!(
            result.get_input("node", "port"),
            Some(&Value::Int(42)),
            "empty remaps should preserve all inputs"
        );
    }

    #[test]
    fn test_remap_input_mocks_multi_target() {
        let mut mocks = BoundaryMocks::new();
        mocks.set_input("subdag", "data", Value::Str("shared".into()));

        let mut remaps: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
        remaps.insert(
            ("subdag".to_string(), "data".to_string()),
            vec![
                ("subdag/inner_a".to_string(), "input_a".to_string()),
                ("subdag/inner_b".to_string(), "input_b".to_string()),
            ],
        );

        let result = remap_input_mocks(&mocks, &remaps);

        // Both targets should receive the value
        assert_eq!(
            result.get_input("subdag/inner_a", "input_a"),
            Some(&Value::Str("shared".into()))
        );
        assert_eq!(
            result.get_input("subdag/inner_b", "input_b"),
            Some(&Value::Str("shared".into()))
        );
    }

    #[test]
    fn test_remap_mode_inputs_dry_run() {
        let mut dry_mocks = BoundaryMocks::new();
        dry_mocks.set_input("subdag", "port", Value::Int(99));

        let mut remaps: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
        remaps.insert(
            ("subdag".to_string(), "port".to_string()),
            vec![("subdag/inner".to_string(), "inner_port".to_string())],
        );

        let mode = ExecutionMode::DryRun(dry_mocks);
        let result = remap_mode_inputs(mode, &remaps);

        match result {
            ExecutionMode::DryRun(mocks) => {
                assert_eq!(
                    mocks.get_input("subdag/inner", "inner_port"),
                    Some(&Value::Int(99)),
                    "DryRun mocks should be remapped"
                );
            }
            _ => panic!("expected DryRun mode"),
        }
    }

    #[test]
    fn test_remap_mode_inputs_real_unchanged() {
        let remaps: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
        let mode = remap_mode_inputs(ExecutionMode::Real, &remaps);
        assert!(matches!(mode, ExecutionMode::Real));
    }
}
