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
use gunbc_dag::{print_tool_header, run_tool, RunToolOptions};
use gunbc_exec::{print_attention, AttentionLevel, BoundaryMocks, ExecutionMode};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_lib_review::dimension::FermiDepth;
use gunbc_lib_review::graph::build_diff_review_graph_with;
use gunbc_lib_review::profile::{
    coding_review_profile_with_context, coding_review_profile_with_requirements, ProjectContext,
};
use gunbc_lib_review::ReviewPipelineConfig;
use std::path::Path;
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

/// Read project context files from the repo path.
///
/// Binary entry points are the I/O boundary — this is where file reads
/// happen before entering the pure DAG world (I6 compliant).
#[allow(clippy::disallowed_methods)] // Binary entry point reads (not DAG runtime I/O)
fn read_project_context(repo: &Path) -> ProjectContext {
    let agent_md = std::fs::read_to_string(repo.join("AGENT.md")).ok();
    let clippy_toml = std::fs::read_to_string(repo.join("clippy.toml")).ok();
    ProjectContext {
        agent_md,
        clippy_toml,
    }
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
    // Phase 2: Build review profile (W5)
    // ========================================================================

    let repo = Path::new(&repo_path);

    // Read project context files (binary is the I/O boundary — I6 compliant)
    let project_context = read_project_context(repo);

    let requirements_text = combined_context.as_deref().unwrap_or("");

    let profile = if requirements_text.is_empty() {
        coding_review_profile_with_context(depth, &project_context)
    } else {
        coding_review_profile_with_requirements(depth, &project_context, requirements_text)
    };

    // Select criteria for the single-pass review config node.
    // When requirements context is available (--pr / --issue), use the
    // requirements dimension so CI/PR/issue context reaches prompt generation.
    // Otherwise fall back to quality criteria for standard code review.
    let criteria = if combined_context.is_some() {
        profile
            .criteria_for(gunbc_lib_review::dimension::ReviewDimension::Requirements)
            .cloned()
            .unwrap_or_else(gunbc_lib_review::default_criteria)
    } else {
        profile
            .criteria_for(gunbc_lib_review::dimension::ReviewDimension::Quality)
            .cloned()
            .unwrap_or_else(gunbc_lib_review::default_criteria)
    };

    // In dry-run mode, use the default config (OpenAI) to match mock specs.
    let (effective_provider, model) = if dry_run {
        let default_config = ReviewPipelineConfig::gunbc_default();
        (default_config.provider, default_config.model)
    } else {
        let m = match provider.as_str() {
            "anthropic" => "claude-sonnet-4-20250514".to_string(),
            _ => "gpt-4o".to_string(),
        };
        (provider.clone(), m)
    };

    let config = ReviewPipelineConfig {
        provider: effective_provider.clone(),
        model: model.clone(),
        criteria,
        default_branch: base_ref.clone().unwrap_or_else(|| "main".to_string()),
    };

    // ========================================================================
    // Phase 3: Build and execute review DAG
    // ========================================================================

    let dag = match build_diff_review_graph_with(config) {
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
        ("profile", profile.name.clone()),
        (
            "dimensions",
            profile
                .active_dimensions()
                .iter()
                .map(|d| d.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        ),
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
