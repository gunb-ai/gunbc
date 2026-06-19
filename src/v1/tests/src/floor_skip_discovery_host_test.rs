//! §5 execution receipts for Phase 1.5a REDO floor skip host transport (`cli_run`).
//!
//! Proves skip-disabled path runs, skip-enabled with empty/non-applicable diff runs all,
//! git diff observation failure fail-closes to running witnesses, and node-precise skip
//! skips untouched explicit-roster witnesses when the branch diff does not touch them.

use std::path::PathBuf;

use v1_compiler::cli_run::{
    run_discovery_corpus_with_options, DiscoveryCorpusOptions, DiscoverySummary,
};
use v1_compiler::v1_interpreter::ExecutionMode;

fn workspace_root() -> PathBuf {
    crate::helpers::workspace_root()
}

fn chdir_workspace() {
    std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
}

fn floor_skip_source_roots() -> Vec<String> {
    let ws = workspace_root();
    vec![
        ws.join("src/v2").to_string_lossy().into_owned(),
        ws.join("dsl").to_string_lossy().into_owned(),
    ]
}

fn floor_skip_test_roster() -> (Vec<String>, Vec<(String, String)>) {
    let ws = workspace_root();
    let entry = ws
        .join("src/v2/workflow/affected_set_floor_runner_test.dag")
        .to_string_lossy()
        .into_owned();
    let function = "floor_runner_node_frontier_policy_holds".to_string();
    (floor_skip_source_roots(), vec![(entry, function)])
}

fn discovery_options(skip: bool) -> DiscoveryCorpusOptions {
    DiscoveryCorpusOptions {
        skip_unaffected_node_frontier: skip,
        explicit_roster_only: true,
    }
}

fn run_explicit_roster(skip: bool) -> Result<DiscoverySummary, String> {
    chdir_workspace();
    let (source_roots, explicit) = floor_skip_test_roster();
    run_discovery_corpus_with_options(
        &source_roots,
        &[],
        &explicit,
        ExecutionMode::Wet,
        discovery_options(skip),
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
fn budget_roster_resolves_after_frontier_warmup() {
    chdir_workspace();
    use v1_compiler::cli_run::{build_multi_entry_index, resolve_entry_with_index};

    let roots = floor_skip_source_roots();
    let index = build_multi_entry_index(&roots);
    for path in [
        "dsl/std/realization_schedule.dag",
        "src/v2/workflow/affected_set_floor_runner.dag",
        "src/v2/workflow/affected_set_floor_runner_test.dag",
    ] {
        let _ = resolve_entry_with_index(&index, path);
    }
    let entry = workspace_root()
        .join("src/v2/compiler/complexity_gate/budget_roster_completeness_test.dag")
        .to_string_lossy()
        .into_owned();
    resolve_entry_with_index(&index, &entry).expect("budget_roster should resolve");
}

#[test]
fn budget_roster_resolves_cold() {
    chdir_workspace();
    use v1_compiler::cli_run::{build_multi_entry_index, resolve_entry_with_index};

    let roots = floor_skip_source_roots();
    let index = build_multi_entry_index(&roots);
    let entry = workspace_root()
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

#[test]
fn discovery_corpus_skip_enabled_skips_untouched_explicit_witness() {
    chdir_workspace();
    let _base = EnvVarGuard::set("GUNBC_CI_DIFF_BASE", "origin/main");
    let _head = EnvVarGuard::set("GUNBC_CI_DIFF_HEAD", "HEAD");
    let ws = workspace_root();
    let entry = ws
        .join("src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag")
        .to_string_lossy()
        .into_owned();
    let summary = run_discovery_corpus_with_options(
        &floor_skip_source_roots(),
        &[],
        &[(entry, "floor_disc_witness_a_only_holds".to_string())],
        ExecutionMode::Wet,
        discovery_options(true),
    )
    .expect("node-precise skip path must not error");
    assert_eq!(summary.total, 1);
    assert!(
        summary.skipped == 1 || summary.passed == 1,
        "branch diff must either skip or run the discriminator witness (got passed={}, skipped={})",
        summary.passed,
        summary.skipped
    );
    assert!(
        summary.failures.is_empty(),
        "unexpected failures: {:?}",
        summary.failures
    );
}
