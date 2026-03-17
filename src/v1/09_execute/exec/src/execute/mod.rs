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
//! Interception is driven by `NodeKind` (set by the lowerer's
//! `stamp_node_kinds`). The executor does **not** fall back to port-type
//! heuristics — nodes with `kind: Pure` (the default) are treated as pure.
//! A pre-flight check (`validate_node_kinds_for_interception`) errors if any
//! `Pure` node has port patterns that indicate it should have been classified.
//! Hand-built DAGs must call `Node::with_kind()` to set the kind explicitly.
//!
//! Boundary detection (`BoundaryInfo`) is still used for signature inference
//! and workflow interface detection, but NOT for DryRun interception.

use crate::error::{ErrorLayer, ExecError, IntoExecResult, NodeRole, NodeTraceLayer};
use crate::intercept::BoundaryMocks;
use crate::lower::{lower, LoopInfo};
use crate::progress::{DagSnapshot, OutputSummary, ProgressObserver};
use crate::topo::topo_sort;
use crate::Executable;
use gunbc_ir::transport::{FileOp, TransportResponse};
use gunbc_ir::{
    canonical_edge_order, detect_boundaries, detect_entrypoints, normalize_resource_id, AccessMode,
    AppliedCoercion, BoundaryInfo, Cardinality, Dag, LogDetailLevel, Node, NodeBody, NodeId,
    NodeKind, PortName, Value, RESOURCE_FILE, RESOURCE_FILE_PREFIX,
};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Whether the executor enforces strict input validation.
///
/// In **Strict** mode, the executor fails with a diagnostic error when a
/// required input port has no value after all edge, mock, and default-list
/// resolution.  This catches modeling gaps early instead of letting them
/// propagate as surprising `Unit` / missing-key failures deep inside op
/// implementations.
///
/// **Lenient** mode (the default) silently tolerates missing required inputs,
/// passing whatever partial inputs were gathered.  This is the legacy behavior
/// preserved for tests that intentionally omit inputs.  The default will be
/// flipped to Strict once all call sites are audited.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DryRunStrictness {
    /// Missing required inputs produce a hard error.
    Strict,
    /// Missing required inputs are silently tolerated (legacy behavior).
    /// Default until all callers are audited for strict-mode readiness.
    #[default]
    Lenient,
}

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

impl ExecutionMode {
    /// Returns true if this is a dry-run or simulate mode (not real execution).
    pub fn is_intercepting(&self) -> bool {
        matches!(self, ExecutionMode::DryRun(_) | ExecutionMode::Simulate(_))
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
    pub inputs: Option<HashMap<String, Value>>,
    pub outputs: HashMap<String, Value>,
    pub was_intercepted: bool,
    /// Coercions applied to this node's inputs during execution.
    ///
    /// Empty when no coercions were needed or when log detail level
    /// is below `IncludeInputs`.
    pub coercions_applied: Vec<AppliedCoercion>,
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

impl LogEntry {
    /// Return the captured input value for a specific port.
    pub fn input_value(&self, port: &str) -> Option<&Value> {
        self.inputs.as_ref()?.get(port)
    }

    /// Return the captured input value associated with an applied coercion.
    ///
    /// This is useful for assertion-oriented observability: tests can inspect
    /// the exact shape delivered to the target port where the coercion landed.
    pub fn coercion_input_value(&self, coercion: &AppliedCoercion) -> Option<&Value> {
        self.input_value(&coercion.to_port)
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

/// Consolidated configuration for DAG execution (CP-45).
///
/// Replaces the 10-variant execute function family with a single entry point.
/// All fields have sensible defaults via `Default`.
pub struct ExecuteConfig<'a> {
    /// Execution mode (Real, DryRun, Simulate). Default: Real.
    pub mode: ExecutionMode,
    /// Optional boundary input mocks for entrypoint ports.
    pub input_mocks: Option<&'a BoundaryMocks>,
    /// Progress observer for live status callbacks.
    pub observer: Option<&'a mut dyn ProgressObserver>,
    /// Execution log detail level. Default: IncludeInputs.
    pub log_detail: LogDetailLevel,
    /// Input strictness (S32). In Strict mode (the default), missing required
    /// inputs produce a hard error.  In Lenient mode, missing inputs are
    /// silently tolerated (legacy test behavior).
    pub strictness: DryRunStrictness,
}

impl Default for ExecuteConfig<'_> {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::default(),
            input_mocks: None,
            observer: None,
            log_detail: LogDetailLevel::IncludeInputs,
            strictness: DryRunStrictness::default(),
        }
    }
}

/// Execute a DAG with the given configuration.
///
/// This is the consolidated entry point (CP-45). All other `execute_*` functions
/// delegate here.
pub fn execute_dag<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    config: ExecuteConfig<'_>,
) -> Result<ExecutionLog, ExecError> {
    let lowered = lower(dag).exec_context("lowering failed")?;
    let remapped_mocks = config
        .input_mocks
        .map(|mocks| remap_input_mocks(mocks, &lowered.input_remaps));
    let effective_mocks = remapped_mocks.as_ref().or(config.input_mocks);
    let effective_mode = remap_mode_inputs(config.mode, &lowered.input_remaps);
    let boundaries = detect_boundaries(&lowered.dag);
    execute_flat(
        &lowered.dag,
        &boundaries,
        &effective_mode,
        config.observer,
        effective_mocks,
        &lowered.loops,
        config.log_detail,
        config.strictness,
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
/// ```text
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
    let file_guard_enabled = runtime_file_guard_enabled();

    // Lower sub-DAGs first (in case the target node is inside a sub-DAG)
    let lowered = lower(dag).exec_context("lowering failed")?;

    if matches!(mode, ExecutionMode::DryRun(_) | ExecutionMode::Simulate(_)) {
        validate_node_kinds_for_interception(&lowered.dag)?;
    }

    // Find the node
    let node = lowered
        .dag
        .nodes
        .iter()
        .find(|n| n.id.0 == node_id)
        .ok_or_else(|| ExecError::new(format!("node '{}' not found in DAG", node_id)))?;

    // Check if this node should be intercepted in DryRun/Simulate mode
    let should_intercept = should_intercept_for_mode(node, &mode);

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
        NodeBody::Opaque(op) => {
            let outputs = op.execute(inputs)?;
            enforce_runtime_file_guard(node, &outputs, file_guard_enabled)?;
            Ok(outputs)
        }
        NodeBody::SubDag(..) => Err(ExecError::new(format!(
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

    // Execute with simulation tracking — simulation uses Lenient by default
    // since it's a planning/estimation tool, not a production path.
    let log = execute_flat(
        &lowered.dag,
        &boundaries,
        &ExecutionMode::Simulate(config.clone()),
        None,
        None,
        &lowered.loops,
        LogDetailLevel::IncludeInputs,
        DryRunStrictness::Lenient,
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

/// Resolve an input mock value for `(node_id, port_name)`.
///
/// For param-source nodes with an `input_alias`, looks up the alias target
/// directly instead of scanning all mocks by port-name convention.
fn resolve_mock_input(
    mocks: &BoundaryMocks,
    node_id: &NodeId,
    port_name: &PortName,
    input_alias: Option<&(NodeId, PortName)>,
) -> Option<Value> {
    if let Some(value) = mocks.get_input(&node_id.0, &port_name.0) {
        return Some(value.clone());
    }
    if let Some((alias_node, alias_port)) = input_alias {
        return mocks.get_input(&alias_node.0, &alias_port.0).cloned();
    }
    None
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
        ExecutionMode::Simulate(config) => ExecutionMode::Simulate(SimConfig {
            boundary_mocks: remap_input_mocks(&config.boundary_mocks, input_remaps),
            ..config
        }),
        other => other,
    }
}

/// Execute a flat (fully lowered) DAG.
///
/// When the observer requires sequential execution (e.g. `CiContext` for proper
/// group nesting), routes to [`execute_flat_sequential`]. Otherwise uses the
/// parallel executor.
#[allow(clippy::too_many_arguments)]
fn execute_flat<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    boundaries: &BoundaryInfo,
    mode: &ExecutionMode,
    observer: Option<&mut dyn ProgressObserver>,
    input_mocks: Option<&BoundaryMocks>,
    loops: &[LoopInfo<T>],
    log_detail: LogDetailLevel,
    strictness: DryRunStrictness,
) -> Result<ExecutionLog, ExecError> {
    if matches!(mode, ExecutionMode::DryRun(_) | ExecutionMode::Simulate(_)) {
        validate_node_kinds_for_interception(dag)?;
    }
    let sequential = observer.as_ref().is_some_and(|o| o.requires_sequential());
    if sequential {
        execute_flat_sequential(
            dag,
            boundaries,
            mode,
            observer.unwrap(),
            input_mocks,
            loops,
            log_detail,
            strictness,
        )
    } else {
        execute_flat_parallel(
            dag,
            boundaries,
            mode,
            observer,
            input_mocks,
            loops,
            log_detail,
            strictness,
        )
    }
}

/// Execute a flat DAG sequentially with a unified observer.
///
/// Used when the observer requires sequential execution (e.g. `CiContext`
/// needs proper group nesting). All CI-specific behaviors (groups, annotations,
/// secret masking, boundary output) are handled through the observer trait.
#[allow(clippy::too_many_arguments)]
fn execute_flat_sequential<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    boundaries: &BoundaryInfo,
    mode: &ExecutionMode,
    observer: &mut dyn ProgressObserver,
    input_mocks: Option<&BoundaryMocks>,
    loops: &[LoopInfo<T>],
    log_detail: LogDetailLevel,
    strictness: DryRunStrictness,
) -> Result<ExecutionLog, ExecError> {
    let file_guard_enabled = runtime_file_guard_enabled();
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

    // Build snapshot and fire on_dag_start
    let dag_start = Instant::now();
    let snapshot = DagSnapshot::from_dag(dag, &order, boundaries);
    observer.on_dag_start(&snapshot);

    for node_id in &order {
        let node = node_map
            .get(node_id.0.as_str())
            .ok_or_else(|| ExecError::new(format!("node '{}' not found", node_id.0)))?;

        // Gather inputs from upstream edges.
        //
        // The merge strategy is determined by the TARGET port's cardinality:
        // - List cardinality (`is_list()`): collect values from all edges into a list
        // - Scalar cardinality: take one value (conditional merge for branches)
        let mut inputs: HashMap<String, Value> = HashMap::new();
        let mut list_buckets: HashMap<String, Vec<(usize, Vec<Value>)>> = HashMap::new();
        let mut scalar_sources: HashMap<String, String> = HashMap::new();
        let applied_coercions: Vec<AppliedCoercion> = Vec::new();

        let list_ports: HashSet<&str> = node
            .inputs
            .iter()
            .filter(|p| p.cardinality.is_list())
            .map(|p| p.name.0.as_str())
            .collect();

        if let Some(edges) = edges_by_to_node.get(node_id) {
            for &edge in edges {
                if !edge.kind.carries_data() {
                    continue;
                }
                if let Some(upstream) = node_outputs.get(&edge.from_node.0) {
                    if let Some(val) = upstream.get(&edge.from_port.0) {
                        if list_ports.contains(edge.to_port.0.as_str()) {
                            let from_cardinality = dag
                                .get_node(&edge.from_node)
                                .and_then(|n| n.outputs.iter().find(|p| p.name == edge.from_port))
                                .map(|p| p.cardinality)
                                .unwrap_or(Cardinality::ONE);

                            if let Some(elements) = collect_fan_in(val, from_cardinality) {
                                let bucket =
                                    list_buckets.entry(edge.to_port.0.clone()).or_default();
                                bucket.push((edge.index, elements));
                            }
                        } else if scalar_sources.contains_key(&edge.to_port.0) {
                            // Conditional merge: multiple upstream edges to a scalar port.
                            // In conditional branches, only one branch produces
                            // a real value; others produce Skipped. Take the
                            // first non-Skipped value.
                            if !matches!(val, Value::Skipped) {
                                if let Some(existing) = inputs.get(&edge.to_port.0) {
                                    if !matches!(existing, Value::Skipped) {
                                        return Err(ExecError::new(format!(
                                            "conditional merge error at node '{}' port '{}': \
                                             multiple non-Skipped values (from '{}' and previous source '{}')",
                                            node_id.0, edge.to_port.0,
                                            edge.from_node.0,
                                            scalar_sources.get(&edge.to_port.0).map(|s| s.as_str()).unwrap_or("unknown"),
                                        )));
                                    }
                                }
                                inputs.insert(edge.to_port.0.clone(), val.clone());
                                scalar_sources.insert(
                                    edge.to_port.0.clone(),
                                    format!("{}.{}", edge.from_node.0, edge.from_port.0),
                                );
                            }
                        } else {
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

        // Merge collected list-port values as Value::List, sorted by edge index
        for (port_name, mut groups) in list_buckets {
            groups.sort_by_key(|(idx, _)| *idx);
            let values: Vec<Value> = groups.into_iter().flat_map(|(_, elems)| elems).collect();
            inputs.insert(port_name, Value::List(values));
        }

        // Inject input mocks for dangling input ports (DAG entry points).
        if let Some(mocks) = input_mocks {
            for port in &node.inputs {
                if !inputs.contains_key(&port.name.0) {
                    if let Some(mock_value) =
                        resolve_mock_input(mocks, &node.id, &port.name, node.input_alias.as_ref())
                    {
                        inputs.insert(port.name.0.clone(), mock_value);
                    }
                }
            }
        }

        if let ExecutionMode::DryRun(ref mocks)
        | ExecutionMode::Simulate(SimConfig {
            boundary_mocks: ref mocks,
            ..
        }) = mode
        {
            for port in &node.inputs {
                if !inputs.contains_key(&port.name.0) {
                    if let Some(mock_value) =
                        resolve_mock_input(mocks, &node.id, &port.name, node.input_alias.as_ref())
                    {
                        inputs.insert(port.name.0.clone(), mock_value);
                    }
                }
            }
        }

        // Default list-cardinality inputs to empty when still missing,
        // but only if the cardinality allows empty (min == 0).
        // Non-empty list ports ([1,∞)) must not silently receive [].
        for port in &node.inputs {
            if port.cardinality.is_list()
                && port.cardinality.allows_empty()
                && !inputs.contains_key(&port.name.0)
            {
                inputs.insert(port.name.0.clone(), Value::List(vec![]));
            }
        }

        // S32: Strict-mode check — fail if any required input port has no value.
        if strictness == DryRunStrictness::Strict {
            check_missing_required_inputs(node, &inputs)?;
        }

        // Capture the final input map for execution logs before ownership can move.
        let captured_inputs = capture_log_inputs_for_node(node, &inputs, log_detail);

        // Check guards BEFORE emitting on_node_start — skipped nodes never
        // enter the "running" state.
        let skip = should_skip_node(node, &inputs);

        // Track node timing for summary computation (set in the else branch).
        let mut node_elapsed = Duration::ZERO;

        let (outputs, was_intercepted) = if skip {
            // Node is skipped — all outputs become Skipped
            let outputs: HashMap<String, Value> = node
                .outputs
                .iter()
                .map(|p| (p.name.0.clone(), Value::Skipped))
                .collect();
            observer.on_node_skipped(node_id);
            (outputs, false)
        } else {
            // Notify observer that node is starting (CiContext opens CI group here)
            let node_start = Instant::now();
            observer.on_node_start(node_id);

            let should_intercept = should_intercept_for_mode(node, mode);
            // Allow input_mocks output mocks (set_value) to intercept only in
            // DryRun/Simulate. Real mode must never silently mask execution.
            let input_mock_intercept = !should_intercept
                && matches!(mode, ExecutionMode::DryRun(_) | ExecutionMode::Simulate(_))
                && input_mocks
                    .map(|m| has_full_mock_for_node(node, m))
                    .unwrap_or(false);

            if should_intercept {
                // Intercept: use mock values for boundary outputs
                let mocks = match mode {
                    ExecutionMode::DryRun(ref m) => m,
                    ExecutionMode::Simulate(ref config) => &config.boundary_mocks,
                    _ => unreachable!(),
                };

                let outputs = mock_intercept_outputs(node, mocks)?;
                node_elapsed = node_start.elapsed();
                (outputs, true)
            } else if input_mock_intercept {
                // Intercept: use output mocks from the input_mocks parameter.
                let outputs = mock_intercept_outputs(node, input_mocks.unwrap())?;
                node_elapsed = node_start.elapsed();
                (outputs, true)
            } else {
                // Execute normally
                match &node.body {
                    NodeBody::Opaque(op) => {
                        // Snapshot inputs for failure diagnostics
                        let saved_inputs = inputs.clone();
                        match op.execute(inputs) {
                            Ok(outputs) => {
                                node_elapsed = node_start.elapsed();
                                (outputs, false)
                            }
                            Err(e) => {
                                // Structural: automatically annotate with node trace
                                let e = e.with_layer(node_trace_layer(node_id, node));
                                // Failure diagnostics and error annotation happen inside
                                // the CI group, then the group is closed by on_node_failed.
                                observer.on_failure_diagnostics(node_id, &saved_inputs);
                                observer.on_node_failed(node_id, &e);
                                observer.on_dag_complete(dag_start.elapsed());
                                return Err(e);
                            }
                        }
                    }
                    NodeBody::SubDag(..) => {
                        let err = ExecError::new(format!(
                            "node '{}' is a SubDag — DAG must be lowered before execution",
                            node_id.0
                        ))
                        .with_layer(node_trace_layer(node_id, node));
                        observer.on_node_failed(node_id, &err);
                        observer.on_dag_complete(dag_start.elapsed());
                        return Err(err);
                    }
                }
            }
        };

        if !skip && !was_intercepted {
            if let Err(e) = enforce_runtime_file_guard(node, &outputs, file_guard_enabled) {
                let e = e.with_layer(node_trace_layer(node_id, node));
                observer.on_node_failed(node_id, &e);
                observer.on_dag_complete(dag_start.elapsed());
                return Err(e);
            }
        }

        // Mask any secret values so CI runners redact them from all output.
        // This happens inside the CI group (before on_node_complete closes it).
        #[allow(clippy::disallowed_methods)] // Approved: CI secret masking at transport boundary
        for value in outputs.values() {
            if let Value::Secret(s) = value {
                observer.on_secret_output(node_id, s.expose_plaintext_for_transport());
            }
        }

        node_outputs.insert(node_id.0.clone(), outputs.clone());
        let entry = LogEntry {
            node_id: node_id.0.clone(),
            inputs: captured_inputs,
            outputs,
            was_intercepted,
            coercions_applied: applied_coercions,
        };

        // Boundary output via observer (appears inside CI group).
        if boundaries.is_boundary_node(node_id) {
            observer.on_boundary_output(node_id, &entry);
        }
        entries.push(entry);

        // Close the CI group by notifying completion/interception.
        // This is deferred from the execution block above so that secret
        // masking and boundary output appear inside the CI group.
        if !skip {
            let summary = OutputSummary::from_outputs(&node_outputs[&node_id.0], node_elapsed);
            if was_intercepted {
                observer.on_node_intercepted(node_id, summary);
            } else {
                observer.on_node_complete(node_id, summary);
            }
        }

        // Loop body execution: if this node is a loop unpack, execute the body
        // template once per element and replace the element output with results.
        if let Some(loop_info) = loops.iter().find(|l| l.unpack_id == *node_id) {
            let body_entries =
                execute_loop_body(loop_info, &node_outputs, mode, log_detail, strictness).map_err(
                    |e| {
                        // Annotate loop body errors with the unpack node as context
                        e.with_layer(node_trace_layer(node_id, node))
                    },
                )?;

            let results: Vec<Value> = body_entries
                .iter()
                .filter_map(|e| e.outputs.get("result").cloned())
                .collect();

            if let Some(unpack_out) = node_outputs.get_mut(&loop_info.unpack_id.0) {
                unpack_out.insert(loop_info.element_port.clone(), Value::List(results));
            }

            entries.extend(body_entries);
        }
    }

    // Notify observer of successful DAG completion
    observer.on_dag_complete(dag_start.elapsed());

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

#[derive(Debug, Clone, Default)]
struct ActiveResourceLock {
    readers: usize,
    writer: bool,
    exclusive: bool,
}

/// Check if two normalized resource IDs conflict for admission control.
///
/// Exact ID equality always conflicts.  Additionally, coarse `file`
/// conflicts with any specific `file:<path>` lock (R2 semantics).
/// Full glob-aware conflict detection is deferred (see `TODO/backlog.md`).
fn resource_ids_conflict(required_id: &str, active_id: &str) -> bool {
    if required_id == active_id {
        return true;
    }

    // Coarse `file` conflicts with any specific `file:<path>` lock.
    let required_is_file = required_id == "file" || required_id.starts_with("file:");
    let active_is_file = active_id == "file" || active_id.starts_with("file:");
    required_is_file && active_is_file && (required_id == "file" || active_id == "file")
}

fn active_lock_allows_mode(lock: &ActiveResourceLock, mode: AccessMode) -> bool {
    match mode {
        AccessMode::Read => !lock.writer && !lock.exclusive,
        AccessMode::Write | AccessMode::Exclusive => {
            lock.readers == 0 && !lock.writer && !lock.exclusive
        }
    }
}

fn derive_node_resource_requirements<T>(
    dag: &Dag<T>,
) -> HashMap<NodeId, Vec<(String, AccessMode)>> {
    dag.nodes
        .iter()
        .map(|node| {
            let requirements = node
                .inputs
                .iter()
                .filter_map(|port| {
                    port.resource_access
                        .map(|mode| (normalize_resource_id(&port.name.0), mode))
                })
                .collect::<Vec<_>>();
            (node.id.clone(), requirements)
        })
        .collect()
}

fn node_requirements_can_acquire(
    requirements: &[(String, AccessMode)],
    active: &HashMap<String, ActiveResourceLock>,
) -> bool {
    requirements.iter().all(|(resource_id, mode)| {
        active
            .iter()
            .filter(|(active_id, _)| resource_ids_conflict(resource_id, active_id))
            .all(|(_, lock)| active_lock_allows_mode(lock, *mode))
    })
}

fn acquire_node_requirements(
    requirements: &[(String, AccessMode)],
    active: &mut HashMap<String, ActiveResourceLock>,
) {
    for (resource_id, mode) in requirements {
        let entry = active.entry(resource_id.clone()).or_default();
        match mode {
            AccessMode::Read => entry.readers += 1,
            AccessMode::Write => entry.writer = true,
            AccessMode::Exclusive => entry.exclusive = true,
        }
    }
}

fn release_node_requirements(
    requirements: &[(String, AccessMode)],
    active: &mut HashMap<String, ActiveResourceLock>,
) {
    let mut touched = HashSet::new();
    for (resource_id, mode) in requirements {
        if let Some(entry) = active.get_mut(resource_id) {
            match mode {
                AccessMode::Read => entry.readers = entry.readers.saturating_sub(1),
                AccessMode::Write => entry.writer = false,
                AccessMode::Exclusive => entry.exclusive = false,
            }
            touched.insert(resource_id.clone());
        }
    }
    active.retain(|resource_id, entry| {
        if !touched.contains(resource_id) {
            return true;
        }
        entry.readers > 0 || entry.writer || entry.exclusive
    });
}

/// Parse optional runtime file guard toggle from `GUNBC_RESOURCE_FILE_GUARD`.
///
/// Enabled values: `1`, `true`, `yes`, `on` (case-insensitive).
/// Disabled values (or unset): everything else.
fn runtime_file_guard_enabled() -> bool {
    std::env::var("GUNBC_RESOURCE_FILE_GUARD")
        .ok()
        .map(|raw| {
            matches!(
                raw.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn file_op_requires_write_declaration(op: FileOp) -> bool {
    matches!(
        op,
        FileOp::Write | FileOp::Append | FileOp::Delete | FileOp::CreateDir
    )
}

fn normalize_file_guard_path(path: &str) -> String {
    let mut normalized = path.replace('\\', "/");
    while normalized.starts_with("./") {
        normalized.drain(..2);
    }
    if normalized == "." {
        normalized.clear();
    }
    normalized
}

/// Match a file resource pattern against a concrete path.
///
/// Since wildcard resource IDs are normalized to coarse `file` at
/// `Port::resource()` construction time (R2), the `pattern` argument
/// here should only ever be `"*"` (from coarse `res:file`) or a
/// literal path (from specific `res:file:<path>`).  The glob-style
/// branches below are retained as defense-in-depth; full glob-aware
/// resource admission is deferred (see `TODO/backlog.md`).
fn file_resource_pattern_matches_path(pattern: &str, path: &str) -> bool {
    let normalized_pattern = normalize_file_guard_path(pattern);
    let normalized_path = normalize_file_guard_path(path);

    if normalized_pattern == "*" || normalized_pattern.is_empty() {
        return true;
    }

    if !normalized_pattern.contains('*') {
        return normalized_pattern == normalized_path;
    }

    if let Some(prefix) = normalized_pattern.strip_suffix('*') {
        return normalized_path.starts_with(prefix);
    }
    if let Some(suffix) = normalized_pattern.strip_prefix('*') {
        return normalized_path.ends_with(suffix);
    }
    if let Some((prefix, suffix)) = normalized_pattern.split_once('*') {
        return normalized_path.starts_with(prefix)
            && normalized_path.ends_with(suffix)
            && normalized_path.len() >= prefix.len() + suffix.len();
    }

    false
}

fn write_file_pattern_for_port_name(
    port_name: &str,
    access_mode: Option<AccessMode>,
) -> Option<&str> {
    if !matches!(
        access_mode,
        Some(AccessMode::Write) | Some(AccessMode::Exclusive)
    ) {
        return None;
    }

    if port_name == RESOURCE_FILE {
        return Some("*");
    }

    if let Some(pattern) = port_name.strip_prefix(RESOURCE_FILE_PREFIX) {
        return Some(if pattern.is_empty() { "*" } else { pattern });
    }

    None
}

fn collect_file_write_ops_from_value(value: &Value, writes: &mut Vec<(FileOp, String)>) {
    match value {
        Value::Response(TransportResponse::File(resp))
            if file_op_requires_write_declaration(resp.operation) =>
        {
            writes.push((resp.operation, normalize_file_guard_path(&resp.path)));
        }
        Value::List(items) | Value::Set(items) => {
            for item in items {
                collect_file_write_ops_from_value(item, writes);
            }
        }
        Value::Map(map) => {
            for item in map.values() {
                collect_file_write_ops_from_value(item, writes);
            }
        }
        _ => {}
    }
}

fn node_has_matching_write_file_input<T>(node: &Node<T>, path: &str) -> bool {
    node.inputs.iter().any(|port| {
        write_file_pattern_for_port_name(port.name.0.as_str(), port.resource_access)
            .is_some_and(|pattern| file_resource_pattern_matches_path(pattern, path))
    })
}

fn node_declared_write_file_inputs<T>(node: &Node<T>) -> Vec<String> {
    node.inputs
        .iter()
        .filter_map(|port| {
            write_file_pattern_for_port_name(port.name.0.as_str(), port.resource_access)
                .map(|_| port.name.0.clone())
        })
        .collect()
}

/// Optional runtime guard that validates file writes against declared res:file inputs.
fn enforce_runtime_file_guard<T>(
    node: &Node<T>,
    outputs: &HashMap<String, Value>,
    enabled: bool,
) -> Result<(), ExecError> {
    if !enabled {
        return Ok(());
    }

    let mut writes = Vec::new();
    for value in outputs.values() {
        collect_file_write_ops_from_value(value, &mut writes);
    }

    for (op, path) in writes {
        if node_has_matching_write_file_input(node, &path) {
            continue;
        }

        let declared = node_declared_write_file_inputs(node);
        let declared_text = if declared.is_empty() {
            "none".to_string()
        } else {
            declared.join(", ")
        };

        return Err(ExecError::new(format!(
            "runtime file guard: node '{}' emitted file {:?} on '{}' without matching write \
             resource input (declared write inputs: {}). declare `res:file:{}` or coarse \
             `res:file` with AccessMode::Write/Exclusive",
            node.id.0, op, path, declared_text, path
        )));
    }

    Ok(())
}

fn should_intercept_for_mode<T>(node: &Node<T>, mode: &ExecutionMode) -> bool {
    let has_full_mock = match mode {
        ExecutionMode::DryRun(mocks) => has_full_mock_for_node(node, mocks),
        ExecutionMode::Simulate(config) => has_full_mock_for_node(node, &config.boundary_mocks),
        ExecutionMode::Real => false,
    };

    (should_intercept_by_kind(node) || has_full_mock)
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
) -> Result<(HashMap<String, Value>, Vec<AppliedCoercion>), ExecError> {
    // Gather inputs from upstream edges.
    // Merge strategy determined by target port cardinality (see main execute loop).
    let mut inputs: HashMap<String, Value> = HashMap::new();
    let mut list_buckets: HashMap<String, Vec<(usize, Vec<Value>)>> = HashMap::new();
    let mut scalar_sources: HashMap<String, String> = HashMap::new();
    let applied_coercions: Vec<AppliedCoercion> = Vec::new();

    let list_ports: HashSet<&str> = node
        .inputs
        .iter()
        .filter(|p| p.cardinality.is_list())
        .map(|p| p.name.0.as_str())
        .collect();

    if let Some(edges) = edges_by_to_node.get(node_id) {
        for &edge in edges {
            if !edge.kind.carries_data() {
                continue;
            }
            if let Some(upstream) = node_outputs.get(&edge.from_node.0) {
                if let Some(val) = upstream.get(&edge.from_port.0) {
                    if list_ports.contains(edge.to_port.0.as_str()) {
                        let from_cardinality = dag
                            .get_node(&edge.from_node)
                            .and_then(|n| n.outputs.iter().find(|p| p.name == edge.from_port))
                            .map(|p| p.cardinality)
                            .unwrap_or(Cardinality::ONE);

                        if let Some(elements) = collect_fan_in(val, from_cardinality) {
                            let bucket = list_buckets.entry(edge.to_port.0.clone()).or_default();
                            bucket.push((edge.index, elements));
                        }
                    } else if scalar_sources.contains_key(&edge.to_port.0) {
                        if !matches!(val, Value::Skipped) {
                            if let Some(existing) = inputs.get(&edge.to_port.0) {
                                if !matches!(existing, Value::Skipped) {
                                    return Err(ExecError::new(format!(
                                        "conditional merge error at node '{}' port '{}': \
                                         multiple non-Skipped values (from '{}' and previous source '{}')",
                                        node_id.0, edge.to_port.0,
                                        edge.from_node.0,
                                        scalar_sources.get(&edge.to_port.0).map(|s| s.as_str()).unwrap_or("unknown"),
                                    )));
                                }
                            }
                            inputs.insert(edge.to_port.0.clone(), val.clone());
                            scalar_sources.insert(
                                edge.to_port.0.clone(),
                                format!("{}.{}", edge.from_node.0, edge.from_port.0),
                            );
                        }
                    } else {
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

    for (port_name, mut groups) in list_buckets {
        groups.sort_by_key(|(idx, _)| *idx);
        let values: Vec<Value> = groups.into_iter().flat_map(|(_, elems)| elems).collect();
        inputs.insert(port_name, Value::List(values));
    }

    if let Some(mocks) = input_mocks {
        for port in &node.inputs {
            if !inputs.contains_key(&port.name.0) {
                if let Some(mock_value) =
                    resolve_mock_input(mocks, &node.id, &port.name, node.input_alias.as_ref())
                {
                    inputs.insert(port.name.0.clone(), mock_value);
                }
            }
        }
    }

    if let ExecutionMode::DryRun(mocks)
    | ExecutionMode::Simulate(SimConfig {
        boundary_mocks: mocks,
        ..
    }) = mode
    {
        for port in &node.inputs {
            if !inputs.contains_key(&port.name.0) {
                if let Some(mock_value) =
                    resolve_mock_input(mocks, &node.id, &port.name, node.input_alias.as_ref())
                {
                    inputs.insert(port.name.0.clone(), mock_value);
                }
            }
        }
    }

    for port in &node.inputs {
        if port.cardinality.is_list()
            && port.cardinality.allows_empty()
            && !inputs.contains_key(&port.name.0)
        {
            inputs.insert(port.name.0.clone(), Value::List(vec![]));
        }
    }

    Ok((inputs, applied_coercions))
}

fn capture_log_inputs_for_node<T>(
    node: &Node<T>,
    inputs: &HashMap<String, Value>,
    root_log_detail: LogDetailLevel,
) -> Option<HashMap<String, Value>> {
    let node_log_detail = node.log_detail.unwrap_or(root_log_detail);

    if node.inputs.is_empty() {
        return node_log_detail
            .includes_inputs()
            .then(HashMap::<String, Value>::new);
    }

    let mut captured = HashMap::new();
    for port in &node.inputs {
        let port_log_detail = port.log_detail.unwrap_or(node_log_detail);
        if !port_log_detail.includes_inputs() {
            continue;
        }
        if let Some(value) = inputs.get(&port.name.0) {
            captured.insert(port.name.0.clone(), value.clone());
        }
    }

    if captured.is_empty() {
        None
    } else {
        Some(captured)
    }
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

#[allow(clippy::too_many_arguments)]
fn finalize_node_parallel<T: Executable + Clone + Send>(
    node_id: &NodeId,
    inputs: Option<HashMap<String, Value>>,
    outputs: HashMap<String, Value>,
    was_intercepted: bool,
    coercions_applied: Vec<AppliedCoercion>,
    mode: &ExecutionMode,
    log_detail: LogDetailLevel,
    strictness: DryRunStrictness,
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
        inputs,
        outputs,
        was_intercepted,
        coercions_applied,
    });

    if let Some(loop_info) = state.loops_by_unpack.get(node_id) {
        let body_entries =
            execute_loop_body(loop_info, &state.node_outputs, mode, log_detail, strictness)
                .map_err(|e| {
                    // Annotate loop body errors with unpack node context (Pure role
                    // since we don't have the full Node<T> here — the node_id is the
                    // important part for trace rendering).
                    e.with_layer(ErrorLayer::NodeTrace(NodeTraceLayer {
                        node_id: node_id.0.clone(),
                        role: NodeRole::Pure,
                    }))
                })?;

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

#[allow(clippy::too_many_arguments)]
fn execute_flat_parallel<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    boundaries: &BoundaryInfo,
    mode: &ExecutionMode,
    observer: Option<&mut dyn ProgressObserver>,
    input_mocks: Option<&BoundaryMocks>,
    loops: &[LoopInfo<T>],
    log_detail: LogDetailLevel,
    strictness: DryRunStrictness,
) -> Result<ExecutionLog, ExecError> {
    struct NodeExecutionResult {
        node_id: NodeId,
        started_at: Instant,
        inputs: Option<HashMap<String, Value>>,
        coercions_applied: Vec<AppliedCoercion>,
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
    let file_guard_enabled = runtime_file_guard_enabled();
    let node_resource_requirements = derive_node_resource_requirements(dag);
    let mut active_resource_locks: HashMap<String, ActiveResourceLock> = HashMap::new();
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
            let mut ready_idx = 0usize;
            while ready_idx < state.ready.len() && in_flight < max_concurrency {
                let node_id = state.ready[ready_idx].clone();
                let requirements = node_resource_requirements
                    .get(&node_id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                if !node_requirements_can_acquire(requirements, &active_resource_locks) {
                    ready_idx += 1;
                    continue;
                }
                state.ready.remove(ready_idx);
                acquire_node_requirements(requirements, &mut active_resource_locks);
                let node = node_map
                    .get(node_id.0.as_str())
                    .ok_or_else(|| ExecError::new(format!("node '{}' not found", node_id.0)))?;

                let (inputs, node_coercions) = build_node_inputs(
                    dag,
                    node,
                    &node_id,
                    &edges_by_to_node,
                    &state.node_outputs,
                    mode,
                    input_mocks,
                )?;

                // S32: Strict-mode check — fail if any required input port has no value.
                if strictness == DryRunStrictness::Strict {
                    check_missing_required_inputs(node, &inputs)?;
                }

                if should_skip_node(node, &inputs) {
                    let captured_inputs = capture_log_inputs_for_node(node, &inputs, log_detail);
                    let outputs: HashMap<String, Value> = node
                        .outputs
                        .iter()
                        .map(|port| (port.name.0.clone(), Value::Skipped))
                        .collect();
                    if let Some(ref mut o) = obs {
                        o.on_node_skipped(&node_id);
                    }
                    finalize_node_parallel(
                        &node_id,
                        captured_inputs,
                        outputs,
                        false,
                        node_coercions,
                        mode,
                        log_detail,
                        strictness,
                        &mut state,
                    )?;
                    release_node_requirements(requirements, &mut active_resource_locks);
                    continue;
                }

                let node_start = Instant::now();
                if let Some(ref mut o) = obs {
                    o.on_node_start(&node_id);
                }

                let input_mock_intercept = !should_intercept_for_mode(node, mode)
                    && matches!(mode, ExecutionMode::DryRun(_) | ExecutionMode::Simulate(_))
                    && input_mocks
                        .map(|m| has_full_mock_for_node(node, m))
                        .unwrap_or(false);

                if should_intercept_for_mode(node, mode) || input_mock_intercept {
                    let captured_inputs = capture_log_inputs_for_node(node, &inputs, log_detail);
                    let mocks = if input_mock_intercept {
                        input_mocks.unwrap()
                    } else {
                        match mode {
                            ExecutionMode::DryRun(mocks) => mocks,
                            ExecutionMode::Simulate(config) => &config.boundary_mocks,
                            ExecutionMode::Real => unreachable!(),
                        }
                    };
                    let outputs = match mock_intercept_outputs(node, mocks) {
                        Ok(outputs) => outputs,
                        Err(e) => {
                            let e = e.with_layer(node_trace_layer(&node_id, node));
                            if let Some(ref mut o) = obs {
                                o.on_node_failed(&node_id, &e);
                            }
                            release_node_requirements(requirements, &mut active_resource_locks);
                            return Err(e);
                        }
                    };
                    if let Some(ref mut o) = obs {
                        let summary = OutputSummary::from_outputs(&outputs, node_start.elapsed());
                        o.on_node_intercepted(&node_id, summary);
                    }
                    finalize_node_parallel(
                        &node_id,
                        captured_inputs,
                        outputs,
                        true,
                        node_coercions,
                        mode,
                        log_detail,
                        strictness,
                        &mut state,
                    )?;
                    release_node_requirements(requirements, &mut active_resource_locks);
                    continue;
                }

                match &node.body {
                    NodeBody::Opaque(op) => {
                        let op = op.clone();
                        let node_id_clone = node_id.clone();
                        let tx = tx.clone();
                        let captured_inputs =
                            capture_log_inputs_for_node(node, &inputs, log_detail);
                        scope.spawn(move || {
                            let result = op.execute(inputs);
                            let _ = tx.send(NodeExecutionResult {
                                node_id: node_id_clone,
                                started_at: node_start,
                                inputs: captured_inputs,
                                coercions_applied: node_coercions,
                                result,
                            });
                        });
                        in_flight += 1;
                    }
                    NodeBody::SubDag(..) => {
                        let err = ExecError::new(format!(
                            "node '{}' is a SubDag — DAG must be lowered before execution",
                            node_id.0
                        ))
                        .with_layer(node_trace_layer(&node_id, node));
                        if let Some(ref mut o) = obs {
                            o.on_node_failed(&node_id, &err);
                        }
                        release_node_requirements(requirements, &mut active_resource_locks);
                        return Err(err);
                    }
                }
            }

            if state.completed >= order.len() {
                break;
            }

            if in_flight == 0 {
                if !state.ready.is_empty() {
                    let blocked = state
                        .ready
                        .iter()
                        .map(|id| id.0.clone())
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(ExecError::new(format!(
                        "execution stalled: ready nodes blocked by resource admission control ({blocked})"
                    )));
                }
                return Err(ExecError::new(
                    "execution stalled: no ready nodes and no running tasks",
                ));
            }

            let completed_node = rx
                .recv()
                .map_err(|_| ExecError::new("execution worker channel closed unexpectedly"))?;
            in_flight = in_flight.saturating_sub(1);
            let requirements = node_resource_requirements
                .get(&completed_node.node_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            release_node_requirements(requirements, &mut active_resource_locks);
            match completed_node.result {
                Ok(outputs) => {
                    let node =
                        node_map
                            .get(completed_node.node_id.0.as_str())
                            .ok_or_else(|| {
                                ExecError::new(format!(
                                    "node '{}' not found",
                                    completed_node.node_id.0
                                ))
                            })?;
                    if let Err(e) = enforce_runtime_file_guard(node, &outputs, file_guard_enabled) {
                        let e = e.with_layer(node_trace_layer(&completed_node.node_id, node));
                        if let Some(ref mut o) = obs {
                            o.on_node_failed(&completed_node.node_id, &e);
                        }
                        return Err(e);
                    }

                    if let Some(ref mut o) = obs {
                        let summary = OutputSummary::from_outputs(
                            &outputs,
                            completed_node.started_at.elapsed(),
                        );
                        o.on_node_complete(&completed_node.node_id, summary);
                    }
                    finalize_node_parallel(
                        &completed_node.node_id,
                        completed_node.inputs,
                        outputs,
                        false,
                        completed_node.coercions_applied,
                        mode,
                        log_detail,
                        strictness,
                        &mut state,
                    )?
                }
                Err(e) => {
                    // Structural: annotate with node trace if node is found
                    let e = if let Some(node) = node_map.get(completed_node.node_id.0.as_str()) {
                        e.with_layer(node_trace_layer(&completed_node.node_id, node))
                    } else {
                        // Fallback: use Pure role if node not found (shouldn't happen)
                        e.with_layer(ErrorLayer::NodeTrace(NodeTraceLayer {
                            node_id: completed_node.node_id.0.clone(),
                            role: NodeRole::Pure,
                        }))
                    };
                    if let Some(ref mut o) = obs {
                        o.on_node_failed(&completed_node.node_id, &e);
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
    use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};

    let mut augmented = existing.clone();
    for node in &body_dag.nodes {
        if node.kind == NodeKind::TransportExecute {
            // Only add default mocks for outputs that don't already have one
            for port in &node.outputs {
                if !existing.has_mock(&node.id, &port.name) {
                    // Choose a type-appropriate default based on resource inputs:
                    // nodes with FilesystemHandle inputs are file transports.
                    let is_file_transport = node
                        .inputs
                        .iter()
                        .any(|p| p.type_id.0 == "FilesystemHandle");
                    let default_response = if is_file_transport {
                        Value::Response(TransportResponse::File(FileResponse {
                            path: String::new(),
                            operation: FileOp::Read,
                            success: true,
                            content: Some(String::new()),
                            bytes: None,
                            exists: None,
                            error: None,
                        }))
                    } else {
                        Value::Response(TransportResponse::Shell(ShellResponse::ok("")))
                    };
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
    log_detail: LogDetailLevel,
    strictness: DryRunStrictness,
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
            Some(&iter_mocks),
            &lowered_body.loops,
            log_detail,
            strictness,
        )?;

        // Prefix iteration entries for unique identification in the log
        let prefix = format!("{}/body_{}", loop_info.unpack_id.0, i);
        for entry in body_log.entries {
            all_entries.push(LogEntry {
                node_id: format!("{}/{}", prefix, entry.node_id),
                inputs: entry.inputs,
                outputs: entry.outputs,
                was_intercepted: entry.was_intercepted,
                coercions_applied: entry.coercions_applied,
            });
        }
    }

    Ok(all_entries)
}

/// S32: Fail if any required input port has no value after all resolution.
///
/// A port is "required" when:
/// 1. Its cardinality min > 0 (i.e., `!allows_empty()`), AND
/// 2. It is not an internal wiring port (`__deps`, `__out:*`), AND
/// 3. It does not have a guard (guarded ports are handled by skip logic).
///
/// Resource and tool ports ARE checked — they must be wired or mocked.
fn check_missing_required_inputs<T>(
    node: &Node<T>,
    inputs: &HashMap<String, Value>,
) -> Result<(), ExecError> {
    let mut missing = Vec::new();
    for port in &node.inputs {
        // Skip ports that allow zero values (optional, empty-allowing lists).
        if port.cardinality.allows_empty() {
            continue;
        }
        // Skip internal wiring ports — they are infrastructure, not user data.
        if port.name.0.starts_with("__") {
            continue;
        }
        // Skip guarded ports — their absence causes node skip, not an error.
        if port.has_guard() {
            continue;
        }
        if !inputs.contains_key(&port.name.0) {
            missing.push(format!(
                "'{}' (type: {}, cardinality: {})",
                port.name.0, port.type_id.0, port.cardinality
            ));
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    Err(ExecError::new(format!(
        "strict mode: node '{}' is missing {} required input{}: {}. \
         Either wire the input via edges/mocks, or use ExecuteConfig {{ strictness: \
         DryRunStrictness::Lenient, .. }} for tests that intentionally omit inputs",
        node.id.0,
        missing.len(),
        if missing.len() == 1 { "" } else { "s" },
        missing.join(", "),
    )))
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

/// Check whether a node should be intercepted in DryRun/Simulate mode
/// based on its structural kind.
///
/// Only nodes with an explicit effectful `NodeKind` are intercepted.
/// Nodes with `kind: Pure` are **not** intercepted — callers must ensure
/// all effectful nodes have `kind` set before execution. See
/// [`validate_node_kinds_for_interception`] for the pre-flight check.
fn should_intercept_by_kind<T>(node: &Node<T>) -> bool {
    matches!(
        node.kind,
        NodeKind::TransportExecute
            | NodeKind::ToolEnvironment
            | NodeKind::ToolConsumer
            | NodeKind::ResourceEnvironment
            | NodeKind::ResourceAcquire
    )
}

/// Pre-flight check: error if any `Pure` node has effectful port patterns.
///
/// `looks_effectful_without_kind()` helper was removed (C18); keep the checks
/// inlined here so accidental `kind: Pure` regressions still fail closed.
fn validate_node_kinds_for_interception<T>(dag: &Dag<T>) -> Result<(), ExecError> {
    for node in &dag.nodes {
        if node.kind != NodeKind::Pure {
            continue;
        }
        for port in &node.inputs {
            if port.type_id.0 == "TransportRequest" {
                return Err(ExecError::new(format!(
                    "node '{}' has kind: Pure but has TransportRequest input",
                    node.id.0
                )));
            }
            if port.type_id.0 == "ToolHandle" {
                return Err(ExecError::new(format!(
                    "node '{}' has kind: Pure but has ToolHandle input",
                    node.id.0
                )));
            }
            if port.name.is_resource() {
                return Err(ExecError::new(format!(
                    "node '{}' has kind: Pure but has resource input '{}'",
                    node.id.0, port.name.0
                )));
            }
        }
        for port in &node.outputs {
            if port.type_id.0 == "ToolHandle" {
                return Err(ExecError::new(format!(
                    "node '{}' has kind: Pure but has ToolHandle output",
                    node.id.0
                )));
            }
            if matches!(
                port.type_id.0.as_str(),
                "FilesystemHandle" | "NetworkHandle" | "Timestamp" | "Credential" | "Platform"
            ) {
                return Err(ExecError::new(format!(
                    "node '{}' has kind: Pure but has resource-environment output '{}'",
                    node.id.0, port.type_id.0
                )));
            }
        }
    }
    Ok(())
}

/// Classify a node's structural role for error reporting.
fn classify_node_role<T>(node: &Node<T>) -> NodeRole {
    match node.kind {
        NodeKind::TransportExecute => NodeRole::TransportExecutor,
        NodeKind::ToolConsumer => NodeRole::ToolConsumer,
        NodeKind::ToolEnvironment
        | NodeKind::ResourceEnvironment
        | NodeKind::ResourceAcquire
        | NodeKind::ResourceRelease => NodeRole::ResourceProvider,
        _ => NodeRole::Pure,
    }
}

/// Build a [`NodeTraceLayer`] for a node — called automatically by the executor
/// on every node failure.
fn node_trace_layer<T>(node_id: &NodeId, node: &Node<T>) -> ErrorLayer {
    ErrorLayer::NodeTrace(NodeTraceLayer {
        node_id: node_id.0.clone(),
        role: classify_node_role(node),
    })
}

/// Build mock outputs for a tool environment node.
/// Build mock outputs for any intercepted node.
fn mock_intercept_outputs<T>(
    node: &Node<T>,
    mocks: &BoundaryMocks,
) -> Result<HashMap<String, Value>, ExecError> {
    let mut outputs = HashMap::new();
    let has_any_mock = node
        .outputs
        .iter()
        .any(|port| mocks.has_mock(&node.id, &port.name));

    for port in &node.outputs {
        if !mocks.has_mock(&node.id, &port.name) {
            if !has_any_mock {
                // No explicit mocks for this node at all — auto-skip.
                // This handles transport nodes from transitively imported
                // modules that the entrypoint doesn't actually exercise.
                outputs.insert(port.name.0.clone(), Value::Skipped);
                continue;
            }
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
        let (value, exhausted) = mock.next_value_with_status();
        if exhausted && mock.has_sequence() {
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
mod tests;
