#![allow(clippy::disallowed_methods)]
use std::path::Path;

mod common;
use common::fixture::CliTestContext;
use std::process::Command;

fn sdlc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_gunbc-sdlc")
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

fn write_infra_intent_template(root: &Path) {
    let todo_dir = root.join("TODO");
    std::fs::create_dir_all(&todo_dir).expect("create TODO directory for infra intent");
    let content = r#"(
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
        lease_ttl_seconds: 120,
        heartbeat_seconds: 30,
        poll_interval_seconds: 15,
    ),
    drift: (
        reconcile_mode: "plan-then-apply",
        reconcile_interval_minutes: 60,
    ),
    notes: Some("test fixture infra intent"),
)"#;
    std::fs::write(todo_dir.join("infra-intent-template.dag"), content)
        .expect("write infra intent template fixture");
}

fn write_intent_file(path: &Path, intent_id: &str, intake_key: &str, issue_id: Option<u64>) {
    let root = path
        .parent()
        .expect("intent fixture path should have parent");
    ensure_git_repo_with_commit(root);
    write_infra_intent_template(root);
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

fn progress_intake_to_accepted(
    ctx: &CliTestContext,
    root: &Path,
    intent_id: &str,
    intake_key: &str,
    issue_id: u64,
) {
    let intent_path = root.join("intent.yaml");
    write_intent_file(&intent_path, intent_id, intake_key, Some(issue_id));

    let intake = ctx
        .command()
        .arg("intake")
        .arg("--intent")
        .arg(&intent_path)
        .current_dir(root)
        .output()
        .expect("run intake");
    assert!(
        intake.status.success(),
        "intake should succeed: {}",
        String::from_utf8_lossy(&intake.stderr)
    );

    for _ in 0..3 {
        let worker = ctx
            .command()
            .arg("worker")
            .arg("--worker-id")
            .arg("test-worker-id")
            .current_dir(root)
            .output()
            .expect("run worker");
        assert!(
            worker.status.success(),
            "worker should succeed: {}",
            String::from_utf8_lossy(&worker.stderr)
        );
    }

    let approve = ctx
        .command()
        .arg("transition")
        .arg("--intake-key")
        .arg(intake_key)
        .arg("--stage")
        .arg("accepted")
        .current_dir(root)
        .output()
        .expect("transition to accepted");
    assert!(
        approve.status.success(),
        "approval transition should succeed: {}",
        String::from_utf8_lossy(&approve.stderr)
    );
}

#[test]
fn worker_polls_running_agent_records_during_sweep() {
    let ctx = CliTestContext::new("worker_agent_polling", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-agent-poll",
        "intent-agent-poll",
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

    let intake_ledger_path = root.join("target/sdlc/intake-ledger.json");
    let intake_ledger_raw =
        std::fs::read_to_string(&intake_ledger_path).expect("read intake ledger");
    let intake_ledger: serde_json::Value =
        serde_json::from_str(&intake_ledger_raw).expect("parse intake ledger");
    let run_key = intake_ledger["entries"]["intent-agent-poll"]["run_key"]
        .as_str()
        .expect("run_key should be populated");

    let agent_ledger_path = root.join("target/sdlc/agent-ledger.json");
    if let Some(parent) = agent_ledger_path.parent() {
        std::fs::create_dir_all(parent).expect("create agent ledger parent");
    }
    let agent_ledger = serde_json::json!({
        "entries": {
            "intent-agent-poll": {
                "intake_key": "intent-agent-poll",
                "handle": {
                    "provider": "stub",
                    "session_id": "stub-intent-agent-poll",
                    "intake_key": "intent-agent-poll",
                    "spawned_at_epoch_ms": 0,
                },
                "status": { "Running": { "progress": serde_json::Value::Null } },
                "pr_number": serde_json::Value::Null,
                "pr_url": serde_json::Value::Null,
                "updated_at_epoch_ms": 0,
            }
        }
    });
    std::fs::write(
        &agent_ledger_path,
        serde_json::to_vec_pretty(&agent_ledger).expect("serialize agent ledger"),
    )
    .expect("write agent ledger fixture");

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
    assert_eq!(
        payload["agent_polls"][0]["intake_key"],
        serde_json::Value::String("intent-agent-poll".to_string())
    );
    assert_eq!(
        payload["agent_polls"][0]["status"]["Completed"]["branch"],
        serde_json::Value::String("sdlc/polled".to_string())
    );

    let agent_ledger_after_raw =
        std::fs::read_to_string(&agent_ledger_path).expect("read agent ledger after worker");
    let agent_ledger_after: serde_json::Value =
        serde_json::from_str(&agent_ledger_after_raw).expect("parse agent ledger after worker");
    assert_eq!(
        agent_ledger_after["entries"]["intent-agent-poll"]["status"]["Completed"]["branch"],
        serde_json::Value::String("sdlc/polled".to_string())
    );
    assert_eq!(
        intake_ledger["entries"]["intent-agent-poll"]["run_key"],
        serde_json::Value::String(run_key.to_string())
    );
}

#[test]
fn worker_spawns_agent_record_when_advancing_from_accepted() {
    let ctx = CliTestContext::new("worker_agent_spawn", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    progress_intake_to_accepted(&ctx, &root, "intent-agent-spawn", "intent-agent-spawn", 84);

    let worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run worker accepted stage");
    assert!(
        worker.status.success(),
        "worker should succeed: {}",
        String::from_utf8_lossy(&worker.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&worker.stdout).expect("worker output should be JSON");
    assert_eq!(
        payload["agent_spawns"][0]["intake_key"],
        serde_json::Value::String("intent-agent-spawn".to_string())
    );
    assert_eq!(
        payload["agent_spawns"][0]["target_branch"],
        serde_json::Value::String("feature/intent-agent-spawn".to_string())
    );
    assert_eq!(
        payload["agent_spawns"][0]["status"]["Completed"]["branch"],
        serde_json::Value::String("feature/intent-agent-spawn".to_string())
    );

    let agent_ledger_path = root.join("target/sdlc/agent-ledger.json");
    let agent_ledger_raw = std::fs::read_to_string(&agent_ledger_path).expect("read agent ledger");
    let agent_ledger: serde_json::Value =
        serde_json::from_str(&agent_ledger_raw).expect("parse agent ledger");
    assert_eq!(
        agent_ledger["entries"]["intent-agent-spawn"]["status"]["Completed"]["branch"],
        serde_json::Value::String("feature/intent-agent-spawn".to_string())
    );
}

#[test]
fn agent_spawn_command_records_completed_handle_for_accepted_intake() {
    let ctx = CliTestContext::new("agent_spawn_command", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    progress_intake_to_accepted(
        &ctx,
        &root,
        "intent-agent-command",
        "intent-agent-command",
        333,
    );

    let spawn = ctx
        .command()
        .arg("agent-spawn")
        .arg("--intake-key")
        .arg("intent-agent-command")
        .current_dir(&root)
        .output()
        .expect("run agent-spawn command");
    assert!(
        spawn.status.success(),
        "agent-spawn should succeed: {}",
        String::from_utf8_lossy(&spawn.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&spawn.stdout).expect("agent-spawn output should be JSON");
    assert_eq!(
        payload["target_branch"],
        serde_json::Value::String("feature/intent-agent-command".to_string())
    );
    assert_eq!(
        payload["status"]["Completed"]["branch"],
        serde_json::Value::String("feature/intent-agent-command".to_string())
    );

    let agent_ledger_path = root.join("target/sdlc/agent-ledger.json");
    let ledger: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(agent_ledger_path).expect("read agent ledger"),
    )
    .expect("parse agent ledger");
    assert_eq!(
        ledger["entries"]["intent-agent-command"]["status"]["Completed"]["branch"],
        serde_json::Value::String("feature/intent-agent-command".to_string())
    );
}

#[test]
fn validate_pr_dry_run_requires_agent_record() {
    let ctx = CliTestContext::new("validate_pr_requires_agent", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    progress_intake_to_accepted(
        &ctx,
        &root,
        "intent-validate-missing-agent",
        "intent-validate-missing-agent",
        404,
    );

    let validate = ctx
        .command()
        .arg("validate-pr")
        .arg("--intake-key")
        .arg("intent-validate-missing-agent")
        .arg("--dry-run")
        .current_dir(&root)
        .output()
        .expect("run validate-pr dry-run");
    assert!(
        !validate.status.success(),
        "validate-pr should fail without an agent record"
    );
    let stderr = String::from_utf8_lossy(&validate.stderr);
    assert!(
        stderr.contains("no agent record for intake key"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn validate_pr_dry_run_reports_ci_and_review_summary_for_completed_agent() {
    let ctx = CliTestContext::new("validate_pr_dry_run", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    progress_intake_to_accepted(
        &ctx,
        &root,
        "intent-validate-success",
        "intent-validate-success",
        505,
    );

    let spawn = ctx
        .command()
        .arg("agent-spawn")
        .arg("--intake-key")
        .arg("intent-validate-success")
        .current_dir(&root)
        .output()
        .expect("run agent-spawn command");
    assert!(
        spawn.status.success(),
        "agent-spawn should succeed: {}",
        String::from_utf8_lossy(&spawn.stderr)
    );

    let validate = ctx
        .command()
        .arg("validate-pr")
        .arg("--intake-key")
        .arg("intent-validate-success")
        .arg("--dry-run")
        .current_dir(&root)
        .output()
        .expect("run validate-pr dry-run");
    assert!(
        validate.status.success(),
        "validate-pr dry-run should succeed: {}",
        String::from_utf8_lossy(&validate.stderr)
    );
    let stdout = String::from_utf8_lossy(&validate.stdout);
    let mut stream = serde_json::Deserializer::from_str(&stdout).into_iter::<serde_json::Value>();
    let mut payload = None;
    for entry in &mut stream {
        payload = Some(entry.expect("validate-pr output stream should contain valid JSON values"));
    }
    let payload = payload.expect("validate-pr output stream should include final summary JSON");
    assert_eq!(payload["review_passed"], serde_json::Value::Bool(true));
    assert_eq!(payload["ci_passed"], serde_json::Value::Bool(true));
    assert_eq!(payload["all_passed"], serde_json::Value::Bool(true));
    assert_eq!(
        payload["stage_after"],
        serde_json::Value::String("closed".to_string())
    );
}

#[test]
fn validate_pr_real_mode_skips_ci_when_review_findings_block() {
    let ctx = CliTestContext::new("validate_pr_review_blocks_ci", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    progress_intake_to_accepted(
        &ctx,
        &root,
        "intent-validate-review-block",
        "intent-validate-review-block",
        515,
    );

    let spawn = ctx
        .command()
        .arg("agent-spawn")
        .arg("--intake-key")
        .arg("intent-validate-review-block")
        .current_dir(&root)
        .output()
        .expect("run agent-spawn command");
    assert!(
        spawn.status.success(),
        "agent-spawn should succeed: {}",
        String::from_utf8_lossy(&spawn.stderr)
    );

    let agent_ledger_path = root.join("target/sdlc/agent-ledger.json");
    let mut agent_ledger: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&agent_ledger_path).expect("read agent ledger"),
    )
    .expect("parse agent ledger");
    agent_ledger["entries"]["intent-validate-review-block"]["pr_number"] =
        serde_json::Value::Number(serde_json::Number::from(515));
    agent_ledger["entries"]["intent-validate-review-block"]["pr_url"] =
        serde_json::Value::String("https://example.test/pr/515".to_string());
    std::fs::write(
        &agent_ledger_path,
        serde_json::to_vec_pretty(&agent_ledger).expect("serialize updated agent ledger"),
    )
    .expect("write updated agent ledger");

    let validate = ctx
        .command()
        .arg("validate-pr")
        .arg("--intake-key")
        .arg("intent-validate-review-block")
        .current_dir(&root)
        .output()
        .expect("run validate-pr in real mode");
    assert!(
        validate.status.success(),
        "validate-pr should succeed with blocked review summary output: {}",
        String::from_utf8_lossy(&validate.stderr)
    );
    let stdout = String::from_utf8_lossy(&validate.stdout);
    let mut stream = serde_json::Deserializer::from_str(&stdout).into_iter::<serde_json::Value>();
    let mut payload = None;
    for entry in &mut stream {
        payload = Some(entry.expect("validate-pr output stream should contain valid JSON values"));
    }
    let payload = payload.expect("validate-pr output stream should include final summary JSON");
    assert_eq!(payload["review_passed"], serde_json::Value::Bool(false));
    assert_eq!(payload["ci_passed"], serde_json::Value::Bool(false));
    assert_eq!(payload["all_passed"], serde_json::Value::Bool(false));
    assert_eq!(
        payload["ci_summary"],
        serde_json::Value::String("skipped: blocking code review findings".to_string())
    );
}

#[test]
fn implementation_stage_fails_closed_without_agent_record() {
    let ctx = CliTestContext::new("implementation_requires_agent", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    progress_intake_to_accepted(
        &ctx,
        &root,
        "intent-impl-no-agent",
        "intent-impl-no-agent",
        606,
    );

    // Advance Accepted -> Implementation once, which normally creates agent ledger entry.
    let first_impl_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run worker accepted stage");
    assert!(
        first_impl_worker.status.success(),
        "worker should succeed: {}",
        String::from_utf8_lossy(&first_impl_worker.stderr)
    );

    let agent_ledger_path = root.join("target/sdlc/agent-ledger.json");
    std::fs::write(
        &agent_ledger_path,
        serde_json::to_vec_pretty(&serde_json::json!({"entries": {}}))
            .expect("serialize empty agent ledger"),
    )
    .expect("clear agent ledger entries");

    // Next pass attempts Implementation -> Closed and must fail closed without agent record.
    let second_impl_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run worker implementation stage");
    assert!(
        second_impl_worker.status.success(),
        "worker should return report even when implementation dispatch fails: {}",
        String::from_utf8_lossy(&second_impl_worker.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&second_impl_worker.stdout).expect("worker output should be JSON");
    assert_eq!(
        payload["executed_runs"]
            .as_array()
            .expect("executed_runs should be an array")
            .len(),
        0,
        "implementation dispatch must not execute without required agent record"
    );

    let intake_ledger_path = root.join("target/sdlc/intake-ledger.json");
    let intake_ledger: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&intake_ledger_path).expect("read intake ledger"),
    )
    .expect("parse intake ledger");
    assert_eq!(
        intake_ledger["entries"]["intent-impl-no-agent"]["stage"],
        serde_json::Value::String("Implementation".to_string())
    );
    assert_eq!(
        intake_ledger["entries"]["intent-impl-no-agent"]["retry"]["attempts"],
        serde_json::Value::Number(serde_json::Number::from(1))
    );
}

#[test]
fn implementation_to_closed_emits_validation_summary() {
    let ctx = CliTestContext::new("implementation_validation_summary", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    progress_intake_to_accepted(
        &ctx,
        &root,
        "intent-impl-validation",
        "intent-impl-validation",
        707,
    );

    let accepted_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run accepted stage worker");
    assert!(
        accepted_worker.status.success(),
        "accepted worker should succeed: {}",
        String::from_utf8_lossy(&accepted_worker.stderr)
    );

    let implementation_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run implementation stage worker");
    assert!(
        implementation_worker.status.success(),
        "implementation worker should succeed: {}",
        String::from_utf8_lossy(&implementation_worker.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&implementation_worker.stdout)
        .expect("implementation worker output should be JSON");
    assert_eq!(
        payload["implementation_validations"][0]["intake_key"],
        serde_json::Value::String("intent-impl-validation".to_string())
    );
    assert_eq!(
        payload["implementation_validations"][0]["validation_mode"],
        serde_json::Value::String("dry-run".to_string())
    );
    assert_eq!(
        payload["implementation_validations"][0]["review_passed"],
        serde_json::Value::Bool(true)
    );
    assert_eq!(
        payload["implementation_validations"][0]["ci_passed"],
        serde_json::Value::Bool(true)
    );
}

#[test]
fn implementation_stage_fails_closed_on_blocking_code_review_findings() {
    let ctx = CliTestContext::new("implementation_review_gate", sdlc_bin());
    let root = ctx.path().to_path_buf();
    std::fs::create_dir_all(&root).expect("create temp root");
    progress_intake_to_accepted(
        &ctx,
        &root,
        "intent-impl-review-gate",
        "intent-impl-review-gate",
        808,
    );

    let accepted_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run accepted stage worker");
    assert!(
        accepted_worker.status.success(),
        "accepted worker should succeed: {}",
        String::from_utf8_lossy(&accepted_worker.stderr)
    );

    let agent_ledger_path = root.join("target/sdlc/agent-ledger.json");
    let mut agent_ledger: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&agent_ledger_path).expect("read agent ledger"),
    )
    .expect("parse agent ledger");
    agent_ledger["entries"]["intent-impl-review-gate"]["handle"]["provider"] =
        serde_json::Value::String("codex".to_string());
    agent_ledger["entries"]["intent-impl-review-gate"]["status"]["Completed"]["branch"] =
        serde_json::Value::String("branch-not-present-in-fixture".to_string());
    std::fs::write(
        &agent_ledger_path,
        serde_json::to_vec_pretty(&agent_ledger).expect("serialize updated agent ledger"),
    )
    .expect("write updated agent ledger");

    let implementation_worker = ctx
        .command()
        .arg("worker")
        .arg("--worker-id")
        .arg("test-worker-id")
        .current_dir(&root)
        .output()
        .expect("run implementation stage worker");
    assert!(
        !implementation_worker.status.success(),
        "implementation worker should fail closed on blocking review findings"
    );
    let dispatch_error = String::from_utf8_lossy(&implementation_worker.stderr);
    assert!(
        dispatch_error.contains("blocking code review findings"),
        "expected review-gate failure details in dispatch error: {dispatch_error}"
    );

    let intake_ledger_path = root.join("target/sdlc/intake-ledger.json");
    let intake_ledger: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&intake_ledger_path).expect("read intake ledger"),
    )
    .expect("parse intake ledger");
    assert_eq!(
        intake_ledger["entries"]["intent-impl-review-gate"]["stage"],
        serde_json::Value::String("Implementation".to_string())
    );
}
