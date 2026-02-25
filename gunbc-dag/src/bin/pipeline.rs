//! gunbc-pipeline main entry point (W7).
//!
//! Orchestrates the full daily flow for a branch/PR:
//! 1. Fetch CI status (via `gh run list`)
//! 2. Run 4-dimension review
//! 3. Output summary with actionable items categorized as must-fix / defer / accept
//!
//! Single command before submitting.
//!
//! # Examples
//!
//! ```text
//! gunbc-pipeline                              # Review current branch
//! gunbc-pipeline -r . -b develop              # Against develop
//! gunbc-pipeline --pr 42                      # Review PR with CI context
//! gunbc-pipeline --depth L                    # Thorough pre-merge review
//! gunbc-pipeline --pr 42 --issue 15           # With issue requirements
//! ```

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_dag::{dsl_builder::build_dsl_graph, print_tool_header, run_tool, RunToolOptions};
use gunbc_exec::{print_attention, AttentionLevel, BoundaryMocks, ExecutionMode};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_lib_review::dimension::FermiDepth;
use std::process;

/// Query CI status for a PR using `gh run list`.
///
/// Returns a formatted string describing CI status, or None if not available.
#[allow(clippy::disallowed_methods)] // Binary entry point context gathering (not DAG runtime I/O)
fn query_ci_status(pr_number: &str) -> Option<String> {
    let output = std::process::Command::new("gh")
        .args([
            "run",
            "list",
            "--limit",
            "5",
            "--json",
            "status,conclusion,name,event",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let runs: Vec<serde_json::Value> = serde_json::from_str(&stdout).ok()?;

    if runs.is_empty() {
        return None;
    }

    let mut summary_parts = vec![format!("CI Status for PR #{}", pr_number)];
    for run in &runs {
        let name = run
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let status = run
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let conclusion = run
            .get("conclusion")
            .and_then(|v| v.as_str())
            .unwrap_or("pending");
        summary_parts.push(format!("  - {}: {} ({})", name, status, conclusion));
    }

    Some(summary_parts.join("\n"))
}

/// Query PR description for requirements context.
#[allow(clippy::disallowed_methods)] // Binary entry point context gathering (not DAG runtime I/O)
fn query_pr_description(pr_number: &str) -> Option<String> {
    let output = std::process::Command::new("gh")
        .args(["pr", "view", pr_number, "--json", "title,body"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pr: serde_json::Value = serde_json::from_str(&stdout).ok()?;

    let title = pr.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let body = pr.get("body").and_then(|v| v.as_str()).unwrap_or("");

    Some(format!("PR Title: {}\n\n{}", title, body))
}

/// Query issue description for requirements context.
#[allow(clippy::disallowed_methods)] // Binary entry point context gathering (not DAG runtime I/O)
fn query_issue_description(issue_number: &str) -> Option<String> {
    let output = std::process::Command::new("gh")
        .args(["issue", "view", issue_number, "--json", "title,body"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let issue: serde_json::Value = serde_json::from_str(&stdout).ok()?;

    let title = issue.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let body = issue.get("body").and_then(|v| v.as_str()).unwrap_or("");

    Some(format!("Issue: {}\n\n{}", title, body))
}

fn main() {
    let parsed = BinaryArgs::new()
        .with_string_param("repo_path", Some('r'), Some("."))
        .with_string_param("base_ref", Some('b'), None)
        .with_string_param("provider", Some('p'), Some("anthropic"))
        .with_string_param("depth", Some('d'), Some("M"))
        .with_string_param("pr", None, None)
        .with_string_param("issue", None, None)
        .parse_env();

    if parsed.help {
        print_help();
        return;
    }

    let dry_run = parsed.dry_run;
    let repo_path = parsed.get_string("repo_path").unwrap_or(".").to_string();
    let base_ref = parsed.get_string("base_ref").map(|s| s.to_string());
    let provider = parsed
        .get_string("provider")
        .unwrap_or("anthropic")
        .to_string();
    let depth_str = parsed.get_string("depth").unwrap_or("M").to_string();
    let pr_number = parsed.get_string("pr").map(|s| s.to_string());
    let issue_number = parsed.get_string("issue").map(|s| s.to_string());

    // Validate provider
    if !["openai", "anthropic"].contains(&provider.as_str()) {
        print_attention(
            AttentionLevel::Error,
            "Unknown provider",
            &format!(
                "'{}' is not supported. Use 'openai' or 'anthropic'.",
                provider
            ),
        );
        process::exit(1);
    }

    // Parse depth
    let depth = FermiDepth::parse(&depth_str).unwrap_or_else(|| {
        print_attention(
            AttentionLevel::Error,
            "Unknown depth",
            &format!("'{}' is not valid. Use XS, S, M, L, or XL.", depth_str),
        );
        process::exit(1);
    });

    // ========================================================================
    // Phase 1: Gather context
    // ========================================================================

    let mut context_parts = Vec::new();

    // CI status (W6)
    if let Some(ref pr) = pr_number {
        if !dry_run {
            if let Some(ci_status) = query_ci_status(pr) {
                context_parts.push(ci_status);
            }
        }
    }

    // Requirements from PR description
    if let Some(ref pr) = pr_number {
        if !dry_run {
            if let Some(pr_desc) = query_pr_description(pr) {
                context_parts.push(format!("--- PR Description ---\n{}", pr_desc));
            }
        }
    }

    // Requirements from issue (W8 integration)
    if let Some(ref issue) = issue_number {
        if !dry_run {
            if let Some(issue_desc) = query_issue_description(issue) {
                context_parts.push(format!("--- Issue Description ---\n{}", issue_desc));
            }
        }
    }

    let combined_context = if context_parts.is_empty() {
        None
    } else {
        Some(context_parts.join("\n\n"))
    };

    // ========================================================================
    // Phase 2: Resolve provider and model
    // ========================================================================

    let (effective_provider, model) = {
        let m = match provider.as_str() {
            "anthropic" => "claude-sonnet-4-20250514".to_string(),
            _ => "gpt-4o".to_string(),
        };
        (provider.clone(), m)
    };

    // ========================================================================
    // Phase 3: Build and execute review DAG (DSL-compiled, D-1)
    // ========================================================================

    let dag = match build_dimension_review_graph_dsl() {
        Ok(d) => d,
        Err(e) => {
            print_attention(
                AttentionLevel::Error,
                "Pipeline graph build failed",
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
                if let Some(ref ctx) = combined_context {
                    input_mocks.set_input(
                        node_id.0.clone(),
                        port_name.0.clone(),
                        Value::Str(ctx.clone()),
                    );
                }
            }
            _ => {}
        }
    }

    // Set up execution mode
    let mode = if dry_run {
        ExecutionMode::DryRun(BoundaryMocks::new())
    } else {
        ExecutionMode::Real
    };

    // Print tool header
    let mut metadata = vec![
        ("exec", if dry_run { "dry-run" } else { "real" }.to_string()),
        ("provider", effective_provider),
        ("model", model),
        ("depth", depth.to_string()),
        ("repo", repo_path),
        ("pipeline", "dsl-dimension-review".to_string()),
    ];
    if let Some(ref br) = base_ref {
        metadata.push(("base_ref", br.clone()));
    }
    if let Some(ref pr) = pr_number {
        metadata.push(("pr", format!("#{}", pr)));
    }
    if let Some(ref issue) = issue_number {
        metadata.push(("issue", format!("#{}", issue)));
    }
    let tool_name = gunbc_ir::cargo::name("pipeline");
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
    let name = gunbc_ir::cargo::name("pipeline");
    println!("{name} - Full development pipeline (CI + review + summary)");
    println!();
    println!("USAGE:");
    println!("    {name} [OPTIONS]");
    println!();
    println!("Orchestrates the daily flow for a branch/PR:");
    println!("  1. Fetch CI status (if --pr given)");
    println!("  2. Run code review with coding profile");
    println!("  3. Output summary with must-fix / defer / accept items");
    println!();
    println!("OPTIONS:");
    println!("    -r, --repo-path PATH    Repository path (default: .)");
    println!("    -b, --base-ref REF      Base branch for diff (default: main)");
    println!("    -p, --provider NAME     LLM provider: openai, anthropic (default: anthropic)");
    println!("    -d, --depth DEPTH       Review depth: XS, S, M, L, XL (default: M)");
    println!("        --pr NUMBER         PR number (enables CI status + PR context)");
    println!("        --issue NUMBER      GitHub issue number (requirements context)");
    println!("    -n, --dry-run           Don't perform actual I/O");
    println!("    -h, --help              Print this help");
    println!();
    println!("REVIEW DIMENSIONS:");
    println!("    Coherence       Internal consistency, bugs, logic errors");
    println!("    Quality         Against project standards (AGENT.md, clippy.toml)");
    println!("    Requirements    Does it accomplish the stated goal?");
    println!("    Aspirational    Classify findings: must-fix / defer / accept");
    println!();
    println!("EXAMPLES:");
    println!("    {name}                     # Quick review before submitting");
    println!("    {name} --pr 42             # Full pipeline with CI + PR context");
    println!("    {name} --pr 42 --issue 15  # With issue requirements");
    println!("    {name} -d L --pr 42        # Thorough pre-merge review");
    println!("    {name} -n                  # Dry-run preview");
}
