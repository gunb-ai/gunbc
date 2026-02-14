//! gunbc-pragma: Generate repo pragma artifacts (clippy.toml + allowlists).
//!
//! Progress display is automatic based on terminal capabilities.

#![deny(dead_code)]
use gunbc_dag::build_pragma_graph;
use gunbc_exec::{
    execute_and_display, execute_and_display_with_result, print_attention, AttentionLevel,
    BoundaryMocks, ExecutionMode, TerminalProfile,
};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_lib_transport::preflight::ensure_lint_upsert;
use std::env;
use std::fmt::Write;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let mut dry_run = false;
    let mut resource_mode = ExecMode::Ensure;
    let mut check_deprecated = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--dry-run" => dry_run = true,
            "-c" | "--check" => {
                resource_mode = ExecMode::Verify;
                check_deprecated = true;
            }
            "--mode" => {
                i += 1;
                if i < args.len() {
                    match ExecMode::parse_strict(&args[i]) {
                        Ok(parsed) => resource_mode = parsed,
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Error: --mode requires a value (verify|ensure)");
                    process::exit(1);
                }
            }
            arg if arg.starts_with("--mode=") => {
                let mode_str = arg.trim_start_matches("--mode=");
                match ExecMode::parse_strict(mode_str) {
                    Ok(parsed) => resource_mode = parsed,
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        process::exit(1);
                    }
                }
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            other => {
                eprintln!("error: unknown flag '{}'", other);
                process::exit(1);
            }
        }
        i += 1;
    }

    if check_deprecated {
        eprintln!("Warning: --check is deprecated; use --mode=verify");
    }

    if let Err(err) = ensure_lint_upsert() {
        eprintln!("preflight failed: {}", err);
        process::exit(1);
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
                    Value::Bool(resource_mode == ExecMode::Verify),
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
    // In verify mode, we run Real (read transports must execute), but check_mode=true
    // forces compare_content to set skip=true on the write transports.
    // In --dry-run mode (without verify), mock all transports.
    let mode = if dry_run && resource_mode != ExecMode::Verify {
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
            mocks.set_value(&write_node, &path_key, Value::Str("<DRY-RUN>".to_string()));
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

    if resource_mode == ExecMode::Verify {
        // Check mode: execute through shared display path and inspect log outputs.
        let profile = TerminalProfile::detect();
        match execute_and_display_with_result(&dag, mode, &profile, None, Some(&input_mocks)) {
            Ok(result) => {
                let log = result.log;
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
                        "pragma --mode=verify: {} file{} up to date",
                        ok_count,
                        if ok_count == 1 { "" } else { "s" }
                    );
                } else {
                    let mut body = String::new();
                    for path in &drifted {
                        writeln!(body, "DRIFT  {path}").unwrap();
                    }
                    if ok_count > 0 {
                        write!(
                            body,
                            "({} file{} ok)",
                            ok_count,
                            if ok_count == 1 { "" } else { "s" }
                        )
                        .unwrap();
                    }
                    print_attention(
                        &profile,
                        AttentionLevel::Error,
                        "pragma --mode=verify: drift detected",
                        body.trim_end(),
                    );
                    process::exit(1);
                }
            }
            Err(e) => {
                print_attention(
                    &profile,
                    AttentionLevel::Error,
                    "pragma --mode=verify failed",
                    &e.to_string(),
                );
                process::exit(1);
            }
        }
    } else {
        // Detect terminal environment
        let profile = TerminalProfile::detect();

        // Print header
        println!("pragma");
        println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
        println!("  resource_mode: {}", resource_mode);
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
    println!("    --mode=MODE          Resource mode: verify (CI) or ensure (default)");
    println!("    -c, --check          Deprecated alias for --mode=verify");
    println!("    -h, --help           Print this help");
    println!();
    println!("Progress display is automatic based on terminal capabilities.");
}
