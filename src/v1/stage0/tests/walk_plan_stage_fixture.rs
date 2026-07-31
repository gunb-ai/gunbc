//! Production-path stage-executor controls driven through real WalkPlan fixtures
//! (`src/v2/test/fixture/walk_plan_stage/plan.dag`). These are the discriminating
//! witnesses #7499's `run_stage` extraction needs: unit tests over `spawn_units`
//! closures cannot catch regressions where grouping, lane selection, admission, or
//! the stage loop itself is wrong.

use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

fn claim_executor_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_claim_executor")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            workspace_root().join(
                std::env::var("PROFILE")
                    .map(|p| format!("target/{p}/claim_executor"))
                    .unwrap_or_else(|_| "target/debug/claim_executor".to_string()),
            )
        })
}

fn run_fixture_plan(plan_function: &str) -> (i32, String) {
    let root = workspace_root();
    let bin = claim_executor_bin();
    assert!(
        bin.exists(),
        "claim_executor binary missing at {}",
        bin.display()
    );
    let output = Command::new(&bin)
        .current_dir(&root)
        .args([
            "--source-root",
            "dag",
            "--source-root",
            "src/v2",
            "--plan-entry",
            "src/v2/test/fixture/walk_plan_stage/plan.dag",
            "--plan-function",
            plan_function,
        ])
        .output()
        .expect("spawn claim_executor");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1);
    (code, stderr)
}

fn stage2_marker_path(root: &PathBuf) -> PathBuf {
    root.join("target/walk_plan_stage_stage2_ran")
}

fn stage1_receipt_path(root: &PathBuf) -> PathBuf {
    root.join("target/floor-on-success-stage-1-receipt.tsv")
}

#[test]
#[ignore = "cold overlap resolve can take minutes; run locally: cargo test -p v1-compiler walk_plan_stage_overlap_barrier -- --ignored"]
fn walk_plan_stage_overlap_barrier_production_path() {
    let root = workspace_root();
    let _ = std::fs::remove_file(stage2_marker_path(&root));
    let (code, stderr) = run_fixture_plan("walk_plan_stage_overlap_barrier_plan");
    assert!(
        stderr.contains("on-success stage 1"),
        "fixture must reach on-success stages, not fail-fast on budget; stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("PASS [on-success stage 1] walk_plan_stage_overlap_peer_a_holds"),
        "overlap peer A must pass; stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("PASS [on-success stage 1] walk_plan_stage_overlap_peer_b_holds"),
        "overlap peer B must pass; stderr:\n{stderr}"
    );
    assert!(
        stage1_receipt_path(&root).is_file(),
        "stage-1 per-stage receipt must exist before stage 2"
    );
    assert!(
        stderr.contains("PASS [on-success stage 2] walk_plan_stage_barrier_witness_holds"),
        "barrier witness must read stage-1 receipt; stderr:\n{stderr}"
    );
    assert_eq!(
        code, 0,
        "overlap+barrier plan must exit zero; stderr:\n{stderr}"
    );
}

#[test]
fn walk_plan_stage_failure_blocks_stage2() {
    let root = workspace_root();
    let marker = stage2_marker_path(&root);
    let _ = std::fs::remove_file(&marker);
    let (code, stderr) = run_fixture_plan("walk_plan_stage_failure_barrier_plan");
    assert!(
        stderr.contains("remaining stage(s) NOT run"),
        "failed stage 1 must block stage 2; stderr:\n{stderr}"
    );
    assert!(
        !marker.exists(),
        "stage-2 marker must not exist when stage-1 claim is red"
    );
    assert_ne!(code, 0, "failure plan must exit nonzero; stderr:\n{stderr}");
}

#[test]
fn walk_plan_stage_panic_blocks_stage2() {
    let root = workspace_root();
    let marker = stage2_marker_path(&root);
    let _ = std::fs::remove_file(&marker);
    let (code, stderr) = run_fixture_plan("walk_plan_stage_panic_barrier_plan");
    assert!(
        stderr.contains("remaining stage(s) NOT run") || stderr.contains("infra=thread_panic"),
        "panicking stage-1 member must block stage 2; stderr:\n{stderr}"
    );
    assert!(
        !marker.exists(),
        "stage-2 marker must not exist when stage-1 thread panicked"
    );
    assert_ne!(code, 0, "panic plan must exit nonzero; stderr:\n{stderr}");
}

#[test]
fn walk_plan_stage_receipt_refusal_blocks_stage2() {
    let root = workspace_root();
    let marker = stage2_marker_path(&root);
    let _ = std::fs::remove_file(&marker);
    let (code, stderr) = run_fixture_plan("walk_plan_stage_receipt_refusal_barrier_plan");
    assert!(
        stderr.contains("receipt write failed") || stderr.contains("remaining stage(s) NOT run"),
        "stage-1 receipt refusal must block stage 2; stderr:\n{stderr}"
    );
    assert!(
        !marker.exists(),
        "stage-2 marker must not exist when stage-1 receipt write refused"
    );
    assert_ne!(
        code, 0,
        "receipt-refusal plan must exit nonzero; stderr:\n{stderr}"
    );
}
