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
//!   true  → channel-driven event loop: executor on background thread,
//!           main thread owns DagProgress + renders on tick/event
//!   false → observer-driven status lines + boundary outputs
//!           (CI environments compose CiGroupObserver for workflow annotations)
//! ```

use crate::box_draw;
use crate::error::{ErrorLayer, FailureDetail};
use crate::frame_build::{build_frame, format_duration};
use crate::frame_write::FrameWriter;
use crate::intercept::BoundaryMocks;
use crate::progress::{
    ComposedObserver, DagPhase, DagProgress, DagSnapshot, ExecutionEvent, OutputSummary,
    ProgressObserver, StageGroup,
};
use crate::render::{Animation, RenderMode};
use crate::terminal::TerminalProfile;
use crate::{
    execute_with_progress_and_mode_and_inputs, lower, topo_sort, ExecError, Executable,
    ExecutionMode, NodeState,
};
use gunbc_ir::layout::compute_layout;
use gunbc_ir::symbols::{SemanticColor, SymbolId, Tier, STANDARD};
use gunbc_ir::{
    detect_boundaries, Dag, NodeId, Value, HUMAN_TEXT_MAX_LINES, HUMAN_TEXT_MAX_LINE_WIDTH,
};
use std::collections::HashMap;
use std::io::{self, IsTerminal, Write};
use std::process;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

const PROGRESS_TICK: Duration = Duration::from_millis(80);

/// Minimum time to show the progress display before clearing.
///
/// Prevents visual flicker for fast operations where the progress display
/// appears and disappears too quickly to read. Matches `gunb.ai`'s
/// `SPINNER_MIN_SECONDS` behavior.
const MIN_DISPLAY_DURATION: Duration = Duration::from_millis(200);

/// All 10 braille spinner symbol IDs (matching gunb.ai's progressSpinnerFrames).
const SPINNER_SYMBOL_IDS: [SymbolId; 10] = [
    SymbolId::Spinner0,
    SymbolId::Spinner1,
    SymbolId::Spinner2,
    SymbolId::Spinner3,
    SymbolId::Spinner4,
    SymbolId::Spinner5,
    SymbolId::Spinner6,
    SymbolId::Spinner7,
    SymbolId::Spinner8,
    SymbolId::Spinner9,
];

/// Resolve spinner frames for the given tier.
fn resolve_spinner_frames(tier: Tier) -> Vec<String> {
    SPINNER_SYMBOL_IDS
        .iter()
        .map(|id| STANDARD.resolve_tier(*id, tier).to_string())
        .collect()
}

/// High-signal attention levels for user-facing terminal messaging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionLevel {
    Info,
    Warning,
    Error,
}

/// Display surface mode for DAG execution output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Animated,
    Plain,
    CiPlain,
}

/// Display verbosity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayVerbosity {
    Normal,
    Verbose,
}

/// Unified display configuration across all execution paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayConfig {
    pub mode: DisplayMode,
    pub verbosity: DisplayVerbosity,
}

impl DisplayConfig {
    /// Resolve display configuration from runtime hints.
    pub fn from_runtime(animated_hint: bool) -> Self {
        Self::from_surface(animated_hint, is_ci_environment())
    }

    fn from_surface(animated_hint: bool, ci_environment: bool) -> Self {
        let mode = if animated_hint {
            DisplayMode::Animated
        } else if ci_environment {
            DisplayMode::CiPlain
        } else {
            DisplayMode::Plain
        };
        Self {
            mode,
            verbosity: DisplayVerbosity::Normal,
        }
    }
}

/// Preamble header displayed before DAG execution begins.
///
/// Rendered as a box with the tool name and a short description.
/// Matches `gunb.ai`'s preamble box style.
#[derive(Debug, Clone, Default)]
pub struct Preamble {
    /// Tool name (e.g., "gist", "ci").
    pub title: String,
    /// Short description of what this tool does.
    pub description: String,
    /// Additional body lines rendered inside the box (e.g., args like "repo_path: .").
    pub body_lines: Vec<String>,
}

impl Preamble {
    /// Create a new preamble.
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            body_lines: Vec::new(),
        }
    }

    /// Create a preamble with body lines (args displayed inside the box).
    pub fn with_body(
        title: impl Into<String>,
        description: impl Into<String>,
        body_lines: Vec<String>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            body_lines,
        }
    }
}

/// Print a preamble box to stderr.
///
/// Uses the box drawing module with `Accent` color for the border.
/// Rendered in both TTY and non-TTY modes (non-TTY gets a plain fallback).
pub fn print_preamble(preamble: &Preamble, tier: Tier, use_color: bool) {
    if preamble.title.is_empty() {
        return;
    }
    let b = box_draw::preamble_box(&preamble.title, tier, use_color);
    let mut stderr = io::stderr();

    // Collect all lines: description first, then body_lines (args).
    let mut lines: Vec<String> = Vec::new();
    if !preamble.description.is_empty() {
        lines.push(preamble.description.clone());
    }
    for line in &preamble.body_lines {
        lines.push(format!("  {}", line));
    }

    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let _ = b.render(&mut stderr, &refs);
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
pub fn execute_and_display<T: Executable + Clone + Send + 'static>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    animated: bool,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) {
    let config = DisplayConfig::from_runtime(animated);
    match execute_and_display_with_result_config(dag, mode, config, success_port, input_mocks) {
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

/// Detect terminal capabilities and print a preamble box.
///
/// Returns `(animated, tier, use_color)` for use by callers that need
/// to make further display decisions.
pub fn print_preamble_auto(preamble: &Preamble) -> bool {
    let profile = TerminalProfile::detect();
    let animated = profile.is_tty && !is_ci_environment();
    print_preamble(preamble, profile.tier, profile.supports_color);
    animated
}

/// Execute a DAG through the shared display path and return execution results.
///
/// Unlike [`execute_and_display`], this function never exits the process.
pub fn execute_and_display_with_result<T: Executable + Clone + Send + 'static>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    animated: bool,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<DisplayResult, ExecError> {
    let config = DisplayConfig::from_runtime(animated);
    execute_and_display_with_result_config(dag, mode, config, success_port, input_mocks)
}

/// Execute a DAG through the shared display path using an explicit display config.
pub fn execute_and_display_with_result_config<T: Executable + Clone + Send + 'static>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    config: DisplayConfig,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<DisplayResult, ExecError> {
    match config.mode {
        DisplayMode::Animated => run_with_progress(dag, mode, config, success_port, input_mocks),
        DisplayMode::Plain | DisplayMode::CiPlain => {
            run_plain(dag, mode, config, success_port, input_mocks)
        }
    }
}

/// Plain execution: observer-driven status lines + boundary outputs.
///
/// Unified path for all non-interactive environments. When in CI, composes
/// the status observer with a `CiContext` observer for workflow commands
/// (groups, error annotations, secret masking).
fn run_plain<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    config: DisplayConfig,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<DisplayResult, ExecError> {
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {}", e)))?;
    let boundaries = detect_boundaries(&flat.dag);

    let mut status_observer = NonTtyProgressObserver::default();

    let is_ci = matches!(config.mode, DisplayMode::CiPlain);

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
    // Only check specific provider markers. The generic CI env var is
    // unreliable (set by editors, tools, etc.).
    std::env::var("GITHUB_ACTIONS").is_ok() || std::env::var("GITLAB_CI").is_ok()
}

/// Mask secret values from an execution log via CI workflow commands.
///
/// Iterates through all log entries and emits `::add-mask::` for each
/// secret value so CI runners redact them from all subsequent output.
#[allow(clippy::disallowed_methods)] // Approved: CI secret masking at transport boundary
fn mask_secrets_in_log(ci: &mut crate::CiContext, log: &crate::ExecutionLog) {
    for entry in &log.entries {
        for value in entry.outputs.values() {
            if let Value::Secret(s) = value {
                ci.mask(s.expose_plaintext_for_transport());
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

/// Progress-display execution: channel-driven event loop on main thread.
///
/// The executor runs on a background thread and sends events via channel.
/// The main thread owns `DagProgress` directly, renders frames on each tick or
/// event, and stops when it sees a terminal phase or channel disconnect.
///
/// No shared mutex, no AtomicBool, no render thread.
fn run_with_progress<T: Executable + Clone + Send + 'static>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    _config: DisplayConfig,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<DisplayResult, ExecError> {
    let display_start = Instant::now();
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
    let (event_tx, event_rx) = mpsc::channel();

    // Clone dag and mocks so the background executor thread owns them.
    // This avoids requiring T: Sync and BoundaryMocks: Sync.
    let dag_owned = dag.clone();
    let mocks_owned = input_mocks.cloned();

    // Executor on background thread, orchestrator loop on main thread.
    let exec_handle = thread::spawn(move || {
        let mut observer = ChannelObserver::new(event_tx);
        execute_with_progress_and_mode_and_inputs(
            &dag_owned,
            mode,
            &mut observer,
            mocks_owned.as_ref(),
        )
    });

    // Orchestrator loop on main thread
    let _cursor_guard = crate::frame_write::CursorGuard::new(profile.is_tty);
    let spinner_frames = resolve_spinner_frames(profile.tier);
    let mut spinner = Animation::cycle(spinner_frames, PROGRESS_TICK);
    let mut writer = FrameWriter::new(
        profile.supports_color,
        profile.tier,
        &STANDARD,
        profile.is_tty,
    );
    let mut stderr = io::stderr();
    let mut last_tick = Instant::now();
    let mut progress = DagProgress::new(snapshot);

    loop {
        match event_rx.recv_timeout(PROGRESS_TICK) {
            Ok(event) => progress.apply(event),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }

        // Skip rendering until the DAG is actually running — avoids
        // flashing a useless "DAG pending" frame.
        if matches!(progress.phase, DagPhase::NotStarted) {
            continue;
        }

        let now = Instant::now();
        spinner.tick(now.saturating_duration_since(last_tick));
        last_tick = now;

        render_progress_frame(
            &progress,
            &layout,
            &spinner,
            &mut writer,
            &mut stderr,
            &profile,
        );

        if matches!(
            progress.phase,
            DagPhase::Failed { .. } | DagPhase::Completed { .. }
        ) {
            // Drain remaining events (e.g. DagComplete after Failed)
            while let Ok(event) = event_rx.try_recv() {
                progress.apply(event);
            }
            break;
        }
    }

    // Anti-flicker: if execution was very fast, wait so the final frame is visible
    let elapsed_display = display_start.elapsed();
    if elapsed_display < MIN_DISPLAY_DURATION {
        thread::sleep(MIN_DISPLAY_DURATION - elapsed_display);
    }

    let last_lines = writer.last_frame_lines();

    // CursorGuard drops here (restores cursor)
    drop(_cursor_guard);

    // Wait for executor to finish (workers may still be draining after failure)
    let log_result = exec_handle.join().unwrap();

    // Render a clean final frame with static icons (no spinner animation),
    // seeded with the last animated frame's line count for seamless overwrite.
    // This MUST happen before the `?` so the display is cleaned up even on failure.
    render_final_static_frame_seeded(&progress, &layout, &profile, last_lines);

    let log = log_result?;

    // Check final node states for hard failures
    let mut should_fail = progress
        .nodes
        .values()
        .any(|np| np.state == NodeState::Failed);
    should_fail = should_fail || success_port_failed(&log, success_port);

    // Render error detail boxes for failed nodes
    print_error_boxes(&progress, profile.tier, profile.supports_color);

    // Surface boundary outputs after progress render so users see
    // the actual tool results (e.g., gist URL) instead of only the DAG view.
    print_boundary_outputs(&log, &boundaries);

    Ok(DisplayResult { log, should_fail })
}

/// Observer that sends events to a channel instead of mutating shared state.
///
/// Send errors are intentionally ignored — the orchestrator may have already
/// stopped reading after seeing a terminal phase.
struct ChannelObserver {
    tx: mpsc::Sender<ExecutionEvent>,
}

impl ChannelObserver {
    fn new(tx: mpsc::Sender<ExecutionEvent>) -> Self {
        Self { tx }
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
    /// Track failed nodes with their structured failure details for the dag_complete summary.
    failures: Vec<(String, FailureDetail)>,
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

    fn on_node_failed(&mut self, node_id: &NodeId, error: &ExecError) {
        self.set_state(node_id, NonTtyNodeState::Failed);
        let label = self.label_for(node_id).to_string();
        let detail = error.to_failure_detail();
        let classification = error.classification();
        let msg = error.to_string();

        // Summary line with classification tag and optional service label
        if let Some(svc) = error.service_label() {
            eprintln!("✗ {} [{}] ({}): {}", label, classification, svc, msg);
        } else {
            eprintln!("✗ {} [{}]: {}", label, classification, msg);
        }

        // Print boxed failure detail, capped at FAILURE_DETAIL_LINES
        eprintln!();
        eprintln!("  ┌─ [ERROR] {}", label);

        // Render layer context lines
        for layer in error.layers() {
            match layer {
                ErrorLayer::Service(s) => {
                    eprintln!("  │ Service: {}.{}", s.provider, s.operation);
                }
                ErrorLayer::Http(h) => {
                    let reason = h.reason.as_deref().unwrap_or("");
                    eprintln!("  │ Http: {} {}", h.status_code, reason);
                }
                ErrorLayer::Rest(r) => {
                    eprintln!("  │ Rest: {} {}", r.method, r.endpoint);
                }
                ErrorLayer::Auth(a) => {
                    if let Some(ref cred) = a.credential_ref {
                        eprintln!("  │ Auth: {} (credential: {})", a.scheme, cred);
                    } else {
                        eprintln!("  │ Auth: {}", a.scheme);
                    }
                }
                ErrorLayer::Shell(s) => {
                    if let Some(code) = s.exit_code {
                        eprintln!("  │ Shell: {} (exit {})", s.command, code);
                    } else {
                        eprintln!("  │ Shell: {}", s.command);
                    }
                }
                ErrorLayer::File(f) => {
                    eprintln!("  │ File: {} ({})", f.path, f.operation);
                }
            }
        }

        let lines: Vec<&str> = msg.lines().collect();
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

        self.failures.push((label, detail));
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

        // Print failure summary listing all failed nodes with classification
        if !self.failures.is_empty() {
            eprintln!();
            for (label, detail) in &self.failures {
                let first_line = detail
                    .message
                    .lines()
                    .next()
                    .unwrap_or(&detail.message);
                let tag = detail.classification();
                eprintln!("  ┌─ [FAILED] {} [{}]", label, tag);
                eprintln!("  │ {}", first_line);
                eprintln!("  └─");
            }
        }
    }
}

impl ProgressObserver for ChannelObserver {
    fn on_dag_start(&mut self, snapshot: &DagSnapshot) {
        let _ = self.tx.send(ExecutionEvent::DagStart(snapshot.clone()));
    }

    fn on_node_start(&mut self, node_id: &NodeId) {
        let _ = self.tx.send(ExecutionEvent::NodeStart(node_id.clone()));
    }

    fn on_node_complete(&mut self, node_id: &NodeId, summary: OutputSummary) {
        let _ = self
            .tx
            .send(ExecutionEvent::NodeComplete(node_id.clone(), summary));
    }

    fn on_node_failed(&mut self, node_id: &NodeId, error: &ExecError) {
        let _ = self.tx.send(ExecutionEvent::NodeFailed(
            node_id.clone(),
            error.to_failure_detail(),
        ));
    }

    fn on_node_skipped(&mut self, node_id: &NodeId) {
        let _ = self.tx.send(ExecutionEvent::NodeSkipped(node_id.clone()));
    }

    fn on_node_intercepted(&mut self, node_id: &NodeId, summary: OutputSummary) {
        let _ = self
            .tx
            .send(ExecutionEvent::NodeIntercepted(node_id.clone(), summary));
    }

    fn on_dag_complete(&mut self, elapsed: Duration) {
        let _ = self.tx.send(ExecutionEvent::DagComplete(elapsed));
    }
}

/// Render a clean final frame with static icons (no spinner animation).
///
/// Called after the render loop stops. Uses an empty `spinner_frame` so
/// `build_frame` resolves to static checkmarks / X marks instead of animated
/// braille dots.
///
/// `seed_lines` is the line count of the last animated frame. The writer is
/// seeded with this value so the final frame cursor-ups over the animated
/// frame and overwrites it cleanly.
/// Returns the line count of the final frame written.
fn render_final_static_frame_seeded(
    progress: &DagProgress,
    layout: &gunbc_ir::layout::DagLayout,
    profile: &TerminalProfile,
    seed_lines: usize,
) -> usize {
    let frame = build_frame(
        progress,
        layout,
        RenderMode::Standard,
        "", // empty → static icons
        profile.tier,
        &STANDARD,
        Some(profile.viewport.width as usize),
    );
    let line_count = frame.lines.len();

    let mut writer = FrameWriter::new(
        profile.supports_color,
        profile.tier,
        &STANDARD,
        profile.is_tty,
    );
    writer.seed_last_frame_lines(seed_lines);
    let _ = writer.write_frame(&frame, &mut io::stderr());
    line_count
}

fn render_progress_frame(
    progress: &DagProgress,
    layout: &gunbc_ir::layout::DagLayout,
    spinner: &Animation,
    writer: &mut FrameWriter,
    sink: &mut dyn io::Write,
    profile: &TerminalProfile,
) {
    let frame = build_frame(
        progress,
        layout,
        RenderMode::Standard,
        spinner.frame(),
        profile.tier,
        &STANDARD,
        Some(profile.viewport.width as usize),
    );
    let _ = writer.write_frame(&frame, sink);
}

/// Print a single output value in the standard format.
///
/// Uses `Value::display_redacted_truncated()` as the single chokepoint
/// for rendering values. Port-name-specific formatting (suppressing empty
/// stderr/stdout, short string inline) is layered on top.
pub fn print_value(port: &str, value: &Value) {
    if let Some(rendered) = render_value_for_port(port, value) {
        println!("  {}: {}", port, rendered);
    }
}

fn render_value_for_port(port: &str, value: &Value) -> Option<String> {
    match value {
        Value::Skipped | Value::Unit => None,
        Value::Str(s) => {
            // Suppress empty stderr/stdout
            if (port.ends_with("stderr") || port.ends_with("stdout")) && s.is_empty() {
                return None;
            }
            // Short single-line strings inline
            if !s.contains('\n') && s.len() < 120 {
                return Some(value.display_redacted());
            }
            // Everything else through the truncating chokepoint
            Some(value.display_redacted_truncated(MAX_LOG_VALUE_LINES, MAX_LINE_WIDTH))
        }
        _ => Some(value.display_redacted()),
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

/// Returns true when any log entry emits a `success_port` value that is not
/// explicitly `Bool(true)`.  `Skipped`, `Bool(false)`, or any other variant
/// all count as failure — the success port must affirmatively be `true`.
fn success_port_failed(log: &crate::ExecutionLog, success_port: Option<&str>) -> bool {
    let Some(port) = success_port else {
        return false;
    };

    log.entries.iter().any(|entry| {
        match entry.outputs.get(port) {
            Some(Value::Bool(true)) => false,  // explicitly passed
            Some(_) => true,                   // Bool(false), Skipped, or unexpected type
            None => false,                     // port not on this node
        }
    })
}

/// Render error detail boxes for all failed nodes in the DAG.
///
/// Each failed node gets an open-right box with `Error` color border and
/// `Dim` content text. Renders structured layer context (Service, Http, Rest,
/// Auth, Shell, File) when available, plus a classification tag. Error text
/// is truncated to [`ERROR_OUTPUT_MAX_LINES`] lines.
///
/// [`ERROR_OUTPUT_MAX_LINES`]: crate::box_draw::ERROR_OUTPUT_MAX_LINES
pub fn print_error_boxes(progress: &DagProgress, tier: Tier, use_color: bool) {
    let failures = progress.failed_nodes();
    if failures.is_empty() {
        return;
    }

    let mut stderr = io::stderr();
    let _ = writeln!(stderr);

    for (label, detail) in &failures {
        // Build a label that includes service label and classification if available
        let box_label = match detail.service_label() {
            Some(svc) => format!("{} ({}) [{}]", label, svc, detail.classification()),
            None => {
                let tag = detail.classification();
                if tag != "UNKNOWN" {
                    format!("{} [{}]", label, tag)
                } else {
                    label.clone()
                }
            }
        };
        let b = box_draw::error_box(&box_label, tier, use_color);

        // Build content lines: layer context first, then error message
        let mut content_lines: Vec<String> = Vec::new();

        for layer in &detail.layers {
            match layer {
                ErrorLayer::Service(s) => {
                    content_lines.push(format!("Service: {}.{}", s.provider, s.operation));
                }
                ErrorLayer::Http(h) => {
                    let reason = h.reason.as_deref().unwrap_or("");
                    content_lines.push(format!("Http: {} {}", h.status_code, reason));
                }
                ErrorLayer::Rest(r) => {
                    content_lines.push(format!("Rest: {} {}", r.method, r.endpoint));
                }
                ErrorLayer::Auth(a) => {
                    if let Some(ref cred) = a.credential_ref {
                        content_lines
                            .push(format!("Auth: {} (credential: {})", a.scheme, cred));
                    } else {
                        content_lines.push(format!("Auth: {}", a.scheme));
                    }
                }
                ErrorLayer::Shell(s) => {
                    if let Some(code) = s.exit_code {
                        content_lines.push(format!("Shell: {} (exit {})", s.command, code));
                    } else {
                        content_lines.push(format!("Shell: {}", s.command));
                    }
                }
                ErrorLayer::File(f) => {
                    content_lines.push(format!("File: {} ({})", f.path, f.operation));
                }
            }
        }

        // Add separator between layers and message if layers are present
        if !content_lines.is_empty() {
            content_lines.push(String::new());
        }

        // Add error message lines
        let msg_lines: Vec<&str> = detail.message.lines().collect();
        for line in &msg_lines {
            content_lines.push(line.to_string());
        }

        // Truncate to max lines
        let max = box_draw::ERROR_OUTPUT_MAX_LINES;
        let refs: Vec<&str> = content_lines.iter().map(|s| s.as_str()).collect();
        if refs.len() <= max {
            let _ = b.render(&mut stderr, &refs);
        } else {
            let _ = b.write_top(&mut stderr);
            let skip = refs.len() - max;
            let truncation_notice = format!("... ({} lines omitted, showing last {})", skip, max);
            let _ = b.write_content(&mut stderr, &truncation_notice);
            for line in &refs[skip..] {
                let _ = b.write_content(&mut stderr, line);
            }
            let _ = b.write_bottom(&mut stderr);
        }
        let _ = writeln!(stderr);
    }
}

/// Print a high-signal attention block.
///
/// TTY with color uses a boxed section keyed by severity color.
/// Non-TTY uses a compact plain fallback.
pub fn print_attention(level: AttentionLevel, title: &str, body: &str) {
    let (label, color) = match level {
        AttentionLevel::Info => ("INFO", SemanticColor::Info),
        AttentionLevel::Warning => ("WARNING", SemanticColor::Warning),
        AttentionLevel::Error => ("ERROR", SemanticColor::Error),
    };
    let ansi = color.ansi();
    let reset = SemanticColor::reset();
    let lines: Vec<&str> = body.lines().collect();
    let use_color = std::io::stdout().is_terminal() && std::env::var("NO_COLOR").is_err();
    if use_color {
        eprintln!();
        eprintln!("  {ansi}┌─ [{label}] {title}{reset}");
        if lines.is_empty() {
            eprintln!("  {ansi}│{reset} ");
        } else {
            for line in &lines {
                eprintln!("  {ansi}│{reset} {line}");
            }
        }
        eprintln!("  {ansi}└─{reset}");
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

/// Maximum lines to display for a single port value in log output.
const MAX_LOG_VALUE_LINES: usize = HUMAN_TEXT_MAX_LINES;

/// Maximum characters per line before truncation.
const MAX_LINE_WIDTH: usize = HUMAN_TEXT_MAX_LINE_WIDTH;

/// Maximum lines to show for a single failure detail in NonTty mode.
const FAILURE_DETAIL_LINES: usize = 30;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct SharedBufferWriter {
        buf: Arc<Mutex<Vec<u8>>>,
    }

    impl Write for SharedBufferWriter {
        fn write(&mut self, data: &[u8]) -> io::Result<usize> {
            self.buf
                .lock()
                .expect("shared buffer lock")
                .extend_from_slice(data);
            Ok(data.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

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
        observer.on_node_failed(&b, &ExecError::new("boom"));
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

    #[test]
    fn test_non_tty_summary_snapshot_success() {
        let line = format_non_tty_summary_line(
            NonTtyProgressCounts {
                total: 3,
                completed: 3,
                ..Default::default()
            },
            Duration::from_millis(250),
        );
        assert_eq!(line, "✓ progress: 3/3 done, 0 skipped [250ms]");
    }

    #[test]
    fn test_non_tty_summary_snapshot_failure() {
        let line = format_non_tty_summary_line(
            NonTtyProgressCounts {
                total: 4,
                completed: 2,
                failed: 1,
                skipped: 1,
                ..Default::default()
            },
            Duration::from_secs(2),
        );
        assert_eq!(line, "✗ progress: 2/4 done, 1 failed, 1 skipped [2.0s]");
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
        observer.on_node_failed(&NodeId::from("a"), &ExecError::new("error line 1\nerror line 2"));

        assert_eq!(observer.failures.len(), 1);
        assert_eq!(observer.failures[0].0, "A");
        assert_eq!(observer.failures[0].1.message, "error line 1\nerror line 2");
    }

    #[test]
    fn test_display_config_mode_resolution() {
        let animated = DisplayConfig::from_surface(true, true);
        assert_eq!(animated.mode, DisplayMode::Animated);
        assert_eq!(animated.verbosity, DisplayVerbosity::Normal);

        let ci_plain = DisplayConfig::from_surface(false, true);
        assert_eq!(ci_plain.mode, DisplayMode::CiPlain);

        let plain = DisplayConfig::from_surface(false, false);
        assert_eq!(plain.mode, DisplayMode::Plain);
    }

    #[test]
    fn test_render_value_for_port_redacts_secrets() {
        let value = Value::Secret(gunbc_ir::SecretString::new("top-secret-token"));
        let rendered = render_value_for_port("api_key", &value).expect("rendered secret");
        assert_eq!(rendered, "***");
    }

    #[test]
    fn test_mask_secrets_in_log_emits_mask_command_without_plaintext_secret() {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer = SharedBufferWriter { buf: buf.clone() };
        let mut ci = crate::CiContext::new(Box::new(gunbc_ir::transport::ci::PlainTextProvider))
            .with_writer(Box::new(writer));
        let secret = "top-secret-token";
        let log = crate::ExecutionLog {
            entries: vec![crate::LogEntry {
                node_id: "node".to_string(),
                inputs: None,
                outputs: HashMap::from([(
                    "secret".to_string(),
                    Value::Secret(gunbc_ir::SecretString::new(secret)),
                )]),
                was_intercepted: false,
                coercions_applied: vec![],
            }],
        };

        mask_secrets_in_log(&mut ci, &log);

        let output = String::from_utf8(buf.lock().expect("shared buffer lock").clone())
            .expect("utf8 output");
        assert!(
            output.contains("[masked value]"),
            "expected mask command output, got: {output}"
        );
        assert!(
            !output.contains(secret),
            "secret plaintext must not be emitted in CI output"
        );
    }

    // -------------------------------------------------------------------
    // success_port_failed tests
    // -------------------------------------------------------------------

    fn log_with_output(port: &str, value: Value) -> crate::ExecutionLog {
        crate::ExecutionLog {
            entries: vec![crate::LogEntry {
                node_id: "node".to_string(),
                inputs: None,
                outputs: HashMap::from([(port.to_string(), value)]),
                was_intercepted: false,
                coercions_applied: vec![],
            }],
        }
    }

    #[test]
    fn success_port_failed_returns_false_for_true() {
        let log = log_with_output("overall_success", Value::Bool(true));
        assert!(!success_port_failed(&log, Some("overall_success")));
    }

    #[test]
    fn success_port_failed_returns_true_for_false() {
        let log = log_with_output("overall_success", Value::Bool(false));
        assert!(success_port_failed(&log, Some("overall_success")));
    }

    #[test]
    fn success_port_failed_returns_true_for_skipped() {
        let log = log_with_output("overall_success", Value::Skipped);
        assert!(success_port_failed(&log, Some("overall_success")));
    }

    #[test]
    fn success_port_failed_returns_false_when_port_absent() {
        let log = log_with_output("other_port", Value::Bool(false));
        assert!(!success_port_failed(&log, Some("overall_success")));
    }

    #[test]
    fn success_port_failed_returns_false_when_no_success_port() {
        let log = log_with_output("overall_success", Value::Bool(false));
        assert!(!success_port_failed(&log, None));
    }
}
