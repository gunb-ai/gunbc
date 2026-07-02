//! Acceptance witnesses for `GUNBC_FLOOR_PHASE_PROFILE` (claim_executor heartbeat).
#![cfg(unix)]

use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

fn claim_executor_bin() -> std::path::PathBuf {
    std::env::var_os("CARGO_BIN_EXE_claim_executor")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            workspace_root().join(
                std::env::var("PROFILE")
                    .map(|p| format!("target/{p}/claim_executor"))
                    .unwrap_or_else(|_| "target/debug/claim_executor".to_string()),
            )
        })
}

#[test]
fn phase_profile_sigterm_mid_walk_flushes_last_tick() {
    let root = workspace_root();
    let bin = claim_executor_bin();
    assert!(
        bin.exists(),
        "claim_executor binary missing at {}",
        bin.display()
    );

    let mut child = Command::new(&bin)
        .current_dir(&root)
        .env("GUNBC_FLOOR_PHASE_PROFILE", "1")
        .env("GUNBC_FLOOR_PHASE_PROFILE_INTERVAL_SECS", "1")
        .args([
            "--source-root",
            "dsl",
            "--source-root",
            "src/v2",
            "--plan-entry",
            "src/v2/workflow/phase_profile_proof_plan.dag",
            "--plan-function",
            "phase_profile_proof_batches",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn claim_executor");

    thread::sleep(Duration::from_secs(3));
    let pid = child.id();
    extern "C" {
        fn kill(pid: LibcPid, sig: i32) -> i32;
    }
    type LibcPid = i32;
    const SIGTERM: i32 = 15;
    unsafe {
        kill(pid as LibcPid, SIGTERM);
    }

    let mut stderr = String::new();
    if let Some(mut err) = child.stderr.take() {
        let _ = err.read_to_string(&mut stderr);
    }
    let _ = child.wait();

    assert!(
        stderr.contains("[phase-profile]"),
        "expected phase-profile ticks on stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("signal=SIGTERM") && stderr.contains("flushed=1"),
        "expected SIGTERM-flushed tick on stderr, got:\n{stderr}"
    );
    let tick_lines: Vec<_> = stderr
        .lines()
        .filter(|l| l.contains("[phase-profile]"))
        .collect();
    assert!(
        tick_lines.len() >= 2,
        "acceptance A: expected >=2 ticks before SIGTERM flush, got {} lines:\n{stderr}",
        tick_lines.len()
    );
    let phases: std::collections::BTreeSet<_> = tick_lines
        .iter()
        .filter_map(|l| l.split("phase=").nth(1)?.split_whitespace().next())
        .collect();
    assert!(
        phases.len() >= 2,
        "acceptance B: expected multiple distinct phase tags, got {:?} in:\n{stderr}",
        phases
    );
}
