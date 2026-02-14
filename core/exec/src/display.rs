//! Shared execute-and-display logic for all CLI tools.
//!
//! Encapsulates the two display paths — progress visualization and classic
//! text output — in a single generic function. All CLI tools (handwritten
//! and code-generated) call [`execute_and_display`] instead of duplicating
//! the progress/classic branching logic.
//!
//! # Architecture
//!
//! ```text
//! TerminalProfile::detect() →
//!   is_ci             → execute_with_mode_and_ci_and_inputs → grouped CI logs
//!   supports_progress → lower → layout → snapshot → execute → live rendering
//!   non_tty           → observer-driven status lines + final summary
//!   fallback          → execute_with_mode_and_inputs → classic log entries
//! ```

use crate::frame_build::{build_frame, format_duration};
use crate::frame_write::FrameWriter;
use crate::intercept::BoundaryMocks;
use crate::progress::{DagProgress, DagSnapshot, OutputSummary, ProgressObserver};
use crate::render::{Animation, RenderMode};
use crate::terminal::TerminalProfile;
use crate::{
    execute_with_mode_and_ci_and_inputs, execute_with_mode_and_inputs,
    execute_with_progress_and_mode_and_inputs, lower, topo_sort, ExecError, Executable,
    ExecutionMode, NodeState,
};
use gunbc_ir::layout::compute_layout;
use gunbc_ir::symbols::{SymbolId, STANDARD};
use gunbc_ir::{detect_boundaries, Dag, NodeId, Value};
use std::collections::HashMap;
use std::io;
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
/// - Terminal profile detection (already done by caller)
/// - Progress display with animated replay (when `profile.supports_progress`)
/// - Classic text output (otherwise)
/// - Exit code handling (exits with code 1 on failure)
///
/// # Arguments
///
/// - `dag`: The DAG to execute.
/// - `mode`: Execution mode (real or dry-run with mocks).
/// - `profile`: Terminal profile from `TerminalProfile::detect()`.
/// - `success_port`: Optional port name to check for `false` → exit(1).
/// - `input_mocks`: Optional input overrides for entrypoint ports.
pub fn execute_and_display<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    profile: &TerminalProfile,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) {
    match execute_and_display_with_result(dag, mode, profile, success_port, input_mocks) {
        Ok(result) => {
            if result.should_fail {
                print_attention(
                    profile,
                    AttentionLevel::Error,
                    "Execution failed",
                    "A required success check returned false.",
                );
                process::exit(1);
            }
        }
        Err(e) => {
            print_attention(
                profile,
                AttentionLevel::Error,
                "Execution failed",
                &e.to_string(),
            );
            process::exit(1);
        }
    }
}

/// Execute a DAG through the shared display path and return execution results.
///
/// Unlike [`execute_and_display`], this function never exits the process.
pub fn execute_and_display_with_result<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    profile: &TerminalProfile,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<DisplayResult, ExecError> {
    if profile.is_ci {
        run_with_ci(dag, mode, success_port, input_mocks)
    } else if profile.supports_progress {
        run_with_progress(dag, mode, profile, success_port, input_mocks)
    } else if !profile.is_tty {
        run_non_tty_summary(dag, mode, success_port, input_mocks)
    } else {
        run_classic(dag, mode, success_port, input_mocks)
    }
}

/// Classic execution: plain text log output.
fn run_classic<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<DisplayResult, ExecError> {
    let log = execute_with_mode_and_inputs(dag, mode, input_mocks)?;

    for entry in &log.entries {
        let marker = if entry.was_intercepted {
            " [DRY-RUN]"
        } else {
            ""
        };
        println!("[{}]{}", entry.node_id, marker);

        for (port, value) in &entry.outputs {
            print_value(port, value);
        }
    }

    Ok(DisplayResult {
        should_fail: success_port_failed(&log, success_port),
        log,
    })
}

/// CI execution: provider-aware grouped output via [`crate::CiContext`].
fn run_with_ci<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<DisplayResult, ExecError> {
    let mut ci = crate::CiContext::detect();
    let log = execute_with_mode_and_ci_and_inputs(dag, mode, &mut ci, input_mocks)?;
    Ok(DisplayResult {
        should_fail: success_port_failed(&log, success_port),
        log,
    })
}

/// Non-TTY execution: concise progress/status lines instead of per-node dumps.
fn run_non_tty_summary<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<DisplayResult, ExecError> {
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {}", e)))?;
    let boundaries = detect_boundaries(&flat.dag);

    let mut observer = NonTtyProgressObserver::default();
    let log = execute_with_progress_and_mode_and_inputs(dag, mode, &mut observer, input_mocks)?;

    print_boundary_outputs(&log, &boundaries);

    Ok(DisplayResult {
        should_fail: observer.failed_count() > 0 || success_port_failed(&log, success_port),
        log,
    })
}

/// Progress-display execution: live DAG visualization driven by observer events.
fn run_with_progress<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    profile: &TerminalProfile,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<DisplayResult, ExecError> {
    // Lower the DAG to get flat topology for layout
    let flat = lower(dag).map_err(|e| ExecError::new(format!("lowering failed: {}", e)))?;

    let boundaries = detect_boundaries(&flat.dag);
    let topo_order = topo_sort(&flat.dag);

    // Build labels from node IDs
    let labels: HashMap<NodeId, String> = flat
        .dag
        .nodes
        .iter()
        .map(|n| (n.id.clone(), n.id.0.clone()))
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
    }

    fn on_node_start(&mut self, node_id: &NodeId) {
        self.set_state(node_id, NonTtyNodeState::Running);
        eprintln!("→ {}...", self.label_for(node_id));
    }

    fn on_node_complete(&mut self, node_id: &NodeId, _summary: OutputSummary) {
        self.set_state(node_id, NonTtyNodeState::Completed);
        eprintln!("✓ {}", self.label_for(node_id));
    }

    fn on_node_failed(&mut self, node_id: &NodeId, error: &str) {
        self.set_state(node_id, NonTtyNodeState::Failed);
        eprintln!("✗ {}: {}", self.label_for(node_id), error);
    }

    fn on_node_skipped(&mut self, node_id: &NodeId) {
        self.set_state(node_id, NonTtyNodeState::Skipped);
    }

    fn on_node_intercepted(&mut self, node_id: &NodeId, _summary: OutputSummary) {
        self.set_state(node_id, NonTtyNodeState::Intercepted);
        eprintln!("✓ {} [dry-run]", self.label_for(node_id));
    }

    fn on_dag_complete(&mut self, elapsed: Duration) {
        let counts = self.counts();
        eprintln!("{}", format_non_tty_summary_line(counts, elapsed));
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

/// Print a single output value in the standard format.
pub fn print_value(port: &str, value: &Value) {
    match value {
        Value::Secret(_) => {
            println!("  {}: ***", port);
        }
        Value::Str(s) => {
            if port.ends_with("stderr") || port.ends_with("stdout") {
                if !s.is_empty() {
                    let t = truncate_log_value(s);
                    println!("  {}: {}", port, t);
                }
            } else if s.contains('\n') {
                let t = truncate_log_value(s);
                println!("  {}: {}", port, t);
            } else if s.len() < 120 {
                println!("  {}: {}", port, s);
            } else {
                println!("  {}: {}...", port, truncate_str(s, 80));
            }
        }
        Value::Int(i) => println!("  {}: {}", port, i),
        Value::Bool(b) => println!("  {}: {}", port, b),
        Value::List(list) => println!("  {}: [{} items]", port, list.len()),
        Value::Set(set) => println!("  {}: {{{} items}}", port, set.len()),
        Value::Map(map) => println!("  {}: {{{} entries}}", port, map.len()),
        Value::Json(_) => println!("  {}: <JSON>", port),
        Value::Skipped => {}
        _ => {}
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
pub fn print_attention(profile: &TerminalProfile, level: AttentionLevel, title: &str, body: &str) {
    let (label, color) = match level {
        AttentionLevel::Info => ("INFO", ANSI_BLUE),
        AttentionLevel::Warning => ("WARNING", ANSI_YELLOW),
        AttentionLevel::Error => ("ERROR", ANSI_RED),
    };
    let lines: Vec<&str> = body.lines().collect();
    if profile.is_tty && profile.supports_color {
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

/// Truncate a string to at most `max_chars` characters (char-boundary safe).
///
/// Returns a borrowed slice when possible. Never panics on multi-byte UTF-8.
fn truncate_str(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((byte_idx, _)) => &s[..byte_idx],
        None => s,
    }
}

/// Maximum lines to display for a single port value in log output.
const MAX_LOG_VALUE_LINES: usize = 40;

/// Truncate a multi-line string for display in CI groups and classic log output.
///
/// Keeps the first 5 and last 35 lines, inserting a truncation marker.
/// Also truncates individual lines longer than 500 characters.
pub(crate) fn truncate_log_value(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() <= MAX_LOG_VALUE_LINES {
        return s.to_string();
    }

    let head = 5;
    let tail = MAX_LOG_VALUE_LINES - head;
    let omitted = lines.len() - head - tail;

    let mut out = String::new();
    for line in &lines[..head] {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str(&format!("    ... ({omitted} lines omitted) ..."));
    out.push('\n');
    for (i, line) in lines[lines.len() - tail..].iter().enumerate() {
        out.push_str(line);
        if i < tail - 1 {
            out.push('\n');
        }
    }
    out
}

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
}
