//! gunbc-docgen main entry point.
//!
//! Generates documentation artifacts from live code/test sources.

#![deny(dead_code)]
#![allow(clippy::disallowed_methods)] // Docgen binary reads source files directly
#![allow(clippy::vec_init_then_push)]
use gunbc_dag::build_docgen_graph;
use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::Value;
use std::env;
use std::process;

const AB_DOC_PATH: &str = "docs/ab-writing-workflows.md";
const AB_GENERATED_DOC_PATH: &str = "docs/ab-writing-workflows-generated.md";

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut dry_run = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--dry-run" => dry_run = true,
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
        }
        i += 1;
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

    if let Err(e) = execute_with_mode(&dag, mode) {
        eprintln!("Execution failed: {}", e);
        process::exit(1);
    }
}

fn build_dry_run_mocks() -> BoundaryMocks {
    let mut mocks = BoundaryMocks::new();
    set_chain_mocks(&mut mocks, "ab_workflows_doc", AB_DOC_PATH);
    set_chain_mocks(&mut mocks, "ab_workflows_generated_doc", AB_GENERATED_DOC_PATH);
    mocks
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

    mocks.set_value(&write_node, format!("{name}_written_path"), Value::Str("<DRY-RUN>".to_string()));
    mocks.set_value(&write_node, format!("{name}_content"), Value::Str("<DRY-RUN>".to_string()));
    mocks.set_value(&write_node, "skip", Value::Bool(false));
    mocks.set_value(&write_node, "skip_reason", Value::Str("<DRY-RUN>".to_string()));
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
