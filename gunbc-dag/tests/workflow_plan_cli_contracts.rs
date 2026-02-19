//! Workflow planner CLI explainability contracts (WF5).

use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn run_workflow_plan(workflow: &str, workspace_root: &PathBuf) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_gunbc-workflow"))
        .arg("--plan")
        .arg(workflow)
        .arg("--workspace-root")
        .arg(workspace_root)
        .output()
        .expect("gunbc-workflow should run");
    assert!(
        output.status.success(),
        "gunbc-workflow failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout should be utf-8")
}

#[test]
fn plan_output_is_deterministic_for_ci_fixture_state() {
    let root = temp_root();
    std::fs::create_dir_all(&root).expect("temp workspace root");

    let first = run_workflow_plan("ci", &root);
    let second = run_workflow_plan("ci", &root);
    assert_eq!(first, second, "ci plan output should be deterministic");
    assert!(first.contains("workflow: ci"));
    assert!(first.contains("execute-set:"));
    assert!(first.contains("cache-hit-set:"));
    assert!(first.contains("critical-path:"));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn plan_output_is_deterministic_for_test_all_fixture_state() {
    let root = temp_root();
    std::fs::create_dir_all(&root).expect("temp workspace root");

    let first = run_workflow_plan("test-all", &root);
    let second = run_workflow_plan("test-all", &root);
    assert_eq!(
        first, second,
        "test-all plan output should be deterministic"
    );
    assert!(first.contains("workflow: test-all"));
    assert!(first.contains("execute-set:"));
    assert!(first.contains("critical-path:"));
    assert!(
        first.contains("miss:no-prior-run"),
        "fresh fixture should classify misses as no-prior-run"
    );

    let _ = std::fs::remove_dir_all(root);
}
