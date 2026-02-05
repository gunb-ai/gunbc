//! gunbc-ci main entry point.
//!
//! This is a handwritten main.rs (not generated) because the CI tool is the
//! bootstrap that runs codegen for all other tools. It cannot depend on
//! generated code because it needs to run BEFORE codegen.
//!
//! The CI pipeline uses the resource acquisition pattern internally - the
//! `prep` node checks if codegen is needed and runs it if so.
//!
//! # GitHub Actions Integration
//!
//! When running in GitHub Actions, this tool emits `::group::` commands
//! for each DAG node, creating collapsible sections in the Actions UI.
//! This gives visibility into each step (prep, build, test, lint, report)
//! without requiring separate workflow steps.

use gunbc_dag::build_ci_graph;
use gunbc_exec::{execute_with_mode_and_ci, BoundaryMocks, CiContext, ExecutionMode};
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse};
use gunbc_ir::Value;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let dry_run = args.iter().any(|a| a == "-n" || a == "--dry-run");
    
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }
    
    // Build the CI graph
    let dag = match build_ci_graph() {
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
        mocks.set_value("execute_deps_exists", "response", Value::Response(
            FileResponse {
                path: "deps.toml".to_string(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(false),
                error: None,
            }.into()
        ));

        // execute_codegen_exists: file exists check (codegen removed, use Cargo.toml)
        mocks.set_value("execute_codegen_exists", "response", Value::Response(
            FileResponse {
                path: "Cargo.toml".to_string(),
                operation: FileOp::Exists,
                success: true,
                content: None,
                exists: Some(true), // Pretend codegen already exists
                error: None,
            }.into()
        ));

        // execute_codegen: shell command (skipped when codegen exists)
        mocks.set_value("execute_codegen", "response", Value::Response(
            ShellResponse::ok("").into()
        ));
        mocks.set_value("execute_codegen", "skip", Value::Bool(true));

        // execute_build: shell command for cargo build
        mocks.set_value("execute_build", "response", Value::Response(
            ShellResponse::ok("<DRY-RUN>").into()
        ));
        mocks.set_value("execute_build", "skip", Value::Bool(false));

        // execute_test: shell command for cargo test
        mocks.set_value("execute_test", "response", Value::Response(
            ShellResponse::ok("<DRY-RUN>").into()
        ));
        mocks.set_value("execute_test", "skip", Value::Bool(false));

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };
    
    // Detect CI environment and create context for workflow commands
    // In GitHub Actions: emits ::group:: for collapsible sections
    // In GitLab CI: emits section_start/end escape sequences
    // Locally: just prints plain text
    let mut ci = CiContext::detect();
    
    // Print header
    println!("{}", gunbc_ir::cargo::name("ci"));
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    // Show CI provider if detected (not "plain")
    if ci.provider_id() != "plain" {
        println!("  ci: {}", ci.provider_name());
    }
    println!();
    
    // Execute the CI pipeline with CI context for step visibility.
    // Node outputs are printed inside their CI groups by the executor,
    // except for the "report" node which prints directly (no group).
    match execute_with_mode_and_ci(&dag, mode, &mut ci) {
        Ok(log) => {
            // Check overall_success and exit with appropriate code
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
}

fn print_help() {
    let name = gunbc_ir::cargo::name("ci");
    println!("{name} - CI orchestration tool");
    println!();
    println!("USAGE:");
    println!("    {name} [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run    Don't perform actual I/O");
    println!("    -h, --help       Print this help");
    println!();
    println!("The CI pipeline runs: SetupDeps -> Prep -> Build -> Test/Lint -> Report");
    println!();
    println!("The Prep stage automatically runs codegen if generated files are missing.");
    println!("This is the resource acquisition (upsert) pattern - check -> create if needed.");
}
