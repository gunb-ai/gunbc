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
//!   supports_progress → lower → layout → snapshot → execute → animated replay
//!   fallback          → execute_with_mode_and_inputs → print log entries
//! ```

use crate::frame_build::build_frame;
use crate::frame_write::FrameWriter;
use crate::intercept::BoundaryMocks;
use crate::progress::{DagPhase, DagProgress, DagSnapshot, OutputSummary, ProgressObserver};
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
use std::thread;
use std::time::Duration;

const ANSI_RED: &str = "\x1b[31m";
const ANSI_RESET: &str = "\x1b[0m";

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
                process::exit(1);
            }
        }
        Err(e) => {
            print_error_attention(profile, "Execution failed", &e.to_string());
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

/// Progress-display execution: live DAG visualization with animated replay.
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

    // Save levels before handing layout to renderer (for parallel animation)
    let levels = layout.levels.clone();

    // Create progress tracker and execute (instant)
    let snapshot = DagSnapshot::from_dag(&flat.dag, &topo_order, &boundaries);
    let mut progress = DagProgress::new(snapshot.clone());
    let result = execute_with_progress_and_mode_and_inputs(dag, mode, &mut progress, input_mocks);

    // Capture actual final state per node (for failure detection)
    let final_states: HashMap<NodeId, NodeState> = progress
        .nodes
        .iter()
        .map(|(id, np)| (id.clone(), np.state))
        .collect();
    let final_phase = progress.phase.clone();

    // Animated replay: create a fresh visual progress and step through
    let mut visual = DagProgress::new(snapshot.clone());
    visual.on_dag_start(&snapshot);

    // Set up spinner and frame writer directly
    let spinner_frames: Vec<String> = [
        SymbolId::Spinner0,
        SymbolId::Spinner1,
        SymbolId::Spinner2,
        SymbolId::Spinner3,
    ]
    .iter()
    .map(|id| STANDARD.resolve_tier(*id, profile.tier).to_string())
    .collect();
    let mut spinner = Animation::cycle(spinner_frames, Duration::from_millis(150));

    let mut writer = FrameWriter::new(
        profile.supports_color,
        profile.tier,
        &STANDARD,
        profile.is_tty,
    );
    let mut stdout = io::stdout();

    let render = |visual: &DagProgress,
                  spinner: &Animation,
                  layout: &gunbc_ir::layout::DagLayout,
                  writer: &mut FrameWriter,
                  stdout: &mut io::Stdout| {
        let frame = build_frame(
            visual,
            layout,
            RenderMode::Standard,
            spinner.frame(),
            profile.tier,
            &STANDARD,
        );
        let _ = writer.write_frame(&frame, stdout);
    };

    // Animation timing: minimum 1 second total, 2 frames per level (start + complete)
    // Execution already ran at full speed — this is purely visual replay.
    const MIN_ANIMATION_MS: u64 = 1000;
    let num_levels = levels.len();
    let total_frames = (num_levels * 2).max(1) as u64;
    let frame_ms = (MIN_ANIMATION_MS / total_frames).max(50);
    let frame_delay = Duration::from_millis(frame_ms);

    // Render initial state (all pending)
    render(&visual, &spinner, &layout, &mut writer, &mut stdout);

    let empty_summary = || OutputSummary {
        fields: vec![],
        elapsed: Duration::from_millis(frame_ms),
    };

    // Animate by level: parallel nodes in each level start and complete together
    for level in &levels {
        // Start all nodes in this level simultaneously
        for node_id in level {
            visual.on_node_start(node_id);
        }
        spinner.tick(frame_delay);
        render(&visual, &spinner, &layout, &mut writer, &mut stdout);
        thread::sleep(frame_delay);

        // Complete all nodes in this level simultaneously
        for node_id in level {
            let final_state = final_states
                .get(node_id)
                .copied()
                .unwrap_or(NodeState::Pending);

            match final_state {
                NodeState::Failed => {
                    visual.on_node_failed(node_id, "failed");
                }
                NodeState::Skipped => {
                    visual.on_node_skipped(node_id);
                }
                NodeState::Intercepted => {
                    visual.on_node_intercepted(node_id, empty_summary());
                }
                _ => {
                    visual.on_node_complete(node_id, empty_summary());
                }
            }
        }
        spinner.tick(frame_delay);
        render(&visual, &spinner, &layout, &mut writer, &mut stdout);
        thread::sleep(frame_delay);
    }

    // Final frame: set actual elapsed and phase
    match &final_phase {
        DagPhase::Completed { elapsed } => {
            visual.on_dag_complete(*elapsed);
        }
        DagPhase::Failed { .. } => {
            // Phase already set by on_node_failed
        }
        _ => {
            visual.on_dag_complete(Duration::ZERO);
        }
    }
    render(&visual, &spinner, &layout, &mut writer, &mut stdout);

    // Check execution result and exit code
    let log = result?;

    // Check final node states for hard failures
    let mut should_fail = final_states.values().any(|s| *s == NodeState::Failed);
    should_fail = should_fail || success_port_failed(&log, success_port);

    // Surface boundary outputs after progress render so users see
    // the actual tool results (e.g., gist URL) instead of only the DAG view.
    print_boundary_outputs(&log, &boundaries);

    Ok(DisplayResult { log, should_fail })
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

/// Print a high-signal error block.
///
/// TTY with color uses a red boxed section. Non-TTY uses a compact plain fallback.
fn print_error_attention(profile: &TerminalProfile, title: &str, body: &str) {
    let lines: Vec<&str> = body.lines().collect();
    if profile.is_tty && profile.supports_color {
        eprintln!();
        eprintln!("  {ANSI_RED}┌─ {title}{ANSI_RESET}");
        if lines.is_empty() {
            eprintln!("  {ANSI_RED}│{ANSI_RESET} ");
        } else {
            for line in &lines {
                eprintln!("  {ANSI_RED}│{ANSI_RESET} {line}");
            }
        }
        eprintln!("  {ANSI_RED}└─{ANSI_RESET}");
        return;
    }

    eprintln!();
    eprintln!("ERROR: {title}");
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
