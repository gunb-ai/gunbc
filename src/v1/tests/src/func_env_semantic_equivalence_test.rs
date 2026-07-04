//! Oracle 4 — whole-corpus semantic equivalence vs a frozen baseline commit.
//!
//! Proves the compiler stays byte-identical to a pinned baseline commit on
//! diagnostics, per-module emit repr, and full `EmitGraphInfo` when run over
//! the frozen baseline corpus (`git archive <BASELINE_COMMIT> dag src/v1`).
//! Fixture captured via capture_func_env_semantic_oracle on that tree.
//!
//! BASELINE_COMMIT must postdate the dsl->dag rename (#6165) — the tree at
//! any earlier commit has `dsl/` not `dag/`, so `git archive ... dag src/v1`
//! fails on a fresh clone (a warm runner with a cached
//! target/func_env_semantic_baseline_corpus dir hides this).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Deserialize;
use v1_compiler::cli_run::{whole_corpus_semantic_oracle_snapshot, FLOOR_DISCOVERY_EXCLUDES};

use crate::helpers::workspace_root;

const BASELINE_COMMIT: &str = "aeb1739ec5c";
const BASELINE_FIXTURE: &str = "src/v1/tests/fixtures/func_env_semantic_baseline.json";

#[derive(Debug, Deserialize)]
struct SemanticBaseline {
    baseline_commit: String,
    diagnostic_fingerprint: String,
    rust_corpus_repr: String,
    emit_graph_fingerprint: String,
    corpus_fingerprint: String,
    modules_resolved: usize,
    per_module_rows: usize,
}

fn whole_tree_probe_excludes() -> Vec<String> {
    let mut exclude_subpaths: Vec<String> = FLOOR_DISCOVERY_EXCLUDES
        .iter()
        .map(|sub| (*sub).to_string())
        .collect();
    exclude_subpaths.extend([
        "test/fixture/".to_string(),
        "/test/".to_string(),
        "nat_semiring_rung".to_string(),
        "lens/application/empty_required_lenses_skip_gate.dag".to_string(),
        "lens/application/rejecting_lens_blocks_before_compile.dag".to_string(),
    ]);
    exclude_subpaths
}

fn git_toplevel() -> PathBuf {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(workspace_root())
        .output()
        .unwrap_or_else(|e| panic!("git rev-parse --show-toplevel: {e}"));
    assert!(
        output.status.success(),
        "git rev-parse --show-toplevel failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_verify_commit(commit: &str, root: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", &format!("{commit}^{{commit}}")])
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn ensure_baseline_commit_available(root: &Path) {
    if git_verify_commit(BASELINE_COMMIT, root) {
        return;
    }
    // PR checkouts can be too shallow to contain the frozen baseline commit.
    let _ = Command::new("git")
        .args(["fetch", "--unshallow"])
        .current_dir(root)
        .status();
    if git_verify_commit(BASELINE_COMMIT, root) {
        return;
    }
    let fetch = Command::new("git")
        .args(["fetch", "origin", BASELINE_COMMIT])
        .current_dir(root)
        .output()
        .unwrap_or_else(|e| panic!("git fetch origin {BASELINE_COMMIT}: {e}"));
    assert!(
        fetch.status.success() && git_verify_commit(BASELINE_COMMIT, root),
        "baseline commit {BASELINE_COMMIT} unavailable after fetch: {}",
        String::from_utf8_lossy(&fetch.stderr)
    );
}

fn baseline_corpus_dir() -> PathBuf {
    let git_root = git_toplevel();
    let dir = workspace_root()
        .join("target")
        .join("func_env_semantic_baseline_corpus");
    if dir.join("dag").is_dir() && dir.join("src/v1").is_dir() {
        return dir;
    }
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("create baseline corpus dir {dir:?}: {e}"));
    ensure_baseline_commit_available(&git_root);
    let archive = Command::new("git")
        .args(["archive", BASELINE_COMMIT, "dag", "src/v1"])
        .current_dir(&git_root)
        .output()
        .unwrap_or_else(|e| panic!("git archive {BASELINE_COMMIT}: {e}"));
    assert!(
        archive.status.success(),
        "git archive {BASELINE_COMMIT} failed: {}",
        String::from_utf8_lossy(&archive.stderr)
    );
    let mut tar = Command::new("tar")
        .args(["-x", "-C"])
        .arg(&dir)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn tar for baseline corpus: {e}"));
    {
        let stdin = tar.stdin.as_mut().expect("tar stdin");
        stdin
            .write_all(&archive.stdout)
            .unwrap_or_else(|e| panic!("write baseline corpus tar: {e}"));
    }
    let status = tar
        .wait()
        .unwrap_or_else(|e| panic!("wait for baseline corpus tar: {e}"));
    assert!(status.success(), "tar extract baseline corpus failed");
    dir
}

fn baseline_corpus_roots(corpus_dir: &Path) -> Vec<String> {
    vec![
        corpus_dir.join("dag").to_string_lossy().into_owned(),
        corpus_dir.join("src/v1").to_string_lossy().into_owned(),
    ]
}

#[test]
#[ignore = "CI witness opt-in inversion (2026-07-04): whole-corpus strict resolve + semantic-oracle snapshot over dag+src/v1 — the rust-lane twin of the corpus witnesses inverted out of the per-PR floor (run-everything had pushed both CI jobs to the 90-min timeout: max cost, zero signal). Run explicitly: cargo nextest run -p v1-compiler-tests -- --ignored func_env_whole_corpus_semantic_oracle_matches_pre_change_baseline. Re-enroll when affected-set selection + floor memoization land (see ci_witness_optin_inversion in gunbc.commit_workflow)."]
fn func_env_whole_corpus_semantic_oracle_matches_pre_change_baseline() {
    let fixture_path = workspace_root().join(BASELINE_FIXTURE);
    let raw = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("read baseline fixture {fixture_path:?}: {e}"));
    let baseline: SemanticBaseline =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse baseline fixture: {e}"));

    let corpus_dir = baseline_corpus_dir();
    let current = whole_corpus_semantic_oracle_snapshot(
        &baseline_corpus_roots(&corpus_dir),
        &whole_tree_probe_excludes(),
    )
    .expect("whole-corpus strict resolve for semantic oracle");

    assert_eq!(
        current.corpus_fingerprint, baseline.corpus_fingerprint,
        "whole-corpus fingerprint must match pre-change baseline {} \
         (per-module diagnostics/emit repr or emit-graph regression)",
        baseline.baseline_commit
    );
    assert_eq!(
        current.diagnostic_fingerprint, baseline.diagnostic_fingerprint,
        "whole-corpus diagnostic fingerprint must match pre-change baseline {}",
        baseline.baseline_commit
    );
    assert_eq!(
        current.emit_graph_fingerprint, baseline.emit_graph_fingerprint,
        "emit_graph fingerprint must match pre-change baseline {}",
        baseline.baseline_commit
    );
    assert_eq!(
        current.rust_corpus_repr, baseline.rust_corpus_repr,
        "rust_corpus_repr must match pre-change baseline {}",
        baseline.baseline_commit
    );
    assert_eq!(
        current.modules_resolved, baseline.modules_resolved,
        "module count must match baseline capture"
    );
    assert_eq!(
        current.per_module_rows, baseline.per_module_rows,
        "per-module row count must match baseline capture"
    );
}
