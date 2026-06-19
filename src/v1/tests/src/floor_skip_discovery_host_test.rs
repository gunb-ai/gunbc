//! §5 execution receipts for Phase 1.5a REDO floor skip host transport (`cli_run`).
//!
//! Proves skip-disabled path runs, skip-enabled with empty/non-applicable diff runs all,
//! and git diff observation failure fail-closes to running witnesses.

use std::path::PathBuf;

use v1_compiler::cli_run::{
    run_discovery_corpus_with_options, DiscoveryCorpusOptions, DiscoverySummary,
};
use v1_compiler::v1_interpreter::ExecutionMode;

fn workspace_root() -> PathBuf {
    crate::helpers::workspace_root()
}

fn floor_skip_test_roster() -> (Vec<String>, Vec<String>, Vec<(String, String)>) {
    let ws = workspace_root();
    let source_roots = vec![
        ws.join("src/v2").to_string_lossy().into_owned(),
        ws.join("dsl").to_string_lossy().into_owned(),
    ];
    let entry = ws
        .join("src/v2/workflow/affected_set_floor_runner_test.dag")
        .to_string_lossy()
        .into_owned();
    let function = "floor_runner_node_frontier_policy_holds".to_string();
    (source_roots, Vec::new(), vec![(entry, function)])
}

fn run_explicit_roster(skip: bool) -> Result<DiscoverySummary, String> {
    let (source_roots, scan_dirs, explicit) = floor_skip_test_roster();
    run_discovery_corpus_with_options(
        &source_roots,
        &scan_dirs,
        &explicit,
        ExecutionMode::Wet,
        DiscoveryCorpusOptions {
            skip_unaffected_node_frontier: skip,
        },
    )
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(v) => std::env::set_var(self.key, v),
            None => std::env::remove_var(self.key),
        }
    }
}

#[test]
fn discovery_corpus_entry_resolves_in_roster_order() {
    use std::collections::BTreeSet;

    use v1_compiler::cli_run::{
        build_multi_entry_index, discover_floor_corpus_rows, resolve_entry_with_index,
    };

    let ws = workspace_root();
    let roots = vec![
        ws.join("src/v2").to_string_lossy().into_owned(),
        ws.join("dsl").to_string_lossy().into_owned(),
    ];
    let scan_dirs = vec![
        ws.join("dsl/test/claim").to_string_lossy().into_owned(),
        ws.join("src/v2/compiler/manual")
            .to_string_lossy()
            .into_owned(),
    ];
    let rows = discover_floor_corpus_rows(&roots, &scan_dirs).expect("discover roster");
    let index = build_multi_entry_index(&roots);
    let mut seen = BTreeSet::new();
    for row in &rows {
        if !seen.insert(row.entry.clone()) {
            continue;
        }
        resolve_entry_with_index(&index, &row.entry).unwrap_or_else(|e| {
            panic!("resolve failed for {}: {e}", row.entry);
        });
    }
}

#[test]
fn budget_roster_resolves_after_frontier_warmup() {
    use v1_compiler::cli_run::{build_multi_entry_index, resolve_entry_with_index};

    let ws = workspace_root();
    let roots = vec![
        ws.join("src/v2").to_string_lossy().into_owned(),
        ws.join("dsl").to_string_lossy().into_owned(),
    ];
    let index = build_multi_entry_index(&roots);
    for path in [
        "dsl/std/realization_schedule.dag",
        "src/v2/workflow/affected_set_floor_runner.dag",
        "src/v2/workflow/affected_set_floor_runner_test.dag",
    ] {
        let _ = resolve_entry_with_index(&index, path);
    }
    let entry = ws
        .join("src/v2/compiler/complexity_gate/budget_roster_completeness_test.dag")
        .to_string_lossy()
        .into_owned();
    resolve_entry_with_index(&index, &entry).expect("budget_roster should resolve");
}

#[test]
fn budget_roster_resolves_cold() {
    use v1_compiler::cli_run::{build_multi_entry_index, resolve_entry_with_index};

    let ws = workspace_root();
    let roots = vec![
        ws.join("src/v2").to_string_lossy().into_owned(),
        ws.join("dsl").to_string_lossy().into_owned(),
    ];
    let index = build_multi_entry_index(&roots);
    let entry = ws
        .join("src/v2/compiler/complexity_gate/budget_roster_completeness_test.dag")
        .to_string_lossy()
        .into_owned();
    resolve_entry_with_index(&index, &entry).expect("budget_roster should resolve cold");
}

#[test]
fn discovery_corpus_skip_disabled_runs_without_panic() {
    let summary = run_explicit_roster(false).expect("skip-disabled discovery must not panic");
    assert_eq!(summary.skipped, 0, "skip disabled → no skips");
    assert!(
        summary.passed >= 1,
        "skip-disabled path must run at least one witness"
    );
}

#[test]
fn discovery_corpus_skip_enabled_empty_diff_runs_corpus() {
    let _base = EnvVarGuard::set("GUNBC_CI_DIFF_BASE", "HEAD");
    let _head = EnvVarGuard::set("GUNBC_CI_DIFF_HEAD", "HEAD");
    let summary = run_explicit_roster(true).expect("empty diff path must not panic");
    assert_eq!(
        summary.skipped, 0,
        "empty diff → fail-closed run-all (no stateless skip)"
    );
    assert!(summary.passed >= 1);
}

#[test]
fn discovery_corpus_skip_enabled_git_observation_fail_closed_runs() {
    let _base = EnvVarGuard::set("GUNBC_CI_DIFF_BASE", "__gunbc_invalid_diff_base__");
    let _head = EnvVarGuard::set("GUNBC_CI_DIFF_HEAD", "HEAD");
    let _merge = EnvVarGuard::set("GUNBC_CI_DIFF_MERGE_BASE", "0");
    let summary = run_explicit_roster(true).expect("git observation fail-closed must not panic");
    assert_eq!(
        summary.skipped, 0,
        "git diff failure → skip inactive → run full explicit roster"
    );
    assert!(summary.passed >= 1);
}
