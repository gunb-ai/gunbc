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
        1,
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
    roster: &[(String, String)],
) -> DiscoverySummary {
    let unified = format!("+++ b/{rel_path}\n@@ -{line},0 +{line},1 @@\n");
    let _diff = EnvVarGuard::set("GUNBC_CI_DIFF_UNIFIED", &unified);
    run_discovery_corpus_with_options(
        &floor_skip_source_roots(),
        &[],
        roster,
        ExecutionMode::Wet,
        1,
        discovery_options(true),
    )
    .expect("node-precise skip path must not error (FreeMonoid decode root fix)")
}

/// §5 node-level (not file-level) discriminator by execution — the acceptance bar. In ONE
/// fixture file, editing a node the witness's claim references RUNS it; editing an ORPHAN
/// node (referenced by no claim), same file, SKIPS it. A file-level skip can never produce
/// the orphan skip — only true node precision can.
///
/// (The earlier intra-witness A/B variant was unsound and is dropped: a witness whose body
/// names another node genuinely depends on it — its Bool result can flip if that node's
/// identity changes — so §5 fail-closed must RUN it, not skip it. Discrimination is sound
/// only between a referenced node and a node referenced by nothing.)
///
/// This also proves the FreeMonoid decode root fix by execution: the skip decision walks
/// `eval_data_item_value` / `eval_data_initializer_values` over the fixture's `List<Node>`
/// initializers, which decode only under `with_active_context`; without the fix the path
/// errors and the `.expect` in `run_injected_diff_roster` panics.
#[test]
fn node_precise_referenced_runs_orphan_skips_by_execution() {
    let _env = DIFF_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    chdir_workspace();
    let ws = workspace_root();
    let rel = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
    let abs = ws.join(rel);
    let text = std::fs::read_to_string(&abs).expect("discriminator fixture readable");
    let entry = abs.to_string_lossy().into_owned();
    let roster = vec![(
        entry.clone(),
        "floor_disc_witness_transitive_holds".to_string(),
    )];

    // Edit node C (the transitive witness's claim references it through helper_conj) → RUN.
    let c_line = fixture_line(&text, "^floor_disc_node_c_symbol");
    let run = run_injected_diff_roster(rel, c_line, &roster);
    assert_eq!(run.total, 1);
    assert!(
        run.failures.is_empty(),
        "referenced-node edit failures: {:?}",
        run.failures
    );
    assert_eq!(run.skipped, 0, "editing a referenced node must NOT skip");
    assert_eq!(
        run.passed, 1,
        "editing a referenced node must RUN the witness"
    );

    // Edit the orphan node (no claim references it), SAME file → SKIP.
    let orphan_line = fixture_line(&text, "^floor_disc_orphan_symbol");
    let skip = run_injected_diff_roster(rel, orphan_line, &roster);
    assert_eq!(skip.total, 1);
    assert!(
        skip.failures.is_empty(),
        "orphan-node edit failures: {:?}",
        skip.failures
    );
    assert_eq!(
        skip.passed, 0,
        "editing an orphan node must NOT run the witness"
    );
    assert_eq!(
        skip.skipped, 1,
        "editing an orphan node must SKIP (node precision; a file-level impl re-runs it)"
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

/// §5 finding-1 receipt (cursor/composer-2.5): editing a witness `test fn`'s OWN body forces
/// that witness to run even though no `data` node it reads changed; editing a DIFFERENT
/// witness's body does not (per-function precision, not file-level). Exercises
/// `edited_test_fns` — the hole the A/B `data`-line fixtures never touched.
#[test]
fn node_precise_test_fn_body_edit_runs_only_that_witness() {
    let _env = DIFF_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    chdir_workspace();
    let ws = workspace_root();
    let rel = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
    let abs = ws.join(rel);
    let text = std::fs::read_to_string(&abs).expect("discriminator fixture readable");
    let entry = abs.to_string_lossy().into_owned();
    let roster = vec![(entry.clone(), "floor_disc_witness_a_only_holds".to_string())];

    // Edit witness A's OWN body → witness A runs (no `data` node it reads changed).
    let a_fn_line = fixture_line(&text, "test fn floor_disc_witness_a_only_holds");
    let a = run_injected_diff_roster(rel, a_fn_line, &roster);
    assert_eq!(a.total, 1);
    assert!(
        a.failures.is_empty(),
        "witness-A body edit failures: {:?}",
        a.failures
    );
    assert_eq!(
        a.skipped, 0,
        "editing witness A's own body must NOT skip it"
    );
    assert_eq!(
        a.passed, 1,
        "editing witness A's own body must RUN it (finding 1)"
    );

    // Edit a DIFFERENT witness's body → witness A skips (per-function, not file-level run-all).
    let b_fn_line = fixture_line(&text, "test fn floor_disc_witness_b_only_holds");
    let b = run_injected_diff_roster(rel, b_fn_line, &roster);
    assert_eq!(b.total, 1);
    assert!(
        b.failures.is_empty(),
        "witness-B body edit failures: {:?}",
        b.failures
    );
    assert_eq!(
        b.passed, 0,
        "editing witness B's body must NOT run witness A"
    );
    assert_eq!(
        b.skipped, 1,
        "editing witness B's body must SKIP witness A (per-function precision)"
    );
}
