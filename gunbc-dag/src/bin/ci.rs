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
use gunbc_cli::{parse, CliParam, ParamType};
use gunbc_dag::build_build_graph;
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
    let argv: Vec<String> = std::env::args().collect();
    let parsed = match parse(
        &argv,
        &[CliParam::new("mode", ParamType::Str).default("ensure")],
    ) {
        Ok(parsed) => parsed,
        Err(error) => {
            print_attention(
                AttentionLevel::Error,
                "CI argument parsing failed",
                &error.to_string(),
            );
            process::exit(1);
        }
    };
    if parsed.help {
        print_help();
        return;
    }

    let dry_run = parsed.dry_run;
    let mode = parsed
        .values
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("ensure");
    let resource_mode = ExecMode::parse_strict(mode).unwrap_or_else(|_| {
        print_attention(
            AttentionLevel::Error,
            "Invalid --mode value",
            "expected one of: ensure, verify",
        );
        process::exit(1);
    });

    // Runtime CI path uses the concrete build/test/lint DAG.
    let dag = match build_build_graph() {
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
        let mut mocks = BoundaryMocks::new();
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
            success_port: Some("success"),
            freshness: gunbc_dag::FreshnessScope::GenerationOnly,
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

    /// Structural enforcement: CI freshness scope must not overlap with
    /// the build tool DAG's cargo operations.
    ///
    /// The build DAG runs Build+Clippy+Test via cargo service operations.
    /// The CI binary uses `FreshnessScope::GenerationOnly` to avoid
    /// re-running those same cargo operations as freshness steps. This test
    /// ensures the two sets don't overlap — if they did, CI would perform
    /// redundant compilation work.
    ///
    /// If this test fails, either:
    /// 1. A new freshness step was added that overlaps with the build DAG
    ///    → move it to `build_verification_steps()` in freshness_policy.rs
    /// 2. The CI binary's `FreshnessScope` was changed from `GenerationOnly`
    ///    → verify no redundancy is introduced
    #[test]
    fn ci_freshness_does_not_overlap_build_operations() {
        // Known mapping: freshness step ID → cargo operation keyword found in
        // build DAG node IDs. This is the overlap surface we guard against.
        const BUILD_CARGO_KEYWORDS: &[&str] = &["Clippy", "Test", "Build"];

        // Get the build DAG and extract node IDs that represent cargo operations
        let dag = gunbc_dag::build_build_graph().expect("build graph should compile");
        let cargo_node_ids: Vec<&str> = dag
            .nodes
            .iter()
            .filter(|n| {
                let id = n.id.0.as_str();
                id.contains("transport_services_cargo")
                    && BUILD_CARGO_KEYWORDS.iter().any(|kw| id.contains(kw))
            })
            .map(|n| n.id.0.as_str())
            .collect();
        assert!(
            !cargo_node_ids.is_empty(),
            "build DAG should contain cargo transport nodes"
        );

        // Get the generation-only freshness steps (what CI actually uses)
        let gen_steps = gunbc_lib_transport::check_and_plan_generation_freshness();
        if let Some(steps) = gen_steps {
            // Freshness step IDs that correspond to build-phase cargo operations
            const BUILD_PHASE_IDS: &[&str] = &["clippy", "test-compile", "release-check"];

            let overlap: Vec<&str> = steps
                .iter()
                .filter(|s| BUILD_PHASE_IDS.contains(&s.id.as_str()))
                .map(|s| s.id.as_str())
                .collect();

            assert!(
                overlap.is_empty(),
                "CI freshness scope contains steps that overlap with the build DAG's \
                 cargo operations: {overlap:?}. This causes redundant compilation work. \
                 Use FreshnessScope::GenerationOnly or move these steps to \
                 build_verification_steps() in freshness_policy.rs."
            );
        }

        // Also verify that Full scope WOULD have overlap (validates the test catches real issues)
        let full_steps = gunbc_lib_transport::check_and_plan_freshness();
        if let Some(steps) = full_steps {
            const BUILD_PHASE_IDS: &[&str] = &["clippy", "test-compile", "release-check"];
            let has_build_steps = steps
                .iter()
                .any(|s| BUILD_PHASE_IDS.contains(&s.id.as_str()));
            assert!(
                has_build_steps,
                "Full freshness scope should contain build-phase steps \
                 (clippy, test-compile, release-check). If these were removed, \
                 this test needs updating."
            );
        }
    }
}
