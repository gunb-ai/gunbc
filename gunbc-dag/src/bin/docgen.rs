//! gunbc-docgen main entry point.
//!
//! Generates documentation artifacts from live code/test sources.

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_dag::{
    build_docgen_graph, print_tool_header, run_tool, wire_fs_env_write_mock, RunToolOptions,
    DOCGEN_READ_TARGETS,
};
use gunbc_exec::{print_attention, AttentionLevel, BoundaryMocks, ExecutionMode};
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::Value;
use std::process;

const AB_DOC_PATH: &str = "docs/ab-writing-workflows.md";

fn main() {
    let parsed = BinaryArgs::new().parse_env();
    if parsed.help {
        print_help();
        return;
    }
    let dry_run = parsed.dry_run;

    let dag = match build_docgen_graph() {
        Ok(d) => d,
        Err(e) => {
            print_attention(AttentionLevel::Error, "Graph build failed", &e.to_string());
            process::exit(1);
        }
    };

    let mode = if dry_run {
        let mut mocks = build_dry_run_mocks();
        wire_fs_env_write_mock(&dag, &mut mocks);
        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    print_tool_header(
        "docgen",
        &[("mode", if dry_run { "dry-run" } else { "real" }.to_string())],
    );
    run_tool(
        dag,
        mode,
        RunToolOptions {
            with_freshness: true,
            ..RunToolOptions::default()
        },
    );
}

fn build_dry_run_mocks() -> BoundaryMocks {
    let mut mocks = BoundaryMocks::new();
    for target in DOCGEN_READ_TARGETS {
        let content = if target.name == "ab_doc_template" {
            dry_run_ab_doc_template()
        } else {
            "<DRY-RUN>"
        };
        set_read_mock_with_content(&mut mocks, target.name, target.path, content);
    }
    set_chain_mocks(
        &mut mocks,
        "ab_workflows_doc",
        AB_DOC_PATH,
        Some(dry_run_ab_doc_template()),
    );
    mocks
}

fn set_read_mock_with_content(mocks: &mut BoundaryMocks, name: &str, path: &str, content: &str) {
    let read_node = format!("execute_{name}");
    mocks.set_value(
        &read_node,
        "response",
        Value::Response(TransportResponse::File(FileResponse {
            path: path.to_string(),
            operation: FileOp::Read,
            success: true,
            content: Some(content.to_string()),
            exists: None,
            error: None,
        })),
    );
}

fn set_chain_mocks(mocks: &mut BoundaryMocks, name: &str, path: &str, read_content: Option<&str>) {
    let read_node = format!("execute_read_{name}");
    let write_node = format!("execute_{name}_transport");
    let read_content = read_content.unwrap_or("<DRY-RUN>").to_string();

    mocks.set_value(
        &read_node,
        "response",
        Value::Response(TransportResponse::File(FileResponse {
            path: path.to_string(),
            operation: FileOp::Read,
            success: true,
            content: Some(read_content),
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

fn dry_run_ab_doc_template() -> &'static str {
    r#"<!-- BEGIN GENERATED:clippy_mock_spec -->
<!-- END GENERATED:clippy_mock_spec -->
<!-- BEGIN GENERATED:clippy_generated_test_excerpt -->
<!-- END GENERATED:clippy_generated_test_excerpt -->
<!-- BEGIN GENERATED:appendix_a_clippy -->
<!-- END GENERATED:appendix_a_clippy -->
<!-- BEGIN GENERATED:appendix_a_gist -->
<!-- END GENERATED:appendix_a_gist -->
<!-- BEGIN GENERATED:appendix_b -->
<!-- END GENERATED:appendix_b -->
<!-- BEGIN GENERATED:appendix_c -->
<!-- END GENERATED:appendix_c -->
<!-- BEGIN GENERATED:appendix_d -->
<!-- END GENERATED:appendix_d -->
"#
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
