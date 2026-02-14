//! Shared execute-and-display logic for all CLI tools.
//!
//! Encapsulates the two display paths in a single generic function. All CLI
//! tools (handwritten and code-generated) call [`execute_and_display`] instead
//! of duplicating the display branching logic.
//!
//! # Architecture
//!
//! ```text
//! animated: bool →
//!   true  → live DAG animation via SharedProgressObserver
//!   false → observer-driven status lines + boundary outputs
//!           (CI environments compose CiGroupObserver for workflow annotations)
//! ```

use crate::frame_build::{build_frame, format_duration};
use crate::frame_write::FrameWriter;
use crate::intercept::BoundaryMocks;
use crate::progress::{
    ComposedObserver, DagProgress, DagSnapshot, OutputSummary, ProgressObserver, StageGroup,
};
use crate::render::{Animation, RenderMode};
use crate::terminal::TerminalProfile;
use crate::{
    execute_with_progress_and_mode_and_inputs, lower, topo_sort, ExecError, Executable,
    ExecutionMode, NodeState,
};
use gunbc_ir::layout::compute_layout;
use gunbc_ir::symbols::{SymbolId, STANDARD};
use gunbc_ir::{detect_boundaries, Dag, NodeId, Value};
use std::collections::HashMap;
use std::io::{self, IsTerminal};
use std::process;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

const ANSI_RED: &str = "\x1b[31m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_RESET: &str = "\x1b[0m";
const PROGRESS_TICK: Duration = Duration::from_millis(100);

/// High-signal attention levels for user-facing terminal messaging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionLevel {
    Info,
    Warning,
    Error,
}

/// Result from display-aware DAG execution.
#[derive(Debug, Clone)]
pub struct DisplayResult {
    pub log: crate::ExecutionLog,
    /// True when execution completed but policy indicates non-zero exit
    /// (for example: `success_port=false`).
    pub should_fail: bool,
}

/// Execute a DAG and display results based on terminal capabilities.
///
/// This is the single entry point for all CLI tools. It handles:
/// - Progress display with animated replay (when `animated` is true)
/// - Classic text output (otherwise)
/// - Exit code handling (exits with code 1 on failure)
///
/// # Arguments
///
/// - `dag`: The DAG to execute.
/// - `mode`: Execution mode (real or dry-run with mocks).
/// - `animated`: Whether to use animated progress display.
/// - `success_port`: Optional port name to check for `false` → exit(1).
/// - `input_mocks`: Optional input overrides for entrypoint ports.
pub fn execute_and_display<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    animated: bool,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) {
    match execute_and_display_with_result(dag, mode, animated, success_port, input_mocks) {
        Ok(result) => {
            if result.should_fail {
                print_attention(
                    AttentionLevel::Error,
                    "Execution failed",
                    "A required success check returned false.",
                );
                process::exit(1);
            }
        }
        Err(e) => {
            print_attention(AttentionLevel::Error, "Execution failed", &e.to_string());
            process::exit(1);
        }
    }
}

/// Execute preflight and then execute/display a DAG with a unified terminal surface.
///
/// `preflight` is passed a [`PreflightObserver`] to report progress.
pub fn execute_and_display_with_preflight<T, F>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    animated: bool,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
    mut preflight: F,
) where
    T: Executable + Clone + Send,
    F: FnMut(Option<&mut dyn PreflightObserver>) -> Result<(), String>,
{
    match execute_and_display_with_preflight_result(
        dag,
        mode,
        animated,
        success_port,
        input_mocks,
        &mut preflight,
    ) {
        Ok(result) => {
            if result.should_fail {
                print_attention(
                    AttentionLevel::Error,
                    "Execution failed",
                    "A required success check returned false.",
                );
                process::exit(1);
            }
        }
        Err(e) => {
            print_attention(AttentionLevel::Error, "Execution failed", &e.to_string());
            process::exit(1);
        }
    }
}

/// Result-returning variant of [`execute_and_display_with_preflight`].
pub fn execute_and_display_with_preflight_result<T, F>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    animated: bool,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
    preflight: &mut F,
) -> Result<DisplayResult, ExecError>
where
    T: Executable + Clone + Send,
    F: FnMut(Option<&mut dyn PreflightObserver>) -> Result<(), String>,
{
    run_preflight_with_display(animated, preflight)?;

    execute_and_display_with_result(dag, mode, animated, success_port, input_mocks)
}

/// Run preflight using the same terminal display surface used by DAG execution.
pub fn run_preflight_with_display<F>(animated: bool, preflight: &mut F) -> Result<(), ExecError>
where
    F: FnMut(Option<&mut dyn PreflightObserver>) -> Result<(), String>,
{
    if animated {
        run_preflight_with_progress(|observer| preflight(Some(observer)))
    } else if is_ci_environment() {
        let mut ci = crate::CiContext::detect();
        preflight(Some(&mut ci)).map_err(|e| ExecError::new(format!("preflight failed: {}", e)))
    } else {
        let mut status = PreflightStatusObserver;
        preflight(Some(&mut status)).map_err(|e| ExecError::new(format!("preflight failed: {}", e)))
    }
}

/// Execute a DAG through the shared display path and return execution results.
///
/// Unlike [`execute_and_display`], this function never exits the process.
pub fn execute_and_display_with_result<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    animated: bool,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<DisplayResult, ExecError> {
    if animated {
        run_with_progress(dag, mode, success_port, input_mocks)
    } else {
        run_plain(dag, mode, success_port, input_mocks)
    }
}

fn run_preflight_with_progress<F>(mut preflight: F) -> Result<(), ExecError>
where
    F: FnMut(&mut dyn PreflightObserver) -> Result<(), String>,
{
    let profile = TerminalProfile::detect();

    let progress_state = Arc::new(Mutex::new(DagProgress::new(initial_preflight_snapshot())));
    let stop_render = Arc::new(AtomicBool::new(false));

    let progress_for_render = Arc::clone(&progress_state);
    let stop_for_render = Arc::clone(&stop_render);
    let profile_for_render = profile.clone();
    let render_handle = thread::spawn(move || {
        let spinner_frames: Vec<String> = [
            SymbolId::Spinner0,
            SymbolId::Spinner1,
            SymbolId::Spinner2,
            SymbolId::Spinner3,
        ]
        .iter()
        .map(|id| {
            STANDARD
                .resolve_tier(*id, profile_for_render.tier)
                .to_string()
        })
        .collect();
        let mut spinner = Animation::cycle(spinner_frames, Duration::from_millis(150));
        let mut writer = FrameWriter::new(
            profile_for_render.supports_color,
            profile_for_render.tier,
            &STANDARD,
            profile_for_render.is_tty,
        );
        let mut stdout = io::stdout();
        let mut last_tick = Instant::now();

        loop {
            let now = Instant::now();
            spinner.tick(now.saturating_duration_since(last_tick));
            last_tick = now;

            let progress = {
                let guard = progress_for_render
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guard.clone()
            };

            let layout = compute_layout(
                &progress.snapshot.topo_order,
                &progress.snapshot.edges,
                &progress.snapshot.labels,
                &profile_for_render.viewport,
            );
            render_progress_frame(
                &progress,
                &layout,
                &spinner,
                &mut writer,
                &mut stdout,
                &profile_for_render,
            );

            if stop_for_render.load(Ordering::Relaxed) {
                break;
            }
            thread::sleep(PROGRESS_TICK);
        }
    });

    let mut observer = SharedPreflightObserver::new(Arc::clone(&progress_state));
    let preflight_result = preflight(&mut observer);

    stop_render.store(true, Ordering::Relaxed);
    if render_handle.join().is_err() {
        return Err(ExecError::new(
            "preflight progress renderer thread panicked",
        ));
    }

    preflight_result.map_err(|e| ExecError::new(format!("preflight failed: {}", e)))
}

/// Plain execution: observer-driven status lines + boundary outputs.
///
/// Unified path for all non-interactive environments. When in CI, composes
/// the status observer with a `CiContext` observer for workflow commands
/// (groups, error annotations, secret masking).
fn run_plain<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<DisplayResult, ExecError> {
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {}", e)))?;
    let boundaries = detect_boundaries(&flat.dag);

    let mut status_observer = NonTtyProgressObserver::default();

    let is_ci = is_ci_environment();

    let log = if is_ci {
        // CI groups require sequential execution (groups must nest properly).
        // The parallel executor already respects GUNBC_EXEC_MAX_CONCURRENCY.
        let _guard = CiConcurrencyGuard::new();

        let mut ci_observer = crate::CiContext::detect();
        let mut composed = ComposedObserver {
            primary: &mut status_observer,
            secondary: &mut ci_observer,
        };
        let log = execute_with_progress_and_mode_and_inputs(dag, mode, &mut composed, input_mocks)?;

        // Mask secrets before printing any outputs.
        // GitHub Actions masks are retroactive within a step, so masking
        // after execution but before boundary output printing is safe.
        mask_secrets_in_log(&mut ci_observer, &log);

        log
    } else {
        execute_with_progress_and_mode_and_inputs(dag, mode, &mut status_observer, input_mocks)?
    };

    print_boundary_outputs(&log, &boundaries);

    Ok(DisplayResult {
        should_fail: status_observer.failed_count() > 0 || success_port_failed(&log, success_port),
        log,
    })
}

fn is_ci_environment() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
}

/// Mask secret values from an execution log via CI workflow commands.
///
/// Iterates through all log entries and emits `::add-mask::` for each
/// secret value so CI runners redact them from all subsequent output.
fn mask_secrets_in_log(ci: &mut crate::CiContext, log: &crate::ExecutionLog) {
    for entry in &log.entries {
        for value in entry.outputs.values() {
            if let Value::Secret(s) = value {
                ci.mask(s.expose());
            }
        }
    }
}

/// RAII guard that sets `GUNBC_EXEC_MAX_CONCURRENCY=1` for CI sequential execution.
///
/// CI groups require sequential execution (groups must nest properly).
/// Restores the previous value (or removes the var) on drop.
struct CiConcurrencyGuard {
    previous: Option<String>,
}

impl CiConcurrencyGuard {
    fn new() -> Self {
        let previous = std::env::var("GUNBC_EXEC_MAX_CONCURRENCY").ok();
        std::env::set_var("GUNBC_EXEC_MAX_CONCURRENCY", "1");
        Self { previous }
    }
}

impl Drop for CiConcurrencyGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(val) => std::env::set_var("GUNBC_EXEC_MAX_CONCURRENCY", val),
            None => std::env::remove_var("GUNBC_EXEC_MAX_CONCURRENCY"),
        }
    }
}

/// Progress-display execution: live DAG visualization driven by observer events.
fn run_with_progress<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<DisplayResult, ExecError> {
    let profile = TerminalProfile::detect();
    // Lower the DAG to get flat topology for layout
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {}", e)))?;

    let boundaries = detect_boundaries(&flat.dag);
    let topo_order = topo_sort(&flat.dag);

    // Build labels from node IDs, stripping SubDag parent prefix for readability
    let labels: HashMap<NodeId, String> = flat
        .dag
        .nodes
        .iter()
        .map(|n| {
            let label = n.id.0.split('/').next_back().unwrap_or(&n.id.0).to_string();
            (n.id.clone(), label)
        })
        .collect();

    // Compute layout from profile viewport
    let layout = compute_layout(&topo_order, &flat.dag.edges, &labels, &profile.viewport);

    let snapshot = DagSnapshot::from_dag(&flat.dag, &topo_order, &boundaries);
    let progress_state = Arc::new(Mutex::new(DagProgress::new(snapshot)));
    let stop_render = Arc::new(AtomicBool::new(false));

    // Render live frames on a background thread while execution runs.
    let progress_for_render = Arc::clone(&progress_state);
    let stop_for_render = Arc::clone(&stop_render);
    let layout_for_render = layout.clone();
    let profile_for_render = profile.clone();
    let render_handle = thread::spawn(move || {
        let spinner_frames: Vec<String> = [
            SymbolId::Spinner0,
            SymbolId::Spinner1,
            SymbolId::Spinner2,
            SymbolId::Spinner3,
        ]
        .iter()
        .map(|id| {
            STANDARD
                .resolve_tier(*id, profile_for_render.tier)
                .to_string()
        })
        .collect();
        let mut spinner = Animation::cycle(spinner_frames, Duration::from_millis(150));
        let mut writer = FrameWriter::new(
            profile_for_render.supports_color,
            profile_for_render.tier,
            &STANDARD,
            profile_for_render.is_tty,
        );
        let mut stdout = io::stdout();
        let mut last_tick = Instant::now();

        loop {
            let now = Instant::now();
            spinner.tick(now.saturating_duration_since(last_tick));
            last_tick = now;

            let progress = {
                let guard = progress_for_render
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guard.clone()
            };
            render_progress_frame(
                &progress,
                &layout_for_render,
                &spinner,
                &mut writer,
                &mut stdout,
                &profile_for_render,
            );

            if stop_for_render.load(Ordering::Relaxed) {
                break;
            }
            thread::sleep(PROGRESS_TICK);
        }
    });

    let mut observer = SharedProgressObserver::new(Arc::clone(&progress_state));
    let log_result =
        execute_with_progress_and_mode_and_inputs(dag, mode, &mut observer, input_mocks);

    stop_render.store(true, Ordering::Relaxed);
    if render_handle.join().is_err() {
        return Err(ExecError::new("progress renderer thread panicked"));
    }

    let log = log_result?;

    let final_progress = {
        let guard = progress_state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.clone()
    };

    // Check final node states for hard failures
    let mut should_fail = final_progress
        .nodes
        .values()
        .any(|np| np.state == NodeState::Failed);
    should_fail = should_fail || success_port_failed(&log, success_port);

    // Surface boundary outputs after progress render so users see
    // the actual tool results (e.g., gist URL) instead of only the DAG view.
    print_boundary_outputs(&log, &boundaries);

    Ok(DisplayResult { log, should_fail })
}

#[derive(Clone)]
struct SharedProgressObserver {
    progress: Arc<Mutex<DagProgress>>,
}

impl SharedProgressObserver {
    fn new(progress: Arc<Mutex<DagProgress>>) -> Self {
        Self { progress }
    }

    fn with_progress<F>(&self, update: F)
    where
        F: FnOnce(&mut DagProgress),
    {
        let mut guard = self
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        update(&mut guard);
    }
}

#[derive(Clone)]
struct SharedPreflightObserver {
    progress: Arc<Mutex<DagProgress>>,
    started: bool,
    initialized_steps: bool,
    total_steps: usize,
    current_step: Option<usize>,
}

impl SharedPreflightObserver {
    fn new(progress: Arc<Mutex<DagProgress>>) -> Self {
        Self {
            progress,
            started: false,
            initialized_steps: false,
            total_steps: 0,
            current_step: None,
        }
    }

    fn with_progress<F>(&self, update: F)
    where
        F: FnOnce(&mut DagProgress),
    {
        let mut guard = self
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        update(&mut guard);
    }

    fn ensure_started(&mut self) {
        if self.started {
            return;
        }
        self.with_progress(|progress| {
            let snapshot = progress.snapshot.clone();
            progress.on_dag_start(&snapshot);
            progress.on_node_start(&NodeId::from("preflight_check"));
        });
        self.started = true;
    }

    fn ensure_initialized_steps(&mut self, total: usize) {
        if self.initialized_steps && self.total_steps == total {
            return;
        }
        let snapshot = preflight_snapshot(total);
        self.with_progress(|progress| {
            *progress = DagProgress::new(snapshot.clone());
            progress.on_dag_start(&snapshot);
        });
        self.initialized_steps = true;
        self.total_steps = total;
        self.current_step = None;
        self.started = true;
    }

    fn step_node_id(step: usize) -> NodeId {
        NodeId::from(format!("preflight_step_{}", step))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NonTtyNodeState {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Intercepted,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct NonTtyProgressCounts {
    total: usize,
    running: usize,
    completed: usize,
    failed: usize,
    skipped: usize,
    intercepted: usize,
}

impl NonTtyProgressCounts {
    fn done(self) -> usize {
        self.completed + self.intercepted
    }
}

#[derive(Default)]
struct NonTtyProgressObserver {
    labels: HashMap<NodeId, String>,
    states: HashMap<NodeId, NonTtyNodeState>,
    /// Track failed nodes with their error first lines for the dag_complete summary.
    failures: Vec<(String, String)>,
    /// Stage groups from the snapshot (empty for non-CI DAGs).
    groups: Vec<StageGroup>,
    /// Maps node_id → group index for quick lookup.
    group_map: HashMap<NodeId, usize>,
    /// Which groups have had their separator printed.
    groups_started: Vec<bool>,
}

impl NonTtyProgressObserver {
    fn label_for<'a>(&'a self, node_id: &'a NodeId) -> &'a str {
        self.labels
            .get(node_id)
            .map(String::as_str)
            .unwrap_or(&node_id.0)
    }

    fn set_state(&mut self, node_id: &NodeId, state: NonTtyNodeState) {
        self.states.insert(node_id.clone(), state);
    }

    fn counts(&self) -> NonTtyProgressCounts {
        let mut counts = NonTtyProgressCounts {
            total: self.states.len(),
            ..Default::default()
        };

        for state in self.states.values() {
            match state {
                NonTtyNodeState::Pending => {}
                NonTtyNodeState::Running => counts.running += 1,
                NonTtyNodeState::Completed => counts.completed += 1,
                NonTtyNodeState::Failed => counts.failed += 1,
                NonTtyNodeState::Skipped => counts.skipped += 1,
                NonTtyNodeState::Intercepted => counts.intercepted += 1,
            }
        }
        counts
    }

    fn failed_count(&self) -> usize {
        self.counts().failed
    }

    /// Check if all nodes in the given group have reached a terminal state.
    fn is_group_done(&self, group_idx: usize) -> bool {
        if let Some(group) = self.groups.get(group_idx) {
            group.node_ids.iter().all(|id| {
                matches!(
                    self.states.get(id),
                    Some(
                        NonTtyNodeState::Completed
                            | NonTtyNodeState::Failed
                            | NonTtyNodeState::Skipped
                            | NonTtyNodeState::Intercepted
                    )
                )
            })
        } else {
            false
        }
    }

    /// Emit a group separator if the node's group hasn't been started yet.
    fn maybe_emit_group_header(&mut self, node_id: &NodeId) {
        if let Some(&group_idx) = self.group_map.get(node_id) {
            if !self.groups_started[group_idx] {
                self.groups_started[group_idx] = true;
                let name = &self.groups[group_idx].name;
                eprintln!("--- {} ---", name);
            }
        }
    }

    /// Check if the node's group just completed and emit a summary if so.
    fn maybe_emit_group_summary(&self, node_id: &NodeId) {
        if let Some(&group_idx) = self.group_map.get(node_id) {
            if self.is_group_done(group_idx) {
                let group = &self.groups[group_idx];
                let done = group
                    .node_ids
                    .iter()
                    .filter(|id| {
                        matches!(
                            self.states.get(*id),
                            Some(NonTtyNodeState::Completed | NonTtyNodeState::Intercepted)
                        )
                    })
                    .count();
                let failed = group
                    .node_ids
                    .iter()
                    .filter(|id| matches!(self.states.get(*id), Some(NonTtyNodeState::Failed)))
                    .count();
                let total = group.node_ids.len();
                if failed > 0 {
                    eprintln!("✗ {} [{}/{}]", group.name, done, total);
                } else {
                    eprintln!("✓ {} [{}/{}]", group.name, done, total);
                }
            }
        }
    }
}

fn format_non_tty_summary_line(counts: NonTtyProgressCounts, elapsed: Duration) -> String {
    let icon = if counts.failed > 0 { "✗" } else { "✓" };
    let elapsed = format_duration(elapsed);
    if counts.failed > 0 {
        return format!(
            "{icon} progress: {}/{} done, {} failed, {} skipped [{}]",
            counts.done(),
            counts.total,
            counts.failed,
            counts.skipped,
            elapsed
        );
    }

    format!(
        "{icon} progress: {}/{} done, {} skipped [{}]",
        counts.done(),
        counts.total,
        counts.skipped,
        elapsed
    )
}

impl ProgressObserver for NonTtyProgressObserver {
    fn on_dag_start(&mut self, snapshot: &DagSnapshot) {
        self.labels = snapshot.labels.clone();
        self.states = snapshot
            .node_ids
            .iter()
            .map(|id| (id.clone(), NonTtyNodeState::Pending))
            .collect();
        // Initialize group tracking
        self.groups = snapshot.groups.clone();
        self.groups_started = vec![false; self.groups.len()];
        self.group_map = HashMap::new();
        for (idx, group) in self.groups.iter().enumerate() {
            for node_id in &group.node_ids {
                self.group_map.insert(node_id.clone(), idx);
            }
        }
    }

    fn on_node_start(&mut self, node_id: &NodeId) {
        self.set_state(node_id, NonTtyNodeState::Running);
        self.maybe_emit_group_header(node_id);
        eprintln!("→ {}...", self.label_for(node_id));
    }

    fn on_node_complete(&mut self, node_id: &NodeId, _summary: OutputSummary) {
        self.set_state(node_id, NonTtyNodeState::Completed);
        eprintln!("✓ {}", self.label_for(node_id));
        self.maybe_emit_group_summary(node_id);
    }

    fn on_node_failed(&mut self, node_id: &NodeId, error: &str) {
        self.set_state(node_id, NonTtyNodeState::Failed);
        let label = self.label_for(node_id).to_string();
        // Save first line for the dag_complete summary
        let first_line = error.lines().next().unwrap_or(error).to_string();
        self.failures.push((label.clone(), first_line));
        eprintln!("✗ {}: {}", label, error);
        // Print boxed failure detail, capped at FAILURE_DETAIL_LINES
        eprintln!();
        eprintln!("  ┌─ [ERROR] {}", label);
        let lines: Vec<&str> = error.lines().collect();
        let display_lines = lines.len().min(FAILURE_DETAIL_LINES);
        for line in &lines[..display_lines] {
            eprintln!("  │ {}", line);
        }
        if lines.len() > FAILURE_DETAIL_LINES {
            eprintln!(
                "  │ ... ({} more lines)",
                lines.len() - FAILURE_DETAIL_LINES
            );
        }
        eprintln!("  └─");
        self.maybe_emit_group_summary(node_id);
    }

    fn on_node_skipped(&mut self, node_id: &NodeId) {
        self.set_state(node_id, NonTtyNodeState::Skipped);
        self.maybe_emit_group_summary(node_id);
    }

    fn on_node_intercepted(&mut self, node_id: &NodeId, _summary: OutputSummary) {
        self.set_state(node_id, NonTtyNodeState::Intercepted);
        eprintln!("✓ {} [dry-run]", self.label_for(node_id));
        self.maybe_emit_group_summary(node_id);
    }

    fn on_dag_complete(&mut self, elapsed: Duration) {
        let counts = self.counts();
        eprintln!("{}", format_non_tty_summary_line(counts, elapsed));

        // Print failure summary listing all failed nodes with error first lines
        if !self.failures.is_empty() {
            eprintln!();
            for (label, first_line) in &self.failures {
                eprintln!("  ┌─ [FAILED] {}", label);
                eprintln!("  │ {}", first_line);
                eprintln!("  └─");
            }
        }
    }
}

impl ProgressObserver for SharedProgressObserver {
    fn on_dag_start(&mut self, snapshot: &DagSnapshot) {
        self.with_progress(|progress| progress.on_dag_start(snapshot));
    }

    fn on_node_start(&mut self, node_id: &NodeId) {
        self.with_progress(|progress| progress.on_node_start(node_id));
    }

    fn on_node_complete(&mut self, node_id: &NodeId, summary: OutputSummary) {
        self.with_progress(|progress| progress.on_node_complete(node_id, summary));
    }

    fn on_node_failed(&mut self, node_id: &NodeId, error: &str) {
        self.with_progress(|progress| progress.on_node_failed(node_id, error));
    }

    fn on_node_skipped(&mut self, node_id: &NodeId) {
        self.with_progress(|progress| progress.on_node_skipped(node_id));
    }

    fn on_node_intercepted(&mut self, node_id: &NodeId, summary: OutputSummary) {
        self.with_progress(|progress| progress.on_node_intercepted(node_id, summary));
    }

    fn on_dag_complete(&mut self, elapsed: Duration) {
        self.with_progress(|progress| progress.on_dag_complete(elapsed));
    }
}

fn render_progress_frame(
    progress: &DagProgress,
    layout: &gunbc_ir::layout::DagLayout,
    spinner: &Animation,
    writer: &mut FrameWriter,
    stdout: &mut io::Stdout,
    profile: &TerminalProfile,
) {
    let frame = build_frame(
        progress,
        layout,
        RenderMode::Standard,
        spinner.frame(),
        profile.tier,
        &STANDARD,
    );
    let _ = writer.write_frame(&frame, stdout);
}

fn initial_preflight_snapshot() -> DagSnapshot {
    let node_id = NodeId::from("preflight_check");
    DagSnapshot {
        node_ids: vec![node_id.clone()],
        edges: Vec::new(),
        topo_order: vec![node_id.clone()],
        boundary_nodes: Vec::new(),
        labels: HashMap::from([(node_id.clone(), "preflight".to_string())]),
        groups: Vec::new(),
    }
}

fn preflight_snapshot(total_steps: usize) -> DagSnapshot {
    let mut node_ids = Vec::new();
    let mut edges = Vec::new();
    let mut labels = HashMap::new();

    for step in 1..=total_steps {
        let id = SharedPreflightObserver::step_node_id(step);
        labels.insert(id.clone(), format!("step {}", step));
        if step > 1 {
            let prev = SharedPreflightObserver::step_node_id(step - 1);
            edges.push(gunbc_ir::Edge::new(
                prev.0.clone(),
                "out",
                id.0.clone(),
                "in",
            ));
        }
        node_ids.push(id);
    }

    let groups = if node_ids.len() > 1 {
        vec![StageGroup {
            name: "preflight".to_string(),
            node_ids: node_ids.clone(),
        }]
    } else {
        Vec::new()
    };

    DagSnapshot {
        topo_order: node_ids.clone(),
        node_ids,
        edges,
        boundary_nodes: Vec::new(),
        labels,
        groups,
    }
}

/// Print a single output value in the standard format.
///
/// Uses `Value::display_redacted_truncated()` as the single chokepoint
/// for rendering values. Port-name-specific formatting (suppressing empty
/// stderr/stdout, short string inline) is layered on top.
pub fn print_value(port: &str, value: &Value) {
    match value {
        Value::Skipped | Value::Unit => {}
        Value::Str(s) => {
            // Suppress empty stderr/stdout
            if (port.ends_with("stderr") || port.ends_with("stdout")) && s.is_empty() {
                return;
            }
            // Short single-line strings inline
            if !s.contains('\n') && s.len() < 120 {
                println!("  {}: {}", port, value.display_redacted());
                return;
            }
            // Everything else through the truncating chokepoint
            let rendered = value.display_redacted_truncated(MAX_LOG_VALUE_LINES, MAX_LINE_WIDTH);
            println!("  {}: {}", port, rendered);
        }
        _ => {
            let rendered = value.display_redacted();
            println!("  {}: {}", port, rendered);
        }
    }
}

/// Print outputs for DAG boundary ports (terminal outputs).
fn print_boundary_outputs(log: &crate::ExecutionLog, boundaries: &gunbc_ir::BoundaryInfo) {
    if boundaries.boundary_ports.is_empty() {
        return;
    }

    println!();
    println!("Outputs:");

    for (node_id, port_name) in &boundaries.boundary_ports {
        let entry = log.get(&node_id.0);
        let value = entry.and_then(|e| e.outputs.get(&port_name.0));
        if let Some(value) = value {
            let label = format!("{}.{}", node_id.0, port_name.0);
            print_value(&label, value);
        }
    }
}

/// Returns true when `success_port` is present and emits `false` in any log entry.
fn success_port_failed(log: &crate::ExecutionLog, success_port: Option<&str>) -> bool {
    let Some(port) = success_port else {
        return false;
    };

    log.entries
        .iter()
        .any(|entry| matches!(entry.outputs.get(port), Some(Value::Bool(false))))
}

/// Print a high-signal attention block.
///
/// TTY with color uses a boxed section keyed by severity color.
/// Non-TTY uses a compact plain fallback.
pub fn print_attention(level: AttentionLevel, title: &str, body: &str) {
    let (label, color) = match level {
        AttentionLevel::Info => ("INFO", ANSI_BLUE),
        AttentionLevel::Warning => ("WARNING", ANSI_YELLOW),
        AttentionLevel::Error => ("ERROR", ANSI_RED),
    };
    let lines: Vec<&str> = body.lines().collect();
    let use_color = std::io::stdout().is_terminal() && std::env::var("NO_COLOR").is_err();
    if use_color {
        eprintln!();
        eprintln!("  {color}┌─ [{label}] {title}{ANSI_RESET}");
        if lines.is_empty() {
            eprintln!("  {color}│{ANSI_RESET} ");
        } else {
            for line in &lines {
                eprintln!("  {color}│{ANSI_RESET} {line}");
            }
        }
        eprintln!("  {color}└─{ANSI_RESET}");
        return;
    }

    eprintln!();
    eprintln!("{label}: {title}");
    if lines.is_empty() {
        eprintln!("  (no details)");
        return;
    }
    for line in &lines {
        eprintln!("  {line}");
    }
}

// ---------------------------------------------------------------------------
// PreflightObserver
// ---------------------------------------------------------------------------

/// Trait for receiving preflight execution events.
///
/// Abstracts the output of preflight operations (lint-upsert, codegen, etc.)
/// so they can emit structured output in CI, status lines in non-TTY, or
/// nothing at all in library usage.
pub trait PreflightObserver: Send {
    /// Called when preflight begins.
    fn on_preflight_start(&mut self, name: &str);

    /// Called when a preflight step begins.
    fn on_preflight_step(&mut self, step: usize, total: usize, label: &str);

    /// Called when a preflight step completes successfully.
    fn on_preflight_step_complete(&mut self, label: &str, elapsed: Duration);

    /// Called when all preflight steps complete.
    fn on_preflight_complete(&mut self, name: &str, elapsed: Duration);

    /// Called when preflight fails.
    fn on_preflight_error(&mut self, name: &str, error: &str);
}

/// Status-line preflight observer for non-interactive environments.
///
/// Emits `→ step...` / `✓ step [0.5s]` style status lines to stderr.
pub struct PreflightStatusObserver;

impl PreflightObserver for PreflightStatusObserver {
    fn on_preflight_start(&mut self, name: &str) {
        eprintln!("--- {} ---", name);
    }

    fn on_preflight_step(&mut self, step: usize, total: usize, label: &str) {
        eprint!("  [{}/{}] {}...", step, total, label);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    fn on_preflight_step_complete(&mut self, _label: &str, elapsed: Duration) {
        eprintln!(" {:.1}s", elapsed.as_secs_f64());
    }

    fn on_preflight_complete(&mut self, name: &str, elapsed: Duration) {
        eprintln!("✓ {} [{:.1}s]", name, elapsed.as_secs_f64());
    }

    fn on_preflight_error(&mut self, name: &str, error: &str) {
        let first_line = error.lines().next().unwrap_or(error);
        eprintln!("✗ {}: {}", name, first_line);
    }
}

impl PreflightObserver for SharedPreflightObserver {
    fn on_preflight_start(&mut self, _name: &str) {
        self.ensure_started();
    }

    fn on_preflight_step(&mut self, step: usize, total: usize, label: &str) {
        self.ensure_initialized_steps(total);
        let node_id = Self::step_node_id(step);
        self.with_progress(|progress| {
            if let Some(existing_label) = progress.snapshot.labels.get_mut(&node_id) {
                *existing_label = label.to_string();
            }
            progress.on_node_start(&node_id);
        });
        self.current_step = Some(step);
    }

    fn on_preflight_step_complete(&mut self, _label: &str, elapsed: Duration) {
        let step = self.current_step.unwrap_or(1);
        let node_id = Self::step_node_id(step);
        self.with_progress(|progress| {
            progress.on_node_complete(
                &node_id,
                OutputSummary {
                    fields: Vec::new(),
                    elapsed,
                },
            );
        });
    }

    fn on_preflight_complete(&mut self, _name: &str, elapsed: Duration) {
        // Fresh-state preflight can complete without explicit steps.
        if !self.initialized_steps {
            self.with_progress(|progress| {
                progress.on_node_complete(
                    &NodeId::from("preflight_check"),
                    OutputSummary {
                        fields: Vec::new(),
                        elapsed,
                    },
                );
            });
        }
        self.with_progress(|progress| progress.on_dag_complete(elapsed));
    }

    fn on_preflight_error(&mut self, _name: &str, error: &str) {
        let node_id = match self.current_step {
            Some(step) => Self::step_node_id(step),
            None => NodeId::from("preflight_check"),
        };
        self.with_progress(|progress| progress.on_node_failed(&node_id, error));
    }
}

/// Maximum lines to display for a single port value in log output.
const MAX_LOG_VALUE_LINES: usize = 40;

/// Maximum characters per line before truncation.
const MAX_LINE_WIDTH: usize = 500;

/// Maximum lines to show for a single failure detail in NonTty mode.
const FAILURE_DETAIL_LINES: usize = 30;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_non_tty_observer_counts_track_states() {
        let a = NodeId::from("a");
        let b = NodeId::from("b");
        let c = NodeId::from("c");

        let snapshot = DagSnapshot {
            node_ids: vec![a.clone(), b.clone(), c.clone()],
            edges: Vec::new(),
            topo_order: vec![a.clone(), b.clone(), c.clone()],
            boundary_nodes: Vec::new(),
            labels: HashMap::from([
                (a.clone(), "A".to_string()),
                (b.clone(), "B".to_string()),
                (c.clone(), "C".to_string()),
            ]),
            groups: vec![],
        };

        let mut observer = NonTtyProgressObserver::default();
        observer.on_dag_start(&snapshot);
        observer.on_node_start(&a);
        observer.on_node_complete(
            &a,
            OutputSummary {
                fields: Vec::new(),
                elapsed: Duration::ZERO,
            },
        );
        observer.on_node_start(&b);
        observer.on_node_failed(&b, "boom");
        observer.on_node_skipped(&c);

        let counts = observer.counts();
        assert_eq!(counts.total, 3);
        assert_eq!(counts.done(), 1);
        assert_eq!(counts.failed, 1);
        assert_eq!(counts.skipped, 1);
        assert_eq!(observer.failed_count(), 1);
    }

    #[test]
    fn test_non_tty_summary_line_success() {
        let line = format_non_tty_summary_line(
            NonTtyProgressCounts {
                total: 5,
                completed: 4,
                intercepted: 1,
                ..Default::default()
            },
            Duration::from_secs(12),
        );
        assert!(line.starts_with("✓ progress: 5/5 done"));
        assert!(line.contains("[12.0s]"));
    }

    #[test]
    fn test_non_tty_summary_line_failure() {
        let line = format_non_tty_summary_line(
            NonTtyProgressCounts {
                total: 5,
                completed: 2,
                failed: 1,
                skipped: 2,
                ..Default::default()
            },
            Duration::from_millis(850),
        );
        assert!(line.starts_with("✗ progress: 2/5 done, 1 failed, 2 skipped"));
        assert!(line.contains("[850ms]"));
    }

    // -------------------------------------------------------------------
    // Phase 6: NonTty group-aware rendering tests
    // -------------------------------------------------------------------

    #[test]
    fn test_non_tty_observer_group_tracking() {
        use crate::progress::StageGroup;

        let snapshot = DagSnapshot {
            node_ids: vec![
                NodeId::from("prepare_build"),
                NodeId::from("execute_build"),
                NodeId::from("parse_build"),
            ],
            edges: Vec::new(),
            topo_order: vec![
                NodeId::from("prepare_build"),
                NodeId::from("execute_build"),
                NodeId::from("parse_build"),
            ],
            boundary_nodes: Vec::new(),
            labels: HashMap::from([
                (NodeId::from("prepare_build"), "prepare_build".to_string()),
                (NodeId::from("execute_build"), "execute_build".to_string()),
                (NodeId::from("parse_build"), "parse_build".to_string()),
            ]),
            groups: vec![StageGroup {
                name: "build".into(),
                node_ids: vec![
                    NodeId::from("prepare_build"),
                    NodeId::from("execute_build"),
                    NodeId::from("parse_build"),
                ],
            }],
        };

        let mut observer = NonTtyProgressObserver::default();
        observer.on_dag_start(&snapshot);

        // Group tracking should be initialized
        assert_eq!(observer.groups.len(), 1);
        assert_eq!(observer.group_map.len(), 3);
        assert!(!observer.groups_started[0]);

        // Starting a node in the group should mark group as started
        observer.on_node_start(&NodeId::from("prepare_build"));
        assert!(observer.groups_started[0]);

        // Group should not be done yet
        assert!(!observer.is_group_done(0));

        // Complete all nodes in the group
        observer.on_node_complete(
            &NodeId::from("prepare_build"),
            OutputSummary {
                fields: Vec::new(),
                elapsed: Duration::ZERO,
            },
        );
        observer.on_node_start(&NodeId::from("execute_build"));
        observer.on_node_complete(
            &NodeId::from("execute_build"),
            OutputSummary {
                fields: Vec::new(),
                elapsed: Duration::ZERO,
            },
        );
        observer.on_node_start(&NodeId::from("parse_build"));
        observer.on_node_complete(
            &NodeId::from("parse_build"),
            OutputSummary {
                fields: Vec::new(),
                elapsed: Duration::ZERO,
            },
        );

        // Group should now be done
        assert!(observer.is_group_done(0));
    }

    #[test]
    fn test_non_tty_observer_failure_tracking() {
        let snapshot = DagSnapshot {
            node_ids: vec![NodeId::from("a"), NodeId::from("b")],
            edges: Vec::new(),
            topo_order: vec![NodeId::from("a"), NodeId::from("b")],
            boundary_nodes: Vec::new(),
            labels: HashMap::from([
                (NodeId::from("a"), "A".to_string()),
                (NodeId::from("b"), "B".to_string()),
            ]),
            groups: vec![],
        };

        let mut observer = NonTtyProgressObserver::default();
        observer.on_dag_start(&snapshot);
        observer.on_node_start(&NodeId::from("a"));
        observer.on_node_failed(&NodeId::from("a"), "error line 1\nerror line 2");

        assert_eq!(observer.failures.len(), 1);
        assert_eq!(observer.failures[0].0, "A");
        assert_eq!(observer.failures[0].1, "error line 1");
    }
}
