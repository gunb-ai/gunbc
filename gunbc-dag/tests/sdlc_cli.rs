#![allow(clippy::disallowed_methods)]
use std::path::Path;
mod common;
use common::fixture::CliTestContext;
use std::process::Command;

fn sdlc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_gunbc-sdlc")
}

fn write_intent_file(path: &Path, intent_id: &str, intake_key: &str, issue_id: Option<u64>) {
    write_intent_file_with_git(path, intent_id, intake_key, issue_id, true);
}

fn write_intent_file_without_git(
    path: &Path,
    intent_id: &str,
    intake_key: &str,
    issue_id: Option<u64>,
) {
    write_intent_file_with_git(path, intent_id, intake_key, issue_id, false);
}

fn write_intent_file_with_git(
    path: &Path,
    intent_id: &str,
    intake_key: &str,
    issue_id: Option<u64>,
    initialize_git: bool,
) {
    let root = path
        .parent()
        .expect("intent fixture path should have parent");
    if initialize_git {
        ensure_git_repo_with_commit(root);
    }
    write_infra_intent_template(root, true);
    let issue_id_yaml = match issue_id {
        Some(value) => value.to_string(),
        None => "null".to_string(),
    };
    let content = format!(
        r#"intent_id: "{intent_id}"
title: "Intent {intent_id}"
objective: "Ship intent {intent_id}"
provider: "github"
success_criteria:
  - "criterion 1"
constraints:
  - "constraint 1"
owner: "team-sdlc"
priority: "M"
scope:
  in:
    - "scope in"
  out:
    - "scope out"
links:
  docs:
    - "docs/design/sdlc/mega-modeling-design.md"
  related_issues:
    - "123"
idempotency:
  intake_key: "{intake_key}"
  policy_version: "1"
update_strategy:
  comment_mode: "upsert-by-marker"
  transition_mode: "compare-and-set"
tracking:
  issue_id: {issue_id_yaml}
  run_key: null
acceptance_tests:
  - "cargo test -q -p gunbc-dag --test sdlc_cli"
notes: "test fixture"
"#
    );
    std::fs::write(path, content).expect("write intent fixture");
}

fn write_infra_intent_template(root: &Path, valid: bool) {
    let todo_dir = root.join("TODO");
    std::fs::create_dir_all(&todo_dir).expect("create TODO directory for infra intent");
    let lease_ttl = if valid { 120 } else { 5 };
    let heartbeat = if valid { 30 } else { 10 };
    let content = format!(
        r#"(
    schema_version: "1",
    intent_id: "infra-20260221-runtime-profile",
    environment: "dev",
    runtime_profile: "stateless-fleet",
    provider: "github",
    policy_version: "1",
    components: (
        claim_store: (
            backend: "sqlite",
            dsn: "var/sdlc/claims.db",
        ),
        outcome_ledger: (
            backend: "sqlite",
            dsn: "var/sdlc/outcomes.db",
        ),
        secrets: (
            credential_policy_profile: "default",
            required_refs: [
                "github-token",
                "openai-api-key",
            ],
        ),
        metrics: (
            sink: "stdout",
            namespace: "gunbc.sdlc",
        ),
    ),
    safety: (
        fail_closed_on_missing_prereqs: true,
        require_capability_gate: true,
    ),
    launch: (
        worker_count: 5,
        lease_ttl_seconds: {lease_ttl},
        heartbeat_seconds: {heartbeat},
        poll_interval_seconds: 15,
    ),
    drift: (
        reconcile_mode: "plan-then-apply",
        reconcile_interval_minutes: 60,
    ),
    notes: Some("test fixture infra intent"),
)"#
    );
    std::fs::write(todo_dir.join("infra-intent-template.dag"), content)
        .expect("write infra intent template fixture");
}

fn ensure_git_repo_with_commit(root: &Path) {
    if root.join(".git").exists() {
        return;
    }
    let init = Command::new("git")
        .arg("init")
        .arg("-q")
        .current_dir(root)
        .output()
        .expect("initialize git repository for test fixture");
    assert!(
        init.status.success(),
        "git init should succeed: {}",
        String::from_utf8_lossy(&init.stderr)
    );
    let email = Command::new("git")
        .args(["config", "user.email", "sdlc-tests@example.com"])
        .current_dir(root)
        .output()
        .expect("configure git user.email");
    assert!(
        email.status.success(),
        "git config user.email should succeed: {}",
        String::from_utf8_lossy(&email.stderr)
    );
    let name = Command::new("git")
        .args(["config", "user.name", "SDLC Tests"])
        .current_dir(root)
        .output()
        .expect("configure git user.name");
    assert!(
        name.status.success(),
        "git config user.name should succeed: {}",
        String::from_utf8_lossy(&name.stderr)
    );
    std::fs::write(root.join(".git-trace"), "trace-seed\n").expect("write git trace seed file");
    let add = Command::new("git")
        .args(["add", ".git-trace"])
        .current_dir(root)
        .output()
        .expect("stage git trace seed file");
    assert!(
        add.status.success(),
        "git add should succeed: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    let commit = Command::new("git")
        .args(["commit", "-q", "-m", "seed trace metadata"])
        .current_dir(root)
        .output()
        .expect("create initial git commit for trace linkage");
    assert!(
        commit.status.success(),
        "git commit should succeed: {}",
        String::from_utf8_lossy(&commit.stderr)
    );
}

#[test]
fn intake_dry_run_computes_run_key_without_writing_ledger() {
    let ctx = CliTestContext::new("dry_run", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-dry-run",
        "intent-20260221-dry-run",
        None,
    );

    let output = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .arg("--dry-run")
        .current_dir(&root)
        .output()
        .expect("run sdlc intake dry-run");

    assert!(
        output.status.success(),
        "dry-run intake should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("dry-run output should be JSON");
    assert_eq!(payload["command"], "intake");
    assert_eq!(payload["mode"], "dry-run");
    assert_eq!(payload["intent_id"], "intent-20260221-dry-run");
    assert!(payload["run_key"]
        .as_str()
        .expect("run_key should be a string")
        .contains("intent-20260221-dry-run"));
    assert!(
        !root.join("target/sdlc/intake-ledger.json").exists(),
        "dry-run intake must not write ledger state"
    );
    assert!(
        !root.join("target/sdlc/artifact-ledger.json").exists(),
        "dry-run intake must not write artifact ledger state"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn intake_real_mode_fails_closed_without_git_trace_context() {
    let ctx = CliTestContext::new("missing_git_trace", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file_without_git(
        &intent_path,
        "intent-20260221-missing-git",
        "intent-20260221-missing-git",
        None,
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake without git metadata");
    assert!(
        !intake.status.success(),
        "intake should fail closed without git trace context"
    );
    let stderr = String::from_utf8_lossy(&intake.stderr);
    assert!(
        stderr.contains("trace linkage requires git metadata"),
        "stderr should explain missing git metadata requirement: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_real_mode_fails_closed_when_infra_preflight_invalid() {
    let ctx = CliTestContext::new("invalid_infra_preflight", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-invalid-preflight",
        "intent-20260221-invalid-preflight",
        Some(1234),
    );
    write_infra_intent_template(&root, false);

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake before worker");
    assert!(
        intake.status.success(),
        "intake should succeed before preflight failure: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run worker with invalid infra preflight");
    assert!(
        !worker.status.success(),
        "worker should fail closed when infra preflight fails"
    );
    let stderr = String::from_utf8_lossy(&worker.stderr);
    assert!(
        stderr.contains("lease_ttl_seconds must be >= launch.heartbeat_seconds"),
        "preflight failure should explain invalid launch configuration: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_real_mode_fails_closed_when_infra_schema_version_unsupported() {
    let ctx = CliTestContext::new("invalid_infra_schema_version", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-invalid-schema",
        "intent-20260221-invalid-schema",
        Some(1200),
    );

    let infra_path = root.join("TODO/infra-intent-template.dag");
    let infra_content = std::fs::read_to_string(&infra_path).expect("read infra template");
    let invalid_schema = infra_content.replace("schema_version: \"1\"", "schema_version: \"2\"");
    std::fs::write(&infra_path, invalid_schema).expect("write invalid infra schema");

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake before worker");
    assert!(
        intake.status.success(),
        "intake should succeed before schema preflight failure: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run worker with unsupported infra schema");
    assert!(
        !worker.status.success(),
        "worker should fail closed when infra schema_version is unsupported"
    );
    let stderr = String::from_utf8_lossy(&worker.stderr);
    assert!(
        stderr.contains("unsupported infra schema_version"),
        "preflight failure should explain unsupported schema version: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_real_mode_fails_closed_when_infra_required_refs_missing_provider_token() {
    let ctx = CliTestContext::new("invalid_infra_required_refs", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-invalid-required-refs",
        "intent-20260221-invalid-required-refs",
        Some(1201),
    );

    let infra_path = root.join("TODO/infra-intent-template.dag");
    let infra_content = std::fs::read_to_string(&infra_path).expect("read infra template");
    let invalid_refs = infra_content.replace("\"github-token\",\n", "");
    std::fs::write(&infra_path, invalid_refs).expect("write invalid required refs");

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake before worker");
    assert!(
        intake.status.success(),
        "intake should succeed before required-ref preflight failure: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run worker with missing github-token reference");
    assert!(
        !worker.status.success(),
        "worker should fail closed when provider-required secret refs are missing"
    );
    let stderr = String::from_utf8_lossy(&worker.stderr);
    assert!(
        stderr.contains("must include `github-token`"),
        "preflight failure should explain missing provider token reference: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn drain_command_toggles_worker_drain_flag() {
    let ctx = CliTestContext::new("drain_toggle", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");

    let activate = ctx
        .command()
        .arg("drain")
        .arg("--activate")
        .current_dir(&root)
        .output()
        .expect("activate drain flag");
    assert!(
        activate.status.success(),
        "drain activate should succeed: {}",
        String::from_utf8_lossy(&activate.stderr)
    );
    let activate_payload: serde_json::Value =
        serde_json::from_slice(&activate.stdout).expect("activate output should be JSON");
    assert_eq!(activate_payload["command"], "drain");
    assert_eq!(activate_payload["active"], true);
    assert!(
        root.join("target/sdlc/worker-drain.flag").exists(),
        "drain flag file should exist after activation"
    );

    let deactivate = ctx
        .command()
        .arg("drain")
        .arg("--deactivate")
        .current_dir(&root)
        .output()
        .expect("deactivate drain flag");
    assert!(
        deactivate.status.success(),
        "drain deactivate should succeed: {}",
        String::from_utf8_lossy(&deactivate.stderr)
    );
    let deactivate_payload: serde_json::Value =
        serde_json::from_slice(&deactivate.stdout).expect("deactivate output should be JSON");
    assert_eq!(deactivate_payload["active"], false);
    assert!(
        !root.join("target/sdlc/worker-drain.flag").exists(),
        "drain flag file should be removed after deactivation"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_real_mode_honors_drain_flag_and_skips_processing() {
    let ctx = CliTestContext::new("worker_drain_active", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-drain",
        "intent-20260221-drain",
        Some(8080),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake before worker drain test");
    assert!(
        intake.status.success(),
        "intake should succeed before drain test: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let activate = ctx
        .command()
        .arg("drain")
        .arg("--activate")
        .current_dir(&root)
        .output()
        .expect("activate drain");
    assert!(
        activate.status.success(),
        "drain activate should succeed: {}",
        String::from_utf8_lossy(&activate.stderr)
    );

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run worker under drain");
    assert!(
        worker.status.success(),
        "worker should succeed under active drain mode: {}",
        String::from_utf8_lossy(&worker.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
    assert_eq!(payload["command"], "worker");
    assert_eq!(payload["drain"]["active"], true);
    assert_eq!(
        payload["acquired_claims"].as_array().map(|v| v.len()),
        Some(0)
    );
    assert_eq!(payload["pending_count"], 0);

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_real_mode_supports_local_co_located_profile_preflight() {
    let ctx = CliTestContext::new("worker_local_co_located_profile", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-local-profile",
        "intent-20260221-local-profile",
        Some(9901),
    );

    let local_profile_intent_path = root.join("TODO/infra-intent-local.yaml");
    let local_profile_intent = r#"schema_version: "1"
intent_id: "infra-20260221-local-profile"
environment: "dev"
runtime_profile: "local-co-located"
provider: "github"
policy_version: "1"
components:
  claim_store:
    backend: "sqlite"
    dsn: "var/sdlc/local-claims.db"
  outcome_ledger:
    backend: "sqlite"
    dsn: "var/sdlc/local-outcomes.db"
  secrets:
    credential_policy_profile: "default"
    required_refs:
      - "github-token"
      - "openai-api-key"
  metrics:
    sink: "stdout"
    namespace: "gunbc.sdlc"
safety:
  fail_closed_on_missing_prereqs: true
  require_capability_gate: true
launch:
  worker_count: 1
  lease_ttl_seconds: 60
  heartbeat_seconds: 10
  poll_interval_seconds: 5
drift:
  reconcile_mode: "plan-then-apply"
  reconcile_interval_minutes: 15
notes: "local co-located profile fixture"
"#;
    std::fs::write(&local_profile_intent_path, local_profile_intent)
        .expect("write local profile infra intent fixture");

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake before worker");
    assert!(
        intake.status.success(),
        "intake should succeed before local profile worker run: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .arg("--infra-intent")
        .arg(&local_profile_intent_path)
        .current_dir(&root)
        .output()
        .expect("run worker with local profile infra intent");
    assert!(
        worker.status.success(),
        "worker should succeed for local co-located profile preflight: {}",
        String::from_utf8_lossy(&worker.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
    assert_eq!(payload["preflight"]["status"], "ok");
    assert_eq!(payload["preflight"]["runtime_profile"], "local-co-located");

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_respects_launch_worker_capacity_from_infra_intent() {
    let ctx = CliTestContext::new("worker_launch_capacity", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_a = root.join("intent_a.yaml");
    let intent_b = root.join("intent_b.yaml");
    write_intent_file(
        &intent_a,
        "intent-20260221-capacity-a",
        "intent-20260221-capacity-a",
        Some(9101),
    );
    write_intent_file(
        &intent_b,
        "intent-20260221-capacity-b",
        "intent-20260221-capacity-b",
        Some(9102),
    );

    for intent in [&intent_a, &intent_b] {
        let intake = ctx
            .command()
            .arg("intake")
            .arg("--intent")
            .arg(intent)
            .current_dir(&root)
            .output()
            .expect("run intake");
        assert!(
            intake.status.success(),
            "intake should succeed: {}",
            String::from_utf8_lossy(&intake.stderr)
        );
    }

    let constrained_intent_path = root.join("TODO/infra-intent-constrained.yaml");
    let constrained_intent = r#"schema_version: "1"
intent_id: "infra-20260221-capacity"
environment: "dev"
runtime_profile: "local-co-located"
provider: "github"
policy_version: "1"
components:
  claim_store:
    backend: "sqlite"
    dsn: "var/sdlc/capacity-claims.db"
  outcome_ledger:
    backend: "sqlite"
    dsn: "var/sdlc/capacity-outcomes.db"
  secrets:
    credential_policy_profile: "default"
    required_refs:
      - "github-token"
      - "openai-api-key"
  metrics:
    sink: "stdout"
    namespace: "gunbc.sdlc"
safety:
  fail_closed_on_missing_prereqs: true
  require_capability_gate: true
launch:
  worker_count: 1
  lease_ttl_seconds: 60
  heartbeat_seconds: 10
  poll_interval_seconds: 5
drift:
  reconcile_mode: "plan-then-apply"
  reconcile_interval_minutes: 15
notes: "capacity constrained local profile fixture"
"#;
    std::fs::write(&constrained_intent_path, constrained_intent)
        .expect("write constrained infra intent fixture");

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .arg("--infra-intent")
        .arg(&constrained_intent_path)
        .current_dir(&root)
        .output()
        .expect("run worker with constrained infra intent");
    assert!(
        worker.status.success(),
        "worker should succeed with constrained launch profile: {}",
        String::from_utf8_lossy(&worker.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
    assert_eq!(payload["preflight"]["worker_count"], 1);
    assert_eq!(payload["summary"]["capacity_deferred_count"], 1);
    assert_eq!(
        payload["skipped_capacity"][0], "intent-20260221-capacity-b",
        "capacity constrained worker should defer extra intake keys in deterministic order"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn issue_command_filters_worker_scope_by_issue_id() {
    let ctx = CliTestContext::new("issue_scope_filter", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_a = root.join("intent_a.yaml");
    let intent_b = root.join("intent_b.yaml");
    write_intent_file(
        &intent_a,
        "intent-20260221-issue-filter-a",
        "intent-20260221-issue-filter-a",
        Some(5001),
    );
    write_intent_file(
        &intent_b,
        "intent-20260221-issue-filter-b",
        "intent-20260221-issue-filter-b",
        Some(5002),
    );

    for intent in [&intent_a, &intent_b] {
        let intake = ctx
            .command()
            .arg("intake")
            .arg("--intent")
            .arg(intent)
            .current_dir(&root)
            .output()
            .expect("run intake");
        assert!(
            intake.status.success(),
            "intake should succeed: {}",
            String::from_utf8_lossy(&intake.stderr)
        );
    }

    let issue = ctx
        .command()
        .arg("issue")
        .arg("--issue-id")
        .arg("5001")
        .arg("--dry-run")
        .current_dir(&root)
        .output()
        .expect("run issue command");
    assert!(
        issue.status.success(),
        "issue command should succeed: {}",
        String::from_utf8_lossy(&issue.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&issue.stdout).expect("issue output should be JSON");
    assert_eq!(payload["command"], "issue");
    assert_eq!(payload["issue_filter"], 5001);
    assert_eq!(payload["issue_binding_found"], true);
    assert_eq!(payload["pending_count"], 1);
    assert_eq!(
        payload["intake_keys"][0], "intent-20260221-issue-filter-a",
        "issue command should only include entries for requested issue id"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn issue_command_reports_unbound_issue_when_issue_id_not_found() {
    let ctx = CliTestContext::new("issue_not_found", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-issue-not-found",
        "intent-20260221-issue-not-found",
        Some(7007),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let issue = ctx
        .command()
        .arg("issue")
        .arg("--issue-id")
        .arg("7999")
        .arg("--dry-run")
        .current_dir(&root)
        .output()
        .expect("run issue command with missing issue id");
    assert!(
        issue.status.success(),
        "issue command should still succeed and report unbound issue state: {}",
        String::from_utf8_lossy(&issue.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&issue.stdout).expect("issue output should be JSON");
    assert!(
        payload["issue_binding_found"] == false,
        "issue command should report missing intake binding for requested issue id"
    );
    assert_eq!(
        payload["pending_count"], 0,
        "missing issue binding should produce empty pending set"
    );
    assert_eq!(
        payload["intake_keys"]
            .as_array()
            .expect("intake_keys should be array")
            .len(),
        0,
        "missing issue binding should produce empty intake key list"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn issue_command_real_mode_persists_execution_report() {
    let ctx = CliTestContext::new("issue_execution_report", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-issue-report",
        "intent-20260221-issue-report",
        Some(7331),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let issue = ctx
        .command()
        .arg("issue")
        .arg("--issue-id")
        .arg("7331")
        .current_dir(&root)
        .output()
        .expect("run issue command");
    assert!(
        issue.status.success(),
        "issue command should succeed: {}",
        String::from_utf8_lossy(&issue.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&issue.stdout).expect("issue output should be JSON");
    assert_eq!(payload["command"], "issue");
    assert_eq!(payload["issue_binding_found"], true);
    assert_eq!(
        payload["execution_report_path"],
        "target/sdlc/execution-report.json"
    );

    let report_raw = std::fs::read_to_string(root.join("target/sdlc/execution-report.json"))
        .expect("read issue execution report");
    let report: serde_json::Value =
        serde_json::from_str(&report_raw).expect("parse issue execution report");
    assert_eq!(report["command"], "issue");
    assert_eq!(report["issue_filter"], 7331);

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn legacy_issue_flag_alias_fails_closed() {
    let ctx = CliTestContext::new("legacy_issue_alias", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-legacy-issue",
        "intent-20260221-legacy-issue",
        Some(6420),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let issue = ctx
        .command()
        .arg("--issue")
        .arg("6420")
        .arg("--dry-run")
        .current_dir(&root)
        .output()
        .expect("run legacy issue alias command");
    assert!(
        !issue.status.success(),
        "legacy issue alias command should fail: {}",
        String::from_utf8_lossy(&issue.stderr)
    );
    let stderr = String::from_utf8_lossy(&issue.stderr);
    assert!(
        stderr.contains("unknown command `--issue`"),
        "legacy alias failure should identify unknown command: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn legacy_issue_equals_alias_fails_closed() {
    let ctx = CliTestContext::new("legacy_issue_equals_alias", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-legacy-issue-equals",
        "intent-20260221-legacy-issue-equals",
        Some(6421),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let issue = ctx
        .command()
        .arg("--issue=6421")
        .arg("--dry-run")
        .current_dir(&root)
        .output()
        .expect("run legacy issue equals alias command");
    assert!(
        !issue.status.success(),
        "legacy issue equals alias command should fail: {}",
        String::from_utf8_lossy(&issue.stderr)
    );
    let stderr = String::from_utf8_lossy(&issue.stderr);
    assert!(
        stderr.contains("unknown command `--issue=6421`"),
        "legacy alias failure should identify unknown command: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn intake_real_mode_is_idempotent_for_same_intake_key() {
    let ctx = CliTestContext::new("idempotent", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-idempotent",
        "intent-20260221-idempotent",
        None,
    );

    let first = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run first sdlc intake");
    assert!(
        first.status.success(),
        "first intake should succeed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_payload: serde_json::Value =
        serde_json::from_slice(&first.stdout).expect("first intake output should be JSON");
    assert_eq!(first_payload["idempotent"], false);
    assert_eq!(first_payload["artifact_status"], "inserted");
    assert_eq!(
        first_payload["trace_linkage"]["intent_id"],
        "intent-20260221-idempotent"
    );
    assert!(
        first_payload["trace_linkage"]["linkage_key"]
            .as_str()
            .expect("linkage key should be string")
            .contains("run_key="),
        "trace linkage key should include run-key metadata"
    );

    let second = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run second sdlc intake");
    assert!(
        second.status.success(),
        "second intake should succeed: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_payload: serde_json::Value =
        serde_json::from_slice(&second.stdout).expect("second intake output should be JSON");
    assert_eq!(second_payload["idempotent"], true);
    assert_eq!(second_payload["artifact_status"], "noop");
    assert_eq!(
        second_payload["trace_linkage"]["intent_id"],
        "intent-20260221-idempotent"
    );

    let ledger_path = root.join("target/sdlc/intake-ledger.json");
    let ledger_raw = std::fs::read_to_string(&ledger_path).expect("read intake ledger");
    let ledger: serde_json::Value =
        serde_json::from_str(&ledger_raw).expect("ledger should be valid JSON");
    let entries = ledger["entries"]
        .as_object()
        .expect("entries should be an object");
    assert_eq!(
        entries.len(),
        1,
        "idempotent re-intake should keep a single ledger record"
    );
    assert!(entries.contains_key("intent-20260221-idempotent"));
    assert!(
        entries["intent-20260221-idempotent"]["trace_linkage"]
            .as_object()
            .is_some(),
        "ledger should persist trace linkage metadata"
    );

    let artifact_ledger_path = root.join("target/sdlc/artifact-ledger.json");
    let artifact_raw =
        std::fs::read_to_string(&artifact_ledger_path).expect("read artifact ledger");
    let artifact: serde_json::Value =
        serde_json::from_str(&artifact_raw).expect("artifact ledger should be valid JSON");
    let records = artifact["records"]
        .as_object()
        .expect("artifact records should be an object");
    assert_eq!(
        records.len(),
        1,
        "idempotent re-intake should not duplicate artifact markers"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn intake_conflict_fails_closed_for_reused_intake_key() {
    let ctx = CliTestContext::new("conflict", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_a = root.join("intent_a.yaml");
    let intent_b = root.join("intent_b.yaml");
    write_intent_file(&intent_a, "intent-20260221-a", "intent-20260221-a", None);
    write_intent_file(&intent_b, "intent-20260221-b", "intent-20260221-a", None);

    let first = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_a)
        .current_dir(&root)
        .output()
        .expect("run first sdlc intake");
    assert!(
        first.status.success(),
        "first intake should succeed: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let conflict = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_b)
        .current_dir(&root)
        .output()
        .expect("run conflicting sdlc intake");
    assert!(
        !conflict.status.success(),
        "conflicting intake must fail closed"
    );
    let stderr = String::from_utf8_lossy(&conflict.stderr);
    assert!(
        stderr.contains("intake key conflict"),
        "conflict error must explain intake key collision: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_dry_run_reports_pending_intake_keys() {
    let ctx = CliTestContext::new("worker", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-worker",
        "intent-20260221-worker",
        None,
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake before worker");
    assert!(
        intake.status.success(),
        "intake should succeed before worker run: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .arg("--dry-run")
        .current_dir(&root)
        .output()
        .expect("run worker dry-run");
    assert!(
        worker.status.success(),
        "worker dry-run should succeed: {}",
        String::from_utf8_lossy(&worker.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
    assert_eq!(payload["command"], "worker");
    assert_eq!(payload["mode"], "dry-run");
    assert_eq!(payload["pending_count"], 1);
    assert_eq!(
        payload["intake_keys"][0]
            .as_str()
            .expect("first intake key should be string"),
        "intent-20260221-worker"
    );
    assert!(
        payload["metrics"]["stage_duration_ms"]
            .as_object()
            .expect("stage_duration metrics should be map")
            .contains_key("intent-20260221-worker"),
        "worker output should include per-intake stage duration metrics"
    );
    assert_eq!(
        payload["metrics"]["llm_cost_units"]["intent-20260221-worker"], 0,
        "worker output should include per-intake llm_cost_units metrics"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_dry_run_executes_compiled_dispatch_preview_without_persisting_stage() {
    let ctx = CliTestContext::new("worker_dry_run_dispatch_preview", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-dry-run-preview",
        "intent-20260221-dry-run-preview",
        Some(5150),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake before worker");
    assert!(
        intake.status.success(),
        "intake should succeed before worker run: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .arg("--dry-run")
        .current_dir(&root)
        .output()
        .expect("run worker dry-run");
    assert!(
        worker.status.success(),
        "worker dry-run should succeed: {}",
        String::from_utf8_lossy(&worker.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
    assert_eq!(payload["mode"], "dry-run");
    assert_eq!(
        payload["dry_run_dispatches"][0]["intake_key"],
        serde_json::Value::String("intent-20260221-dry-run-preview".to_string())
    );
    assert_eq!(
        payload["dry_run_dispatches"][0]["from_stage"],
        serde_json::Value::String("idea".to_string())
    );
    assert_eq!(
        payload["dry_run_dispatches"][0]["next_stage"],
        serde_json::Value::String("design".to_string())
    );
    assert_eq!(
        payload["dry_run_dispatches"][0]["awaiting_approval"],
        serde_json::Value::Bool(false)
    );

    let intake_ledger_path = root.join("target/sdlc/intake-ledger.json");
    let intake_ledger: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&intake_ledger_path).expect("read intake ledger"),
    )
    .expect("parse intake ledger");
    assert_eq!(
        intake_ledger["entries"]["intent-20260221-dry-run-preview"]["stage"],
        serde_json::Value::String("Idea".to_string()),
        "dry-run dispatch preview must not persist stage mutations"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_dry_run_reports_runtime_params_and_fails_closed_for_unknown_param_keys() {
    let ctx = CliTestContext::new("worker_dry_run_runtime_params", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-runtime-params",
        "intent-20260221-runtime-params",
        Some(5151),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake before worker");
    assert!(
        intake.status.success(),
        "intake should succeed before worker run: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .arg("--dry-run")
        .arg("--param")
        .arg("unknown_param=unexpected")
        .arg("--param")
        .arg("run_key=override-run-key")
        .current_dir(&root)
        .output()
        .expect("run worker dry-run with runtime params");
    assert!(
        worker.status.success(),
        "worker dry-run should succeed even when dispatch preview reports param error: {}",
        String::from_utf8_lossy(&worker.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
    assert_eq!(
        payload["runtime_params"]["run_key"],
        serde_json::Value::String("override-run-key".to_string())
    );
    assert_eq!(
        payload["runtime_params"]["unknown_param"],
        serde_json::Value::String("unexpected".to_string())
    );
    assert!(
        payload["dry_run_dispatches"][0]["error"]
            .as_str()
            .expect("dry-run dispatch error should be string")
            .contains("unsupported --param key(s): unknown_param"),
        "dry-run dispatch should fail closed for unsupported runtime params"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_rejects_malformed_param_flag() {
    let ctx = CliTestContext::new("worker_rejects_malformed_param", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .arg("--dry-run")
        .arg("--param")
        .arg("invalid-param-format")
        .current_dir(&root)
        .output()
        .expect("run worker dry-run with malformed param");
    assert!(
        !worker.status.success(),
        "worker should fail closed for malformed --param input"
    );
    let stderr = String::from_utf8_lossy(&worker.stderr);
    assert!(
        stderr.contains("invalid --param `invalid-param-format`; expected <key>=<value>"),
        "stderr should explain malformed --param input: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_real_mode_persists_retry_state_on_claim_conflict() {
    let ctx = CliTestContext::new("worker_conflict_retry", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_a = root.join("intent_a.yaml");
    let intent_b = root.join("intent_b.yaml");
    write_intent_file(
        &intent_a,
        "intent-20260221-worker-a",
        "intent-20260221-worker-a",
        Some(4242),
    );
    write_intent_file(
        &intent_b,
        "intent-20260221-worker-b",
        "intent-20260221-worker-b",
        Some(4242),
    );

    for intent in [&intent_a, &intent_b] {
        let intake = ctx
            .command()
            .arg("intake")
            .arg("--intent")
            .arg(intent)
            .current_dir(&root)
            .output()
            .expect("run intake");
        assert!(
            intake.status.success(),
            "intake should succeed: {}",
            String::from_utf8_lossy(&intake.stderr)
        );
    }

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run worker");
    assert!(
        worker.status.success(),
        "worker should succeed: {}",
        String::from_utf8_lossy(&worker.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
    assert_eq!(payload["command"], "worker");
    assert_eq!(payload["mode"], "real");
    assert_eq!(payload["claim_conflicts"][0], "intent-20260221-worker-b");
    assert_eq!(payload["acquired_claims"][0], "intent-20260221-worker-a");

    let ledger_path = root.join("target/sdlc/intake-ledger.json");
    let ledger_raw = std::fs::read_to_string(&ledger_path).expect("read intake ledger");
    let ledger: serde_json::Value = serde_json::from_str(&ledger_raw).expect("parse intake ledger");
    assert_eq!(
        ledger["entries"]["intent-20260221-worker-b"]["retry"]["attempts"],
        1
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_respects_retry_backoff_and_defers_claim_attempts() {
    let ctx = CliTestContext::new("worker_retry_backoff", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_a = root.join("intent_a.yaml");
    let intent_b = root.join("intent_b.yaml");
    write_intent_file(
        &intent_a,
        "intent-20260221-backoff-a",
        "intent-20260221-backoff-a",
        Some(8181),
    );
    write_intent_file(
        &intent_b,
        "intent-20260221-backoff-b",
        "intent-20260221-backoff-b",
        Some(8181),
    );

    for intent in [&intent_a, &intent_b] {
        let intake = ctx
            .command()
            .arg("intake")
            .arg("--intent")
            .arg(intent)
            .current_dir(&root)
            .output()
            .expect("run intake");
        assert!(
            intake.status.success(),
            "intake should succeed: {}",
            String::from_utf8_lossy(&intake.stderr)
        );
    }

    let first_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run first worker");
    assert!(
        first_worker.status.success(),
        "first worker should succeed: {}",
        String::from_utf8_lossy(&first_worker.stderr)
    );
    let first_payload: serde_json::Value =
        serde_json::from_slice(&first_worker.stdout).expect("first worker output should be JSON");
    assert_eq!(
        first_payload["claim_conflicts"][0],
        "intent-20260221-backoff-b"
    );

    let second_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run second worker");
    assert!(
        second_worker.status.success(),
        "second worker should succeed: {}",
        String::from_utf8_lossy(&second_worker.stderr)
    );
    let second_payload: serde_json::Value =
        serde_json::from_slice(&second_worker.stdout).expect("second worker output should be JSON");
    assert_eq!(
        second_payload["skipped_retry_backoff"][0], "intent-20260221-backoff-b",
        "second worker should defer conflicted intake while retry backoff is active"
    );
    assert_eq!(
        second_payload["summary"]["retry_backoff_deferred_count"], 1,
        "summary should report deferred backoff count"
    );
    assert_eq!(
        second_payload["claim_conflicts"]
            .as_array()
            .expect("claim_conflicts should be array")
            .len(),
        0,
        "second worker should not re-attempt conflicted claim during backoff window"
    );

    let ledger_path = root.join("target/sdlc/intake-ledger.json");
    let ledger_raw = std::fs::read_to_string(&ledger_path).expect("read intake ledger");
    let ledger: serde_json::Value = serde_json::from_str(&ledger_raw).expect("parse intake ledger");
    assert_eq!(
        ledger["entries"]["intent-20260221-backoff-b"]["retry"]["attempts"], 1,
        "retry attempts should remain stable while backoff defers claim attempts"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_real_mode_persists_execution_report_with_metrics() {
    let ctx = CliTestContext::new("worker_execution_report", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-execution-report",
        "intent-20260221-execution-report",
        Some(6767),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run worker");
    assert!(
        worker.status.success(),
        "worker should succeed: {}",
        String::from_utf8_lossy(&worker.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
    let report_path = root.join("target/sdlc/execution-report.json");
    assert_eq!(
        payload["execution_report_path"], "target/sdlc/execution-report.json",
        "worker output should expose persisted execution report path"
    );
    let report_raw = std::fs::read_to_string(&report_path).expect("read execution report");
    let report: serde_json::Value =
        serde_json::from_str(&report_raw).expect("parse execution report");
    assert_eq!(report["command"], "worker");
    assert!(
        report["report_generated_at_epoch_ms"].as_u64().is_some(),
        "execution report should include report generation timestamp"
    );
    assert_eq!(report["summary"]["intake_total"], 1);
    assert_eq!(report["summary"]["executed_count"], 1);
    assert_eq!(report["summary"]["terminalized_count"], 0);
    assert!(
        report["metrics"]["stage_duration_ms"]
            .as_object()
            .expect("stage_duration metrics should be map")
            .contains_key("intent-20260221-execution-report"),
        "execution report should include stage duration metrics"
    );
    assert!(
        report["metrics"]["llm_cost_units"]
            .as_object()
            .expect("llm cost metrics should be map")
            .contains_key("intent-20260221-execution-report"),
        "execution report should include llm cost metrics"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_terminalizes_after_retry_budget_exhaustion() {
    let ctx = CliTestContext::new("worker_terminalize", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-term-b",
        "intent-20260221-term-b",
        Some(777),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let mut final_payload = serde_json::Value::Null;
    for _ in 0..3 {
        // Inject a foreign claim for issue:777:stage:Idea owned by another worker.
        // This blocks our worker from acquiring the claim, producing a conflict.
        let claim_ledger_path = root.join("target/sdlc/claim-ledger.json");
        let foreign_claim = serde_json::json!({
            "claims": {
                "issue:777:stage:idea": {
                    "owner": "gunbc-sdlc-worker:foreign-worker:foreign-intent",
                    "claimed_at_epoch_ms": 1000,
                    "lease_expires_at_epoch_ms": 9999999999999u64
                }
            }
        });
        std::fs::write(
            &claim_ledger_path,
            serde_json::to_string_pretty(&foreign_claim).expect("serialize foreign claim"),
        )
        .expect("write claim ledger with foreign claim");

        let worker = ctx
            .command()
            .arg("worker")
            .arg("--worker-id")
            .arg("test-worker-id")
            .current_dir(&root)
            .output()
            .expect("run worker");
        assert!(
            worker.status.success(),
            "worker should succeed: {}",
            String::from_utf8_lossy(&worker.stderr)
        );
        final_payload =
            serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
        let ledger_path = root.join("target/sdlc/intake-ledger.json");
        let mut ledger: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&ledger_path).expect("read intake ledger"),
        )
        .expect("parse intake ledger");
        if ledger["entries"]["intent-20260221-term-b"]["terminalized"] != true {
            ledger["entries"]["intent-20260221-term-b"]["retry"]["next_retry_at_epoch_ms"] =
                serde_json::Value::Null;
            std::fs::write(
                &ledger_path,
                serde_json::to_string_pretty(&ledger).expect("serialize intake ledger"),
            )
            .expect("write intake ledger with forced immediate retry");
        }
    }

    let ledger_path = root.join("target/sdlc/intake-ledger.json");
    let ledger_raw = std::fs::read_to_string(&ledger_path).expect("read intake ledger");
    let ledger: serde_json::Value = serde_json::from_str(&ledger_raw).expect("parse intake ledger");
    assert_eq!(
        ledger["entries"]["intent-20260221-term-b"]["terminalized"],
        true
    );
    assert_eq!(
        ledger["entries"]["intent-20260221-term-b"]["retry"]["attempts"],
        3
    );
    assert!(
        final_payload["terminal_failures"]["intent-20260221-term-b"]
            .as_str()
            .expect("terminal failure reason should be string")
            .contains("claim conflict"),
        "worker output should expose terminal failure reason for exhausted retries"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn await_approval_releases_claim_on_next_worker_pass() {
    let ctx = CliTestContext::new("await_approval_release", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-await-approval",
        "intent-20260221-await-approval",
        Some(9001),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let first_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run first worker");
    assert!(
        first_worker.status.success(),
        "first worker should succeed: {}",
        String::from_utf8_lossy(&first_worker.stderr)
    );

    let await_approval = ctx
        .command()
        .arg("await-approval")
        .arg("--intake-key")
        .arg("intent-20260221-await-approval")
        .current_dir(&root)
        .output()
        .expect("run await-approval");
    assert!(
        await_approval.status.success(),
        "await-approval should succeed: {}",
        String::from_utf8_lossy(&await_approval.stderr)
    );

    let second_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run second worker");
    assert!(
        second_worker.status.success(),
        "second worker should succeed: {}",
        String::from_utf8_lossy(&second_worker.stderr)
    );
    let second_payload: serde_json::Value =
        serde_json::from_slice(&second_worker.stdout).expect("second worker output should be JSON");
    assert_eq!(
        second_payload["released_claims"][0],
        "intent-20260221-await-approval"
    );
    assert!(
        second_payload["metrics"]["approval_latency_ms"]
            .as_object()
            .expect("approval latency metrics should be map")
            .contains_key("intent-20260221-await-approval"),
        "worker metrics should include approval latency for awaiting approval entries"
    );

    let claim_ledger_path = root.join("target/sdlc/claim-ledger.json");
    let claim_ledger_raw = std::fs::read_to_string(&claim_ledger_path).expect("read claim ledger");
    let claim_ledger: serde_json::Value =
        serde_json::from_str(&claim_ledger_raw).expect("parse claim ledger");
    let claims = claim_ledger["claims"]
        .as_object()
        .expect("claims should be object");
    assert!(
        claims.is_empty(),
        "awaiting approval should release held claim slot"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_can_emit_pending_approval_exit_code_for_workflow_yield() {
    let ctx = CliTestContext::new("worker_pending_exit_code", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-pending-exit",
        "intent-20260221-pending-exit",
        Some(4040),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let first_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run first worker");
    assert!(
        first_worker.status.success(),
        "first worker should succeed: {}",
        String::from_utf8_lossy(&first_worker.stderr)
    );

    let await_approval = ctx
        .command()
        .arg("await-approval")
        .arg("--intake-key")
        .arg("intent-20260221-pending-exit")
        .current_dir(&root)
        .output()
        .expect("run await-approval");
    assert!(
        await_approval.status.success(),
        "await-approval should succeed: {}",
        String::from_utf8_lossy(&await_approval.stderr)
    );

    let pending_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .arg("--emit-pending-exit-code")
        .current_dir(&root)
        .output()
        .expect("run worker with pending exit code");
    assert_eq!(
        pending_worker.status.code(),
        Some(42),
        "worker should exit with pending approval code"
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&pending_worker.stdout).expect("worker output should be JSON");
    assert_eq!(
        payload["awaiting_approval"][0],
        "intent-20260221-pending-exit"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn issue_command_can_emit_pending_approval_exit_code_for_filtered_issue() {
    let ctx = CliTestContext::new("issue_pending_exit_code", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-issue-pending-exit",
        "intent-20260221-issue-pending-exit",
        Some(5050),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let first_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run first worker");
    assert!(
        first_worker.status.success(),
        "first worker should succeed: {}",
        String::from_utf8_lossy(&first_worker.stderr)
    );

    let await_approval = ctx
        .command()
        .arg("await-approval")
        .arg("--intake-key")
        .arg("intent-20260221-issue-pending-exit")
        .current_dir(&root)
        .output()
        .expect("run await-approval");
    assert!(
        await_approval.status.success(),
        "await-approval should succeed: {}",
        String::from_utf8_lossy(&await_approval.stderr)
    );

    let pending_issue = ctx
        .command()
        .arg("issue")
        .arg("--issue-id")
        .arg("5050")
        .arg("--emit-pending-exit-code")
        .current_dir(&root)
        .output()
        .expect("run issue command with pending exit code");
    assert_eq!(
        pending_issue.status.code(),
        Some(42),
        "issue command should emit pending approval exit code for filtered issue"
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&pending_issue.stdout).expect("issue output should be JSON");
    assert_eq!(payload["command"], "issue");
    assert_eq!(payload["issue_filter"], 5050);
    assert_eq!(
        payload["awaiting_approval"][0],
        "intent-20260221-issue-pending-exit"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn transition_enforces_stage_ordering_fail_closed() {
    let ctx = CliTestContext::new("transition_order", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-transition",
        "intent-20260221-transition",
        Some(51),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let to_design = ctx
        .command()
        .arg("transition")
        .arg("--intake-key")
        .arg("intent-20260221-transition")
        .arg("--stage")
        .arg("design")
        .current_dir(&root)
        .output()
        .expect("run transition to design");
    assert!(
        to_design.status.success(),
        "idea -> design transition should succeed: {}",
        String::from_utf8_lossy(&to_design.stderr)
    );
    let to_design_payload: serde_json::Value =
        serde_json::from_slice(&to_design.stdout).expect("transition output should be JSON");
    assert_eq!(to_design_payload["stage_labels_after_cas"][0], "design");

    let invalid = ctx
        .command()
        .arg("transition")
        .arg("--intake-key")
        .arg("intent-20260221-transition")
        .arg("--stage")
        .arg("accepted")
        .current_dir(&root)
        .output()
        .expect("run invalid transition");
    assert!(
        !invalid.status.success(),
        "design -> accepted transition must fail closed"
    );
    let stderr = String::from_utf8_lossy(&invalid.stderr);
    assert!(
        stderr.contains("invalid stage transition"),
        "invalid transition should explain ordering violation: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_replay_skips_completed_run_key() {
    let ctx = CliTestContext::new("worker_replay_skip", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-replay",
        "intent-20260221-replay",
        Some(1337),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    // Run 3 workers to reach the DesignReview approval gate.
    for _ in 0..3 {
        let worker = ctx
            .command()
            .arg("worker")
            .arg("--worker-id")
            .arg("test-worker-id")
            .current_dir(&root)
            .output()
            .expect("run worker");
        assert!(
            worker.status.success(),
            "worker should succeed: {}",
            String::from_utf8_lossy(&worker.stderr)
        );
    }

    // Explicitly approve by transitioning DesignReview -> Accepted.
    let approve = ctx
        .command()
        .arg("transition")
        .arg("--intake-key")
        .arg("intent-20260221-replay")
        .arg("--stage")
        .arg("accepted")
        .current_dir(&root)
        .output()
        .expect("approve transition to accepted");
    assert!(
        approve.status.success(),
        "approval transition should succeed: {}",
        String::from_utf8_lossy(&approve.stderr)
    );

    // Run 3 workers to progress Accepted -> Implementation -> Closed, then
    // execute terminal Closed no-op.
    for _ in 0..3 {
        let worker = ctx
            .command()
            .arg("worker")
            .arg("--worker-id")
            .arg("test-worker-id")
            .current_dir(&root)
            .output()
            .expect("run worker");
        assert!(
            worker.status.success(),
            "worker should succeed: {}",
            String::from_utf8_lossy(&worker.stderr)
        );
    }

    // 7th worker: the terminal stage run_key is already completed → replay skip
    let replay_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run replay worker");
    assert!(
        replay_worker.status.success(),
        "replay worker should succeed: {}",
        String::from_utf8_lossy(&replay_worker.stderr)
    );
    let replay_payload: serde_json::Value =
        serde_json::from_slice(&replay_worker.stdout).expect("replay worker output should be JSON");
    assert_eq!(
        replay_payload["replay_skipped"][0],
        "intent-20260221-replay"
    );
    assert_eq!(
        replay_payload["ready_to_run"]
            .as_array()
            .expect("ready_to_run should be an array")
            .len(),
        0
    );

    let run_state_path = root.join("target/sdlc/run-state-ledger.json");
    let run_state_raw = std::fs::read_to_string(&run_state_path).expect("read run state ledger");
    let run_state: serde_json::Value =
        serde_json::from_str(&run_state_raw).expect("parse run state ledger");
    assert_eq!(
        run_state["entries"]["intent-20260221-replay"]["status"],
        "Completed"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_heartbeats_existing_claim_lease_on_subsequent_pass() {
    let ctx = CliTestContext::new("worker_heartbeat_claim", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-heartbeat",
        "intent-20260221-heartbeat",
        Some(9090),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    // Run 3 workers to reach the DesignReview approval gate.
    for _ in 0..3 {
        let worker = ctx
            .command()
            .arg("worker")
            .arg("--worker-id")
            .arg("test-worker-id")
            .current_dir(&root)
            .output()
            .expect("run worker");
        assert!(
            worker.status.success(),
            "worker should succeed: {}",
            String::from_utf8_lossy(&worker.stderr)
        );
    }

    // Explicitly approve by transitioning DesignReview -> Accepted.
    let approve = ctx
        .command()
        .arg("transition")
        .arg("--intake-key")
        .arg("intent-20260221-heartbeat")
        .arg("--stage")
        .arg("accepted")
        .current_dir(&root)
        .output()
        .expect("approve transition to accepted");
    assert!(
        approve.status.success(),
        "approval transition should succeed: {}",
        String::from_utf8_lossy(&approve.stderr)
    );

    // Run 3 workers to progress Accepted -> Implementation -> Closed, then
    // execute terminal Closed no-op. After pass 6 overall, the terminal claim
    // slot (issue:9090:stage:closed) is held.
    for _ in 0..3 {
        let worker = ctx
            .command()
            .arg("worker")
            .arg("--worker-id")
            .arg("test-worker-id")
            .current_dir(&root)
            .output()
            .expect("run worker");
        assert!(
            worker.status.success(),
            "worker should succeed: {}",
            String::from_utf8_lossy(&worker.stderr)
        );
    }

    let claim_ledger_path = root.join("target/sdlc/claim-ledger.json");
    let first_claim_ledger: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&claim_ledger_path).expect("read claim ledger after terminal"),
    )
    .expect("parse claim ledger");
    let first_claims = first_claim_ledger["claims"]
        .as_object()
        .expect("claims should be object");
    let (slot_key, first_claim) = first_claims
        .iter()
        .find(|(k, _)| k.contains("stage:closed"))
        .expect("terminal worker should persist closed-stage claim entry");
    assert!(
        slot_key.contains("issue:9090"),
        "claim slot key should be scoped to requested issue id"
    );
    let first_expires_at = first_claim["lease_expires_at_epoch_ms"]
        .as_u64()
        .expect("lease expiry should be numeric");

    // 7th worker: replay-skips at terminal stage but heartbeats the claim
    let heartbeat_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run heartbeat worker");
    assert!(
        heartbeat_worker.status.success(),
        "heartbeat worker should succeed: {}",
        String::from_utf8_lossy(&heartbeat_worker.stderr)
    );
    let heartbeat_payload: serde_json::Value = serde_json::from_slice(&heartbeat_worker.stdout)
        .expect("heartbeat worker output should be JSON");
    assert_eq!(
        heartbeat_payload["replay_skipped"][0],
        "intent-20260221-heartbeat"
    );

    let second_claim_ledger: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&claim_ledger_path).expect("read claim ledger after heartbeat"),
    )
    .expect("parse claim ledger after heartbeat");
    let second_claim = &second_claim_ledger["claims"][slot_key];
    let second_expires_at = second_claim["lease_expires_at_epoch_ms"]
        .as_u64()
        .expect("heartbeat lease expiry should be numeric");
    assert!(
        second_expires_at >= first_expires_at,
        "heartbeat worker pass should extend/refresh claim lease"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_replay_skips_when_canonical_artifact_already_exists() {
    let ctx = CliTestContext::new("worker_canonical_replay_skip", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");

    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-canonical-replay",
        "intent-20260221-canonical-replay",
        Some(6601),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let artifact_ledger_path = root.join("target/sdlc/artifact-ledger.json");
    let artifact_raw =
        std::fs::read_to_string(&artifact_ledger_path).expect("read artifact ledger after intake");
    let mut artifact: serde_json::Value =
        serde_json::from_str(&artifact_raw).expect("parse artifact ledger");
    let provisional =
        artifact["records"]["sdlc:artifact:provisional:intent-20260221-canonical-replay"].clone();
    let mut canonical = provisional;
    canonical["marker"] = serde_json::Value::String(
        "sdlc:artifact:canonical:intent-20260221-canonical-replay".to_string(),
    );
    canonical["canonical"] = serde_json::Value::Bool(true);
    artifact["records"]["sdlc:artifact:canonical:intent-20260221-canonical-replay"] = canonical;
    std::fs::write(
        &artifact_ledger_path,
        serde_json::to_string_pretty(&artifact).expect("serialize artifact ledger"),
    )
    .expect("write artifact ledger with canonical marker");

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .arg("--dry-run")
        .current_dir(&root)
        .output()
        .expect("run worker dry-run");
    assert!(
        worker.status.success(),
        "worker should succeed: {}",
        String::from_utf8_lossy(&worker.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
    assert!(
        payload["replay_skipped"]
            .as_array()
            .expect("replay_skipped should be array")
            .iter()
            .any(|value| value == "intent-20260221-canonical-replay"),
        "worker should replay-skip intake with existing canonical artifact"
    );
    assert!(
        payload["replay_skipped_canonical"]
            .as_array()
            .expect("replay_skipped_canonical should be array")
            .iter()
            .any(|value| value == "intent-20260221-canonical-replay"),
        "worker should surface canonical-based replay skip attribution"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_metrics_include_stage_specific_llm_cost_units() {
    let ctx = CliTestContext::new("worker_llm_cost_units", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-llm-cost",
        "intent-20260221-llm-cost",
        Some(7711),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let transition = ctx
        .command()
        .arg("transition")
        .arg("--intake-key")
        .arg("intent-20260221-llm-cost")
        .arg("--stage")
        .arg("design")
        .current_dir(&root)
        .output()
        .expect("run transition to design");
    assert!(
        transition.status.success(),
        "transition to design should succeed: {}",
        String::from_utf8_lossy(&transition.stderr)
    );

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .arg("--dry-run")
        .current_dir(&root)
        .output()
        .expect("run worker dry-run");
    assert!(
        worker.status.success(),
        "worker dry-run should succeed: {}",
        String::from_utf8_lossy(&worker.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
    assert_eq!(
        payload["metrics"]["llm_cost_units"]["intent-20260221-llm-cost"], 8,
        "design stage should contribute expected llm cost unit estimate"
    );
    assert_eq!(
        payload["metrics"]["cost_units"]["llm_estimated_total_units"], 8,
        "cost unit summary should include total estimated llm units"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn transition_to_accepted_promotes_canonical_artifact() {
    let ctx = CliTestContext::new("transition_promote_canonical", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-canonical",
        "intent-20260221-canonical",
        Some(31337),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    for stage in ["design", "design-review"] {
        let transition = ctx
            .command()
            .arg("transition")
            .arg("--intake-key")
            .arg("intent-20260221-canonical")
            .arg("--stage")
            .arg(stage)
            .current_dir(&root)
            .output()
            .expect("run pre-accepted transition");
        assert!(
            transition.status.success(),
            "transition to {stage} should succeed: {}",
            String::from_utf8_lossy(&transition.stderr)
        );
    }

    let accepted = ctx
        .command()
        .arg("transition")
        .arg("--intake-key")
        .arg("intent-20260221-canonical")
        .arg("--stage")
        .arg("accepted")
        .current_dir(&root)
        .output()
        .expect("run transition to accepted");
    assert!(
        accepted.status.success(),
        "transition to accepted should succeed: {}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    let accepted_payload: serde_json::Value =
        serde_json::from_slice(&accepted.stdout).expect("accepted output should be JSON");
    assert_eq!(accepted_payload["canonical_artifact_status"], "inserted");

    let artifact_ledger_path = root.join("target/sdlc/artifact-ledger.json");
    let artifact_raw =
        std::fs::read_to_string(&artifact_ledger_path).expect("read artifact ledger");
    let artifact: serde_json::Value =
        serde_json::from_str(&artifact_raw).expect("parse artifact ledger");
    assert!(
        artifact["records"]
            .as_object()
            .expect("artifact records should be object")
            .contains_key("sdlc:artifact:canonical:intent-20260221-canonical"),
        "accepted transition should create canonical artifact marker"
    );
    let canonical_payload =
        &artifact["records"]["sdlc:artifact:canonical:intent-20260221-canonical"]["payload"];
    assert!(
        canonical_payload["Inline"]["body"]
            .as_str()
            .map(|body| !body.is_empty())
            .unwrap_or(false),
        "canonical artifact should persist normalized inline payload body"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_executes_idea_to_design_stage() {
    let ctx = CliTestContext::new("worker_executes_idea_to_design", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-idea-to-design",
        "intent-idea-to-design",
        Some(42),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run worker");
    assert!(
        worker.status.success(),
        "worker should succeed: {}",
        String::from_utf8_lossy(&worker.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
    assert!(
        payload["executed_runs"]
            .as_array()
            .expect("executed_runs should be array")
            .iter()
            .any(|value| value == "intent-idea-to-design"),
        "worker should execute the idea->design stage"
    );

    let issue_transport_path = root.join("target/sdlc/issue-transport-ledger.json");
    let issue_transport_raw =
        std::fs::read_to_string(&issue_transport_path).expect("read issue transport ledger");
    let issue_transport: serde_json::Value =
        serde_json::from_str(&issue_transport_raw).expect("parse issue transport ledger");
    let issue_record = &issue_transport["issues"]["42"];
    assert_eq!(
        issue_record["labels"][0], "design",
        "idea->design pass should update issue transport stage label"
    );
    assert_eq!(
        issue_record["comments_by_marker"]["design-marker"], "Generated design prompt",
        "idea->design pass should upsert design comment marker"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn worker_multi_pass_progresses_stage_to_closed() {
    let ctx = CliTestContext::new("worker_multi_pass_progression", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-multi-pass",
        "intent-multi-pass",
        Some(42),
    );

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(&root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    // Run 3 workers to reach the DesignReview approval gate.
    for _ in 0..3 {
        let worker = ctx
            .command()
            .arg("worker")
            .arg("--worker-id")
            .arg("test-worker-id")
            .current_dir(&root)
            .output()
            .expect("run worker");
        assert!(
            worker.status.success(),
            "worker should succeed: {}",
            String::from_utf8_lossy(&worker.stderr)
        );
    }

    // Approval is explicit: transition DesignReview -> Accepted.
    let approve = ctx
        .command()
        .arg("transition")
        .arg("--intake-key")
        .arg("intent-multi-pass")
        .arg("--stage")
        .arg("accepted")
        .current_dir(&root)
        .output()
        .expect("approve transition to accepted");
    assert!(
        approve.status.success(),
        "approval transition should succeed: {}",
        String::from_utf8_lossy(&approve.stderr)
    );

    // Run 2 workers to progress Accepted -> Implementation -> Closed.
    for _ in 0..2 {
        let worker = ctx
            .command()
            .arg("worker")
            .arg("--worker-id")
            .arg("test-worker-id")
            .current_dir(&root)
            .output()
            .expect("run worker");
        assert!(
            worker.status.success(),
            "worker should succeed: {}",
            String::from_utf8_lossy(&worker.stderr)
        );
    }

    let ledger_path = root.join("target/sdlc/intake-ledger.json");
    let ledger_raw = std::fs::read_to_string(&ledger_path).expect("read intake ledger");
    let ledger: serde_json::Value = serde_json::from_str(&ledger_raw).expect("parse intake ledger");
    assert_eq!(
        ledger["entries"]["intent-multi-pass"]["stage"], "Closed",
        "worker multi-pass progression should reach terminal closed stage"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}
