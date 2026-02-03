//! gunbc-bootstrap main entry point.
//!
//! Bootstrap tool for initializing gunbc projects.
//! Supports progress display with `--progress` flag (default: auto-detect TTY).

use gunbc_dag::build_bootstrap_graph;
use gunbc_exec::{
    execute_with_mode, execute_with_progress_and_mode, BoundaryMocks, DagProgress, ExecutionMode,
    FrameLoop, TerminalRenderer,
};
use gunbc_ir::layout::{compute_layout, Viewport, ViewportUnit};
use gunbc_ir::symbols::{Tier, STANDARD};
use gunbc_ir::{detect_boundaries, Value};
use std::env;
use std::io;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let mut dry_run = false;
    let mut progress_mode = ProgressMode::Auto;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--dry-run" => dry_run = true,
            "--progress" => progress_mode = ProgressMode::On,
            "--no-progress" => progress_mode = ProgressMode::Off,
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    // Build the graph
    let dag = match build_bootstrap_graph() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error building graph: {}", e);
            process::exit(1);
        }
    };

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        mocks.set_value(
            "write_makefile",
            "written_path",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "write_gitignore",
            "written_path",
            Value::Str("<DRY-RUN>".to_string()),
        );

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    // Decide whether to use progress display
    let use_progress = match progress_mode {
        ProgressMode::On => true,
        ProgressMode::Off => false,
        ProgressMode::Auto => atty_check(),
    };

    // Print header
    println!("bootstrap");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    if use_progress {
        run_with_progress(&dag, mode);
    } else {
        run_classic(&dag, mode);
    }
}

/// Classic execution: log output like before.
fn run_classic(dag: &gunbc_ir::Dag<gunbc_dag::BootstrapGraphOp>, mode: ExecutionMode) {
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
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

/// Progress-display execution: live DAG visualization.
fn run_with_progress(dag: &gunbc_ir::Dag<gunbc_dag::BootstrapGraphOp>, mode: ExecutionMode) {
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
    let labels: std::collections::HashMap<gunbc_ir::NodeId, String> = flat
        .nodes
        .iter()
        .map(|n| (n.id.clone(), n.id.0.clone()))
        .collect();

    // Build viewport from terminal size (fallback 80×24)
    let vp = terminal_viewport();

    // Compute layout
    let layout = compute_layout(&topo_order, &flat.edges, &labels, &vp);

    // Create progress tracker
    let snapshot = gunbc_exec::DagSnapshot::from_dag(&flat, &topo_order, &boundaries);
    let mut progress = DagProgress::new(snapshot);

    // Execute with progress observer
    let result = execute_with_progress_and_mode(dag, mode, &mut progress);

    // Render final state
    let mut renderer = TerminalRenderer::new(io::stdout(), &STANDARD, detect_tier(), layout);
    renderer.set_tty(atty_check());
    renderer.render(&progress);

    if let Err(e) = result {
        eprintln!("\nError: {}", e);
        process::exit(1);
    }
}

/// Detect whether stdout is a TTY.
fn atty_check() -> bool {
    // Simple heuristic: check if TERM is set (most TTYs set it)
    env::var("TERM").is_ok()
}

/// Get terminal viewport dimensions.
fn terminal_viewport() -> Viewport {
    // Try to get terminal size from environment
    let cols = env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(80);
    let rows = env::var("LINES")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(24);

    Viewport::new(cols, rows, ViewportUnit::Chars)
}

/// Detect the best symbol tier for the current terminal.
fn detect_tier() -> Tier {
    // Use Emoji if the terminal likely supports it, otherwise Unicode
    // Simple heuristic: check for known modern terminals
    if env::var("TERM_PROGRAM").is_ok() || env::var("WT_SESSION").is_ok() {
        Tier::Emoji
    } else if env::var("LANG")
        .unwrap_or_default()
        .contains("UTF-8")
    {
        Tier::Unicode
    } else {
        Tier::Ascii
    }
}

#[derive(Clone, Copy)]
enum ProgressMode {
    Auto,
    On,
    Off,
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
    println!("bootstrap - Generate Makefile and .gitignore");
    println!();
    println!("USAGE:");
    println!("    bootstrap [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    -h, --help           Print this help");
}
