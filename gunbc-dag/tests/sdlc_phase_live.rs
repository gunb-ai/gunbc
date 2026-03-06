#![allow(clippy::disallowed_methods)] // Live integration harness shells out intentionally.

use gunbc_dag::extern_ops::GunbcExternResolver;
use gunbc_exec::{execute_with_mode_and_inputs, lower, BoundaryMocks, ExecutionMode};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_resolve::{builder::build_dsl_graph, BuildOpts};
use gunbc_test::{guard_test, FermiCost, TestClass};
use serde_json::Value as JsonValue;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const SDLC_LOCAL_PROFILE: &str = "profiles.sdlc.local";
const SDLC_OWNER: &str = "gunb-ai";
const SDLC_REPO: &str = "integration_testing";
const GITHUB_SECRET_PROJECT: &str = "gunbai-secrets";
const GITHUB_SECRET_NAME: &str = "github-token";

fn should_run(name: &str, cost: FermiCost, requires: &[&str], secrets: &[&str]) -> bool {
    guard_test(name, TestClass::Integration, cost, requires, secrets)
}

fn env_var_nonempty(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_flag_enabled(name: &str) -> bool {
    matches!(
        env::var(name).ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

fn llm_api_key_for_provider(provider: &str) -> Option<String> {
    match provider {
        "openai" => env_var_nonempty("OPENAI_API_KEY"),
        _ => env_var_nonempty("ANTHROPIC_API_KEY"),
    }
}

fn github_token_from_secret_manager() -> Option<String> {
    let output = Command::new("gcloud")
        .args([
            "secrets",
            "versions",
            "access",
            "latest",
            "--secret",
            GITHUB_SECRET_NAME,
            "--project",
            GITHUB_SECRET_PROJECT,
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let token = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

fn build_local_worker_graph() -> gunbc_ir::Dag<gunbc_exec::DynOp> {
    build_dsl_graph(
        "funcs/sdlc_worker.dag",
        &GunbcExternResolver,
        BuildOpts {
            entry_func: Some("dispatch_sdlc"),
            profile: Some(SDLC_LOCAL_PROFILE),
        },
    )
    .map(|result| result.dag)
    .unwrap_or_else(|e| panic!("local SDLC worker graph should build: {e}"))
}

fn node_ids(dag: &gunbc_ir::Dag<gunbc_exec::DynOp>) -> Vec<String> {
    dag.nodes.iter().map(|node| node.id.0.clone()).collect()
}

fn has_prefix(node_ids: &[String], prefix: &str) -> bool {
    node_ids.iter().any(|id| id.starts_with(prefix))
}

fn run_command(mut cmd: Command, description: &str) -> String {
    let output = cmd
        .output()
        .unwrap_or_else(|err| panic!("failed to spawn `{description}`: {err}"));
    assert!(
        output.status.success(),
        "command `{description}` failed (status: {:?})\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("command stdout should be valid utf-8")
}

fn github_request(
    token: &str,
    method: &str,
    path: &str,
    body: Option<&JsonValue>,
) -> (u16, JsonValue) {
    let mut cmd = Command::new("curl");
    cmd.arg("-sS")
        .arg("-L")
        .arg("-X")
        .arg(method)
        .arg("-H")
        .arg("Accept: application/vnd.github+json")
        .arg("-H")
        .arg("X-GitHub-Api-Version: 2022-11-28")
        .arg("-H")
        .arg(format!("Authorization: Bearer {token}"));
    if let Some(payload) = body {
        cmd.arg("-H").arg("Content-Type: application/json");
        cmd.arg("-d").arg(payload.to_string());
    }
    cmd.arg("-w").arg("\n%{http_code}");
    cmd.arg(format!("https://api.github.com{path}"));

    let stdout = run_command(cmd, &format!("curl {method} {path}"));
    let (body_text, code_text) = stdout
        .rsplit_once('\n')
        .unwrap_or_else(|| panic!("curl output missing HTTP status footer for `{path}`"));
    let status = code_text
        .trim()
        .parse::<u16>()
        .unwrap_or_else(|err| panic!("invalid HTTP status `{code_text}` for `{path}`: {err}"));
    let json = if body_text.trim().is_empty() {
        JsonValue::Null
    } else {
        serde_json::from_str(body_text).unwrap_or(JsonValue::String(body_text.to_string()))
    };
    (status, json)
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{nanos}", std::process::id())
}

fn create_ephemeral_issue(token: &str) -> String {
    let suffix = unique_suffix();
    let body = serde_json::json!({
        "title": format!("[sdlc-live] design-stage {}", suffix),
        "body": format!("Ephemeral SDLC integration test issue.\n\nmarker: sdlc-live:{suffix}"),
        "labels": ["sdlc:idea", "sdlc:test"],
    });
    let (status, json) = github_request(
        token,
        "POST",
        &format!("/repos/{SDLC_OWNER}/{SDLC_REPO}/issues"),
        Some(&body),
    );
    assert_eq!(status, 201, "ephemeral issue creation should return HTTP 201");
    json["number"]
        .as_i64()
        .unwrap_or_else(|| panic!("created issue should contain numeric `number`: {json}"))
        .to_string()
}

fn close_issue(token: &str, issue_number: &str) {
    let body = serde_json::json!({ "state": "closed" });
    let (status, _) = github_request(
        token,
        "PATCH",
        &format!("/repos/{SDLC_OWNER}/{SDLC_REPO}/issues/{issue_number}"),
        Some(&body),
    );
    assert_eq!(status, 200, "ephemeral issue cleanup should close the issue");
}

struct EphemeralIssue {
    token: String,
    issue_number: String,
}

impl EphemeralIssue {
    fn create(token: &str) -> Self {
        Self {
            token: token.to_string(),
            issue_number: create_ephemeral_issue(token),
        }
    }
}

impl Drop for EphemeralIssue {
    fn drop(&mut self) {
        let _ = std::panic::catch_unwind(|| close_issue(&self.token, &self.issue_number));
    }
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, files);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
}

fn run_dispatch_local(owner: &str, repo: &str, auth_token: &str, api_key: &str, llm_provider: &str, llm_model: &str) {
    let dag = build_local_worker_graph();
    let lowered = lower(&dag).expect("lower local worker graph for entrypoint detection");
    let entrypoints = detect_entrypoints(&lowered.dag);
    let mut input_mocks = BoundaryMocks::new();
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "auth_token" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str(auth_token.to_string()),
            ),
            "api_key" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str(api_key.to_string()),
            ),
            "owner" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str(owner.to_string()),
            ),
            "repo" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str(repo.to_string()),
            ),
            "worker_id" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str("sdlc-local-live-test".to_string()),
            ),
            "llm_provider" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str(llm_provider.to_string()),
            ),
            "llm_model" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str(llm_model.to_string()),
            ),
            _ => {}
        }
    }

    let _log = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&input_mocks))
        .expect("local SDLC dispatch should succeed");
}

#[test]
fn s10_local_profile_binds_real_local_providers() {
    if !should_run(
        "s10_local_profile_binds_real_local_providers",
        FermiCost::S,
        &["gcloud"],
        &[],
    ) {
        return;
    }

    let Some(token) = github_token_from_secret_manager() else {
        eprintln!(
            "skipping S-10 local bindings: gcloud could not access {}/{}",
            GITHUB_SECRET_PROJECT, GITHUB_SECRET_NAME
        );
        return;
    };
    env::set_var("GITHUB_TOKEN", token);

    let dag = build_local_worker_graph();
    let ids = node_ids(&dag);

    assert!(
        has_prefix(
            &ids,
            "execute_transport_extdeps_sdlc_providers_github_issue_provider_github_IssueProvider_discover"
        ),
        "local profile should bind IssueProvider to the GitHub implementation"
    );
    assert!(
        has_prefix(
            &ids,
            "execute_transport_extdeps_sdlc_providers_file_claim_store_file_ClaimStore_acquire"
        ),
        "local profile should bind ClaimStore to the file implementation"
    );
    assert!(
        has_prefix(
            &ids,
            "execute_transport_extdeps_sdlc_providers_file_outcome_ledger_file_OutcomeLedger_upsert"
        ),
        "local profile should bind OutcomeLedger to the file implementation"
    );
    assert!(
        has_prefix(
            &ids,
            "execute_transport_extdeps_sdlc_providers_codex_agent_provider_codex_AgentProvider_spawn"
        ),
        "local profile should bind AgentProvider to the codex implementation"
    );
}

#[test]
fn s11_local_profile_design_stage_e2e() {
    if !should_run(
        "s11_local_profile_design_stage_e2e",
        FermiCost::XL,
        &["shell", "curl", "gcloud"],
        &[],
    ) {
        return;
    }
    if !env_flag_enabled("SDLC_ALLOW_MUTATION") {
        eprintln!("skipping S-11 local e2e: set SDLC_ALLOW_MUTATION=1 to enable");
        return;
    }

    let Some(token) = github_token_from_secret_manager() else {
        eprintln!(
            "skipping S-11 local e2e: gcloud could not access {}/{}",
            GITHUB_SECRET_PROJECT, GITHUB_SECRET_NAME
        );
        return;
    };
    env::set_var("GITHUB_TOKEN", token.clone());

    let llm_provider =
        env_var_nonempty("SDLC_LLM_PROVIDER").unwrap_or_else(|| "anthropic".to_string());
    let llm_model = env_var_nonempty("SDLC_LLM_MODEL")
        .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
    let api_key = match llm_api_key_for_provider(&llm_provider) {
        Some(key) => key,
        None => {
            let missing = if llm_provider == "openai" {
                "OPENAI_API_KEY"
            } else {
                "ANTHROPIC_API_KEY"
            };
            eprintln!("skipping S-11 local e2e: set {missing}");
            return;
        }
    };

    let issue = EphemeralIssue::create(&token);
    let issue_number = issue.issue_number.as_str();

    let labels_body = serde_json::json!(["sdlc:idea"]);
    let (labels_status, _) = github_request(
        &token,
        "PUT",
        &format!("/repos/{SDLC_OWNER}/{SDLC_REPO}/issues/{issue_number}/labels"),
        Some(&labels_body),
    );
    assert_eq!(labels_status, 200, "issue relabel to sdlc:idea should succeed");

    run_dispatch_local(
        SDLC_OWNER,
        SDLC_REPO,
        &token,
        &api_key,
        &llm_provider,
        &llm_model,
    );

    let (issue_status, issue_json) = github_request(
        &token,
        "GET",
        &format!("/repos/{SDLC_OWNER}/{SDLC_REPO}/issues/{issue_number}"),
        None,
    );
    assert_eq!(issue_status, 200, "issue fetch should succeed after dispatch");
    let labels = issue_json["labels"].as_array().cloned().unwrap_or_default();
    let has_design_label = labels
        .iter()
        .any(|label| label["name"].as_str() == Some("sdlc:design"));
    assert!(
        has_design_label,
        "issue should advance to `sdlc:design` after one local dispatch"
    );

    let (comments_status, comments_json) = github_request(
        &token,
        "GET",
        &format!(
            "/repos/{SDLC_OWNER}/{SDLC_REPO}/issues/{issue_number}/comments?per_page=100"
        ),
        None,
    );
    assert_eq!(comments_status, 200, "comments fetch should succeed");
    let has_design_comment = comments_json
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|comment| comment["body"].as_str())
        .any(|body| body.contains("Generated Design"));
    assert!(
        has_design_comment,
        "issue should have a generated design comment after the local run"
    );

    let outcomes_root = Path::new("target/sdlc/outcomes");
    let mut files = Vec::new();
    collect_files(outcomes_root, &mut files);
    assert!(
        !files.is_empty(),
        "local run should leave at least one outcome ledger file under target/sdlc/outcomes"
    );
}
