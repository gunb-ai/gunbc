//! gunbc-pragma: Generate repo pragma artifacts (clippy.toml + allowlists).
//!
//! Progress display is automatic based on terminal capabilities.

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_dag::build_pragma_graph;
use gunbc_exec::{
    execute_and_display, execute_and_display_with_result, print_attention,
    AttentionLevel, BoundaryMocks, ExecutionMode,
};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::{detect_entrypoints, Value};
use std::fmt::Write;
use std::io::IsTerminal;
use std::process;

fn main() {
    let parsed = BinaryArgs::new().with_mode().parse_env();
    if parsed.help {
        print_help();
        return;
    }
    let dry_run = parsed.dry_run;
    let resource_mode = parsed.resource_mode.unwrap_or(ExecMode::Ensure);

    // Build the graph
    let dag = match build_pragma_graph() {
        Ok(d) => d,
        Err(e) => {
            print_attention(AttentionLevel::Error, "Graph build failed", &e.to_string());
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

    let animated = std::io::stdout().is_terminal();

    if resource_mode == ExecMode::Verify {
        // Check mode: execute through shared display path and inspect log outputs.
        match execute_and_display_with_result(
            &dag,
            mode,
            animated,
            None,
            Some(&input_mocks),
        ) {
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
                        AttentionLevel::Error,
                        "pragma --mode=verify: drift detected",
                        body.trim_end(),
                    );
                    process::exit(1);
                }
            }
            Err(e) => {
                print_attention(
                    AttentionLevel::Error,
                    "pragma --mode=verify failed",
                    &e.to_string(),
                );
                process::exit(1);
            }
        }
    } else {
        // Print header
        println!("pragma");
        println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
        println!("  resource_mode: {}", resource_mode);
        println!();

        // Execute and display (progress or classic based on terminal)
        execute_and_display(
            &dag,
            mode,
            animated,
            None,
            Some(&input_mocks),
        );
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
    println!("    -h, --help           Print this help");
    println!();
    println!("Progress display is automatic based on terminal capabilities.");
}
