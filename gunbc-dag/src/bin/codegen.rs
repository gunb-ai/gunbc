//! gunbc-codegen-dag main entry point.
//!
//! Upsert-style codegen prep: checks for generated CLI entrypoints,
//! runs the bootstrapper if missing, and writes a stamp file.

#![deny(dead_code)]
use gunbc_dag::codegen::build_codegen_graph_with_mode;
use gunbc_dag::CODEGEN_STAMP_PATH;
use gunbc_exec::{execute_and_display, BoundaryMocks, ExecutionMode, TerminalProfile};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_lib_transport::preflight::ensure_lint_upsert;
use std::env;
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
                    if let Some(parsed) = ExecMode::parse(&args[i]) {
                        resource_mode = parsed;
                    } else {
                        eprintln!(
                            "Warning: Unknown mode '{}', using '{}'",
                            args[i], resource_mode
                        );
                    }
                } else {
                    eprintln!("Warning: --mode requires a value (verify|ensure)");
                }
            }
            arg if arg.starts_with("--mode=") => {
                let mode_str = arg.trim_start_matches("--mode=");
                if let Some(parsed) = ExecMode::parse(mode_str) {
                    resource_mode = parsed;
                } else {
                    eprintln!(
                        "Warning: Unknown mode '{}', using '{}'",
                        mode_str, resource_mode
                    );
                }
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
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

    // Detect terminal environment
    let profile = TerminalProfile::detect();

    // Build the graph
    let dag = match build_codegen_graph_with_mode(resource_mode) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error building graph: {}", e);
            process::exit(1);
        }
    };

    // Set up execution mode
    let mode = if dry_run && resource_mode != ExecMode::Verify {
        let mut mocks = BoundaryMocks::new();
        let ok_shell = || Value::Response(TransportResponse::Shell(ShellResponse::ok("")));
        let missing_shell = || {
            Value::Response(TransportResponse::Shell(ShellResponse::failed(
                1, "missing",
            )))
        };

        // Simulate missing codegen outputs so the codegen step runs.
        mocks.set_value("execute_codegen_exists", "response", missing_shell());

        // Codegen command execution
        mocks.set_value("execute_codegen", "response", ok_shell());
        mocks.set_value("execute_codegen", "skip", Value::Bool(false));

        // Stamp write
        mocks.set_value(
            "execute_stamp_write",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: CODEGEN_STAMP_PATH.to_string(),
                operation: FileOp::Write,
                success: true,
                content: None,
                exists: None,
                error: None,
            })),
        );
        mocks.set_value("execute_stamp_write", "skip", Value::Bool(false));

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    // Print header
    println!("codegen");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!("  resource_mode: {}", resource_mode);
    println!();

    // Execute and display (progress or classic based on terminal)
    execute_and_display(&dag, mode, &profile, Some("prep_success"), None);
}

fn print_help() {
    println!("codegen - Upsert CLI entrypoints");
    println!();
    println!("USAGE:");
    println!("    gunbc-codegen-dag [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    --mode=MODE          Resource mode: verify or ensure");
    println!("    -c, --check          Deprecated alias for --mode=verify");
    println!("    -h, --help           Print this help");
    println!();
    println!("Checks for generated CLI entrypoints and runs gunbc-codegen if missing.");
}
