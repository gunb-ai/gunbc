//! gunbc-review main entry point.
//!
//! Binary entry point that builds the review DAG, resolves credentials
//! from env/policy, and executes in Real mode.
//!
//! Input: git diff (via `--repo-path` / `-r` and optional `--base-ref` / `-b`).
//! Output: structured findings JSON to stdout.
//!
//! This is the first real end-to-end execution of the full stack (W1).
//!
//! # Provider Selection
//!
//! The `--provider` flag selects the LLM provider ("openai" or "anthropic").
//! Default: "anthropic". Credentials are resolved via env vars
//! (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`) or credential policy
//! (`GUNBC_CREDENTIAL_POLICY_JSON`).
//!
//! # Depth Control
//!
//! The `--depth` flag controls cost/quality tradeoff (XS/S/M/L/XL).
//! Default: M. Maps to prompt detail and review thoroughness.
//!
//! # Examples
//!
//! ```text
//! gunbc-review -r .                          # Review current branch diff
//! gunbc-review -r . -b develop               # Diff against develop
//! gunbc-review -r . --provider anthropic     # Use Anthropic
//! gunbc-review -r . --depth L                # Thorough review
//! gunbc-review -r . --pr 42                  # Review a PR's diff
//! gunbc-review -n                            # Dry-run (no I/O)
//! ```

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_dag::{print_tool_header, run_tool, RunToolOptions};
use gunbc_exec::{print_attention, AttentionLevel, BoundaryMocks, ExecutionMode};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_lib_review::graph::build_diff_review_graph_with;
use gunbc_lib_review::ReviewPipelineConfig;
use std::process;

/// Supported LLM providers and their default models.
fn default_model_for_provider(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "claude-sonnet-4-20250514",
        "openai" => "gpt-4o",
        _ => "gpt-4o",
    }
}

/// Parse a FermiDepth string (XS/S/M/L/XL) and return a depth-appropriate
/// review criteria description suffix.
fn depth_suffix(depth: &str) -> &'static str {
    match depth.to_uppercase().as_str() {
        "XS" => " (quick sanity check — focus on critical issues only)",
        "S" => " (focused review — key issues only)",
        "M" => "",
        "L" => " (thorough review — explore edge cases)",
        "XL" => " (exhaustive deep-dive — multi-pass analysis)",
        _ => "",
    }
}

fn main() {
    let parsed = BinaryArgs::new()
        .with_string_param("repo_path", Some('r'), Some("."))
        .with_string_param("base_ref", Some('b'), None)
        .with_string_param("provider", Some('p'), Some("anthropic"))
        .with_string_param("depth", Some('d'), Some("M"))
        .with_string_param("pr", None, None)
        .parse_env();

    if parsed.help {
        print_help();
        return;
    }

    let dry_run = parsed.dry_run;
    let repo_path = parsed
        .get_string("repo_path")
        .unwrap_or(".")
        .to_string();
    let base_ref = parsed.get_string("base_ref").map(|s| s.to_string());
    let provider = parsed
        .get_string("provider")
        .unwrap_or("anthropic")
        .to_string();
    let depth = parsed
        .get_string("depth")
        .unwrap_or("M")
        .to_string();
    let pr_number = parsed.get_string("pr").map(|s| s.to_string());

    // Validate provider
    if !["openai", "anthropic"].contains(&provider.as_str()) {
        print_attention(
            AttentionLevel::Error,
            "Unknown provider",
            &format!(
                "'{}' is not a supported provider. Use 'openai' or 'anthropic'.",
                provider
            ),
        );
        process::exit(1);
    }

    // Validate depth
    let depth_upper = depth.to_uppercase();
    if !["XS", "S", "M", "L", "XL"].contains(&depth_upper.as_str()) {
        print_attention(
            AttentionLevel::Error,
            "Unknown depth",
            &format!(
                "'{}' is not a valid depth. Use XS, S, M, L, or XL.",
                depth
            ),
        );
        process::exit(1);
    }

    // In dry-run mode, use the default config (OpenAI) to match mock specs.
    // In real mode, use the user-selected provider.
    let (effective_provider, model) = if dry_run {
        let default_config = ReviewPipelineConfig::gunbc_default();
        (default_config.provider, default_config.model)
    } else {
        let m = default_model_for_provider(&provider).to_string();
        (provider.clone(), m)
    };

    // Build criteria with depth adjustment
    let mut criteria = gunbc_lib_review::graph_mock::default_criteria();
    let suffix = depth_suffix(&depth_upper);
    if !suffix.is_empty() {
        criteria.description = format!("{}{}", criteria.description, suffix);
    }

    // Build pipeline config
    let config = ReviewPipelineConfig {
        provider: effective_provider.clone(),
        model: model.clone(),
        criteria,
        default_branch: base_ref.clone().unwrap_or_else(|| "main".to_string()),
    };

    // Build the review DAG
    let dag = match build_diff_review_graph_with(config) {
        Ok(d) => d,
        Err(e) => {
            print_attention(
                AttentionLevel::Error,
                "Review graph build failed",
                &e.to_string(),
            );
            process::exit(1);
        }
    };

    // Wire entrypoint inputs
    let mut input_mocks = BoundaryMocks::new();
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "repo_path" => {
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Str(repo_path.clone()),
                );
            }
            "base_ref" => {
                if let Some(ref br) = base_ref {
                    input_mocks.set_input(
                        node_id.0.clone(),
                        port_name.0.clone(),
                        Value::Str(br.clone()),
                    );
                }
            }
            "context" => {
                // If reviewing a PR, inject PR context
                if let Some(ref pr) = pr_number {
                    input_mocks.set_input(
                        node_id.0.clone(),
                        port_name.0.clone(),
                        Value::Str(format!("Reviewing PR #{}", pr)),
                    );
                }
            }
            _ => {}
        }
    }

    // Set up execution mode
    let mode = if dry_run {
        let mocks = gunbc_lib_review::graph_mock::diff_review_mock_spec().to_boundary_mocks();
        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    // Print tool header
    let mut metadata = vec![
        ("exec", if dry_run { "dry-run" } else { "real" }.to_string()),
        ("provider", effective_provider),
        ("model", model),
        ("depth", depth_upper),
        ("repo", repo_path),
    ];
    if let Some(ref br) = base_ref {
        metadata.push(("base_ref", br.clone()));
    }
    if let Some(ref pr) = pr_number {
        metadata.push(("pr", format!("#{}", pr)));
    }
    let tool_name = gunbc_ir::cargo::name("review");
    print_tool_header(&tool_name, &metadata);

    run_tool(
        dag,
        mode,
        RunToolOptions {
            success_port: Some("output"),
            with_freshness: false,
            input_mocks: Some(&input_mocks),
        },
    );
}

fn print_help() {
    let name = gunbc_ir::cargo::name("review");
    println!("{name} - AI-powered code review tool");
    println!();
    println!("USAGE:");
    println!("    {name} [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -r, --repo-path PATH    Repository path (default: .)");
    println!("    -b, --base-ref REF      Base branch for diff (default: main)");
    println!("    -p, --provider NAME     LLM provider: openai, anthropic (default: anthropic)");
    println!("    -d, --depth DEPTH       Review depth: XS, S, M, L, XL (default: M)");
    println!("        --pr NUMBER         PR number for context injection");
    println!("    -n, --dry-run           Don't perform actual I/O");
    println!("    -h, --help              Print this help");
    println!();
    println!("DEPTH LEVELS:");
    println!("    XS   Quick sanity check — critical issues only");
    println!("    S    Focused review — key issues only");
    println!("    M    Standard review — good coverage (default)");
    println!("    L    Thorough review — edge cases explored");
    println!("    XL   Exhaustive deep-dive — multi-pass analysis");
    println!();
    println!("ENVIRONMENT:");
    println!("    ANTHROPIC_API_KEY              Anthropic API key");
    println!("    OPENAI_API_KEY                 OpenAI API key");
    println!("    GUNBC_CREDENTIAL_POLICY_JSON   Credential policy override");
    println!();
    println!("EXAMPLES:");
    println!("    {name}                         # Review current branch vs main");
    println!("    {name} -b develop              # Diff against develop");
    println!("    {name} -p openai               # Use OpenAI GPT-4o");
    println!("    {name} -d L                    # Thorough pre-merge review");
    println!("    {name} --pr 42                 # Review with PR context");
    println!("    {name} -n                      # Dry-run preview");
}
