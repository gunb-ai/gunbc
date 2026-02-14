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
use gunbc_dag::build_ci_graph_with_mode;
use gunbc_exec::{execute_and_display, BoundaryMocks, CiContext, ExecutionMode, TerminalProfile};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse};
use gunbc_ir::Value;
use gunbc_ir::CODEGEN_STAMP_PATH;
use gunbc_lib_transport::preflight::ensure_lint_upsert;
use gunbc_primitives::filename;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let dry_run = args.iter().any(|a| a == "-n" || a == "--dry-run");

    // Parse resource mode: --mode=verify or --mode=ensure
    let resource_mode = parse_resource_mode(&args);

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }

    if let Err(err) = ensure_lint_upsert() {
        eprintln!("preflight failed: {}", err);
        process::exit(1);
    }

    // Build the CI graph with the exec mode embedded in the inlined codegen DAG
    let dag = match build_ci_graph_with_mode(resource_mode) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error building CI graph: {}", e);
            process::exit(1);
        }
    };

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();

        // Resource environment: filesystem handle used by transport executors.
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        mocks.set_value("fs_env", "fs:write", fs.into());

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

    // Print header
    println!("{}", gunbc_ir::cargo::name("ci"));
    println!("  exec: {}", if dry_run { "dry-run" } else { "real" });
    println!(
        "  resource_mode: {}",
        match resource_mode {
            ExecMode::Verify => "verify (fail on stale)",
            ExecMode::Ensure => "ensure (fix stale)",
        }
    );
    if is_ci {
        println!("  ci: {}", ci.provider_name());
    }
    println!();

    // Shared execution/display path: CI grouping, local progress, and classic mode
    // are selected internally from TerminalProfile and CI environment.
    let profile = TerminalProfile::detect();
    execute_and_display(&dag, mode, &profile, Some("overall_success"), None);
}

/// Parse the resource mode from command-line arguments.
///
/// Defaults to `Ensure` (dev-friendly behavior).
fn parse_resource_mode(args: &[String]) -> ExecMode {
    for arg in args {
        if let Some(mode_str) = arg.strip_prefix("--mode=") {
            return match ExecMode::parse_strict(mode_str) {
                Ok(parsed) => parsed,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            };
        }
    }
    // Default: ensure mode (run codegen if needed)
    ExecMode::Ensure
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
