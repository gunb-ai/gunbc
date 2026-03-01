//! gunbc-sdlc main entry point (BT10).
//!
//! Runs the SDLC worker-dispatch workflow: discovers issues labeled `sdlc:*`,
//! acquires claims, dispatches per-stage handlers, and records outcomes.
//!
//! This compiles `workflows/sdlc.dag` (the worker-dispatch DAG that calls
//! `dispatch_sdlc()`), NOT `pipelines/sdlc.dag` (the issue-centric pipeline).
//!
//! # Examples
//!
//! ```text
//! gunbc-sdlc --profile unit_test --dry-run      # Validate workflow structure
//! gunbc-sdlc --profile local --repo gunb-ai/gunbc  # Real execution
//! gunbc-sdlc --profile local --issue 42          # Process specific issue
//! ```

#![deny(dead_code)]
use gunbc_cli::{parse, CliParam, ParamType};
use gunbc_dag::{
    dsl_builder::build_dsl_graph_with_profile, print_tool_header,
    run_tool, RunToolOptions,
};
use gunbc_test::auto_mock_spec;
use gunbc_exec::{lower, print_attention, AttentionLevel, BoundaryMocks, ExecutionMode};
use gunbc_ir::{detect_entrypoints, Value};
use std::process;

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let schema = vec![
        CliParam::new("profile", ParamType::Str)
            .short('p')
            .default("unit_test"),
        CliParam::new("repo", ParamType::Str).short('r'),
        CliParam::new("issue", ParamType::Str).short('i'),
        CliParam::new("worker_id", ParamType::Str)
            .short('w')
            .default("gunbc-sdlc"),
        CliParam::new("llm_provider", ParamType::Str).default("anthropic"),
        CliParam::new("llm_model", ParamType::Str).default("claude-sonnet-4-20250514"),
    ];
    let parsed = match parse(&argv, &schema) {
        Ok(parsed) => parsed,
        Err(error) => {
            print_attention(
                AttentionLevel::Error,
                "SDLC argument parsing failed",
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
    let profile = parsed
        .values
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("unit_test")
        .to_string();
    let repo = parsed
        .values
        .get("repo")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let issue = parsed
        .values
        .get("issue")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let worker_id = parsed
        .values
        .get("worker_id")
        .and_then(|v| v.as_str())
        .unwrap_or("gunbc-sdlc")
        .to_string();
    let llm_provider = parsed
        .values
        .get("llm_provider")
        .and_then(|v| v.as_str())
        .unwrap_or("anthropic")
        .to_string();
    let llm_model = parsed
        .values
        .get("llm_model")
        .and_then(|v| v.as_str())
        .unwrap_or("claude-sonnet-4-20250514")
        .to_string();

    // Validate profile
    if !["unit_test", "local", "cloud_run"].contains(&profile.as_str()) {
        print_attention(
            AttentionLevel::Error,
            "Unknown profile",
            &format!(
                "'{}' is not supported. Use 'unit_test', 'local', or 'cloud_run'.",
                profile
            ),
        );
        process::exit(1);
    }

    // Parse repo into owner/name
    let (owner, repo_name) = if let Some(ref r) = repo {
        match r.split_once('/') {
            Some((o, n)) => (o.to_string(), n.to_string()),
            None => {
                print_attention(
                    AttentionLevel::Error,
                    "Invalid repo format",
                    &format!("'{}' should be 'owner/name' (e.g., 'gunb-ai/gunbc').", r),
                );
                process::exit(1);
            }
        }
    } else {
        ("gunb-ai".to_string(), "gunbc".to_string())
    };

    // ========================================================================
    // Build SDLC worker-dispatch workflow DAG with profile
    // ========================================================================

    let dag = match build_dsl_graph_with_profile("workflows/sdlc.dag", &profile) {
        Ok(d) => d,
        Err(e) => {
            print_attention(
                AttentionLevel::Error,
                "SDLC workflow build failed",
                &e.to_string(),
            );
            process::exit(1);
        }
    };

    // ========================================================================
    // Wire entrypoint inputs
    // ========================================================================

    let mut input_mocks = BoundaryMocks::new();

    let lowered = match lower(&dag) {
        Ok(l) => l,
        Err(e) => {
            print_attention(
                AttentionLevel::Error,
                "SDLC workflow lower failed",
                &e.to_string(),
            );
            process::exit(1);
        }
    };

    let entrypoints = detect_entrypoints(&lowered.dag);
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        let val = match port_name.0.as_str() {
            p if p.contains("owner") => Some(Value::Str(owner.clone())),
            p if p.contains("repo") => Some(Value::Str(repo_name.clone())),
            p if p.contains("worker_id") => Some(Value::Str(worker_id.clone())),
            p if p.contains("llm_provider") => Some(Value::Str(llm_provider.clone())),
            p if p.contains("llm_model") => Some(Value::Str(llm_model.clone())),
            p if p.contains("issue") => issue.as_ref().map(|i| Value::Str(i.clone())),
            _ => None,
        };
        if let Some(v) = val {
            input_mocks.set_input(node_id.0.clone(), port_name.0.clone(), v);
        }
    }

    // ========================================================================
    // Set up execution mode
    // ========================================================================

    let mode = if dry_run || profile == "unit_test" {
        let spec = auto_mock_spec(&dag, "sdlc");
        let dry_run_mocks = spec.to_dry_run_mocks();

        // Seed entrypoint inputs from auto-mock spec
        let boundary = spec.to_boundary_mocks();
        for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
            if input_mocks.get_input(&node_id.0, &port_name.0).is_none() {
                if let Some(val) = boundary.get_input(&node_id.0, &port_name.0) {
                    input_mocks.set_input(node_id.0.clone(), port_name.0.clone(), val.clone());
                }
            }
        }

        ExecutionMode::DryRun(dry_run_mocks)
    } else {
        ExecutionMode::Real
    };

    // ========================================================================
    // Print tool header and execute
    // ========================================================================

    let mut metadata = vec![
        ("exec", if dry_run { "dry-run" } else { "real" }.to_string()),
        ("profile", profile),
        ("repo", format!("{}/{}", owner, repo_name)),
        ("worker", worker_id),
        ("llm", format!("{}/{}", llm_provider, llm_model)),
    ];
    if let Some(ref i) = issue {
        metadata.push(("issue", format!("#{}", i)));
    }
    let tool_name = gunbc_ir::cargo::name("sdlc");
    print_tool_header(&tool_name, &metadata);

    run_tool(
        dag,
        mode,
        RunToolOptions {
            success_port: None,
            with_freshness: false,
            input_mocks: Some(&input_mocks),
        },
    );
}

fn print_help() {
    let name = gunbc_ir::cargo::name("sdlc");
    println!("{name} - SDLC worker-dispatch: discover → claim → dispatch → record → release");
    println!();
    println!("USAGE:");
    println!("    {name} [OPTIONS]");
    println!();
    println!("Discovers GitHub issues labeled 'sdlc:*', claims them, and dispatches");
    println!("stage handlers to drive issues through the full lifecycle.");
    println!();
    println!("OPTIONS:");
    println!("    -p, --profile PROFILE     Execution profile: unit_test, local, cloud_run");
    println!("                              (default: unit_test)");
    println!("    -r, --repo OWNER/NAME     Target repository (default: gunb-ai/gunbc)");
    println!("    -i, --issue NUMBER        Process specific issue only");
    println!("    -w, --worker-id ID        Worker identity for claim ownership");
    println!("                              (default: gunbc-sdlc)");
    println!("        --llm-provider NAME   LLM provider: openai, anthropic");
    println!("                              (default: anthropic)");
    println!("        --llm-model NAME      LLM model name");
    println!("                              (default: claude-sonnet-4-20250514)");
    println!("    -n, --dry-run             Validate pipeline without real I/O");
    println!("    -h, --help                Print this help");
    println!();
    println!("PROFILES:");
    println!("    unit_test     All stubs — validate pipeline structure (always dry-run)");
    println!("    local         Real GitHub API + file-based stores");
    println!("    cloud_run     GCS stores + PubSub signals (Cloud Run)");
    println!();
    println!("STAGE LIFECYCLE:");
    println!("    idea → design → design-review → accepted → implementing →");
    println!("    code-review → testing → done");
    println!();
    println!("EXAMPLES:");
    println!("    {name} --profile unit_test --dry-run");
    println!("    {name} --profile local --repo gunb-ai/gunbc");
    println!("    {name} --profile local --issue 42");
}
