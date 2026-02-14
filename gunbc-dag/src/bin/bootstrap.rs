//! gunbc-bootstrap main entry point.
//!
//! Bootstrap tool for initializing gunbc projects.
//! Progress display is automatic based on terminal capabilities.

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_dag::build_bootstrap_graph;
use gunbc_exec::{
    execute_and_display, execute_and_display_with_result, print_attention, AttentionLevel,
    BoundaryMocks, ExecutionMode, PreflightStatusObserver,
};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_lib_transport::preflight::ensure_lint_upsert_with_observer;
use std::fmt::Write;
use std::io::IsTerminal;
use std::process;

fn main() {
    let parsed = BinaryArgs::new()
        .with_mode()
        .with_check_deprecated()
        .parse_env();
    if parsed.help {
        print_help();
        return;
    }
    let dry_run = parsed.dry_run;
    let resource_mode = parsed.resource_mode.unwrap_or(ExecMode::Ensure);

    if let Err(err) = ensure_lint_upsert_with_observer(Some(&mut PreflightStatusObserver)) {
        print_attention(AttentionLevel::Error, "Preflight failed", &err);
        process::exit(1);
    }

    let animated = std::io::stdout().is_terminal();

    // Build the graph
    let dag = match build_bootstrap_graph() {
        Ok(d) => d,
        Err(e) => {
            print_attention(AttentionLevel::Error, "Graph build failed", &e.to_string());
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
                    Value::Bool(resource_mode == ExecMode::Verify),
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
    // In verify mode, we run Real (read transports must execute), but check_mode=true
    // forces compare_content to set skip=true on the write transports.
    // In --dry-run mode (without verify), mock all transports.
    let mode = if dry_run && resource_mode != ExecMode::Verify {
        let mut mocks = BoundaryMocks::new();
        let ok_shell = || Value::Response(TransportResponse::Shell(ShellResponse::ok("")));

        // Scan workspace
        mocks.set_value(
            "execute_scan_workspace",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse::ok(
                "crates/example\n",
            ))),
        );

        // Makefile read
        mocks.set_value(
            "execute_read_makefile",
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
        mocks.set_value("execute_makefile_transport", "skip", Value::Bool(false));
        mocks.set_value(
            "execute_makefile_transport",
            "skip_reason",
            Value::Str(String::new()),
        );

        // Gitignore read
        mocks.set_value(
            "execute_read_gitignore",
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
        mocks.set_value("execute_gitignore_transport", "skip", Value::Bool(false));
        mocks.set_value(
            "execute_gitignore_transport",
            "skip_reason",
            Value::Str(String::new()),
        );

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    if resource_mode == ExecMode::Verify {
        // Check mode: execute through shared display path and inspect log outputs.
        match execute_and_display_with_result(&dag, mode, animated, None, Some(&input_mocks)) {
            Ok(result) => {
                let log = result.log;
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
                        "bootstrap --mode=verify: {} file{} up to date",
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
                        "bootstrap --mode=verify: drift detected",
                        body.trim_end(),
                    );
                    process::exit(1);
                }
            }
            Err(e) => {
                print_attention(
                    AttentionLevel::Error,
                    "bootstrap --mode=verify failed",
                    &e.to_string(),
                );
                process::exit(1);
            }
        }
    } else {
        // Print header
        println!("bootstrap");
        println!(
            "  mode: {}",
            if dry_run && resource_mode != ExecMode::Verify {
                "dry-run"
            } else {
                "real"
            }
        );
        println!("  resource_mode: {}", resource_mode);
        println!();

        // Execute and display (progress or classic based on terminal)
        execute_and_display(&dag, mode, animated, None, Some(&input_mocks));
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
    println!("    --mode=MODE          Resource mode: verify (CI) or ensure (default)");
    println!("    -c, --check          Deprecated alias for --mode=verify");
    println!("    -h, --help           Print this help");
    println!();
    println!("Progress display is automatic based on terminal capabilities.");
}
