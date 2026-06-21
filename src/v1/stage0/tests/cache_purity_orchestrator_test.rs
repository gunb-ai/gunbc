//! Executable proof of the cache-purity audit ORCHESTRATOR's fail-closed join (DESIGN §5; ROADMAP
//! §2 P3 TOOTH 1/2). The orchestrator backgrounds K sharded leaf processes beside the floor and must
//! go RED if ANY shard reports a violation (incl. the deepest/last) OR a shard crashes/OOMs/drops
//! (no result file), and must fail closed when the residual budget admits width 0. These run the REAL
//! release bin end-to-end (`CARGO_BIN_EXE_cache_purity_audit`, which Cargo builds before the test);
//! the leaf test hooks short-circuit before any heavy resolve so the join teeth are fast.
//!
//! A discriminating control: each tooth's expectation FLIPS if the join were fail-OPEN — a dropped or
//! violating shard that the orchestrator silently passed would make these assert exit 0 and go RED.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_cache_purity_audit")
}

/// An empty roots dir → leaf shards discover 0 entries → trivially clean & fast (no real resolve).
fn empty_roots() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cpa-orch-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn orchestrate(width: &str, hook: Option<(&str, &str)>) -> std::process::Output {
    let roots = empty_roots();
    let mut cmd = Command::new(bin());
    cmd.arg("--orchestrate")
        .arg("--width")
        .arg(width)
        .arg("--source-root")
        .arg(&roots)
        .arg("--scan-dir")
        .arg(&roots);
    if let Some((k, v)) = hook {
        cmd.env(k, v);
    }
    cmd.output().expect("run orchestrator")
}

/// TOOTH 2 (realized): width 0 — the residual budget cannot fit one audit shard beside the floor —
/// is itself fail-closed: no fan-out, non-zero exit, loud error. NEVER a silent width-1 OOM.
#[test]
fn orchestrator_fails_closed_on_width_zero() {
    let out = orchestrate("0", None);
    assert!(!out.status.success(), "width 0 must fail closed (non-zero exit)");
    let log = String::from_utf8_lossy(&out.stdout);
    assert!(
        log.contains("residual budget too small"),
        "width 0 must emit the fail-closed residual-budget error; got:\n{log}"
    );
}

/// TOOTH 1a: a warm!=cold violation in the LAST/deepest shard (3 of 4) must turn the whole run RED —
/// not just shard 0. RED-on-revert: if the join only checked shard 0, this would (wrongly) exit 0.
#[test]
fn orchestrator_red_on_violation_in_last_shard() {
    let out = orchestrate("4", Some(("GUNBC_CPA_TEST_VIOLATE_SHARD", "3")));
    assert!(!out.status.success(), "a violation in the last shard must fail the run");
    let log = String::from_utf8_lossy(&out.stdout);
    assert!(
        log.contains("shard 3 RED") && log.contains("violations 1"),
        "the last shard's violation must be reported and aggregated; got:\n{log}"
    );
}

/// TOOTH 1b: a shard that crashes/OOMs/drops (exits without writing its result file) must be treated
/// as FAILED — the co-process fail-open risk. A dropped-but-silently-passed shard would exit 0 here.
#[test]
fn orchestrator_red_on_dropped_shard() {
    let out = orchestrate("4", Some(("GUNBC_CPA_TEST_DROP_SHARD", "2")));
    assert!(!out.status.success(), "a dropped/crashed shard must fail the run (fail-closed)");
    let log = String::from_utf8_lossy(&out.stdout);
    assert!(
        log.contains("shard 2 DROPPED") && log.contains("no parseable result"),
        "a dropped shard must be reported as a fail-closed drop, never silently passed; got:\n{log}"
    );
}

/// GREEN control: with no injected fault and an empty corpus, all shards report clean and the
/// orchestrator exits 0 — so the RED teeth above are discriminating, not always-red.
#[test]
fn orchestrator_green_when_all_shards_clean() {
    let out = orchestrate("4", None);
    assert!(
        out.status.success(),
        "clean shards must pass; got:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
}
