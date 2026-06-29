//! Oracle 4 — whole-corpus semantic equivalence vs pre-scope-chain baseline.
//!
//! Proves the post-refactor compiler is byte-identical to commit 57223267a2 on
//! diagnostics, per-module emit repr, and full `EmitGraphInfo` when run over
//! the frozen baseline corpus (`git archive 57223267a2 dsl src/v1`). Fixture
//! captured via capture_func_env_semantic_oracle on that tree.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Deserialize;
use v1_compiler::cli_run::{whole_corpus_semantic_oracle_snapshot, FLOOR_DISCOVERY_EXCLUDES};

use crate::helpers::workspace_root;

const BASELINE_COMMIT: &str = "57223267a2";
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

fn baseline_corpus_dir() -> PathBuf {
    let dir = workspace_root()
        .join("target")
        .join("func_env_semantic_baseline_corpus");
    if dir.join("dsl").is_dir() && dir.join("src/v1").is_dir() {
        return dir;
    }
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("create baseline corpus dir {dir:?}: {e}"));
    let archive = Command::new("git")
        .args(["archive", BASELINE_COMMIT, "dsl", "src/v1"])
        .current_dir(workspace_root())
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
        corpus_dir.join("dsl").to_string_lossy().into_owned(),
        corpus_dir.join("src/v1").to_string_lossy().into_owned(),
    ]
}

#[test]
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
