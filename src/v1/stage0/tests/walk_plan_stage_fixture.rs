//! Production-path stage-executor controls driven through real WalkPlan fixtures
//! (`src/v2/test/fixture/walk_plan_stage/plan.dag`). These are the discriminating
//! witnesses #7499's `run_stage` extraction needs: unit tests over `spawn_units`
//! closures cannot catch regressions where grouping, lane selection, admission, or
//! the stage loop itself is wrong.
//!
//! SCAFFOLD (§7 seed-retained HAND-RUST — authority: gunbc.walk_plan_stage_fixture_scaffold;
//! witness: dag/test/claim/walk_plan_stage_fixture_hand_rust_witness_test.dag).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard};

/// Production-path walks share `target/` hygiene paths; one session holds this lock
/// from pre-cleanup through post-walk assertions so parallel `#[ignore]` tests cannot
/// interleave marker/receipt mutations.
static FIXTURE_RUN_LOCK: Mutex<()> = Mutex::new(());

struct FixtureSession {
    _guard: MutexGuard<'static, ()>,
    root: PathBuf,
    authority: HarnessAuthority,
}

impl FixtureSession {
    fn begin() -> Self {
        let guard = FIXTURE_RUN_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let root = workspace_root();
        let authority = materialize_harness_authority(&root);
        Self {
            _guard: guard,
            root,
            authority,
        }
    }

    fn stage2_marker(&self) -> PathBuf {
        self.root.join(&self.authority.stage2_marker_rel)
    }

    fn stage1_receipt(&self) -> PathBuf {
        self.root.join(&self.authority.stage1_receipt_rel)
    }

    fn run_plan(&self, plan_function: &str) -> FixtureOutput {
        run_fixture_plan_unlocked(&self.root, &self.authority, plan_function)
    }
}

#[derive(Clone, Debug)]
struct HarnessAuthority {
    harness_authority_rel: String,
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

fn resolve_v1_bin(name: &str) -> PathBuf {
    let env_key = format!("CARGO_BIN_EXE_{name}");
    if let Ok(path) = std::env::var(&env_key) {
        return PathBuf::from(path);
    }
    let root = workspace_root();
    let mut candidates = Vec::new();
    if let Ok(profile) = std::env::var("PROFILE") {
        candidates.push(root.join(format!("target/{profile}/{name}")));
    }
    candidates.push(root.join(format!("target/release/{name}")));
    candidates.push(root.join(format!("target/debug/{name}")));
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .unwrap_or_else(|| root.join(format!("target/debug/{name}")))
}

fn claim_executor_bin() -> PathBuf {
    resolve_v1_bin("claim_executor")
}

fn claim_batch_bin() -> PathBuf {
    resolve_v1_bin("claim_batch")
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
        harness_authority_rel: require("harness_authority_path"),
        attempt_id: require("attempt_id"),
        plan_entry: require("plan_entry"),
        stage1_receipt_rel: require("stage1_receipt_path"),
        stage2_marker_rel: require("stage2_marker_path"),
    }
}

fn relative_path_from_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn looks_like_harness_authority(content: &str) -> bool {
    content
        .lines()
        .any(|line| line.starts_with("harness_authority_path="))
}

fn locate_harness_authority_file(root: &Path) -> PathBuf {
    let target = root.join("target");
    let mut candidates = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&target) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            if !looks_like_harness_authority(&content) {
                continue;
            }
            let authority = parse_harness_authority(&content);
            let rel = relative_path_from_root(root, &path);
            if authority.harness_authority_rel == rel {
                candidates.push(path);
            }
        }
    }
    match candidates.len() {
        0 => panic!(
            "harness authority file not found under {}; run walk_plan_stage_materialize_harness_authority_holds first",
            target.display()
        ),
        1 => candidates.remove(0),
        n => panic!(
            "ambiguous harness authority files under {} ({n} matches)",
            target.display()
        ),
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
    let authority_path = locate_harness_authority_file(root);
    let content = std::fs::read_to_string(&authority_path).unwrap_or_else(|e| {
        panic!(
            "failed to read {} after materialize: {e}",
            authority_path.display()
        )
    });
    parse_harness_authority(&content)
}

fn run_fixture_plan_unlocked(
    root: &Path,
    authority: &HarnessAuthority,
    plan_function: &str,
) -> FixtureOutput {
    let bin = claim_executor_bin();
    assert!(
        bin.exists(),
        "claim_executor binary missing at {}",
        bin.display()
    );
    let output = Command::new(&bin)
        .current_dir(root)
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

#[test]
#[ignore = "cold overlap resolve can take minutes; run locally: cargo test -p v1-compiler walk_plan_stage_overlap_barrier -- --ignored --test-threads=1"]
fn walk_plan_stage_overlap_barrier_production_path() {
    let session = FixtureSession::begin();
    let _ = std::fs::remove_file(session.stage2_marker());
    let out = session.run_plan("walk_plan_stage_overlap_barrier_plan");
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
        session.stage1_receipt().is_file(),
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
    let session = FixtureSession::begin();
    let marker = session.stage2_marker();
    let _ = std::fs::remove_file(&marker);
    let out = session.run_plan("walk_plan_stage_failure_barrier_plan");
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
fn walk_plan_stage_recursion_refusal_blocks_stage2() {
    let session = FixtureSession::begin();
    let marker = session.stage2_marker();
    let _ = std::fs::remove_file(&marker);
    let out = session.run_plan("walk_plan_stage_recursion_refusal_barrier_plan");
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
    let session = FixtureSession::begin();
    let marker = session.stage2_marker();
    let _ = std::fs::remove_file(&marker);
    let out = session.run_plan("walk_plan_stage_receipt_refusal_barrier_plan");
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
