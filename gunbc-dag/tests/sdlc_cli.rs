use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn sdlc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_gunbc-sdlc")
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "gunbc_sdlc_cli_{label}_{}_{}",
        std::process::id(),
        nanos
    ))
}

fn write_intent_file(path: &Path, intent_id: &str, intake_key: &str) {
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
  issue_id: null
  run_key: null
acceptance_tests:
  - "cargo test -q -p gunbc-dag --test sdlc_cli"
notes: "test fixture"
"#
    );
    std::fs::write(path, content).expect("write intent fixture");
}

#[test]
fn intake_dry_run_computes_run_key_without_writing_ledger() {
    let root = unique_temp_dir("dry_run");
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(&intent_path, "intent-20260221-dry-run", "intent-20260221-dry-run");

    let output = Command::new(sdlc_bin())
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
    assert!(
        payload["run_key"]
            .as_str()
            .expect("run_key should be a string")
            .contains("intent-20260221-dry-run")
    );
    assert!(
        !root.join("target/sdlc/intake-ledger.json").exists(),
        "dry-run intake must not write ledger state"
    );

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn intake_real_mode_is_idempotent_for_same_intake_key() {
    let root = unique_temp_dir("idempotent");
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(
        &intent_path,
        "intent-20260221-idempotent",
        "intent-20260221-idempotent",
    );

    let first = Command::new(sdlc_bin())
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

    let second = Command::new(sdlc_bin())
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

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}

#[test]
fn intake_conflict_fails_closed_for_reused_intake_key() {
    let root = unique_temp_dir("conflict");
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_a = root.join("intent_a.yaml");
    let intent_b = root.join("intent_b.yaml");
    write_intent_file(&intent_a, "intent-20260221-a", "intent-20260221-a");
    write_intent_file(&intent_b, "intent-20260221-b", "intent-20260221-a");

    let first = Command::new(sdlc_bin())
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

    let conflict = Command::new(sdlc_bin())
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
    let root = unique_temp_dir("worker");
    std::fs::create_dir_all(&root).expect("create temp root");
    let intent_path = root.join("intent.yaml");
    write_intent_file(&intent_path, "intent-20260221-worker", "intent-20260221-worker");

    let intake = Command::new(sdlc_bin())
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

    let worker = Command::new(sdlc_bin())
        .arg("worker")
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

    std::fs::remove_dir_all(root).expect("cleanup temp root");
}
