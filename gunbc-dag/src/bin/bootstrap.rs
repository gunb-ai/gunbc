//! gunbc-bootstrap main entry point.
//!
//! Bootstrap tool for initializing gunbc projects.
//! Progress display is automatic based on terminal capabilities.

#![deny(dead_code)]
use gunbc_dag::build_bootstrap_graph;
use gunbc_exec::{
    execute_and_display, execute_with_mode_and_inputs, BoundaryMocks, ExecutionMode,
    TerminalProfile,
};
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::{detect_entrypoints, Value};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let mut dry_run = false;
    let mut check = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--dry-run" => dry_run = true,
            "--check" => check = true,
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
    let dag = match build_bootstrap_graph() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error building graph: {}", e);
            process::exit(1);
        }
    };

    // Set up entrypoint inputs
    let mut input_mocks = BoundaryMocks::new();
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "check_mode" => {
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Bool(check),
                );
            }
            "path" => {
                // Set read paths for the file upsert check
                let path = if node_id.0.contains("makefile") {
                    "Makefile"
                } else if node_id.0.contains("gitignore") {
                    ".gitignore"
                } else {
                    continue;
                };
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Str(path.to_string()),
                );
            }
            _ => {}
        }
    }

    // Set up execution mode
    // In --check mode, we run Real (read transports must execute), but check_mode=true
    // forces compare_content to set skip=true on the write transports.
    // In --dry-run mode (without --check), mock all transports.
    let mode = if dry_run && !check {
        let mut mocks = BoundaryMocks::new();
        let ok_shell =
            || Value::Response(TransportResponse::Shell(ShellResponse::ok("")));

        // Scan workspace
        mocks.set_value(
            "execute_scan_workspace",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse::ok("crates/example\n"))),
        );

        // Makefile read
        mocks.set_value(
            "execute_makefile_read",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("<DRY-RUN>".into()),
                exists: None,
                error: None,
            })),
        );

        // Makefile write transport
        mocks.set_value(
            "execute_makefile_transport",
            "makefile_response",
            ok_shell(),
        );
        mocks.set_value(
            "execute_makefile_transport",
            "makefile_written_path",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "execute_makefile_transport",
            "makefile_content",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "execute_makefile_transport",
            "skip",
            Value::Bool(false),
        );
        mocks.set_value(
            "execute_makefile_transport",
            "skip_reason",
            Value::Str(String::new()),
        );

        // Gitignore read
        mocks.set_value(
            "execute_gitignore_read",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: ".gitignore".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("<DRY-RUN>".into()),
                exists: None,
                error: None,
            })),
        );

        // Gitignore write transport
        mocks.set_value(
            "execute_gitignore_transport",
            "gitignore_response",
            ok_shell(),
        );
        mocks.set_value(
            "execute_gitignore_transport",
            "gitignore_written_path",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "execute_gitignore_transport",
            "gitignore_content",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "execute_gitignore_transport",
            "skip",
            Value::Bool(false),
        );
        mocks.set_value(
            "execute_gitignore_transport",
            "skip_reason",
            Value::Str(String::new()),
        );

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    if check {
        // Check mode: bypass display, use execute_with_mode_and_inputs directly
        match execute_with_mode_and_inputs(&dag, mode, Some(&input_mocks)) {
            Ok(log) => {
                // Scan log for compare_*_content.fresh
                let makefile_fresh = log
                    .entries
                    .iter()
                    .find(|e| e.node_id == "compare_makefile_content")
                    .and_then(|e| e.outputs.get("fresh"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let gitignore_fresh = log
                    .entries
                    .iter()
                    .find(|e| e.node_id == "compare_gitignore_content")
                    .and_then(|e| e.outputs.get("fresh"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let mut ok_count = 0;
                let mut drifted = Vec::new();

                if makefile_fresh {
                    ok_count += 1;
                } else {
                    drifted.push("Makefile");
                }
                if gitignore_fresh {
                    ok_count += 1;
                } else {
                    drifted.push(".gitignore");
                }

                if drifted.is_empty() {
                    println!(
                        "bootstrap --check: {} file{} up to date",
                        ok_count,
                        if ok_count == 1 { "" } else { "s" }
                    );
                } else {
                    eprintln!("bootstrap --check: drift detected");
                    for path in &drifted {
                        eprintln!("  DRIFT  {}", path);
                    }
                    if ok_count > 0 {
                        eprintln!(
                            "  ({} file{} ok)",
                            ok_count,
                            if ok_count == 1 { "" } else { "s" }
                        );
                    }
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Print header
        println!("bootstrap");
        println!(
            "  mode: {}",
            if dry_run && !check {
                "dry-run"
            } else {
                "real"
            }
        );
        println!();

        // Execute and display (progress or classic based on terminal)
        execute_and_display(&dag, mode, &profile, None, Some(&input_mocks));
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
    println!("        --check          Verify generated files match disk");
    println!("    -h, --help           Print this help");
    println!();
    println!("Progress display is automatic based on terminal capabilities.");
}
