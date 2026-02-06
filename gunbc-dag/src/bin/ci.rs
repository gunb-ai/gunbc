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
use gunbc_exec::{
    execute_and_display, execute_with_mode_and_ci, BoundaryMocks, CiContext, ExecutionMode,
    TerminalProfile,
};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse};
use gunbc_ir::CODEGEN_STAMP_PATH;
use gunbc_ir::Value;
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

        // execute_build: shell command for cargo build
        mocks.set_value(
            "execute_build",
            "response",
            Value::Response(ShellResponse::ok("<DRY-RUN>").into()),
        );
        mocks.set_value("execute_build", "skip", Value::Bool(false));

        // execute_test: shell command for cargo test
        mocks.set_value(
            "execute_test",
            "response",
            Value::Response(ShellResponse::ok("<DRY-RUN>").into()),
        );
        mocks.set_value("execute_test", "skip", Value::Bool(false));

        // clippy_lint: tool consumer (intercepted)
        mocks.set_value("clippy_lint", "success", Value::Bool(true));
        mocks.set_value("clippy_lint", "stdout", Value::Str(String::new()));
        mocks.set_value("clippy_lint", "stderr", Value::Str(String::new()));
        mocks.set_value("clippy_lint", "skip", Value::Bool(false));

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    // Detect CI environment
    let ci = CiContext::detect();
    let is_ci = ci.provider_id() != "plain";

    // Print header
    println!("{}", gunbc_ir::cargo::name("ci"));
    println!(
        "  exec: {}",
        if dry_run { "dry-run" } else { "real" }
    );
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

    if is_ci {
        // CI environment: use CI context for workflow commands (::group::, etc.)
        let mut ci = ci;
        match execute_with_mode_and_ci(&dag, mode, &mut ci) {
            Ok(log) => {
                for entry in &log.entries {
                    if let Some(Value::Bool(false)) = entry.outputs.get("overall_success") {
                        process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Local environment: use progress display
        let profile = TerminalProfile::detect();
        execute_and_display(&dag, mode, &profile, Some("overall_success"), None);
    }
}

/// Parse the resource mode from command-line arguments.
///
/// Defaults to `Ensure` (dev-friendly behavior).
fn parse_resource_mode(args: &[String]) -> ExecMode {
    for arg in args {
        if let Some(mode_str) = arg.strip_prefix("--mode=") {
            return match mode_str {
                "verify" => ExecMode::Verify,
                "ensure" => ExecMode::Ensure,
                other => {
                    eprintln!("Warning: Unknown mode '{}', using 'ensure'", other);
                    ExecMode::Ensure
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
