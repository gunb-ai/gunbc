//! Production-path stage-executor controls driven through real WalkPlan fixtures
//! (`src/v2/test/fixture/walk_plan_stage/plan.dag`). These are the discriminating
//! witnesses #7499's `run_stage` extraction needs: unit tests over `spawn_units`
//! closures cannot catch regressions where grouping, lane selection, admission, or
//! the stage loop itself is wrong.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

/// Production-path walks share `target/` hygiene paths; serialize harness runs so
/// parallel `cargo test` threads cannot interleave cleanup with overlap markers.
static FIXTURE_RUN_LOCK: Mutex<()> = Mutex::new(());

/// Must match `walk_plan_stage_attempt_id` in `common.dag` — the executor refuses
/// staged walks without `GUNBC_WALK_ATTEMPT_ID` or the GitHub triple.
const ATTEMPT_ID: &str = "walk-plan-stage-fixture";

/// Must match `walk_plan_stage_stage1_receipt_path` in `common.dag`.
const STAGE1_RECEIPT_REL: &str =
    "target/floor-attempt-walk-plan-stage-fixture/on-success-stage-1-receipt.tsv";

struct FixtureOutput {
    code: i32,
    stdout: String,
    stderr: String,
}

impl FixtureOutput {
    fn combined(&self) -> String {
        format!(
            "--- stdout ---\n{}\n--- stderr ---\n{}",
            self.stdout, self.stderr
        )
    }
}

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

fn run_fixture_plan(plan_function: &str) -> FixtureOutput {
    let _guard = FIXTURE_RUN_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = workspace_root();
    let bin = claim_executor_bin();
    assert!(
        bin.exists(),
        "claim_executor binary missing at {}",
        bin.display()
    );
    let output = Command::new(&bin)
        .current_dir(&root)
        .env("GUNBC_WALK_ATTEMPT_ID", ATTEMPT_ID)
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
    FixtureOutput {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn stage2_marker_path(root: &PathBuf) -> PathBuf {
    root.join("target/walk_plan_stage_stage2_ran")
}

fn stage1_receipt_path(root: &PathBuf) -> PathBuf {
    root.join(STAGE1_RECEIPT_REL)
}

#[test]
#[ignore = "cold overlap resolve can take minutes; run locally: cargo test -p v1-compiler walk_plan_stage_overlap_barrier -- --ignored"]
fn walk_plan_stage_overlap_barrier_production_path() {
    let root = workspace_root();
    let _ = std::fs::remove_file(stage2_marker_path(&root));
    let out = run_fixture_plan("walk_plan_stage_overlap_barrier_plan");
    assert!(
        out.stderr.contains("on-success stage 1"),
        "fixture must reach on-success stages, not fail-fast on budget;\n{}",
        out.combined()
    );
    assert!(
        out.stdout
            .contains("PASS [on-success stage 1] walk_plan_stage_overlap_peer_a_holds"),
        "overlap peer A must pass;\n{}",
        out.combined()
    );
    assert!(
        out.stdout
            .contains("PASS [on-success stage 1] walk_plan_stage_overlap_peer_b_holds"),
        "overlap peer B must pass;\n{}",
        out.combined()
    );
    assert!(
        stage1_receipt_path(&root).is_file(),
        "stage-1 per-stage receipt must exist before stage 2"
    );
    assert!(
        out.stdout
            .contains("PASS [on-success stage 2] walk_plan_stage_barrier_witness_holds"),
        "barrier witness must read stage-1 receipt;\n{}",
        out.combined()
    );
    assert_eq!(
        out.code,
        0,
        "overlap+barrier plan must exit zero;\n{}",
        out.combined()
    );
}

#[test]
#[ignore = "production-path fixture: cold claim_executor walk (~7m); recipe in plan.dag"]
fn walk_plan_stage_failure_blocks_stage2() {
    let root = workspace_root();
    let marker = stage2_marker_path(&root);
    let _ = std::fs::remove_file(&marker);
    let out = run_fixture_plan("walk_plan_stage_failure_barrier_plan");
    assert!(
        out.stderr.contains("remaining stage(s) NOT run"),
        "failed stage 1 must block stage 2;\n{}",
        out.combined()
    );
    assert!(
        !marker.exists(),
        "stage-2 marker must not exist when stage-1 claim is red"
    );
    assert_ne!(
        out.code,
        0,
        "failure plan must exit nonzero;\n{}",
        out.combined()
    );
}

#[test]
#[ignore = "production-path fixture: cold claim_executor walk (~7m); recipe in plan.dag"]
fn walk_plan_stage_panic_blocks_stage2() {
    let root = workspace_root();
    let marker = stage2_marker_path(&root);
    let _ = std::fs::remove_file(&marker);
    let out = run_fixture_plan("walk_plan_stage_panic_barrier_plan");
    assert!(
        out.stdout.contains("<claim thread panicked>"),
        "panicking member must surface structural thread-panic evidence on stdout;\n{}",
        out.combined()
    );
    assert!(
        out.stderr.contains("infra=thread_panic") || out.stdout.contains("infra=thread_panic"),
        "panicking member must classify as infra=thread_panic;\n{}",
        out.combined()
    );
    assert!(
        out.stderr.contains("remaining stage(s) NOT run"),
        "panicking stage 1 must block stage 2;\n{}",
        out.combined()
    );
    assert!(
        !marker.exists(),
        "stage-2 marker must not exist when stage-1 thread panicked"
    );
    assert_ne!(
        out.code,
        0,
        "panic plan must exit nonzero;\n{}",
        out.combined()
    );
}

#[test]
#[ignore = "production-path fixture: cold claim_executor walk (~7m); recipe in plan.dag"]
fn walk_plan_stage_receipt_refusal_blocks_stage2() {
    let root = workspace_root();
    let marker = stage2_marker_path(&root);
    let _ = std::fs::remove_file(&marker);
    let out = run_fixture_plan("walk_plan_stage_receipt_refusal_barrier_plan");
    assert!(
        out.stdout
            .contains("PASS [on-success stage 1] walk_plan_stage_receipt_refusal_poison_holds"),
        "poison claim must pass before receipt write is attempted;\n{}",
        out.combined()
    );
    assert!(
        out.stderr.contains("receipt write failed"),
        "stage-1 receipt write must fail closed;\n{}",
        out.combined()
    );
    assert!(
        out.stderr.contains("remaining stage(s) NOT run"),
        "receipt refusal must block stage 2;\n{}",
        out.combined()
    );
    assert!(
        !marker.exists(),
        "stage-2 marker must not exist when stage-1 receipt write refused"
    );
    assert_ne!(
        out.code,
        0,
        "receipt-refusal plan must exit nonzero;\n{}",
        out.combined()
    );
}
