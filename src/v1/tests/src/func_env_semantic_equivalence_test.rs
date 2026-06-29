//! Oracle 4 — whole-corpus semantic equivalence vs pre-scope-chain baseline.
//!
//! Proves dsl+src/v1 strict whole-tree typecheck is byte-identical on diagnostic
//! fingerprint and rust_corpus_repr vs commit 57223267a2 (design sketch, flat
//! func_env.signatures closure). Fixture captured via capture_func_env_semantic_oracle.

use std::fs;

use serde::Deserialize;
use v1_compiler::cli_run::{
    whole_corpus_semantic_oracle_snapshot, FLOOR_DISCOVERY_EXCLUDES,
};

use crate::helpers::workspace_root;

const BASELINE_FIXTURE: &str =
    "src/v1/tests/fixtures/func_env_semantic_baseline.json";

#[derive(Debug, Deserialize)]
struct SemanticBaseline {
    baseline_commit: String,
    diagnostic_fingerprint: String,
    rust_corpus_repr: String,
    modules_resolved: usize,
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

fn whole_tree_probe_roots() -> Vec<String> {
    vec![
        workspace_root().join("dsl").to_string_lossy().into_owned(),
        workspace_root()
            .join("src/v1")
            .to_string_lossy()
            .into_owned(),
    ]
}

#[test]
fn func_env_whole_corpus_semantic_oracle_matches_pre_change_baseline() {
    let fixture_path = workspace_root().join(BASELINE_FIXTURE);
    let raw = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("read baseline fixture {fixture_path:?}: {e}"));
    let baseline: SemanticBaseline = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("parse baseline fixture: {e}"));

    let current = whole_corpus_semantic_oracle_snapshot(
        &whole_tree_probe_roots(),
        &whole_tree_probe_excludes(),
    )
    .expect("whole-corpus strict resolve for semantic oracle");

    assert_eq!(
        current.diagnostic_fingerprint, baseline.diagnostic_fingerprint,
        "whole-corpus diagnostic fingerprint must match pre-change baseline {} \
         (shadowing-order or typecheck behavior regression)",
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
}
