//! gunbc-build main entry point.
//!
//! Local development build pipeline: build → (test + clippy) → summary.
//! Progress display is automatic based on terminal capabilities.

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_dag::build::build_build_graph;
use gunbc_dag::{print_tool_header, run_tool, wire_fs_env_write_mock, RunToolOptions};
use gunbc_exec::{print_attention, AttentionLevel, BoundaryMocks, ExecutionMode};
use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::Value;
use std::process;

fn main() {
    let parsed = BinaryArgs::new().parse_env();
    if parsed.help {
        print_help();
        return;
    }
    let dry_run = parsed.dry_run;

    // Build the graph
    let dag = match build_build_graph() {
        Ok(d) => d,
        Err(e) => {
            print_attention(AttentionLevel::Error, "Graph build failed", &e.to_string());
            process::exit(1);
        }
    };

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        let ok_shell = || Value::Response(TransportResponse::Shell(ShellResponse::ok("")));
        wire_fs_env_write_mock(&dag, &mut mocks);

        // Build transport
        mocks.set_value("execute_build", "response", ok_shell());

        // Test transport
        mocks.set_value("execute_test", "response", ok_shell());
        mocks.set_value("execute_test", "skip", Value::Bool(false));
        mocks.set_value("execute_test", "skip_reason", Value::Str(String::new()));

        // Clippy transport
        mocks.set_value("execute_clippy", "response", ok_shell());
        mocks.set_value("execute_clippy", "skip", Value::Bool(false));
        mocks.set_value("execute_clippy", "skip_reason", Value::Str(String::new()));

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    print_tool_header(
        "build",
        &[("mode", if dry_run { "dry-run" } else { "real" }.to_string())],
    );
    run_tool(
        dag,
        mode,
        RunToolOptions {
            success_port: Some("overall_success"),
            with_freshness: true,
            ..RunToolOptions::default()
        },
    );
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
