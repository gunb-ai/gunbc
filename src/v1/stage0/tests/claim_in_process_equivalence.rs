//! RED control #2 (v1-run-stability M2, Deliverable 2 — single-binary claim
//! execution): a claim's verdict is IDENTICAL whether it runs in-process
//! (`cli_run::run_claims_in_process`) or via the spawned `claim_batch` child. Proven
//! both directions on the same corpus slice — a passing claim is `true` / exit 0 on
//! both paths; a failing claim is `false` / nonzero on both (the discriminating red).
//! This retires the risk the in-process fold-in of the `run_gunbc_claims` transport
//! carries: folding claims into the executor must not change a single verdict.
#![cfg(unix)]

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

/// Write a tiny probe corpus under the workspace's `target/` (gitignored, and under
/// the workspace root so path normalization accepts it). Returns (source_root_abs,
/// entry_abs, entry_relpath).
fn write_probe_corpus() -> (String, String, String) {
    let dir = workspace_root()
        .join("target")
        .join(format!("claim_d2_probe_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir probe corpus");
    std::fs::write(
        dir.join("probe.dag"),
        "module claim_d2_probe.probe\n\nfn pass_claim() -> Bool { true }\n\nfn fail_claim() -> Bool { false }\n",
    )
    .expect("write probe");
    let root = dir.to_string_lossy().to_string();
    let entry_abs = dir.join("probe.dag").to_string_lossy().to_string();
    let entry_rel = entry_abs
        .strip_prefix(&format!("{}/", workspace_root().to_string_lossy()))
        .unwrap_or(&entry_abs)
        .to_string();
    let root_rel = root
        .strip_prefix(&format!("{}/", workspace_root().to_string_lossy()))
        .unwrap_or(&root)
        .to_string();
    (root_rel, entry_abs, entry_rel)
}

/// Spawn-path verdict: run one claim through the `claim_batch` child, return whether
/// it passed (exit 0).
fn spawn_verdict(root_rel: &str, entry_rel: &str, function: &str) -> bool {
    let bin = claim_batch_bin();
    assert!(bin.exists(), "claim_batch missing at {}", bin.display());
    let status = Command::new(&bin)
        .current_dir(workspace_root())
        .args([
            "--source-root",
            root_rel,
            "--entry",
            entry_rel,
            "--function",
            function,
            "--claim-run",
        ])
        .status()
        .expect("spawn claim_batch");
    status.success()
}

#[test]
fn claim_in_process_matches_spawn_verdict() {
    let (root_rel, entry_abs, entry_rel) = write_probe_corpus();

    for (function, expected) in [("pass_claim", true), ("fail_claim", false)] {
        // In-process path (the new single-binary vehicle).
        let in_process = v1_compiler::cli_run::run_claims_in_process(
            &[root_rel.clone()],
            &[(entry_abs.clone(), function.to_string())],
            v1_compiler::v1_interpreter::ExecutionMode::Hermetic,
        );
        // Spawn path (the child that folds away).
        let spawned = spawn_verdict(&root_rel, &entry_rel, function);

        assert_eq!(
            in_process, expected,
            "in-process verdict for {function} should be {expected}"
        );
        assert_eq!(
            spawned, expected,
            "spawn verdict for {function} should be {expected}"
        );
        assert_eq!(
            in_process, spawned,
            "in-process and spawn verdicts must be identical for {function}"
        );
    }

    let _ = std::fs::remove_dir_all(
        workspace_root()
            .join("target")
            .join(format!("claim_d2_probe_{}", std::process::id())),
    );
}
