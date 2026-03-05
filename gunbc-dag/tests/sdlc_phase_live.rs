#![allow(clippy::disallowed_methods)] // Live integration harnesses shell out intentionally.

use gunbc_dag::dsl_builder::build_dsl_graph_with_profile;
use gunbc_exec::{execute_with_mode_and_inputs, lower, BoundaryMocks, ExecutionMode};
use gunbc_ir::{detect_entrypoints, Dag, Value as DagValue};
use gunbc_test::{guard_test, FermiCost, TestClass};
use serde_json::Value as JsonValue;
use std::collections::{HashSet, VecDeque};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const SDLC_LOCAL_PROFILE: &str = "profiles.sdlc.local";
const SDLC_CLOUD_RUN_PROFILE: &str = "profiles.sdlc.cloud_run";

fn should_run(name: &str, cost: FermiCost, requires: &[&str], secrets: &[&str]) -> bool {
    guard_test(name, TestClass::Integration, cost, requires, secrets)
}

fn connected_subdag<T: Clone>(dag: &Dag<T>, seed_prefix: &str) -> Dag<T> {
    let seed = dag
        .nodes
        .iter()
        .find(|node| node.id.0.starts_with(seed_prefix))
        .map(|node| node.id.0.clone())
        .unwrap_or_else(|| panic!("seed node prefix `{seed_prefix}` not found in DAG"));

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    visited.insert(seed.clone());
    queue.push_back(seed);

    while let Some(current) = queue.pop_front() {
        for edge in &dag.edges {
            let neighbor = if edge.from_node.0 == current {
                Some(edge.to_node.0.clone())
            } else if edge.to_node.0 == current {
                Some(edge.from_node.0.clone())
            } else {
                None
            };
            if let Some(node_id) = neighbor {
                if visited.insert(node_id.clone()) {
                    queue.push_back(node_id);
                }
            }
        }
    }

    let mut subdag = dag.clone();
    subdag.nodes.retain(|node| visited.contains(&node.id.0));
    subdag.edges.retain(|edge| {
        visited.contains(&edge.from_node.0) && visited.contains(&edge.to_node.0)
    });
    subdag
}

fn node_ids(dag: &Dag<gunbc_exec::DynOp>) -> Vec<String> {
    dag.nodes.iter().map(|node| node.id.0.clone()).collect()
}

fn has_prefix(node_ids: &[String], prefix: &str) -> bool {
    node_ids.iter().any(|id| id.starts_with(prefix))
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{}", std::process::id(), nanos)
}

fn run_command(mut cmd: Command, description: &str) -> String {
    let output = cmd.output().unwrap_or_else(|err| {
        panic!("failed to spawn command for `{description}`: {err}")
    });
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

fn env_var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("missing required env var `{name}`"))
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

fn run_dispatch_local(owner: &str, repo: &str, llm_provider: &str, llm_model: &str) {
    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", SDLC_LOCAL_PROFILE)
        .expect("local profile worker graph should compile");
    let dag = connected_subdag(&dag, "funcs.sdlc_worker::dispatch_sdlc");

    let lowered = lower(&dag).expect("lower worker graph for entrypoint detection");
    let entrypoints = detect_entrypoints(&lowered.dag);
    let mut input_mocks = BoundaryMocks::new();
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "owner" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                DagValue::Str(owner.to_string()),
            ),
            "repo" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                DagValue::Str(repo.to_string()),
            ),
            "worker_id" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                DagValue::Str("sdlc-live-test-worker".to_string()),
            ),
            "llm_provider" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                DagValue::Str(llm_provider.to_string()),
            ),
            "llm_model" => input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                DagValue::Str(llm_model.to_string()),
            ),
            _ => {}
        }
    }

    let _log = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&input_mocks))
        .expect("local dispatch run should succeed");
}

#[test]
fn s9_issue_provider_live_operations_against_github() {
    if !should_run(
        "s9_issue_provider_live_operations_against_github",
        FermiCost::L,
        &["shell", "curl"],
        &["SDLC_GITHUB_OWNER", "SDLC_GITHUB_REPO", "GITHUB_TOKEN"],
    ) {
        return;
    }
    if !env_flag_enabled("SDLC_ALLOW_MUTATION") {
        eprintln!("skipping S-9 live ops: set SDLC_ALLOW_MUTATION=1 to enable");
        return;
    }

    let token = match env_var_nonempty("GITHUB_TOKEN") {
        Some(token) => token,
        None => {
            eprintln!("skipping S-9 live ops: set GITHUB_TOKEN");
            return;
        }
    };
    let owner = env_var("SDLC_GITHUB_OWNER");
    let repo = env_var("SDLC_GITHUB_REPO");
    let suffix = unique_suffix();

    let create_body = serde_json::json!({
        "title": format!("[sdlc-live] provider ops {}", suffix),
        "body": "Integration check for S-9 IssueProvider operations.",
        "labels": ["sdlc:idea"]
    });
    let (create_status, created_issue) = github_request(
        &token,
        "POST",
        &format!("/repos/{owner}/{repo}/issues"),
        Some(&create_body),
    );
    assert_eq!(create_status, 201, "create should return HTTP 201");
    let issue_number = created_issue["number"]
        .as_i64()
        .expect("created issue should contain numeric `number`");
    let issue_id = issue_number.to_string();

    let (discover_status, discovered) = github_request(
        &token,
        "GET",
        &format!("/repos/{owner}/{repo}/issues?labels=sdlc:idea&state=open&per_page=100"),
        None,
    );
    assert_eq!(discover_status, 200, "discover should return HTTP 200");
    assert!(
        discovered
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .any(|issue| issue["number"].as_i64() == Some(issue_number)),
        "discover response should include created issue #{issue_number}"
    );

    let (get_status, got_issue) = github_request(
        &token,
        "GET",
        &format!("/repos/{owner}/{repo}/issues/{issue_id}"),
        None,
    );
    assert_eq!(get_status, 200, "get should return HTTP 200");
    assert_eq!(
        got_issue["number"].as_i64(),
        Some(issue_number),
        "get should return requested issue"
    );

    let comment_body = serde_json::json!({ "body": format!("S-9 integration comment {}", suffix) });
    let (comment_status, comment) = github_request(
        &token,
        "POST",
        &format!("/repos/{owner}/{repo}/issues/{issue_id}/comments"),
        Some(&comment_body),
    );
    assert_eq!(comment_status, 201, "comment should return HTTP 201");
    assert!(
        comment["id"].as_i64().is_some(),
        "comment response should include numeric id"
    );

    let labels_body = serde_json::json!(["sdlc:idea", "sdlc:design"]);
    let (labels_status, labels) = github_request(
        &token,
        "PUT",
        &format!("/repos/{owner}/{repo}/issues/{issue_id}/labels"),
        Some(&labels_body),
    );
    assert_eq!(labels_status, 200, "set_labels should return HTTP 200");
    assert!(
        labels
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .any(|label| label["name"].as_str() == Some("sdlc:design")),
        "set_labels response should include `sdlc:design`"
    );

    let (events_status, events) = github_request(
        &token,
        "GET",
        &format!("/repos/{owner}/{repo}/issues/{issue_id}/events?per_page=100"),
        None,
    );
    assert_eq!(events_status, 200, "list_events should return HTTP 200");
    assert!(events.is_array(), "list_events should return an array");

    let close_body = serde_json::json!({ "state": "closed" });
    let (close_status, closed_issue) = github_request(
        &token,
        "PATCH",
        &format!("/repos/{owner}/{repo}/issues/{issue_id}"),
        Some(&close_body),
    );
    assert_eq!(close_status, 200, "close should return HTTP 200");
    assert_eq!(
        closed_issue["state"].as_str(),
        Some("closed"),
        "issue should be closed"
    );
}

#[test]
fn s10_local_profile_credential_wiring_compiles_and_authenticates() {
    if !should_run(
        "s10_local_profile_credential_wiring_compiles_and_authenticates",
        FermiCost::M,
        &["shell", "curl"],
        &["GITHUB_TOKEN", "CODEX_API_KEY"],
    ) {
        return;
    }

    if env_var_nonempty("CODEX_API_KEY").is_none() {
        eprintln!("skipping S-10 local profile auth: set CODEX_API_KEY");
        return;
    }

    let token = match env_var_nonempty("GITHUB_TOKEN") {
        Some(token) => token,
        None => {
            eprintln!("skipping S-10 local profile auth: set GITHUB_TOKEN");
            return;
        }
    };

    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", SDLC_LOCAL_PROFILE)
        .expect("local profile worker graph should compile");
    let ids = node_ids(&dag);
    assert!(
        has_prefix(
            &ids,
            "execute_transport_extdeps_sdlc_providers_github_issue_provider_github_IssueProvider_discover"
        ),
        "local profile should include GitHub IssueProvider transport nodes"
    );
    assert!(
        has_prefix(
            &ids,
            "execute_transport_extdeps_sdlc_providers_gcp_credential_provider_GcpWifCredentialProvider_acquire"
        ),
        "local profile should include credential provider transport nodes"
    );

    let (status, user_json) = github_request(&token, "GET", "/user", None);
    assert_eq!(
        status, 200,
        "authenticated /user call should succeed (token wiring should not be 401)"
    );
    assert!(
        user_json["login"].as_str().is_some(),
        "authenticated /user response should contain `login`"
    );
}

#[test]
fn s11_local_profile_design_stage_e2e() {
    if !should_run(
        "s11_local_profile_design_stage_e2e",
        FermiCost::XL,
        &["shell", "curl"],
        &[
            "SDLC_GITHUB_OWNER",
            "SDLC_GITHUB_REPO",
            "SDLC_TEST_ISSUE_NUMBER",
            "SDLC_LLM_PROVIDER",
            "SDLC_LLM_MODEL",
            "GITHUB_TOKEN",
            "CODEX_API_KEY",
        ],
    ) {
        return;
    }
    if !env_flag_enabled("SDLC_ALLOW_MUTATION") {
        eprintln!("skipping S-11 local e2e: set SDLC_ALLOW_MUTATION=1 to enable");
        return;
    }

    let token = match env_var_nonempty("GITHUB_TOKEN") {
        Some(token) => token,
        None => {
            eprintln!("skipping S-11 local e2e: set GITHUB_TOKEN");
            return;
        }
    };
    if env_var_nonempty("CODEX_API_KEY").is_none() {
        eprintln!("skipping S-11 local e2e: set CODEX_API_KEY");
        return;
    }
    let owner = env_var("SDLC_GITHUB_OWNER");
    let repo = env_var("SDLC_GITHUB_REPO");
    let issue_number = env_var("SDLC_TEST_ISSUE_NUMBER");
    let llm_provider = env_var("SDLC_LLM_PROVIDER");
    let llm_model = env_var("SDLC_LLM_MODEL");

    let labels_body = serde_json::json!(["sdlc:idea"]);
    let (labels_status, _) = github_request(
        &token,
        "PUT",
        &format!("/repos/{owner}/{repo}/issues/{issue_number}/labels"),
        Some(&labels_body),
    );
    assert_eq!(labels_status, 200, "issue relabel to sdlc:idea should succeed");

    run_dispatch_local(&owner, &repo, &llm_provider, &llm_model);
    run_dispatch_local(&owner, &repo, &llm_provider, &llm_model);

    let (issue_status, issue_json) = github_request(
        &token,
        "GET",
        &format!("/repos/{owner}/{repo}/issues/{issue_number}"),
        None,
    );
    assert_eq!(issue_status, 200, "issue fetch should succeed after dispatch");
    let labels = issue_json["labels"].as_array().cloned().unwrap_or_default();
    let has_design_flow_label = labels.iter().any(|label| {
        matches!(
            label["name"].as_str(),
            Some("sdlc:design") | Some("sdlc:design-review") | Some("sdlc:accepted")
        )
    });
    assert!(
        has_design_flow_label,
        "issue should advance into design flow labels after dispatch"
    );

    let (comments_status, comments_json) = github_request(
        &token,
        "GET",
        &format!("/repos/{owner}/{repo}/issues/{issue_number}/comments?per_page=100"),
        None,
    );
    assert_eq!(comments_status, 200, "comments fetch should succeed");
    let has_design_comment = comments_json
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|comment| comment["body"].as_str())
        .any(|body| body.contains("Generated Design") || body.contains("Design Review"));
    assert!(
        has_design_comment,
        "issue should have design or design-review artifact comment after local run"
    );

    let outcomes_root = Path::new("target/sdlc/outcomes");
    let mut files = Vec::new();
    collect_files(outcomes_root, &mut files);
    assert!(
        !files.is_empty(),
        "local run should produce at least one outcome ledger file under target/sdlc/outcomes"
    );
}

#[test]
fn s12_to_s15_local_pipeline_wiring_is_present() {
    if !should_run(
        "s12_to_s15_local_pipeline_wiring_is_present",
        FermiCost::XS,
        &[],
        &["GITHUB_TOKEN", "CODEX_API_KEY"],
    ) {
        return;
    }
    if env_var_nonempty("GITHUB_TOKEN").is_none() {
        eprintln!("skipping S-12..S-15 local wiring: set GITHUB_TOKEN");
        return;
    }
    if env_var_nonempty("CODEX_API_KEY").is_none() {
        eprintln!("skipping S-12..S-15 local wiring: set CODEX_API_KEY");
        return;
    }

    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", SDLC_LOCAL_PROFILE)
        .expect("local profile worker graph should compile");
    let ids = node_ids(&dag);

    let expected_prefixes = [
        "execute_transport_extdeps_sdlc_providers_codex_agent_provider_codex_AgentProvider_spawn",
        "execute_transport_extdeps_sdlc_providers_codex_agent_provider_codex_AgentProvider_poll",
        "execute_transport_extdeps_github_pull_requests_github_PullRequest_Create",
        "execute_transport_extdeps_github_pull_requests_github_PullRequest_ListFiles",
        "execute_transport_extdeps_github_pull_requests_github_PullRequest_AddComment",
        "execute_transport_extdeps_llm_anthropic_llm_Anthropic_Messages",
        "execute_transport_extdeps_cargo_cargo_Build_Test",
        "execute_transport_extdeps_cargo_cargo_Build_Clippy",
        "funcs.sdlc_stages::handle_accepted_to_implementing",
        "funcs.sdlc_stages::handle_implementing_to_code_review",
        "funcs.sdlc_stages::handle_code_review_to_testing",
        "funcs.sdlc_stages::handle_testing_to_done",
    ];

    for prefix in &expected_prefixes {
        assert!(
            has_prefix(&ids, prefix),
            "local profile graph should include phase 3 marker node `{prefix}`"
        );
    }
}

#[test]
fn s16_to_s19_cloud_run_wiring_is_present() {
    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", SDLC_CLOUD_RUN_PROFILE)
        .expect("cloud_run profile worker graph should compile");
    let ids = node_ids(&dag);

    let expected_prefixes = [
        "execute_transport_extdeps_sdlc_providers_gcs_claim_store_gcs_ClaimStore_acquire",
        "execute_transport_extdeps_sdlc_providers_gcs_outcome_ledger_gcs_OutcomeLedger_upsert",
        "execute_transport_extdeps_sdlc_providers_gcs_artifact_store_gcs_ArtifactStore_store",
        "execute_transport_extdeps_sdlc_providers_pubsub_signal_store_pubsub_SignalStore_emit",
        "execute_transport_extdeps_sdlc_providers_pubsub_signal_store_pubsub_SignalStore_consume",
        "execute_transport_extdeps_sdlc_providers_github_issue_provider_github_IssueProvider_discover",
    ];

    for prefix in &expected_prefixes {
        assert!(
            has_prefix(&ids, prefix),
            "cloud_run profile graph should include phase 4 marker node `{prefix}`"
        );
    }
}
