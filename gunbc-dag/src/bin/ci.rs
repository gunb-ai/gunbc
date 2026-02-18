//! gunbc-ci main entry point.
//!
//! This is a handwritten main.rs (not generated) because the CI tool is the
//! bootstrap that runs codegen for all other tools. It cannot depend on
//! generated code because it needs to run BEFORE codegen.
//!
//! The CI pipeline uses the resource acquisition pattern internally - the
//! `prep` node checks if codegen is needed and runs it if so.
//!
//! # Resource Mode
//!
//! The `--mode` flag controls how stale resources are handled:
//! - `--mode=ensure` (default): Run codegen if stale/missing
//! - `--mode=verify`: Fail if codegen is stale/missing (CI strict mode)
//!
//! # GitHub Actions Integration
//!
//! When running in GitHub Actions, this tool emits `::group::` commands
//! for each DAG node, creating collapsible sections in the Actions UI.
//! This gives visibility into each step (prep, build, test, lint, report)
//! without requiring separate workflow steps.

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_dag::{
    build_ci_graph_with_mode, print_tool_header, run_tool, wire_fs_env_write_mock, RunToolOptions,
};
use gunbc_exec::{print_attention, AttentionLevel, BoundaryMocks, CiContext, ExecutionMode};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse};
use gunbc_ir::Value;
use gunbc_ir::CODEGEN_STAMP_PATH;
use std::process;

fn main() {
    let parsed = BinaryArgs::new().with_mode().parse_env();
    if parsed.help {
        print_help();
        return;
    }
    // Safety default: enable runtime file declaration guard in CI runs
    // unless the caller explicitly sets GUNBC_RESOURCE_FILE_GUARD.
    if std::env::var_os("GUNBC_RESOURCE_FILE_GUARD").is_none() {
        std::env::set_var("GUNBC_RESOURCE_FILE_GUARD", "1");
    }

    let dry_run = parsed.dry_run;
    let resource_mode = parsed.resource_mode.unwrap_or(ExecMode::Ensure);

    // Build the CI graph with the exec mode embedded in the inlined codegen DAG
    let dag = match build_ci_graph_with_mode(resource_mode) {
        Ok(d) => d,
        Err(e) => {
            print_attention(
                AttentionLevel::Error,
                "CI graph build failed",
                &e.to_string(),
            );
            process::exit(1);
        }
    };

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();

        // Resource environment: filesystem handle used by transport executors.
        wire_fs_env_write_mock(&dag, &mut mocks);

        // Transport execution nodes need properly-typed Response mocks.
        // The default mock is Value::Str("<DRY-RUN>"), but downstream parse
        // nodes call v.as_response() which only matches Value::Response.

        // execute_deps_exists: file exists check for deps.toml
        mocks.set_value(
            "execute_deps_exists",
            "response",
            Value::Response(
                FileResponse {
                    path: "deps.toml".to_string(),
                    operation: FileOp::Exists,
                    success: true,
                    content: None,
                    exists: Some(false),
                    error: None,
                }
                .into(),
            ),
        );

        // execute_codegen_exists: shell exists check
        mocks.set_value(
            "execute_codegen_exists",
            "response",
            Value::Response(ShellResponse::ok("").into()),
        );

        // execute_codegen: shell command (skipped when codegen exists)
        mocks.set_value(
            "execute_codegen",
            "response",
            Value::Response(ShellResponse::ok("").into()),
        );
        mocks.set_value("execute_codegen", "skip", Value::Bool(true));

        // execute_stamp_write: file write (codegen prep succeeded)
        mocks.set_value(
            "execute_stamp_write",
            "response",
            Value::Response(
                FileResponse {
                    path: CODEGEN_STAMP_PATH.to_string(),
                    operation: FileOp::Write,
                    success: true,
                    content: None,
                    exists: None,
                    error: None,
                }
                .into(),
            ),
        );
        mocks.set_value("execute_stamp_write", "skip", Value::Bool(false));

        let mut set_skippable_shell = |node: &str| {
            mocks.set_value(
                node,
                "response",
                Value::Response(ShellResponse::ok("<DRY-RUN>").into()),
            );
            mocks.set_value(node, "skip", Value::Bool(false));
            mocks.set_value(node, "skip_reason", Value::Str(String::new()));
        };

        // Main CI stage transports (skippable triplets).
        for node in [
            "execute_testgen",
            "execute_bootstrap",
            "execute_pragma",
            "execute_build",
            "execute_test",
            "execute_clippy_lint",
            "execute_guardrail_check",
        ] {
            set_skippable_shell(node);
        }

        // Verify checks: per-generator --mode=verify commands (skippable triplets).
        for node in [
            "execute_verify_makegen_check",
            "execute_verify_deps_config_check",
            "execute_verify_bootstrap_check",
            "execute_verify_testgen_check",
            "execute_verify_pragma_check",
        ] {
            set_skippable_shell(node);
        }

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    // Detect CI environment
    let ci = CiContext::detect();
    let is_ci = ci.provider_id() != "plain";

    let mut metadata = vec![
        ("exec", if dry_run { "dry-run" } else { "real" }.to_string()),
        (
            "resource_mode",
            match resource_mode {
                ExecMode::Verify => "verify (fail on stale)",
                ExecMode::Ensure => "ensure (fix stale)",
            }
            .to_string(),
        ),
    ];
    if is_ci {
        metadata.push(("ci", ci.provider_name().to_string()));
    }
    let tool_name = gunbc_ir::cargo::name("ci");
    print_tool_header(&tool_name, &metadata);

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
    let name = gunbc_ir::cargo::name("ci");
    println!("{name} - CI orchestration tool");
    println!();
    println!("USAGE:");
    println!("    {name} [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run       Don't perform actual I/O");
    println!("    --mode=MODE         Resource acquisition mode:");
    println!("                          ensure - run codegen if stale/missing (default)");
    println!("                          verify - fail if codegen is stale/missing");
    println!("    -h, --help          Print this help");
    println!();
    println!("EXAMPLES:");
    println!("    {name}              # Run CI with auto-codegen if needed");
    println!("    {name} --mode=verify  # Strict mode: fail if codegen stale");
    println!("    {name} --dry-run    # Preview without I/O");
    println!();
    println!("The CI pipeline runs: SetupDeps -> Prep -> Build -> Test/Lint -> Report");
    println!();
    println!("The Prep stage uses manifest-based freshness checking.");
    println!("In 'ensure' mode, stale resources are regenerated automatically.");
    println!("In 'verify' mode, stale resources cause CI to fail immediately.");
}
