//! gunbc-build main entry point.
//!
//! Local development build pipeline: build → (test + clippy) → summary.
//! Progress display is automatic based on terminal capabilities.

use gunbc_dag::build::build_build_graph;
use gunbc_exec::{
    execute_with_mode, execute_with_progress_and_mode, BoundaryMocks, DagProgress, ExecutionMode,
    FrameLoop, OutputSummary, ProgressObserver, TerminalProfile, TerminalRenderer,
};
use gunbc_ir::layout::compute_layout;
use gunbc_ir::symbols::STANDARD;
use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::{detect_boundaries, NodeId, Value};
use std::env;
use std::io;
use std::process;
use std::thread;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let mut dry_run = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--dry-run" => dry_run = true,
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    // Detect terminal environment
    let profile = TerminalProfile::detect();

    // Build the graph
    let dag = match build_build_graph() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error building graph: {}", e);
            process::exit(1);
        }
    };

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        let ok_shell = || {
            Value::Response(TransportResponse::Shell(ShellResponse {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            }))
        };

        // Build transport
        mocks.set_value("execute_build", "response", ok_shell());

        // Test transport
        mocks.set_value("execute_test", "response", ok_shell());
        mocks.set_value("execute_test", "skip", Value::Bool(false));

        // Clippy transport
        mocks.set_value("execute_clippy", "response", ok_shell());
        mocks.set_value("execute_clippy", "skip", Value::Bool(false));

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    // Print header
    println!("build");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    if profile.supports_progress {
        run_with_progress(&dag, mode, &profile);
    } else {
        run_classic(&dag, mode);
    }
}

/// Classic execution: log output.
fn run_classic(dag: &gunbc_ir::Dag<gunbc_dag::build::BuildGraphOp>, mode: ExecutionMode) {
    match execute_with_mode(dag, mode) {
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
            // Check overall_success
            for entry in &log.entries {
                if let Some(Value::Bool(false)) = entry.outputs.get("overall_success") {
                    process::exit(1);
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
fn run_with_progress(
    dag: &gunbc_ir::Dag<gunbc_dag::build::BuildGraphOp>,
    mode: ExecutionMode,
    profile: &TerminalProfile,
) {
    // Lower the DAG to get flat topology for layout
    let flat = match gunbc_exec::lower(dag) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error lowering DAG: {}", e);
            process::exit(1);
        }
    };

    let boundaries = detect_boundaries(&flat);
    let topo_order = gunbc_exec::topo_sort(&flat);

    // Build labels from node IDs
    let labels: std::collections::HashMap<NodeId, String> = flat
        .nodes
        .iter()
        .map(|n| (n.id.clone(), n.id.0.clone()))
        .collect();

    // Compute layout from profile viewport
    let layout = compute_layout(&topo_order, &flat.edges, &labels, &profile.viewport);

    // Save levels for parallel animation
    let levels = layout.levels.clone();

    // Create progress tracker and execute
    let snapshot = gunbc_exec::DagSnapshot::from_dag(&flat, &topo_order, &boundaries);
    let mut progress = DagProgress::new(snapshot.clone());
    let result = execute_with_progress_and_mode(dag, mode, &mut progress);

    // Capture actual final state per node
    let final_states: std::collections::HashMap<NodeId, gunbc_exec::NodeState> = progress
        .nodes
        .iter()
        .map(|(id, np)| (id.clone(), np.state))
        .collect();
    let final_phase = progress.phase.clone();

    // Animated replay: create a fresh visual progress and step through
    let mut visual = DagProgress::new(snapshot.clone());
    visual.on_dag_start(&snapshot);

    let mut renderer =
        TerminalRenderer::new(io::stdout(), &STANDARD, profile.tier, layout, profile.is_tty, profile.supports_color);

    // Animation timing: minimum 1 second total, 2 frames per level (start + complete)
    // Execution already ran at full speed — this is purely visual replay.
    const MIN_ANIMATION_MS: u64 = 1000;
    let num_levels = levels.len();
    let total_frames = (num_levels * 2).max(1) as u64;
    let frame_ms = (MIN_ANIMATION_MS / total_frames).max(50);
    let frame_delay = Duration::from_millis(frame_ms);

    // Render initial state (all pending)
    renderer.render(&visual);

    let empty_summary = || OutputSummary {
        fields: vec![],
        elapsed: Duration::from_millis(frame_ms),
    };

    // Animate by level: parallel nodes start and complete together
    for level in &levels {
        for node_id in level {
            visual.on_node_start(node_id);
        }
        renderer.render(&visual);
        thread::sleep(frame_delay);

        for node_id in level {
            let final_state = final_states
                .get(node_id)
                .copied()
                .unwrap_or(gunbc_exec::NodeState::Pending);

            match final_state {
                gunbc_exec::NodeState::Failed => {
                    visual.on_node_failed(node_id, "failed");
                }
                gunbc_exec::NodeState::Skipped => {
                    visual.on_node_skipped(node_id);
                }
                gunbc_exec::NodeState::Intercepted => {
                    visual.on_node_intercepted(node_id, empty_summary());
                }
                _ => {
                    visual.on_node_complete(node_id, empty_summary());
                }
            }
        }
        renderer.render(&visual);
        thread::sleep(frame_delay);
    }

    // Final frame
    match &final_phase {
        gunbc_exec::DagPhase::Completed { elapsed } => {
            visual.on_dag_complete(*elapsed);
        }
        gunbc_exec::DagPhase::Failed { .. } => {}
        _ => {
            visual.on_dag_complete(Duration::ZERO);
        }
    }
    renderer.render(&visual);

    if let Err(e) = result {
        eprintln!("\nError: {}", e);
        process::exit(1);
    }

    // Check final states for exit code
    let any_failed = final_states
        .values()
        .any(|s| *s == gunbc_exec::NodeState::Failed);
    if any_failed {
        process::exit(1);
    }
}

fn print_value(port: &str, value: &Value) {
    match value {
        Value::Str(s) => {
            if port.ends_with("stderr") || port.ends_with("stdout") {
                if !s.is_empty() {
                    println!("  {}: {}", port, s);
                }
            } else if s.len() < 80 {
                println!("  {}: {}", port, s);
            } else {
                println!("  {}: {}...", port, &s[..60.min(s.len())]);
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

fn print_help() {
    println!("build - Build, test, and lint the project");
    println!();
    println!("USAGE:");
    println!("    gunbc-build [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    -h, --help           Print this help");
    println!();
    println!("Pipeline: build -> (test + clippy) -> summary");
    println!("Progress display is automatic based on terminal capabilities.");
}
