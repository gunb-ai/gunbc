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
use gunbc_dag::ci::build_ci_graph;
use gunbc_dag::resources::MAKEFILE_OUTPUT_PATH;
use gunbc_dag::{print_tool_header, run_tool, wire_fs_env_write_mock, RunToolOptions};
use gunbc_exec::{print_attention, AttentionLevel, BoundaryMocks, CiContext, ExecutionMode};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_testgen_registry::iter_dag_specs;
use std::process;

fn ci_generated_tests_path() -> Option<&'static str> {
    iter_dag_specs()
        .find(|spec| spec.name == "ci")
        .map(|spec| spec.meta.output_path)
}

fn ci_path_for_node(node_id: &str) -> Option<&'static str> {
    if node_id.contains("Find_ListDirs") {
        Some("crates")
    } else if node_id.contains("makegen") {
        Some(MAKEFILE_OUTPUT_PATH)
    } else if node_id.contains("render_and_upsert")
        || node_id == "std.patterns::content_upsert"
        || node_id == "std.patterns::file_content_matches"
    {
        ci_generated_tests_path()
    } else {
        None
    }
}

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

    // Build the CI graph from DSL
    let dag = match build_ci_graph() {
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

    // Set up entrypoint inputs for lower-time content_upsert/service args that
    // are still injected by tool frontends.
    let mut input_mocks = BoundaryMocks::new();
    let mut unresolved_path_nodes = Vec::<String>::new();
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "check_mode" => {
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Bool(resource_mode == ExecMode::Verify),
                );
            }
            "path" => {
                if let Some(path) = ci_path_for_node(&node_id.0) {
                    input_mocks.set_input(
                        node_id.0.clone(),
                        port_name.0.clone(),
                        Value::Str(path.to_string()),
                    );
                } else {
                    unresolved_path_nodes.push(node_id.0.clone());
                }
            }
            "max_depth" if node_id.0.contains("Find_ListDirs") => {
                input_mocks.set_input(node_id.0.clone(), port_name.0.clone(), Value::Int(1));
            }
            "min_depth" if node_id.0.contains("Find_ListDirs") => {
                input_mocks.set_input(node_id.0.clone(), port_name.0.clone(), Value::Int(1));
            }
            _ => {}
        }
    }
    if !unresolved_path_nodes.is_empty() {
        unresolved_path_nodes.sort();
        unresolved_path_nodes.dedup();
        print_attention(
            AttentionLevel::Error,
            "CI entrypoint path wiring is incomplete",
            &format!(
                "unmapped path entrypoints: {}",
                unresolved_path_nodes.join(", ")
            ),
        );
        process::exit(1);
    }

    // Set up execution mode
    let mode = if dry_run {
        // Keep dry-run mocks in sync with the latest lowered CI graph shape.
        let mut mocks = gunbc_dag::ci::graph_mock::ci_mock_spec().to_boundary_mocks();
        wire_fs_env_write_mock(&dag, &mut mocks);
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
            input_mocks: Some(&input_mocks),
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

#[cfg(test)]
mod tests {
    use super::ci_path_for_node;
    use gunbc_dag::resources::MAKEFILE_OUTPUT_PATH;

    #[test]
    fn maps_makegen_entrypoints_to_makefile_path() {
        assert_eq!(
            ci_path_for_node("param_source_tools_makegen_makegen_path"),
            Some(MAKEFILE_OUTPUT_PATH)
        );
        assert_eq!(
            ci_path_for_node("tools.makegen::makegen"),
            Some(MAKEFILE_OUTPUT_PATH)
        );
    }
}
