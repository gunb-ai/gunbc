//! Production-path stage-executor controls driven through real WalkPlan fixtures
//! (`src/v2/test/fixture/walk_plan_stage/plan.dag`). These are the discriminating
//! witnesses #7499's `run_stage` extraction needs: unit tests over `spawn_units`
//! closures cannot catch regressions where grouping, lane selection, admission, or
//! the stage loop itself is wrong.
//!
//! SCAFFOLD (§7 seed-retained HAND-RUST — authority: gunbc.walk_plan_stage_fixture_scaffold;
//! witness: dag/test/claim/walk_plan_stage_fixture_hand_rust_witness_test.dag).

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

/// Production-path walks share `target/` hygiene paths; serialize harness runs so
/// parallel `cargo test` threads cannot interleave cleanup with overlap markers.
static FIXTURE_RUN_LOCK: Mutex<()> = Mutex::new(());

thread_local! {
    static LAST_HARNESS_AUTHORITY: RefCell<Option<HarnessAuthority>> = const { RefCell::new(None) };
}

#[derive(Clone, Debug)]
struct HarnessAuthority {
    attempt_id: String,
    plan_entry: String,
    stage1_receipt_rel: String,
    stage2_marker_rel: String,
}

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

fn claim_batch_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_claim_batch")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            workspace_root().join(
                std::env::var("PROFILE")
                    .map(|p| format!("target/{p}/claim_batch"))
                    .unwrap_or_else(|_| "target/debug/claim_batch".to_string()),
            )
        })
}

fn parse_harness_authority(content: &str) -> HarnessAuthority {
    let mut fields = HashMap::new();
    for line in content.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        fields.insert(key.to_string(), value.to_string());
    }
    let require = |key: &str| -> String {
        fields.get(key).cloned().unwrap_or_else(|| {
            panic!("harness authority missing {key}:\n{content}");
        })
    };
    HarnessAuthority {
        attempt_id: require("attempt_id"),
        plan_entry: require("plan_entry"),
        stage1_receipt_rel: require("stage1_receipt_path"),
        stage2_marker_rel: require("stage2_marker_path"),
    }
}

fn materialize_harness_authority(root: &Path) -> HarnessAuthority {
    let batch = claim_batch_bin();
    assert!(
        batch.exists(),
        "claim_batch binary missing at {}",
        batch.display()
    );
    let status = Command::new(&batch)
        .current_dir(root)
        .args([
            "--source-root",
            "dag",
            "--source-root",
            "src/v2",
            "--entry",
            "src/v2/test/fixture/walk_plan_stage/common.dag",
            "--function",
            "walk_plan_stage_materialize_harness_authority_holds",
            "--wet",
            "--claim-run",
        ])
        .status()
        .expect("spawn claim_batch for harness authority");
    assert!(
        status.success(),
        "walk_plan_stage_materialize_harness_authority_holds must pass before harness walks"
    );
    let authority_path = root.join("target/walk_plan_stage_harness_authority.txt");
    let content = std::fs::read_to_string(&authority_path).unwrap_or_else(|e| {
        panic!(
            "failed to read {} after materialize: {e}",
            authority_path.display()
        )
    });
    parse_harness_authority(&content)
}

fn last_harness_authority() -> HarnessAuthority {
    LAST_HARNESS_AUTHORITY.with(|cell| {
        cell.borrow()
            .clone()
            .expect("run_fixture_plan must run before reading harness authority")
    })
}

fn run_fixture_plan(plan_function: &str) -> FixtureOutput {
    let _guard = FIXTURE_RUN_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = workspace_root();
    let authority = materialize_harness_authority(&root);
    LAST_HARNESS_AUTHORITY.with(|cell| *cell.borrow_mut() = Some(authority.clone()));
    let bin = claim_executor_bin();
    assert!(
        bin.exists(),
        "claim_executor binary missing at {}",
        bin.display()
    );
    let output = Command::new(&bin)
        .current_dir(&root)
        .env("GUNBC_WALK_ATTEMPT_ID", &authority.attempt_id)
        .args([
            "--source-root",
            "dag",
            "--source-root",
            "src/v2",
            "--plan-entry",
            &authority.plan_entry,
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
    root.join(&last_harness_authority().stage2_marker_rel)
}

fn stage1_receipt_path(root: &PathBuf) -> PathBuf {
    root.join(&last_harness_authority().stage1_receipt_rel)
}

#[test]
#[ignore = "cold overlap resolve can take minutes; run locally: cargo test -p v1-compiler walk_plan_stage_overlap_barrier -- --ignored --test-threads=1"]
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
#[ignore = "production-path fixture: cold claim_executor walk (~7m); recipe in plan.dag; use --test-threads=1"]
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
#[ignore = "production-path fixture: cold claim_executor walk (~7m); recipe in plan.dag; use --test-threads=1"]
fn walk_plan_stage_panic_blocks_stage2() {
    let root = workspace_root();
    let marker = stage2_marker_path(&root);
    let _ = std::fs::remove_file(&marker);
    let out = run_fixture_plan("walk_plan_stage_panic_barrier_plan");
    assert!(
        out.stdout.contains("call depth exceeded") || out.stdout.contains("unbounded recursion"),
        "panicking member must surface structural recursion-depth refusal on stdout;\n{}",
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
#[ignore = "production-path fixture: cold claim_executor walk (~7m); recipe in plan.dag; use --test-threads=1"]
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
