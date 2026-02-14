//! gunbc-docgen main entry point.
//!
//! Generates documentation artifacts from live code/test sources.

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_dag::{build_docgen_graph, DOCGEN_READ_TARGETS};
use gunbc_exec::{execute_and_display, BoundaryMocks, ExecutionMode};
use std::io::IsTerminal;
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_lib_transport::preflight::ensure_lint_upsert;
use std::process;

const AB_DOC_PATH: &str = "docs/ab-writing-workflows.md";

fn main() {
    let parsed = BinaryArgs::new().parse_env();
    if parsed.help {
        print_help();
        return;
    }
    let dry_run = parsed.dry_run;

    if let Err(err) = ensure_lint_upsert() {
        eprintln!("preflight failed: {}", err);
        process::exit(1);
    }

    let dag = match build_docgen_graph() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error building graph: {}", e);
            process::exit(1);
        }
    };

    let mode = if dry_run {
        ExecutionMode::DryRun(build_dry_run_mocks())
    } else {
        ExecutionMode::Real
    };

    let animated = std::io::stdout().is_terminal();
    execute_and_display(&dag, mode, animated, None, None);
}

fn build_dry_run_mocks() -> BoundaryMocks {
    let mut mocks = BoundaryMocks::new();
    for target in DOCGEN_READ_TARGETS {
        set_read_mock(&mut mocks, target.name, target.path);
    }
    set_chain_mocks(&mut mocks, "ab_workflows_doc", AB_DOC_PATH);
    mocks
}

fn set_read_mock(mocks: &mut BoundaryMocks, name: &str, path: &str) {
    let read_node = format!("execute_{name}");
    mocks.set_value(
        &read_node,
        "response",
        Value::Response(TransportResponse::File(FileResponse {
            path: path.to_string(),
            operation: FileOp::Read,
            success: true,
            content: Some("<DRY-RUN>".to_string()),
            exists: None,
            error: None,
        })),
    );
}

fn set_chain_mocks(mocks: &mut BoundaryMocks, name: &str, path: &str) {
    let read_node = format!("execute_read_{name}");
    let write_node = format!("execute_{name}_transport");

    mocks.set_value(
        &read_node,
        "response",
        Value::Response(TransportResponse::File(FileResponse {
            path: path.to_string(),
            operation: FileOp::Read,
            success: true,
            content: Some("<DRY-RUN>".to_string()),
            exists: None,
            error: None,
        })),
    );

    mocks.set_value(
        &write_node,
        "response",
        Value::Response(TransportResponse::File(FileResponse {
            path: path.to_string(),
            operation: FileOp::Write,
            success: true,
            content: Some("<DRY-RUN>".to_string()),
            exists: None,
            error: None,
        })),
    );

    mocks.set_value(
        &write_node,
        format!("{name}_written_path"),
        Value::Str("<DRY-RUN>".to_string()),
    );
    mocks.set_value(
        &write_node,
        format!("{name}_content"),
        Value::Str("<DRY-RUN>".to_string()),
    );
    mocks.set_value(&write_node, "skip", Value::Bool(false));
    mocks.set_value(
        &write_node,
        "skip_reason",
        Value::Str("<DRY-RUN>".to_string()),
    );
}

fn print_help() {
    println!("gunbc-docgen - generate docs from live code/test sources");
    println!();
    println!("USAGE:");
    println!("    gunbc-docgen [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run    Don't write files, only simulate");
    println!("    -h, --help       Print this help");
}
