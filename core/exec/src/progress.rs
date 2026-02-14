//! Progress observation and DAG execution state tracking.
//!
//! Provides:
//! - [`ProgressObserver`]: Trait for receiving execution events (formalizes CiContext pattern)
//! - [`DagSnapshot`]: Static topology snapshot for layout computation
//! - [`OutputSummary`]: Summarized node output (extracts from print_log_entry logic)
//! - [`DagProgress`]: Live state machine tracking execution progress
//!
//! # Architecture
//!
//! `ProgressObserver` is called by `execute_flat()` at the same hook points
//! where `CiContext` already fires (start_group, end_group, error). The trait
//! generalizes these ad-hoc callbacks into a formal interface.
//!
//! `DagProgress` implements `ProgressObserver` — it's a concrete observer that
//! maintains the "power flow" state machine (NodeState, EdgeState transitions).

use gunbc_ir::{Edge, NodeId, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// ProgressObserver trait
// ---------------------------------------------------------------------------

/// Trait for receiving execution progress events.
///
/// Formalizes the existing CiContext ad-hoc callback pattern into a trait.
/// Called by `execute_flat()` at the same hook points where CI context fires.
pub trait ProgressObserver: Send {
    /// Called before execution begins with a static snapshot of the DAG topology.
    fn on_dag_start(&mut self, snapshot: &DagSnapshot);

    /// Called when a node begins execution.
    fn on_node_start(&mut self, node_id: &NodeId);

    /// Called when a node completes successfully.
    fn on_node_complete(&mut self, node_id: &NodeId, summary: OutputSummary);

    /// Called when a node fails.
    fn on_node_failed(&mut self, node_id: &NodeId, error: &str);

    /// Called when a node is skipped (guard predicate false).
    fn on_node_skipped(&mut self, node_id: &NodeId);

    /// Called when a node is intercepted (DryRun mock).
    fn on_node_intercepted(&mut self, node_id: &NodeId, summary: OutputSummary);

    /// Called when DAG execution completes (success or after failure).
    fn on_dag_complete(&mut self, elapsed: Duration);

    /// Called when a node produces a secret value that should be masked in CI output.
    fn on_secret_output(&mut self, _node_id: &NodeId, _secret_value: &str) {}

    /// Called when a node fails, providing the inputs snapshot for diagnostics.
    fn on_failure_diagnostics(&mut self, _node_id: &NodeId, _inputs: &HashMap<String, Value>) {}

    /// Called when a boundary node produces output that should be displayed.
    fn on_boundary_output(&mut self, _node_id: &NodeId, _entry: &crate::execute::LogEntry) {}

    /// Whether this observer requires sequential (non-parallel) execution.
    ///
    /// Returns `true` for observers like `CiContext` that emit nested group
    /// commands requiring proper sequential ordering.
    fn requires_sequential(&self) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// DagSnapshot — reuses existing types
// ---------------------------------------------------------------------------

/// Static topology snapshot of a DAG, built once before execution.
///
/// Reuses `Edge` from core/ir directly. Node metadata comes from the
/// actual `Dag<T>` nodes; boundary info from `detect_boundaries()`.
#[derive(Debug, Clone)]
pub struct DagSnapshot {
    pub node_ids: Vec<NodeId>,
    pub edges: Vec<Edge>,
    pub topo_order: Vec<NodeId>,
    pub boundary_nodes: Vec<NodeId>,
    pub labels: HashMap<NodeId, String>,
    /// Stage groups derived from DAG structure (SubDag parents, transport triplets).
    /// When empty, rendering falls back to the ungrouped per-node view.
    pub groups: Vec<StageGroup>,
}

impl DagSnapshot {
    /// Build a snapshot from a flat (lowered) DAG.
    pub fn from_dag<T>(
        dag: &gunbc_ir::Dag<T>,
        topo_order: &[NodeId],
        boundaries: &gunbc_ir::BoundaryInfo,
    ) -> Self {
        let node_ids: Vec<NodeId> = dag.nodes.iter().map(|n| n.id.clone()).collect();
        let edges = dag.edges.clone();
        let boundary_nodes = boundaries.boundary_nodes.clone();
        let labels: HashMap<NodeId, String> = dag
            .nodes
            .iter()
            .map(|n| {
                // For SubDag children like "rev_list/prepare_rev_list",
                // strip the parent prefix to show just "prepare_rev_list".
                let label = n
                    .id
                    .0
                    .split('/')
                    .next_back()
                    .unwrap_or(&n.id.0)
                    .to_string();
                (n.id.clone(), label)
            })
            .collect();

        let groups = derive_stage_groups(&node_ids);

        Self {
            node_ids,
            edges,
            topo_order: topo_order.to_vec(),
            boundary_nodes,
            labels,
            groups,
        }
    }
}

// ---------------------------------------------------------------------------
// OutputSummary — extracts from print_log_entry logic
// ---------------------------------------------------------------------------

/// Summarized output from a completed node.
///
/// Extracts the summarization logic from `print_log_entry()` into a
/// structured type that observers can use without coupling to stdout.
#[derive(Debug, Clone)]
pub struct OutputSummary {
    pub fields: Vec<FieldSummary>,
    pub elapsed: Duration,
}

/// Summary of a single output field.
#[derive(Debug, Clone)]
pub struct FieldSummary {
    pub name: String,
    pub kind: FieldKind,
    pub preview: String,
}

/// Semantic categorization of a Value for display purposes.
///
/// Adds semantic meaning on top of Value's structural types
/// (e.g., detecting URLs within Str values).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Scalar,
    Number,
    Boolean,
    List(usize),
    Map(usize),
    Secret,
    Url,
    Empty,
}

impl OutputSummary {
    /// Build a summary from node outputs and elapsed time.
    pub fn from_outputs(outputs: &HashMap<String, Value>, elapsed: Duration) -> Self {
        let fields = outputs
            .iter()
            .filter_map(|(name, value)| {
                let (kind, preview) = match value {
                    Value::Unit => return None,
                    Value::Skipped => return None,
                    Value::Bool(b) => (FieldKind::Boolean, b.to_string()),
                    Value::Int(i) => (FieldKind::Number, i.to_string()),
                    Value::Str(s) => {
                        if s.starts_with("http://") || s.starts_with("https://") {
                            (FieldKind::Url, truncate_str(s, 80))
                        } else {
                            (FieldKind::Scalar, truncate_str(s, 80))
                        }
                    }
                    Value::List(l) => (FieldKind::List(l.len()), format!("[{} items]", l.len())),
                    Value::Map(m) => (FieldKind::Map(m.len()), format!("{{{} entries}}", m.len())),
                    Value::Json(_) => (FieldKind::Scalar, "<JSON>".to_string()),
                    Value::Secret(_) => (FieldKind::Secret, value.display_redacted()),
                    Value::Request(_) => (FieldKind::Scalar, "<Request>".to_string()),
                    Value::Response(_) => (FieldKind::Scalar, "<Response>".to_string()),
                    Value::Set(s) => (FieldKind::List(s.len()), format!("{{{} items}}", s.len())),
                };
                Some(FieldSummary {
                    name: name.clone(),
                    kind,
                    preview,
                })
            })
            .collect();

        Self { fields, elapsed }
    }
}

/// Truncate a string to at most `max` characters (char-boundary safe).
///
/// Uses `char_indices` to find the cut point, avoiding panics on multi-byte
/// UTF-8 codepoints.
fn truncate_str(s: &str, max: usize) -> String {
    match s.char_indices().nth(max) {
        Some((byte_idx, _)) => format!("{}...", &s[..byte_idx]),
        None => s.to_string(),
    }
}

// ---------------------------------------------------------------------------
// StageGroup — logical grouping of nodes for CI pipelines
// ---------------------------------------------------------------------------

/// A named group of DAG nodes representing a logical stage.
///
/// Groups are derived automatically from the lowered DAG structure:
/// SubDag children are grouped by parent prefix, and flat transport
/// triplets (`prepare_X`, `execute_X`, `parse_X`) are grouped by suffix.
/// When groups are empty, rendering falls back to the ungrouped view.
#[derive(Debug, Clone)]
pub struct StageGroup {
    pub name: String,
    pub node_ids: Vec<NodeId>,
}

/// Computed progress for a stage group.
#[derive(Debug, Clone, Default)]
pub struct GroupProgress {
    pub total: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub skipped: usize,
}

impl GroupProgress {
    /// Whether all nodes in the group have reached a terminal state.
    pub fn is_done(&self) -> bool {
        self.completed + self.failed + self.skipped == self.total
    }

    /// Whether any node in the group has failed.
    pub fn is_failed(&self) -> bool {
        self.failed > 0
    }
}

/// Derive stage groups from node structure after lowering.
///
/// Recognizes two grouping patterns:
///
/// 1. **SubDag children**: Nodes with `parent/child` IDs (created by lowering)
///    are grouped by their parent prefix (the part before `/`).
///    Example: `rev_list/prepare_rev_list`, `rev_list/execute_rev_list` → group `rev_list`
///
/// 2. **Flat transport triplets**: Top-level nodes matching `prepare_X`,
///    `execute_X`, `parse_X` are grouped by suffix `X`.
///    Example: `prepare_build`, `execute_build`, `parse_build` → group `build`
///
/// Single-node groups are dropped (they add noise without value).
/// Returns empty `Vec` when no multi-node groups are found.
pub fn derive_stage_groups(node_ids: &[NodeId]) -> Vec<StageGroup> {
    const PREFIXES: &[&str] = &["prepare_", "execute_", "parse_"];

    let mut seen = std::collections::HashSet::new();
    let mut ordered_names: Vec<String> = Vec::new();
    let mut group_nodes: HashMap<String, Vec<NodeId>> = HashMap::new();

    for node_id in node_ids {
        let name = &node_id.0;

        // Strategy 1: SubDag children — group by parent prefix (before `/`)
        // Strategy 2: Flat transport triplet — group by suffix (after `prepare_`/etc.)
        // Fallback: standalone node — uses its own name as group key
        let group_name = if let Some(slash_pos) = name.find('/') {
            &name[..slash_pos]
        } else {
            PREFIXES
                .iter()
                .find_map(|p| name.strip_prefix(p))
                .unwrap_or(name)
        };

        group_nodes
            .entry(group_name.to_string())
            .or_default()
            .push(node_id.clone());
        if seen.insert(group_name.to_string()) {
            ordered_names.push(group_name.to_string());
        }
    }

    // Build groups in topo order, keeping only multi-node groups
    let groups: Vec<StageGroup> = ordered_names
        .into_iter()
        .filter_map(|name| {
            group_nodes
                .remove(&name)
                .filter(|nodes| nodes.len() > 1)
                .map(|node_ids| StageGroup { name, node_ids })
        })
        .collect();

    groups
}

impl StageGroup {
    /// Compute progress for this group from the current DAG progress state.
    pub fn progress(&self, dag_progress: &DagProgress) -> GroupProgress {
        let mut gp = GroupProgress {
            total: self.node_ids.len(),
            ..Default::default()
        };
        for node_id in &self.node_ids {
            if let Some(np) = dag_progress.nodes.get(node_id) {
                match np.state {
                    NodeState::Running => gp.running += 1,
                    NodeState::Completed | NodeState::Intercepted => gp.completed += 1,
                    NodeState::Failed => gp.failed += 1,
                    NodeState::Skipped => gp.skipped += 1,
                    NodeState::Pending => {}
                }
            }
        }
        gp
    }
}

// ---------------------------------------------------------------------------
// DagProgress — live state machine
// ---------------------------------------------------------------------------

/// Node execution state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Intercepted,
}

/// Edge data flow state (the "power flow" visual).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeState {
    /// No data has flowed through this edge yet.
    Idle,
    /// Data is flowing (source completed, destination not yet started).
    Flowing,
    /// Data has been consumed (destination started or completed).
    Done,
    /// Edge will never carry data (source failed or skipped).
    Dead,
}

/// Overall DAG execution phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagPhase {
    NotStarted,
    Running { current_node: NodeId },
    Completed { elapsed: Duration },
    Failed { node: NodeId, error: String },
}

/// Per-node progress tracking.
#[derive(Debug, Clone)]
pub struct NodeProgress {
    pub state: NodeState,
    pub start_time: Option<Instant>,
    pub elapsed: Option<Duration>,
    pub summary: Option<OutputSummary>,
}

/// Per-edge progress tracking.
#[derive(Debug, Clone)]
pub struct EdgeProgress {
    pub state: EdgeState,
}

/// Live state machine tracking DAG execution progress.
///
/// Implements [`ProgressObserver`] — it's a concrete observer that
/// maintains the "power flow" state machine. Renderer-agnostic:
/// knows nothing about terminals, HTML, or symbols.
///
/// # Edge Identity
///
/// Edges are keyed by `(from_node, to_node)` — intentionally collapsing
/// multiple port-level edges between the same node pair into a single
/// visual edge. This is the correct abstraction for progress display:
/// the renderer shows data flow between nodes, not between ports.
/// If the DAG has edges A:out1→B:in1 and A:out2→B:in2, they appear as
/// one visual edge A→B with a single state.
#[derive(Debug, Clone)]
pub struct DagProgress {
    pub snapshot: DagSnapshot,
    pub nodes: HashMap<NodeId, NodeProgress>,
    /// Visual edges keyed by `(from_node, to_node)`.
    /// Multiple port-level edges between the same nodes are intentionally
    /// collapsed into one visual edge.
    pub edges: HashMap<(NodeId, NodeId), EdgeProgress>,
    pub phase: DagPhase,
    pub start_time: Option<Instant>,
}

impl DagProgress {
    /// Create a new progress tracker from a DAG snapshot.
    ///
    /// All nodes start as Pending, all edges as Idle, phase as NotStarted.
    /// Port-level edges are collapsed to node-pair visual edges.
    pub fn new(snapshot: DagSnapshot) -> Self {
        let nodes: HashMap<NodeId, NodeProgress> = snapshot
            .node_ids
            .iter()
            .map(|id| {
                (
                    id.clone(),
                    NodeProgress {
                        state: NodeState::Pending,
                        start_time: None,
                        elapsed: None,
                        summary: None,
                    },
                )
            })
            .collect();

        // Collapse port-level edges to node-pair visual edges.
        // Multiple edges between the same (from, to) pair become one EdgeProgress.
        let mut edges: HashMap<(NodeId, NodeId), EdgeProgress> = HashMap::new();
        for e in &snapshot.edges {
            edges
                .entry((e.from_node.clone(), e.to_node.clone()))
                .or_insert(EdgeProgress {
                    state: EdgeState::Idle,
                });
        }

        Self {
            snapshot,
            nodes,
            edges,
            phase: DagPhase::NotStarted,
            start_time: None,
        }
    }

    /// Total elapsed time since DAG execution started.
    pub fn elapsed(&self) -> Duration {
        self.start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// Check if all nodes have reached a terminal state.
    pub fn all_terminal(&self) -> bool {
        self.nodes.values().all(|n| {
            matches!(
                n.state,
                NodeState::Completed
                    | NodeState::Failed
                    | NodeState::Skipped
                    | NodeState::Intercepted
            )
        })
    }

    /// Transition outgoing edges from a completed/intercepted node to Flowing.
    fn flow_edges_from(&mut self, node_id: &NodeId) {
        for edge in &self.snapshot.edges {
            if edge.from_node == *node_id {
                if let Some(ep) = self
                    .edges
                    .get_mut(&(edge.from_node.clone(), edge.to_node.clone()))
                {
                    ep.state = EdgeState::Flowing;
                }
            }
        }
    }

    /// Transition incoming edges to a starting node to Done.
    fn settle_edges_to(&mut self, node_id: &NodeId) {
        for edge in &self.snapshot.edges {
            if edge.to_node == *node_id {
                if let Some(ep) = self
                    .edges
                    .get_mut(&(edge.from_node.clone(), edge.to_node.clone()))
                {
                    if ep.state == EdgeState::Flowing {
                        ep.state = EdgeState::Done;
                    }
                }
            }
        }
    }

    /// Kill outgoing edges from a failed/skipped node.
    fn kill_edges_from(&mut self, node_id: &NodeId) {
        for edge in &self.snapshot.edges {
            if edge.from_node == *node_id {
                if let Some(ep) = self
                    .edges
                    .get_mut(&(edge.from_node.clone(), edge.to_node.clone()))
                {
                    ep.state = EdgeState::Dead;
                }
            }
        }
    }
}

impl ProgressObserver for DagProgress {
    fn on_dag_start(&mut self, _snapshot: &DagSnapshot) {
        self.start_time = Some(Instant::now());
        self.phase = DagPhase::Running {
            current_node: self
                .snapshot
                .topo_order
                .first()
                .cloned()
                .unwrap_or_else(|| NodeId::from("unknown")),
        };
    }

    fn on_node_start(&mut self, node_id: &NodeId) {
        if let Some(np) = self.nodes.get_mut(node_id) {
            np.state = NodeState::Running;
            np.start_time = Some(Instant::now());
        }
        self.phase = DagPhase::Running {
            current_node: node_id.clone(),
        };
        // Incoming edges transition from Flowing → Done
        self.settle_edges_to(node_id);
    }

    fn on_node_complete(&mut self, node_id: &NodeId, summary: OutputSummary) {
        if let Some(np) = self.nodes.get_mut(node_id) {
            np.state = NodeState::Completed;
            np.elapsed = np.start_time.map(|t| t.elapsed());
            np.summary = Some(summary);
        }
        // Outgoing edges transition to Flowing ("power flow" moment)
        self.flow_edges_from(node_id);
    }

    fn on_node_failed(&mut self, node_id: &NodeId, error: &str) {
        if let Some(np) = self.nodes.get_mut(node_id) {
            np.state = NodeState::Failed;
            np.elapsed = np.start_time.map(|t| t.elapsed());
        }
        self.kill_edges_from(node_id);
        self.phase = DagPhase::Failed {
            node: node_id.clone(),
            error: error.to_string(),
        };
    }

    fn on_node_skipped(&mut self, node_id: &NodeId) {
        if let Some(np) = self.nodes.get_mut(node_id) {
            np.state = NodeState::Skipped;
        }
        self.kill_edges_from(node_id);
    }

    fn on_node_intercepted(&mut self, node_id: &NodeId, summary: OutputSummary) {
        if let Some(np) = self.nodes.get_mut(node_id) {
            np.state = NodeState::Intercepted;
            np.elapsed = np.start_time.map(|t| t.elapsed());
            np.summary = Some(summary);
        }
        // Intercepted nodes still produce outputs — edges flow
        self.flow_edges_from(node_id);
    }

    fn on_dag_complete(&mut self, elapsed: Duration) {
        if !matches!(self.phase, DagPhase::Failed { .. }) {
            self.phase = DagPhase::Completed { elapsed };
        }
        // Settle any remaining Flowing edges to Done
        let flowing: Vec<(NodeId, NodeId)> = self
            .edges
            .iter()
            .filter(|(_, ep)| ep.state == EdgeState::Flowing)
            .map(|(k, _)| k.clone())
            .collect();
        for key in flowing {
            if let Some(ep) = self.edges.get_mut(&key) {
                ep.state = EdgeState::Done;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ComposedObserver — fan-out to two observers
// ---------------------------------------------------------------------------

/// Adapter that fans out observer callbacks to two observers.
///
/// Enables composing e.g. `NonTtyProgressObserver` with `CiContext`
/// without changing the executor — the executor sees a single observer.
pub struct ComposedObserver<'a, 'b> {
    pub primary: &'a mut dyn ProgressObserver,
    pub secondary: &'b mut dyn ProgressObserver,
}

impl ProgressObserver for ComposedObserver<'_, '_> {
    fn on_dag_start(&mut self, snapshot: &DagSnapshot) {
        self.primary.on_dag_start(snapshot);
        self.secondary.on_dag_start(snapshot);
    }

    fn on_node_start(&mut self, node_id: &NodeId) {
        self.primary.on_node_start(node_id);
        self.secondary.on_node_start(node_id);
    }

    fn on_node_complete(&mut self, node_id: &NodeId, summary: OutputSummary) {
        self.primary.on_node_complete(node_id, summary.clone());
        self.secondary.on_node_complete(node_id, summary);
    }

    fn on_node_failed(&mut self, node_id: &NodeId, error: &str) {
        self.primary.on_node_failed(node_id, error);
        self.secondary.on_node_failed(node_id, error);
    }

    fn on_node_skipped(&mut self, node_id: &NodeId) {
        self.primary.on_node_skipped(node_id);
        self.secondary.on_node_skipped(node_id);
    }

    fn on_node_intercepted(&mut self, node_id: &NodeId, summary: OutputSummary) {
        self.primary.on_node_intercepted(node_id, summary.clone());
        self.secondary.on_node_intercepted(node_id, summary);
    }

    fn on_dag_complete(&mut self, elapsed: Duration) {
        self.primary.on_dag_complete(elapsed);
        self.secondary.on_dag_complete(elapsed);
    }

    fn on_secret_output(&mut self, node_id: &NodeId, secret_value: &str) {
        self.primary.on_secret_output(node_id, secret_value);
        self.secondary.on_secret_output(node_id, secret_value);
    }

    fn on_failure_diagnostics(&mut self, node_id: &NodeId, inputs: &HashMap<String, Value>) {
        self.primary.on_failure_diagnostics(node_id, inputs);
        self.secondary.on_failure_diagnostics(node_id, inputs);
    }

    fn on_boundary_output(&mut self, node_id: &NodeId, entry: &crate::execute::LogEntry) {
        self.primary.on_boundary_output(node_id, entry);
        self.secondary.on_boundary_output(node_id, entry);
    }

    fn requires_sequential(&self) -> bool {
        self.primary.requires_sequential() || self.secondary.requires_sequential()
    }
}

// ---------------------------------------------------------------------------
// RecordingObserver — test helper
// ---------------------------------------------------------------------------

/// A recording observer that collects events into a Vec for testing.
#[derive(Debug, Default)]
pub struct RecordingObserver {
    pub events: Vec<ProgressEvent>,
}

/// A recorded progress event.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    DagStart,
    NodeStart(NodeId),
    NodeComplete(NodeId),
    NodeFailed(NodeId, String),
    NodeSkipped(NodeId),
    NodeIntercepted(NodeId),
    DagComplete(Duration),
}

impl RecordingObserver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the sequence of node start events.
    pub fn node_starts(&self) -> Vec<NodeId> {
        self.events
            .iter()
            .filter_map(|e| match e {
                ProgressEvent::NodeStart(id) => Some(id.clone()),
                _ => None,
            })
            .collect()
    }

    /// Check if all nodes reached a terminal event.
    pub fn all_terminal(&self, expected_count: usize) -> bool {
        let terminal_count = self
            .events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    ProgressEvent::NodeComplete(_)
                        | ProgressEvent::NodeFailed(_, _)
                        | ProgressEvent::NodeSkipped(_)
                        | ProgressEvent::NodeIntercepted(_)
                )
            })
            .count();
        terminal_count == expected_count
    }
}

impl ProgressObserver for RecordingObserver {
    fn on_dag_start(&mut self, _snapshot: &DagSnapshot) {
        self.events.push(ProgressEvent::DagStart);
    }
    fn on_node_start(&mut self, node_id: &NodeId) {
        self.events.push(ProgressEvent::NodeStart(node_id.clone()));
    }
    fn on_node_complete(&mut self, node_id: &NodeId, _summary: OutputSummary) {
        self.events
            .push(ProgressEvent::NodeComplete(node_id.clone()));
    }
    fn on_node_failed(&mut self, node_id: &NodeId, error: &str) {
        self.events.push(ProgressEvent::NodeFailed(
            node_id.clone(),
            error.to_string(),
        ));
    }
    fn on_node_skipped(&mut self, node_id: &NodeId) {
        self.events
            .push(ProgressEvent::NodeSkipped(node_id.clone()));
    }
    fn on_node_intercepted(&mut self, node_id: &NodeId, _summary: OutputSummary) {
        self.events
            .push(ProgressEvent::NodeIntercepted(node_id.clone()));
    }
    fn on_dag_complete(&mut self, elapsed: Duration) {
        self.events.push(ProgressEvent::DagComplete(elapsed));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::Edge;

    fn simple_snapshot() -> DagSnapshot {
        // A → B → C
        DagSnapshot {
            node_ids: vec![NodeId::from("A"), NodeId::from("B"), NodeId::from("C")],
            edges: vec![
                Edge::new("A", "out", "B", "in"),
                Edge::new("B", "out", "C", "in"),
            ],
            topo_order: vec![NodeId::from("A"), NodeId::from("B"), NodeId::from("C")],
            boundary_nodes: vec![],
            labels: [
                (NodeId::from("A"), "A".to_string()),
                (NodeId::from("B"), "B".to_string()),
                (NodeId::from("C"), "C".to_string()),
            ]
            .into_iter()
            .collect(),
            groups: vec![],
        }
    }

    fn empty_summary() -> OutputSummary {
        OutputSummary {
            fields: vec![],
            elapsed: Duration::from_millis(10),
        }
    }

    #[test]
    fn test_initial_state() {
        let snap = simple_snapshot();
        let progress = DagProgress::new(snap);

        assert!(matches!(progress.phase, DagPhase::NotStarted));
        assert!(progress
            .nodes
            .values()
            .all(|n| n.state == NodeState::Pending));
        assert!(progress.edges.values().all(|e| e.state == EdgeState::Idle));
    }

    #[test]
    fn test_happy_path_transitions() {
        let snap = simple_snapshot();
        let mut progress = DagProgress::new(snap.clone());

        // Start DAG
        progress.on_dag_start(&snap);
        assert!(matches!(progress.phase, DagPhase::Running { .. }));

        // A starts → A running
        progress.on_node_start(&NodeId::from("A"));
        assert_eq!(progress.nodes[&NodeId::from("A")].state, NodeState::Running);

        // A completes → A completed, A→B edge flowing
        progress.on_node_complete(&NodeId::from("A"), empty_summary());
        assert_eq!(
            progress.nodes[&NodeId::from("A")].state,
            NodeState::Completed
        );
        assert_eq!(
            progress.edges[&(NodeId::from("A"), NodeId::from("B"))].state,
            EdgeState::Flowing
        );

        // B starts → B running, A→B edge done
        progress.on_node_start(&NodeId::from("B"));
        assert_eq!(progress.nodes[&NodeId::from("B")].state, NodeState::Running);
        assert_eq!(
            progress.edges[&(NodeId::from("A"), NodeId::from("B"))].state,
            EdgeState::Done
        );

        // B completes → B completed, B→C edge flowing
        progress.on_node_complete(&NodeId::from("B"), empty_summary());
        assert_eq!(
            progress.edges[&(NodeId::from("B"), NodeId::from("C"))].state,
            EdgeState::Flowing
        );

        // C starts and completes
        progress.on_node_start(&NodeId::from("C"));
        progress.on_node_complete(&NodeId::from("C"), empty_summary());

        // DAG completes
        progress.on_dag_complete(Duration::from_secs(1));
        assert!(matches!(progress.phase, DagPhase::Completed { .. }));
        assert!(progress.all_terminal());

        // All edges should be Done
        assert!(progress.edges.values().all(|e| e.state == EdgeState::Done));
    }

    #[test]
    fn test_failure_kills_edges() {
        let snap = simple_snapshot();
        let mut progress = DagProgress::new(snap.clone());

        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("A"));
        progress.on_node_complete(&NodeId::from("A"), empty_summary());

        // B fails
        progress.on_node_start(&NodeId::from("B"));
        progress.on_node_failed(&NodeId::from("B"), "something broke");

        assert_eq!(progress.nodes[&NodeId::from("B")].state, NodeState::Failed);
        assert_eq!(
            progress.edges[&(NodeId::from("B"), NodeId::from("C"))].state,
            EdgeState::Dead
        );
        assert!(matches!(progress.phase, DagPhase::Failed { .. }));
    }

    #[test]
    fn test_skip_kills_edges() {
        let snap = simple_snapshot();
        let mut progress = DagProgress::new(snap.clone());

        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("A"));
        progress.on_node_skipped(&NodeId::from("A"));

        assert_eq!(progress.nodes[&NodeId::from("A")].state, NodeState::Skipped);
        assert_eq!(
            progress.edges[&(NodeId::from("A"), NodeId::from("B"))].state,
            EdgeState::Dead
        );
    }

    #[test]
    fn test_intercepted_flows_edges() {
        let snap = simple_snapshot();
        let mut progress = DagProgress::new(snap.clone());

        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("A"));
        progress.on_node_intercepted(&NodeId::from("A"), empty_summary());

        // Intercepted nodes still produce outputs — edges flow
        assert_eq!(
            progress.nodes[&NodeId::from("A")].state,
            NodeState::Intercepted
        );
        assert_eq!(
            progress.edges[&(NodeId::from("A"), NodeId::from("B"))].state,
            EdgeState::Flowing
        );
    }

    #[test]
    fn test_recording_observer() {
        let snap = simple_snapshot();
        let mut observer = RecordingObserver::new();

        observer.on_dag_start(&snap);
        observer.on_node_start(&NodeId::from("A"));
        observer.on_node_complete(&NodeId::from("A"), empty_summary());
        observer.on_node_start(&NodeId::from("B"));
        observer.on_node_complete(&NodeId::from("B"), empty_summary());
        observer.on_node_start(&NodeId::from("C"));
        observer.on_node_complete(&NodeId::from("C"), empty_summary());
        observer.on_dag_complete(Duration::from_secs(1));

        assert_eq!(
            observer.node_starts(),
            vec![NodeId::from("A"), NodeId::from("B"), NodeId::from("C")]
        );
        assert!(observer.all_terminal(3));
        assert_eq!(observer.events.len(), 8); // 1 start + 3*(start+complete) + 1 complete
    }

    #[test]
    fn test_output_summary_from_outputs() {
        let mut outputs = HashMap::new();
        outputs.insert("count".to_string(), Value::Int(42));
        outputs.insert(
            "url".to_string(),
            Value::Str("https://example.com".to_string()),
        );
        outputs.insert(
            "items".to_string(),
            Value::List(vec![Value::Int(1), Value::Int(2)]),
        );
        outputs.insert(
            "secret".to_string(),
            Value::Secret(gunbc_ir::SecretString::new("s3cr3t")),
        );
        outputs.insert("skipped".to_string(), Value::Skipped);

        let summary = OutputSummary::from_outputs(&outputs, Duration::from_millis(100));

        // Skipped values should be filtered out
        assert_eq!(summary.fields.len(), 4);

        let url_field = summary.fields.iter().find(|f| f.name == "url").unwrap();
        assert_eq!(url_field.kind, FieldKind::Url);

        let secret_field = summary.fields.iter().find(|f| f.name == "secret").unwrap();
        assert_eq!(secret_field.kind, FieldKind::Secret);
        assert_eq!(secret_field.preview, "***");

        let list_field = summary.fields.iter().find(|f| f.name == "items").unwrap();
        assert_eq!(list_field.kind, FieldKind::List(2));
    }

    // -----------------------------------------------------------------------
    // StageGroup + GroupProgress tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_group_progress_computes_states() {
        let snap = DagSnapshot {
            node_ids: vec![
                NodeId::from("prepare_build"),
                NodeId::from("execute_build"),
                NodeId::from("parse_build"),
            ],
            edges: vec![
                Edge::new("prepare_build", "out", "execute_build", "in"),
                Edge::new("execute_build", "out", "parse_build", "in"),
            ],
            topo_order: vec![
                NodeId::from("prepare_build"),
                NodeId::from("execute_build"),
                NodeId::from("parse_build"),
            ],
            boundary_nodes: vec![],
            labels: HashMap::new(),
            groups: vec![StageGroup {
                name: "build".into(),
                node_ids: vec![
                    NodeId::from("prepare_build"),
                    NodeId::from("execute_build"),
                    NodeId::from("parse_build"),
                ],
            }],
        };

        let mut progress = DagProgress::new(snap.clone());
        progress.on_dag_start(&snap);
        progress.on_node_start(&NodeId::from("prepare_build"));
        progress.on_node_complete(&NodeId::from("prepare_build"), empty_summary());
        progress.on_node_start(&NodeId::from("execute_build"));

        let gp = snap.groups[0].progress(&progress);
        assert_eq!(gp.total, 3);
        assert_eq!(gp.completed, 1);
        assert_eq!(gp.running, 1);
        assert!(!gp.is_done());
        assert!(!gp.is_failed());
    }

    #[test]
    fn test_group_progress_is_done() {
        let gp = GroupProgress {
            total: 3,
            running: 0,
            completed: 2,
            failed: 0,
            skipped: 1,
        };
        assert!(gp.is_done());
        assert!(!gp.is_failed());
    }

    #[test]
    fn test_group_progress_is_failed() {
        let gp = GroupProgress {
            total: 3,
            running: 0,
            completed: 1,
            failed: 1,
            skipped: 1,
        };
        assert!(gp.is_done());
        assert!(gp.is_failed());
    }

    #[test]
    fn test_derive_stage_groups_ci_naming() {
        let node_ids = vec![
            NodeId::from("prepare_build"),
            NodeId::from("execute_build"),
            NodeId::from("parse_build"),
            NodeId::from("prepare_test"),
            NodeId::from("execute_test"),
            NodeId::from("parse_test"),
            NodeId::from("report"),
        ];

        let groups = derive_stage_groups(&node_ids);
        assert_eq!(groups.len(), 2); // build (3), test (3); report (1) dropped
        assert_eq!(groups[0].name, "build");
        assert_eq!(groups[0].node_ids.len(), 3);
        assert_eq!(groups[1].name, "test");
        assert_eq!(groups[1].node_ids.len(), 3);
    }

    #[test]
    fn test_derive_stage_groups_subdag_prefixed() {
        // Simulates a lowered gist-recent DAG
        let node_ids = vec![
            NodeId::from("fs_env"),
            NodeId::from("resolve_auth"),
            NodeId::from("rev_list/prepare_rev_list"),
            NodeId::from("rev_list/execute_rev_list"),
            NodeId::from("rev_list/parse_rev_list"),
            NodeId::from("diff/prepare_diff"),
            NodeId::from("diff/execute_diff"),
            NodeId::from("diff/parse_diff"),
            NodeId::from("render_markdown"),
        ];

        let groups = derive_stage_groups(&node_ids);
        assert_eq!(groups.len(), 2); // rev_list (3), diff (3); singletons dropped
        assert_eq!(groups[0].name, "rev_list");
        assert_eq!(groups[0].node_ids.len(), 3);
        assert_eq!(groups[1].name, "diff");
        assert_eq!(groups[1].node_ids.len(), 3);
    }

    #[test]
    fn test_derive_stage_groups_no_structure_returns_empty() {
        let node_ids = vec![
            NodeId::from("fetch"),
            NodeId::from("transform"),
            NodeId::from("upload"),
        ];

        let groups = derive_stage_groups(&node_ids);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_derive_stage_groups_preserves_topo_order() {
        let node_ids = vec![
            NodeId::from("prepare_lint"),
            NodeId::from("prepare_build"),
            NodeId::from("execute_lint"),
            NodeId::from("execute_build"),
            NodeId::from("parse_lint"),
            NodeId::from("parse_build"),
        ];

        let groups = derive_stage_groups(&node_ids);
        assert_eq!(groups.len(), 2);
        // lint appears first in topo order
        assert_eq!(groups[0].name, "lint");
        assert_eq!(groups[1].name, "build");
    }

    #[test]
    fn test_derive_stage_groups_mixed_subdag_and_flat() {
        // Mix of SubDag children and flat transport triplets
        let node_ids = vec![
            NodeId::from("cloud_credential/resolve"),
            NodeId::from("cloud_credential/bind"),
            NodeId::from("prepare_build"),
            NodeId::from("execute_build"),
            NodeId::from("parse_build"),
            NodeId::from("standalone"),
        ];

        let groups = derive_stage_groups(&node_ids);
        assert_eq!(groups.len(), 2); // cloud_credential (2), build (3); standalone (1) dropped
        assert_eq!(groups[0].name, "cloud_credential");
        assert_eq!(groups[0].node_ids.len(), 2);
        assert_eq!(groups[1].name, "build");
        assert_eq!(groups[1].node_ids.len(), 3);
    }
}
