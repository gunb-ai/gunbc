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
//! TerminalProfile::detect() → supports_progress?
//!   yes → lower → layout → snapshot → execute → animated replay
//!   no  → execute_with_mode_and_inputs → print log entries
//! ```

use crate::frame_build::build_frame;
use crate::frame_write::FrameWriter;
use crate::intercept::BoundaryMocks;
use crate::progress::{DagPhase, DagProgress, DagSnapshot, OutputSummary, ProgressObserver};
use crate::render::{Animation, RenderMode};
use crate::terminal::TerminalProfile;
use crate::{
    execute_with_mode_and_inputs, execute_with_progress_and_mode_and_inputs, lower, topo_sort,
    Executable, ExecutionMode, NodeState,
};
use gunbc_ir::layout::compute_layout;
use gunbc_ir::symbols::{SymbolId, STANDARD};
use gunbc_ir::{detect_boundaries, Dag, NodeId, Value};
use std::collections::HashMap;
use std::io;
use std::process;
use std::thread;
use std::time::Duration;

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
    if profile.supports_progress {
        run_with_progress(dag, mode, profile, success_port, input_mocks);
    } else {
        run_classic(dag, mode, success_port, input_mocks);
    }
}

/// Classic execution: plain text log output.
fn run_classic<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) {
    match execute_with_mode_and_inputs(dag, mode, input_mocks) {
        Ok(log) => {
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
            // Check success port if specified
            if let Some(port) = success_port {
                for entry in &log.entries {
                    if let Some(Value::Bool(false)) = entry.outputs.get(port) {
                        process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

/// Progress-display execution: live DAG visualization with animated replay.
fn run_with_progress<T: Executable + Clone + Send>(
    dag: &Dag<T>,
    mode: ExecutionMode,
    profile: &TerminalProfile,
    success_port: Option<&str>,
    input_mocks: Option<&BoundaryMocks>,
) {
    // Lower the DAG to get flat topology for layout
    let flat = match lower(dag) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error lowering DAG: {}", e);
            process::exit(1);
        }
    };

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

    let mut writer = FrameWriter::new(profile.supports_color, profile.tier, &STANDARD, profile.is_tty);
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
    match result {
        Ok(log) => {
            // Check final node states for hard failures
            let mut should_fail = final_states.values().any(|s| *s == NodeState::Failed);

            // Check success port from execution log — same policy as classic path.
            // A node can complete successfully (NodeState::Completed) while still
            // emitting overall_success=false. Both paths must apply the same check.
            if let Some(port) = success_port {
                for entry in &log.entries {
                    if let Some(Value::Bool(false)) = entry.outputs.get(port) {
                        should_fail = true;
                        break;
                    }
                }
            }

            // Surface boundary outputs after progress render so users see
            // the actual tool results (e.g., gist URL) instead of only the DAG view.
            print_boundary_outputs(&log, &boundaries);

            if should_fail {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("\nError: {}", e);
            process::exit(1);
        }
    }
}

/// Print a single output value in the standard format.
pub fn print_value(port: &str, value: &Value) {
    match value {
        Value::Str(s) => {
            if port.ends_with("stderr") || port.ends_with("stdout") {
                if !s.is_empty() {
                    println!("  {}: {}", port, s);
                }
            } else if s.contains('\n') {
                // Multi-line values (reports, etc.) — print in full
                println!("  {}: {}", port, s);
            } else if s.len() < 80 {
                println!("  {}: {}", port, s);
            } else {
                println!("  {}: {}...", port, truncate_str(s, 60));
            }
        }
        Value::Int(i) => println!("  {}: {}", port, i),
        Value::Bool(b) => println!("  {}: {}", port, b),
        Value::List(list) => println!("  {}: [{} items]", port, list.len()),
        Value::Set(set) => println!("  {}: {{{} items}}", port, set.len()),
        Value::Map(map) => println!("  {}: {{{} entries}}", port, map.len()),
        Value::Json(_) => println!("  {}: <JSON>", port),
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

/// Truncate a string to at most `max_chars` characters (char-boundary safe).
///
/// Returns a borrowed slice when possible. Never panics on multi-byte UTF-8.
fn truncate_str(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((byte_idx, _)) => &s[..byte_idx],
        None => s,
    }
}
