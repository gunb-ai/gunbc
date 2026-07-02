//! Acceptance witnesses for `GUNBC_FLOOR_PHASE_PROFILE` (process-global phase heartbeat).
#![cfg(unix)]

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

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

    let tick_count = Arc::new(AtomicUsize::new(0));
    let stderr_buf = Arc::new(Mutex::new(String::new()));
    let ticks = Arc::clone(&tick_count);
    let buf = Arc::clone(&stderr_buf);
    let stderr_pipe = child.stderr.take().expect("stderr pipe");
    let reader = thread::spawn(move || {
        let mut reader = BufReader::new(stderr_pipe);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if line.contains("[phase-profile]") {
                        ticks.fetch_add(1, Ordering::Relaxed);
                    }
                    if let Ok(mut guard) = buf.lock() {
                        guard.push_str(&line);
                    }
                }
                Err(_) => break,
            }
        }
    });

    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        if let Some(status) = child.try_wait().expect("try_wait claim_executor") {
            reader.join().expect("stderr reader");
            let stderr = stderr_buf.lock().expect("stderr buf").clone();
            panic!(
                "claim_executor exited early with {status:?} before SIGTERM mid-walk \
                 (got {} profile ticks); stderr:\n{stderr}",
                tick_count.load(Ordering::Relaxed)
            );
        }
        if tick_count.load(Ordering::Relaxed) >= 2 {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            reader.join().expect("stderr reader");
            let stderr = stderr_buf.lock().expect("stderr buf").clone();
            panic!("timed out waiting for 2 profile ticks; stderr:\n{stderr}");
        }
        thread::sleep(Duration::from_millis(50));
    }

    let pid = child.id();
    extern "C" {
        fn kill(pid: LibcPid, sig: i32) -> i32;
    }
    type LibcPid = i32;
    const SIGTERM: i32 = 15;
    unsafe {
        kill(pid as LibcPid, SIGTERM);
    }

    let status = child.wait().expect("wait for claim_executor");
    reader.join().expect("stderr reader");
    let stderr = stderr_buf.lock().expect("stderr buf").clone();

    assert_eq!(
        status.code(),
        Some(143),
        "acceptance C: process must flush then exit 143 after SIGTERM, got {status:?}\nstderr:\n{stderr}"
    );

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
