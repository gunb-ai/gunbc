//! Workflow planner CLI explainability contracts (WF5).
#![allow(clippy::disallowed_methods)]

use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "gunbc-workflow-cli-contracts-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

fn run_workflow_plan_json(workflow: &str, workspace_root: &PathBuf) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_gunbc-workflow"))
        .arg("--plan")
        .arg(workflow)
        .arg("--format")
        .arg("json")
        .arg("--workspace-root")
        .arg(workspace_root)
        .output()
        .expect("gunbc-workflow should run");
    assert!(
        output.status.success(),
        "gunbc-workflow failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("workflow plan output should be valid json")
}

#[test]
fn plan_output_is_deterministic_for_ci_fixture_state() {
    let root = temp_root();
    std::fs::create_dir_all(&root).expect("temp workspace root");

    let first = run_workflow_plan_json("ci", &root);
    let second = run_workflow_plan_json("ci", &root);
    assert_eq!(first, second, "ci plan output should be deterministic");
    assert_eq!(first["workflow"], Value::String("ci".to_string()));
    assert!(first["execute_set"].is_array());
    assert!(first["cache_hit_set"].is_array());
    assert!(first["critical_path"].is_array());
    assert!(first["blocked"].is_object());
    assert!(first["ready"].is_array());

    for entry in first["execute_set"]
        .as_array()
        .expect("execute_set should be an array")
    {
        assert!(entry["node_id"].is_string());
        assert!(entry["miss_reason"].is_string());
    }
}

#[test]
fn plan_output_is_deterministic_for_test_all_fixture_state() {
    let root = temp_root();
    std::fs::create_dir_all(&root).expect("temp workspace root");

    let first = run_workflow_plan_json("test-all", &root);
    let second = run_workflow_plan_json("test-all", &root);
    assert_eq!(
        first, second,
        "test-all plan output should be deterministic"
    );
    assert_eq!(first["workflow"], Value::String("test-all".to_string()));
    assert!(first["execute_set"].is_array());
    assert!(first["critical_path"].is_array());
    assert!(
        first["execute_set"]
            .as_array()
            .expect("execute_set should be an array")
            .iter()
            .all(|entry| {
                entry["miss_reason"]
                    .as_str()
                    .is_some_and(|reason| reason.starts_with("miss:no-prior-run"))
            }),
        "fresh fixture should classify execute set misses as no-prior-run"
    );
}

#[test]
fn plan_output_is_deterministic_for_sdlc_fixture_state() {
    let root = temp_root();
    std::fs::create_dir_all(&root).expect("temp workspace root");

    let first = run_workflow_plan_json("sdlc", &root);
    let second = run_workflow_plan_json("sdlc", &root);
    assert_eq!(first, second, "sdlc plan output should be deterministic");
    assert_eq!(first["workflow"], Value::String("sdlc".to_string()));
    assert!(first["execute_set"].is_array());
    assert!(first["critical_path"].is_array());
    assert!(
        first["execute_set"]
            .as_array()
            .expect("execute_set should be an array")
            .iter()
            .any(|entry| entry["node_id"] == Value::String("sdlc.intake".to_string())),
        "sdlc plan execute_set should include sdlc.intake on cold fixtures"
    );
}
