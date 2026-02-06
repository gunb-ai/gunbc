//! gunbc-pragma: Generate repo pragma artifacts (clippy.toml + allowlists).
//!
//! Progress display is automatic based on terminal capabilities.

#![deny(dead_code)]
use gunbc_dag::build_pragma_graph;
use gunbc_exec::{
    execute_and_display, execute_with_mode_and_inputs, BoundaryMocks, ExecutionMode,
    TerminalProfile,
};
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
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
            "-c" | "--check" => check = true,
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    // Build the graph
    let dag = match build_pragma_graph() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error building graph: {}", e);
            process::exit(1);
        }
    };

    // File paths for the three pragma outputs
    let file_paths: &[(&str, &str)] = &[
        ("clippy", "clippy.toml"),
        ("allowlist", "tools/disallowed-methods-allowlist.txt"),
        ("policy", "tools/pragma-lint-policy.txt"),
    ];

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
                // Match node_id to file path
                let path = file_paths
                    .iter()
                    .find(|(key, _)| node_id.0.contains(key))
                    .map(|(_, path)| *path);
                if let Some(path) = path {
                    input_mocks.set_input(
                        node_id.0.clone(),
                        port_name.0.clone(),
                        Value::Str(path.to_string()),
                    );
                }
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

        for (key, path) in file_paths {
            let read_node = format!("execute_read_{}", key);
            let write_node = format!("execute_{}_transport", key);

            // Read transport mock
            mocks.set_value(
                &read_node,
                "response",
                Value::Response(TransportResponse::File(FileResponse {
                    path: (*path).into(),
                    operation: FileOp::Read,
                    success: true,
                    content: Some("<DRY-RUN>".into()),
                    exists: None,
                    error: None,
                })),
            );

            // Write transport mock
            let response_key = format!("{}_response", key);
            let path_key = format!("{}_written_path", key);
            let content_key = format!("{}_content", key);

            mocks.set_value(
                &write_node,
                &response_key,
                Value::Response(TransportResponse::File(FileResponse {
                    path: (*path).into(),
                    operation: FileOp::Write,
                    success: true,
                    content: Some("<DRY-RUN>".into()),
                    exists: Some(true),
                    error: None,
                })),
            );
            mocks.set_value(
                &write_node,
                &path_key,
                Value::Str("<DRY-RUN>".to_string()),
            );
            mocks.set_value(
                &write_node,
                &content_key,
                Value::Str("<DRY-RUN>".to_string()),
            );
            mocks.set_value(&write_node, "skip", Value::Bool(false));
            mocks.set_value(&write_node, "skip_reason", Value::Str(String::new()));
        }

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    if check {
        // Check mode: bypass display, use execute_with_mode_and_inputs directly
        match execute_with_mode_and_inputs(&dag, mode, Some(&input_mocks)) {
            Ok(log) => {
                let mut ok_count = 0;
                let mut drifted = Vec::new();

                for (key, path) in file_paths {
                    let compare_node = format!("compare_{}_content", key);
                    let fresh = log
                        .entries
                        .iter()
                        .find(|e| e.node_id == compare_node)
                        .and_then(|e| e.outputs.get("fresh"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    if fresh {
                        ok_count += 1;
                    } else {
                        drifted.push(*path);
                    }
                }

                if drifted.is_empty() {
                    println!(
                        "pragma --check: {} file{} up to date",
                        ok_count,
                        if ok_count == 1 { "" } else { "s" }
                    );
                } else {
                    eprintln!("pragma --check: drift detected");
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
        // Detect terminal environment
        let profile = TerminalProfile::detect();

        // Print header
        println!("pragma");
        println!(
            "  mode: {}",
            if dry_run { "dry-run" } else { "real" }
        );
        println!();

        // Execute and display (progress or classic based on terminal)
        execute_and_display(&dag, mode, &profile, None, Some(&input_mocks));
    }
}

fn print_help() {
    println!("pragma - Generate clippy.toml and pragma allowlists");
    println!();
    println!("USAGE:");
    println!("    gunbc-pragma [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    -c, --check          Fail if generated files are stale");
    println!("    -h, --help           Print this help");
    println!();
    println!("Progress display is automatic based on terminal capabilities.");
}
