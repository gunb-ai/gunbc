//! gunbc-codegen-dag main entry point.
//!
//! Upsert-style codegen prep: checks for generated CLI entrypoints,
//! runs the bootstrapper if missing, and writes a stamp file.

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_dag::codegen::build_codegen_graph_with_mode;
use gunbc_dag::{wire_fs_env_write_mock, CODEGEN_STAMP_PATH};
use gunbc_exec::{
    execute_and_display, print_attention, AttentionLevel, BoundaryMocks, ExecutionMode,
};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::Value;
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
    let dag = match build_codegen_graph_with_mode(resource_mode) {
        Ok(d) => d,
        Err(e) => {
            print_attention(AttentionLevel::Error, "Graph build failed", &e.to_string());
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

        // Resource environment: filesystem handle used by write transports.
        wire_fs_env_write_mock(&dag, &mut mocks);

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
    let animated = std::io::stdout().is_terminal();
    execute_and_display(&dag, mode, animated, Some("prep_success"), None);
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
    println!("    -h, --help           Print this help");
    println!();
    println!("Checks for generated CLI entrypoints and runs gunbc-codegen if missing.");
}
