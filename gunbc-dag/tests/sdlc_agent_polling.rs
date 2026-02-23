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
