//! §5 execution receipts for Phase 1.5a REDO floor skip host transport (`cli_run`).
//!
//! Proves skip-disabled path runs, skip-enabled with empty/non-applicable diff runs all,
//! git diff observation failure fail-closes to running witnesses, and node-precise skip
//! skips untouched explicit-roster witnesses when the branch diff does not touch them.

use std::path::PathBuf;
use std::sync::Mutex;

use v1_compiler::cli_run::{
    run_discovery_corpus_with_options, DiscoveryCorpusOptions, DiscoverySummary,
};
use v1_compiler::v1_interpreter::ExecutionMode;

/// Serializes tests that mutate the process-global `GUNBC_CI_DIFF_*` env so `cargo test`'s
/// default multi-threaded harness cannot let one test's injected diff leak into another.
static DIFF_ENV_LOCK: Mutex<()> = Mutex::new(());

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
    let _env = DIFF_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
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
    let _env = DIFF_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
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

fn fixture_line(text: &str, needle: &str) -> i64 {
    text.lines()
        .position(|l| l.contains(needle))
        .map(|i| (i + 1) as i64)
        .unwrap_or_else(|| panic!("discriminator fixture missing line containing `{needle}`"))
}

/// Run the node-precise floor skip over a single deterministic injected unified diff
/// (`git diff -U0` shape) touching exactly `line` of `rel_path`.
fn run_injected_diff_roster(
    rel_path: &str,
    line: i64,
    entry: &str,
    roster: &[(String, String)],
) -> DiscoverySummary {
    let unified = format!("+++ b/{rel_path}\n@@ -{line},0 +{line},1 @@\n");
    let _diff = EnvVarGuard::set("GUNBC_CI_DIFF_UNIFIED", &unified);
    run_discovery_corpus_with_options(
        &floor_skip_source_roots(),
        &[],
        roster,
        ExecutionMode::Wet,
        discovery_options(true),
    )
    .expect("node-precise skip path must not error (FreeMonoid decode root fix)")
}

fn run_injected_diff(rel_path: &str, line: i64, entry: &str) -> DiscoverySummary {
    run_injected_diff_roster(
        rel_path,
        line,
        entry,
        &[(
            entry.to_string(),
            "floor_disc_witness_a_only_holds".to_string(),
        )],
    )
}

/// §5 same-file node-precision discriminator — the acceptance bar. ONE fixture file holds
/// two independent nodes A and B; witness `floor_disc_witness_a_only_holds`'s claim
/// references node A only. A unified diff touching node A's declaration RUNS the witness;
/// a diff touching ONLY node B's declaration SKIPS it. Both edits are in the SAME file, so
/// a file-level skip can never produce the B-only skip, and a file-level run-all can never
/// produce it either — only true node precision discriminates A from B.
///
/// This also proves the FreeMonoid decode root fix by execution: the skip decision walks
/// `eval_data_item_value` / `eval_data_initializer_values` over the fixture's `List<Node>`
/// initializers, which decode only under `with_active_context`; without the fix the path
/// errors and the `.expect` in `run_injected_diff` panics (and a re-masking regression that
/// fail-closed to run-all would flip the B-only `skipped == 1` assertion red).
#[test]
fn node_precise_same_file_a_runs_b_skips_by_execution() {
    let _env = DIFF_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    chdir_workspace();
    let ws = workspace_root();
    let rel = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
    let abs = ws.join(rel);
    let text = std::fs::read_to_string(&abs).expect("discriminator fixture readable");
    // Interior, unique lines of each node's declaration span.
    let a_line = fixture_line(&text, "^floor_disc_node_a_symbol");
    let b_line = fixture_line(&text, "^floor_disc_node_b_symbol");
    let entry = abs.to_string_lossy().into_owned();

    // A-edit → witness RUNS (node A is in the claim-on-A closure).
    let a = run_injected_diff(rel, a_line, &entry);
    assert_eq!(a.total, 1);
    assert!(
        a.failures.is_empty(),
        "A-edit produced failures: {:?}",
        a.failures
    );
    assert_eq!(a.skipped, 0, "A-edit must NOT skip the claim-on-A witness");
    assert_eq!(a.passed, 1, "A-edit must RUN the claim-on-A witness");

    // B-only edit → witness SKIPS (node B is NOT in the claim-on-A closure).
    let b = run_injected_diff(rel, b_line, &entry);
    assert_eq!(b.total, 1);
    assert!(
        b.failures.is_empty(),
        "B-edit produced failures: {:?}",
        b.failures
    );
    assert_eq!(
        b.passed, 0,
        "B-only edit must NOT run the claim-on-A witness"
    );
    assert_eq!(
        b.skipped, 1,
        "B-only edit must SKIP the claim-on-A witness (node precision; a file-level impl runs it)"
    );
}

/// B-only edit in the same file: witness-on-A skips, witness-on-B runs.
#[test]
fn node_precise_same_file_b_edit_runs_b_witness_skips_a_witness() {
    let _env = DIFF_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    chdir_workspace();
    let ws = workspace_root();
    let rel = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
    let abs = ws.join(rel);
    let text = std::fs::read_to_string(&abs).expect("discriminator fixture readable");
    let b_line = fixture_line(&text, "^floor_disc_node_b_symbol");
    let entry = abs.to_string_lossy().into_owned();
    let roster = vec![
        (entry.clone(), "floor_disc_witness_a_only_holds".to_string()),
        (entry.clone(), "floor_disc_witness_b_only_holds".to_string()),
    ];

    let summary = run_injected_diff_roster(rel, b_line, &entry, &roster);
    assert_eq!(summary.total, 2);
    assert!(
        summary.failures.is_empty(),
        "B-edit failures: {:?}",
        summary.failures
    );
    assert_eq!(
        summary.skipped, 1,
        "witness-on-A must skip when only node B's span changed"
    );
    assert_eq!(
        summary.passed, 1,
        "witness-on-B must run when node B's span changed"
    );
}

/// Transitive closure soundness: edit inner node C; conj-wrapped witness must RUN.
#[test]
fn node_precise_transitive_c_edit_runs_conj_witness() {
    let _env = DIFF_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    chdir_workspace();
    let ws = workspace_root();
    let rel = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
    let abs = ws.join(rel);
    let text = std::fs::read_to_string(&abs).expect("discriminator fixture readable");
    let c_line = fixture_line(&text, "^floor_disc_node_c_symbol");
    let entry = abs.to_string_lossy().into_owned();

    let summary = run_injected_diff_roster(
        rel,
        c_line,
        &entry,
        &[(
            entry.clone(),
            "floor_disc_witness_transitive_holds".to_string(),
        )],
    );
    assert_eq!(summary.total, 1);
    assert!(
        summary.failures.is_empty(),
        "C-edit transitive witness failures: {:?}",
        summary.failures
    );
    assert_eq!(
        summary.skipped, 0,
        "transitive witness must RUN when inner node C's span changed (shallow closure would skip)"
    );
    assert_eq!(summary.passed, 1);
}
