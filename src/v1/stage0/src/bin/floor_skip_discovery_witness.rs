#![allow(clippy::disallowed_macros)]

use std::path::PathBuf;
use std::process::ExitCode;

use v1_compiler::cli_run::{
    build_multi_entry_index, resolve_entry_graph_shared, resolve_entry_with_index,
    run_discovery_corpus_with_options, workspace_root, DiscoveryCorpusOptions, DiscoveryRow,
    DiscoverySummary, DiscoveryWidthPolicy, LiveReadSelectionContext, NodeFrontierSelectionMode,
    SELECTION_CONTROL_BUDGET_ROSTER_REL, SELECTION_CONTROL_CI_FLOOR_PLAN_REL,
    SELECTION_CONTROL_DOC_REACHABILITY_REL, SELECTION_CONTROL_FALSIFIER_CONTROL_REL,
    SELECTION_CONTROL_FLOOR_RUNNER_REL, SELECTION_CONTROL_FLOOR_RUNNER_TEST_REL,
    SELECTION_CONTROL_LIVE_TREE_DECLARED_REL, SELECTION_CONTROL_NODE_PRECISE_REL,
    SELECTION_CONTROL_REALIZATION_SCHEDULE_REL, SELECTION_CONTROL_SHARED_HELPER_REL,
};
use v1_compiler::v1_interpreter::ExecutionMode;

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("floor_skip_discovery_witness: {msg}");
    ExitCode::from(1)
}

fn chdir_workspace() {
    std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
}

fn floor_skip_source_roots() -> Vec<String> {
    let ws = workspace_root();
    vec![
        ws.join("src/v2").to_string_lossy().into_owned(),
        ws.join("dag").to_string_lossy().into_owned(),
    ]
}

fn floor_skip_test_roster() -> (Vec<String>, Vec<(String, String)>) {
    let ws = workspace_root();
    let entry = ws
        .join(SELECTION_CONTROL_FLOOR_RUNNER_TEST_REL)
        .to_string_lossy()
        .into_owned();
    let function = "floor_test_untouched_skips_assumed_green_holds".to_string();
    (floor_skip_source_roots(), vec![(entry, function)])
}

fn discovery_options(skip: bool) -> DiscoveryCorpusOptions {
    DiscoveryCorpusOptions {
        node_frontier_selection: if skip {
            NodeFrontierSelectionMode::Applied
        } else {
            NodeFrontierSelectionMode::Off
        },
        execution_authority_source_roots: floor_skip_source_roots(),
        explicit_roster_only: true,
        ..Default::default()
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
        DiscoveryWidthPolicy::Serial,
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

fn fixture_line(text: &str, needle: &str) -> i64 {
    text.lines()
        .position(|l| l.contains(needle))
        .map(|i| (i + 1) as i64)
        .unwrap_or_else(|| panic!("discriminator fixture missing line containing `{needle}`"))
}

/// The `git diff --name-status -z` record for a single MODIFIED file, coherent with the unified
/// hunk injected on the other channel, in the injection transport's AT-REST encoding: each
/// wire-format NUL separator is written as the octal escape `\000`, because a POSIX environment
/// variable is a C string and cannot carry a raw NUL byte (`std::env::set_var` panics on one —
/// proven by execution 2026-07-10). The injection arm in `floor_diff_observe.dag`
/// (`floor_name_status_injection_encoding_note`) decodes with `printf %b`, so the consumer sees
/// the identical NUL-separated bytes the real-git arm produces and `git.dag`'s
/// `from_code_point(0)` split stays the single parse authority. Without this channel injected,
/// the name-status read falls through to the real checkout diff — a non-hermetic two-channel
/// observation measuring the working tree instead of the fixture.
fn injected_name_status_modify(rel_path: &str) -> String {
    format!("M\\000{rel_path}\\000")
}

fn run_injected_diff_roster(
    rel_path: &str,
    line: i64,
    roster: &[(String, String)],
) -> DiscoverySummary {
    let unified = format!("+++ b/{rel_path}\n@@ -{line},0 +{line},1 @@\n+// synthetic touch\n");
    let _diff = EnvVarGuard::set("GUNBC_CI_DIFF_UNIFIED", &unified);
    let _name_status = EnvVarGuard::set(
        "GUNBC_CI_DIFF_NAME_STATUS",
        &injected_name_status_modify(rel_path),
    );
    run_discovery_corpus_with_options(
        &floor_skip_source_roots(),
        &[],
        roster,
        ExecutionMode::Wet,
        DiscoveryWidthPolicy::Serial,
        discovery_options(true),
    )
    .expect("node-precise skip path must not error (FreeMonoid decode root fix)")
}

fn run_injected_diff_roster_with_mode(
    rel_path: &str,
    line: i64,
    roster: &[(String, String)],
    mode: NodeFrontierSelectionMode,
) -> DiscoverySummary {
    let unified = format!("+++ b/{rel_path}\n@@ -{line},0 +{line},1 @@\n+// synthetic touch\n");
    let _diff = EnvVarGuard::set("GUNBC_CI_DIFF_UNIFIED", &unified);
    let _name_status = EnvVarGuard::set(
        "GUNBC_CI_DIFF_NAME_STATUS",
        &injected_name_status_modify(rel_path),
    );
    run_discovery_corpus_with_options(
        &floor_skip_source_roots(),
        &[],
        roster,
        ExecutionMode::Wet,
        DiscoveryWidthPolicy::Serial,
        DiscoveryCorpusOptions {
            node_frontier_selection: mode,
            execution_authority_source_roots: floor_skip_source_roots(),
            explicit_roster_only: true,
            ..Default::default()
        },
    )
    .expect("predict-only path must not error")
}

fn live_tree_declared_roster() -> Vec<(String, String)> {
    let ws = workspace_root();
    vec![(
        ws.join(SELECTION_CONTROL_LIVE_TREE_DECLARED_REL)
            .to_string_lossy()
            .into_owned(),
        "live_tree_declared_control_holds".to_string(),
    )]
}

fn doc_reachability_roster(function: &str) -> Vec<(String, String)> {
    let ws = workspace_root();
    vec![(
        ws.join(SELECTION_CONTROL_DOC_REACHABILITY_REL)
            .to_string_lossy()
            .into_owned(),
        function.to_string(),
    )]
}

fn declared_live_tree_row_runs_on_unrelated_diff() {
    chdir_workspace();
    // The diff touches an UNRELATED fixture; a declared-ReadsLiveTree row must
    // still RUN (its inputs are outside the diff, so no diff proves it unaffected).
    let summary = run_injected_diff_roster(
        SELECTION_CONTROL_NODE_PRECISE_REL,
        3,
        &live_tree_declared_roster(),
    );
    assert_eq!(summary.total, 1);
    assert_eq!(
        summary.skipped, 0,
        "a declared ReadsLiveTree row must never take the node-frontier skip"
    );
    assert_eq!(summary.passed, 1, "the live-tree row runs and is green");
}

fn declared_live_tree_row_never_predicted_unaffected() {
    chdir_workspace();
    let summary = run_injected_diff_roster_with_mode(
        SELECTION_CONTROL_NODE_PRECISE_REL,
        3,
        &live_tree_declared_roster(),
        NodeFrontierSelectionMode::PredictOnly,
    );
    assert!(
        summary.predicted_unaffected.is_empty(),
        "predict-only must never predict a declared ReadsLiveTree row unaffected \
         (live-tree rows never predict-skip)"
    );
    assert_eq!(summary.passed, 1);
}

/// #7023 discriminating witness: a docs-only diff must RUN the doc-graph wall, not bypass it
/// via the retired `documentation_only_skip` shell shortcut. The entry already declares
/// `ReadsLiveTree`; selection's never-predict-skip lane is the sole authority — no hand edge.
fn doc_reachability_runs_on_docs_only_diff() {
    chdir_workspace();
    let summary = run_injected_diff_roster(
        "docs/plans/synthetic-orphan-doc-reachability-red-control.md",
        1,
        &doc_reachability_roster("doc_graph_has_no_orphan_docs"),
    );
    assert_eq!(summary.total, 1);
    assert_eq!(
        summary.skipped, 0,
        "doc-reachability must RUN on a docs-only diff (ReadsLiveTree never predict-skips; \
         #7023 class: shell documentation_only_skip bypass retired)"
    );
    assert_eq!(summary.passed, 1, "clean tree: orphan wall is green");
}

fn doc_reachability_never_predicted_unaffected_on_docs_only_diff() {
    chdir_workspace();
    let summary = run_injected_diff_roster_with_mode(
        "docs/plans/synthetic-orphan-doc-reachability-red-control.md",
        1,
        &doc_reachability_roster("doc_graph_has_no_orphan_docs"),
        NodeFrontierSelectionMode::PredictOnly,
    );
    assert!(
        summary.predicted_unaffected.is_empty(),
        "predict-only must never predict doc-reachability unaffected on a docs-only diff"
    );
    assert_eq!(summary.passed, 1);
}

fn falsifier_control_roster(function: &str) -> Vec<(String, String)> {
    let ws = workspace_root();
    vec![(
        ws.join(SELECTION_CONTROL_FALSIFIER_CONTROL_REL)
            .to_string_lossy()
            .into_owned(),
        function.to_string(),
    )]
}

fn predict_only_red_predicted_unaffected_is_divergence() {
    chdir_workspace();
    // Diff touches an UNRELATED fixture, so the red control is predicted-unaffected.
    let summary = run_injected_diff_roster_with_mode(
        SELECTION_CONTROL_NODE_PRECISE_REL,
        3,
        &falsifier_control_roster("falsifier_red_control_holds"),
        NodeFrontierSelectionMode::PredictOnly,
    );
    assert_eq!(summary.skipped, 0, "predict-only never applies the skip");
    assert_eq!(
        summary.predicted_unaffected.len(),
        1,
        "the prediction must be recorded per row"
    );
    assert_eq!(
        summary.failures.len(),
        1,
        "the red row still fails the batch"
    );
    assert_eq!(
        summary.divergences.len(),
        1,
        "predicted-unaffected + red = exactly one counted divergence"
    );
}

fn predict_only_green_predicted_unaffected_no_divergence() {
    chdir_workspace();
    let summary = run_injected_diff_roster_with_mode(
        SELECTION_CONTROL_NODE_PRECISE_REL,
        3,
        &falsifier_control_roster("falsifier_green_control_holds"),
        NodeFrontierSelectionMode::PredictOnly,
    );
    assert_eq!(summary.predicted_unaffected.len(), 1);
    assert_eq!(summary.passed, 1, "predicted-unaffected rows still run");
    assert!(
        summary.divergences.is_empty(),
        "a green predicted-unaffected row is a confirmed prediction, not a divergence"
    );
}

fn predict_only_red_predicted_affected_is_not_divergence() {
    chdir_workspace();
    // Diff edits the red control's own declaration line → predicted-AFFECTED → its red
    // is an ordinary failure. Divergence must discriminate, not count every red.
    let ws = workspace_root();
    let text = std::fs::read_to_string(ws.join(SELECTION_CONTROL_FALSIFIER_CONTROL_REL))
        .expect("falsifier control fixture readable");
    let line = fixture_line(&text, "test fn falsifier_red_control_holds");
    let summary = run_injected_diff_roster_with_mode(
        SELECTION_CONTROL_FALSIFIER_CONTROL_REL,
        line,
        &falsifier_control_roster("falsifier_red_control_holds"),
        NodeFrontierSelectionMode::PredictOnly,
    );
    assert!(
        summary.predicted_unaffected.is_empty(),
        "editing the declaration line predicts the row affected"
    );
    assert_eq!(summary.failures.len(), 1, "the red still fails the batch");
    assert!(
        summary.divergences.is_empty(),
        "predicted-affected + red is NOT a divergence — divergence must discriminate"
    );
}

fn budget_roster_completeness_entry(ws: &std::path::Path) -> String {
    ws.join(SELECTION_CONTROL_BUDGET_ROSTER_REL)
        .to_string_lossy()
        .into_owned()
}

// Selection-control step D3 (CI floor endgame, 2026-07-23): this suite paid THREE cold
// whole-pool index builds per run — the two budget_roster cases each called bare
// build_multi_entry_index, then the first corpus case built the thread-shared index —
// which is why the step measured 4m51s in CI against 80s local (the cold-index class,
// attribution doc section 9, under the post-floor slot's swapped-out cgroup). The warmup
// case now resolves through resolve_entry_graph_shared, so its warmup entries AND every
// later corpus case share ONE process index. The cold case below deliberately keeps its
// own bare build — it is the named cold-resolution control (the import-strip Class-B
// pool-coincidence class needs a fresh-index resolution witness), one cold build by design.
//
// REGRESSED, and this note read as resolved while it was not (2026-07-29): the step measured
// 9m02s on run 30482171871 (job 90679506428) — ~1.9x the 4m51s state the D3 fix above was
// written to repair. Accounting from that job log: the assertions cost nothing (every witness
// row reports 0.0-0.3ms), and the wall is 18 serial scenario setups — ~2m15s for the shared
// index build in the warmup case below, ~2m34s for the deliberate cold build after it, ~34s
// total for the eight scenarios that hit the shared index, ~3m30s for the ~7 that land in the
// affected closure and pay ~30s each resolving to decide the node frontier, and 11.2s of
// thread-local teardown at exit. Two of those are addressed in the same change as this line:
// the teardown (see main — removed by construction via process::exit, with the 11.2s being the
// CI-measured cost of the removed path rather than a fresh paired before/after timing, which
// the suite's runtime put outside the local measurement window) and the step's placement, which
// is now affected-set scoped by gunbc.ci_workflow's selection-control step so most PRs do not
// pay any of it — though that scoping is itself a trade, priced in that step's note. The ~30s
// per-resolve unit is NOT explained — it is 4x the ~7.2s/resolve the floor resolve receipt
// records, so it is either a genuinely larger closure or a PROCESS_RESOLVE_STORE miss, and it
// is left as a named measurement rather than a guessed fix.
fn budget_roster_resolves_after_frontier_warmup() {
    chdir_workspace();
    let roots = floor_skip_source_roots();
    for path in [
        SELECTION_CONTROL_REALIZATION_SCHEDULE_REL,
        SELECTION_CONTROL_FLOOR_RUNNER_REL,
        SELECTION_CONTROL_FLOOR_RUNNER_TEST_REL,
    ] {
        let _ = resolve_entry_graph_shared(&roots, path);
    }
    let entry = budget_roster_completeness_entry(&workspace_root());
    resolve_entry_graph_shared(&roots, &entry).expect("budget_roster should resolve");
}

// The ONE deliberate cold build in this suite (see the note above): a fresh index with no
// warmup, proving the budget_roster entry resolves without pool-membership coincidence.
fn budget_roster_resolves_cold() {
    chdir_workspace();
    let roots = floor_skip_source_roots();
    let index = build_multi_entry_index(&roots);
    let entry = budget_roster_completeness_entry(&workspace_root());
    resolve_entry_with_index(&index, &entry).expect("budget_roster should resolve cold");
}

fn discovery_corpus_skip_disabled_runs_without_panic() {
    let summary = run_explicit_roster(false).expect("skip-disabled discovery must not panic");
    assert_eq!(summary.skipped, 0, "skip disabled → no skips");
    assert!(
        summary.passed >= 1,
        "skip-disabled path must run at least one witness"
    );
}

fn discovery_corpus_skip_enabled_empty_diff_skips_unaffected() {
    let _base = EnvVarGuard::set("GUNBC_CI_DIFF_BASE", "HEAD");
    let _head = EnvVarGuard::set("GUNBC_CI_DIFF_HEAD", "HEAD");
    let summary = run_explicit_roster(true).expect("empty diff path must not panic");
    // Ruling 2026-07-05: empty diff is not a state — an empty touched-path set
    // flows through the general disposition, so every unaffected row skips.
    // The old assertion here (skipped == 0, "fail-closed run-all") enshrined
    // the absorbing fallback this contract now forbids.
    assert!(
        summary.skipped >= 1,
        "empty diff = computed ∅ — unaffected rows must skip (got {} skips)",
        summary.skipped
    );
    assert_eq!(
        summary.passed, 0,
        "empty diff must not run unaffected witnesses (run-all is the absorbing arm)"
    );
}

fn discovery_corpus_skip_enabled_git_observation_refuses() {
    let _base = EnvVarGuard::set("GUNBC_CI_DIFF_BASE", "__gunbc_invalid_diff_base__");
    let _head = EnvVarGuard::set("GUNBC_CI_DIFF_HEAD", "HEAD");
    let _merge = EnvVarGuard::set("GUNBC_CI_DIFF_MERGE_BASE", "0");
    // Ruling 2026-07-05: observation failure is the one ignorance state and it
    // REFUSES — a typed, counted error naming its cause — never a silent
    // run-everything or a silent selection-off.
    let err = run_explicit_roster(true)
        .expect_err("git observation failure must refuse, not run or skip");
    assert!(
        err.contains("DiffObservationRefusal"),
        "refusal must name its cause (got: {err})"
    );
}

fn node_precise_referenced_runs_orphan_skips_by_execution() {
    chdir_workspace();
    let ws = workspace_root();
    let rel = SELECTION_CONTROL_NODE_PRECISE_REL;
    let abs = ws.join(rel);
    let text = std::fs::read_to_string(&abs).expect("discriminator fixture readable");
    let entry = abs.to_string_lossy().into_owned();
    let roster = vec![(
        entry.clone(),
        "floor_disc_witness_transitive_holds".to_string(),
    )];

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

fn node_precise_transitive_c_edit_runs_conj_witness() {
    chdir_workspace();
    let ws = workspace_root();
    let rel = SELECTION_CONTROL_NODE_PRECISE_REL;
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

fn node_precise_test_fn_body_edit_runs_only_that_witness() {
    chdir_workspace();
    let ws = workspace_root();
    let rel = SELECTION_CONTROL_NODE_PRECISE_REL;
    let abs = ws.join(rel);
    let text = std::fs::read_to_string(&abs).expect("discriminator fixture readable");
    let entry = abs.to_string_lossy().into_owned();
    let roster = vec![(entry.clone(), "floor_disc_witness_a_only_holds".to_string())];

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

fn entry_file_helper_fn_edit_scopes_runs_to_touched_entry_only() {
    chdir_workspace();
    let ws = workspace_root();
    let disc_rel = SELECTION_CONTROL_NODE_PRECISE_REL;
    let disc_abs = ws.join(disc_rel);
    let text = std::fs::read_to_string(&disc_abs).expect("discriminator fixture readable");
    let helper_line = fixture_line(&text, "fn floor_disc_helper_fn");
    let disc_entry = disc_abs.to_string_lossy().into_owned();
    let runner_entry = ws
        .join(SELECTION_CONTROL_FLOOR_RUNNER_TEST_REL)
        .to_string_lossy()
        .into_owned();
    let roster = vec![
        (
            disc_entry.clone(),
            "floor_disc_witness_a_only_holds".to_string(),
        ),
        (
            runner_entry,
            "floor_test_untouched_skips_assumed_green_holds".to_string(),
        ),
    ];

    let summary = run_injected_diff_roster(disc_rel, helper_line, &roster);
    assert_eq!(summary.total, 2);
    assert!(
        summary.failures.is_empty(),
        "helper-fn edit cross-entry roster failures: {:?}",
        summary.failures
    );
    assert_eq!(
        summary.passed, 1,
        "helper-fn edit must RUN the witness in the touched entry"
    );
    assert_eq!(
        summary.skipped, 1,
        "helper-fn edit must SKIP witnesses in unrelated entries (replaces force_run_all)"
    );
}

fn import_closure_helper_fn_edit_runs_importer_entry_only() {
    chdir_workspace();
    let ws = workspace_root();
    let helper_rel = SELECTION_CONTROL_SHARED_HELPER_REL;
    let helper_abs = ws.join(helper_rel);
    let text = std::fs::read_to_string(&helper_abs).expect("shared helper readable");
    let helper_line = fixture_line(&text, "fn floor_disc_shared_helper");
    let disc_entry = ws
        .join(SELECTION_CONTROL_NODE_PRECISE_REL)
        .to_string_lossy()
        .into_owned();
    let runner_entry = ws
        .join(SELECTION_CONTROL_FLOOR_RUNNER_TEST_REL)
        .to_string_lossy()
        .into_owned();
    let roster = vec![
        (disc_entry, "floor_disc_witness_a_only_holds".to_string()),
        (
            runner_entry,
            "floor_test_untouched_skips_assumed_green_holds".to_string(),
        ),
    ];

    let summary = run_injected_diff_roster(helper_rel, helper_line, &roster);
    assert_eq!(summary.total, 2);
    assert!(
        summary.failures.is_empty(),
        "cross-file helper-fn edit roster failures: {:?}",
        summary.failures
    );
    assert_eq!(
        summary.passed, 1,
        "cross-file helper-fn edit must RUN the witness in the importing entry"
    );
    assert_eq!(
        summary.skipped, 1,
        "cross-file helper-fn edit must SKIP witnesses in unrelated entries"
    );
}

fn frontier_warmup_does_not_poison_corpus_resolution() {
    chdir_workspace();
    let ws = workspace_root();
    let poisoner_rel = SELECTION_CONTROL_CI_FLOOR_PLAN_REL;
    let poisoner_abs = ws.join(poisoner_rel);
    let text = std::fs::read_to_string(&poisoner_abs).expect("ci_floor_plan readable");
    let data_line = fixture_line(&text, "data floor_corpus_node");
    let budget = budget_roster_completeness_entry(&ws);
    let roster = vec![(
        budget,
        "complexity_budget_roster_family_gate_holds".to_string(),
    )];

    let summary = run_injected_diff_roster(poisoner_rel, data_line, &roster);
    assert_eq!(summary.total, 1);
    assert!(
        summary.failures.is_empty(),
        "corpus must resolve cleanly after frontier warmup (no FreeMonoid poison): {:?}",
        summary.failures
    );
    assert_eq!(
        summary.skipped, 1,
        "budget_roster does not reference the edited ci_floor_plan node → skips after a clean resolve"
    );
}

/// The two-entry incident subject, named once so every control below decides the SAME subject.
///
/// A = the node-precise discriminator fixture (the entry the diff touches).
/// B = the affected-set floor runner test (the unrelated entry whose 474-module closure reaches a
/// live-read carrier home). B is what the retired predicate made unskippable on any nonempty
/// `.dag` diff, which is the `2 selected / 1 expected` incident.
fn incident_subject_roster() -> Vec<(String, String)> {
    let ws = workspace_root();
    vec![
        (
            ws.join(SELECTION_CONTROL_NODE_PRECISE_REL)
                .to_string_lossy()
                .into_owned(),
            "floor_disc_witness_a_only_holds".to_string(),
        ),
        (
            ws.join(SELECTION_CONTROL_FLOOR_RUNNER_TEST_REL)
                .to_string_lossy()
                .into_owned(),
            "floor_test_untouched_skips_assumed_green_holds".to_string(),
        ),
    ]
}

/// THE INCIDENT CONTROL, at the grain the incident was measured in.
///
/// `entry_file_helper_fn_edit_scopes_runs_to_touched_entry_only` already asserts the row totals
/// (2/1/1). This asserts the ENTRY-GROUP selection those totals are supposed to come from:
/// exactly one of the two entry groups is selected. The distinction is the whole finding — a
/// summary can report one skipped row while both entry groups were selected and resolved, which
/// is `SelectionSuperset` wearing a passing row count. Under the retired carrier-home predicate
/// this assertion reads 2, which is the `2 selected / 1 expected` line from the incident.
fn live_read_selection_narrows_incident_subject_to_touched_entry() {
    chdir_workspace();
    let ws = workspace_root();
    let disc_rel = SELECTION_CONTROL_NODE_PRECISE_REL;
    let text = std::fs::read_to_string(ws.join(disc_rel)).expect("discriminator fixture readable");
    let helper_line = fixture_line(&text, "fn floor_disc_helper_fn");

    let summary = run_injected_diff_roster(disc_rel, helper_line, &incident_subject_roster());
    assert!(
        summary.failures.is_empty(),
        "incident-subject roster failures: {:?}",
        summary.failures
    );
    assert_eq!(
        summary.total_entry_groups, 2,
        "the incident subject is two entry groups"
    );
    assert_eq!(
        summary.selected_entry_groups, 1,
        "affected-set selection must narrow the incident subject to the touched entry \
         (selected/expected — 2 here is the incident: the unrelated entry's import closure \
         reaches a live-read carrier home, which the retired G1 predicate treated as a hit on \
         any nonempty diff)"
    );

    // The counter above proves B ran no witness. It does NOT prove B's cold entry resolve was
    // elided, because a row skipped as assumed-green AFTER its entry resolved also records no
    // outcome — and eliding that resolve is the entire cost win this axis exists to buy. So
    // assert the provenance: B must have taken the skip-BEFORE-resolve fast path.
    let runner_entry = workspace_root()
        .join(SELECTION_CONTROL_FLOOR_RUNNER_TEST_REL)
        .to_string_lossy()
        .into_owned();
    let b_skip = summary
        .selection_skipped_rows
        .iter()
        .find(|r| r.entry == runner_entry)
        .unwrap_or_else(|| {
            panic!(
                "the unrelated entry must appear as a selection-skipped row; skipped rows: {:?}",
                summary.selection_skipped_rows
            )
        });
    assert_eq!(
        b_skip.provenance, "skip-before-resolve-fast-path",
        "the unrelated entry must skip BEFORE resolving — a resolve-then-skip still pays the \
         cold entry resolve this axis exists to elide, and reports identically in every count"
    );
}

/// The stop-line population must not re-widen what selection narrowed.
///
/// The incident trace carried `stopline_n=1` beside the carrier-home hit, so both facts were
/// present and either could have been the cause. This separates them: the unified diff still
/// touches only A, while the name-status population additionally names two floor_skip fixtures
/// that are not in B's closure. B must still skip. If a later change routes the stop-line
/// population into the runtime-dependency axis, this reds while the narrowing control above
/// stays green, so the two axes report separately rather than as one aggregate.
fn stop_line_population_does_not_widen_incident_subject() {
    chdir_workspace();
    let ws = workspace_root();
    let disc_rel = SELECTION_CONTROL_NODE_PRECISE_REL;
    let text = std::fs::read_to_string(ws.join(disc_rel)).expect("discriminator fixture readable");
    let helper_line = fixture_line(&text, "fn floor_disc_helper_fn");

    let unified =
        format!("+++ b/{disc_rel}\n@@ -{helper_line},0 +{helper_line},1 @@\n+// synthetic touch\n");
    // The separator is the literal four-character sequence `\000`, not a NUL byte —
    // `floor_diff_observe` splits the injected value on the string "\\000". An actual NUL here
    // decodes as one unsplit path and the control would exercise a population it does not name.
    let name_status = format!(
        "M\\000{disc_rel}\\000M\\000{}\\000M\\000{}\\000",
        SELECTION_CONTROL_LIVE_TREE_DECLARED_REL, SELECTION_CONTROL_FALSIFIER_CONTROL_REL,
    );
    let _diff = EnvVarGuard::set("GUNBC_CI_DIFF_UNIFIED", &unified);
    let _ns = EnvVarGuard::set("GUNBC_CI_DIFF_NAME_STATUS", &name_status);

    let summary = run_discovery_corpus_with_options(
        &floor_skip_source_roots(),
        &[],
        &incident_subject_roster(),
        ExecutionMode::Wet,
        DiscoveryWidthPolicy::Serial,
        discovery_options(true),
    )
    .expect("stop-line population must not error");
    assert!(
        summary.failures.is_empty(),
        "stop-line population roster failures: {:?}",
        summary.failures
    );
    assert_eq!(summary.total_entry_groups, 2);
    assert_eq!(
        summary.selected_entry_groups, 1,
        "a wider stop-line population must not re-select the unrelated entry — axis (iv) is \
         decided by classified runtime reads, not by how many paths the diff names"
    );
}

/// A missing classification REFUSES; it never licenses a skip.
///
/// Executed rather than argued, because the failure mode this guards is precisely the one that
/// looks green: an entry whose classification is unavailable answering "no runtime read touched"
/// is the under-selection direction, which presents as a faster floor rather than as a failure.
fn live_read_missing_classification_refuses() {
    chdir_workspace();
    let ws = workspace_root();
    let roots = floor_skip_source_roots();
    let index = build_multi_entry_index(&roots);
    let disc_entry = ws
        .join(SELECTION_CONTROL_NODE_PRECISE_REL)
        .to_string_lossy()
        .into_owned();
    let runner_entry = ws
        .join(SELECTION_CONTROL_FLOOR_RUNNER_TEST_REL)
        .to_string_lossy()
        .into_owned();

    // A context built from a roster that names only A.
    let roster = vec![DiscoveryRow {
        label: "a".to_string(),
        entry: disc_entry.clone(),
        function: "floor_disc_witness_a_only_holds".to_string(),
        reads_live_tree: false,
    }];
    let live = LiveReadSelectionContext::build(&index, &roster)
        .expect("context builds over its own complete roster");

    let touched = vec![SELECTION_CONTROL_NODE_PRECISE_REL.to_string()];
    // POSITIVE CONTROL: the enrolled entry answers rather than refusing, so the refusal below is
    // a fact about the missing entry and not about the context being broken.
    live.runtime_dependency_touched_for_entry(&index, &disc_entry, &touched)
        .expect("an enrolled entry is decidable");

    // THE REFUSAL: B is outside the roster the context was built from.
    let err = live
        .runtime_dependency_touched_for_entry(&index, &runner_entry, &touched)
        .expect_err("an entry absent from the context must refuse, never answer 'untouched'");
    assert!(
        err.contains("LiveReadEntryAbsent"),
        "the refusal must name its cause, not merely be an error: {err}"
    );
}

/// THE DEFECT CONTROL: a declaration OUTSIDE the classification lens's own import closure must
/// classify, not refuse.
///
/// This is the control the branch was missing, and its absence is why an earlier head claimed a
/// narrowing it did not have. `fn_arrow_decl_facts_live()` reflects the eval context's modules, so
/// when the manifest producer resolved the lens entry alone, the declaration population was the
/// LENS'S closure. Every enrolled witness outside it failed to bind, every row was a modelled
/// refusal, and consuming that manifest would have run the whole corpus on any nonempty diff —
/// strictly more selection than the predicate the lane replaces.
///
/// The subject is deliberately a floor_skip fixture: it is a real enrolled witness entry and it is
/// not imported by `v2.lens.live_read_classification`, so a classification for it can only come
/// from a fact universe wider than the lens closure. Mutating the producer back to a lens-entry
/// context makes this red — that mutation is the receipt that this control discriminates.
fn live_read_classifies_a_declaration_outside_the_lens_closure() {
    chdir_workspace();
    let ws = workspace_root();
    let roots = floor_skip_source_roots();
    let index = build_multi_entry_index(&roots);
    let disc_entry = ws
        .join(SELECTION_CONTROL_NODE_PRECISE_REL)
        .to_string_lossy()
        .into_owned();
    let roster = vec![DiscoveryRow {
        label: "outside".to_string(),
        entry: disc_entry.clone(),
        function: "floor_disc_witness_a_only_holds".to_string(),
        reads_live_tree: false,
    }];
    let live = LiveReadSelectionContext::build(&index, &roster).expect("manifest builds");

    // An UNRELATED touched path. The declaration is a local read, so a working classification
    // answers `false` — and answering at all is the point: an unbound root would have returned a
    // typed refusal from the modelled-refusal arm instead.
    let unrelated = vec![SELECTION_CONTROL_BUDGET_ROSTER_REL.to_string()];
    let touched = live
        .runtime_dependency_touched_for_entry(&index, &disc_entry, &unrelated)
        .expect(
            "a declaration outside the lens closure must CLASSIFY — a refusal here means the \
             fact universe is narrower than the index the consumer decides over, which widens \
             selection to the whole corpus while reporting green",
        );
    assert!(
        !touched,
        "a local-read declaration must not report the unrelated diff as touching a runtime read"
    );
}

/// A PRESENT row carrying a modelled refusal must reach the consumer as a typed `Err`.
///
/// This is a different arm from `live_read_missing_classification_refuses`, and the difference is
/// the one that matters. That control removes the entry from the context, so the refusal comes
/// from the ROSTER lookup — it proves nothing about what happens when the manifest DOES carry a
/// row and that row is `LiveReadSelectionRefused`. `touched_by` answers `true` for such a row,
/// which is conservative for a path-intersection question but wrong for a consumer to act on:
/// routed through the boolean, an undecidable classification is indistinguishable from a decided
/// "this diff touches a runtime read", and a producer that cannot classify anything presents as
/// `SelectionSuperset` — everything ran, slow but apparently valid — rather than as "the manifest
/// cannot decide".
///
/// The request names a function that does not exist in its entry, so `bind_g2_root` answers
/// `G2RootUnbound` and the producer emits a row that is present and refused. The consumer must
/// stop the line on it.
fn live_read_present_but_refused_row_propagates_refusal() {
    chdir_workspace();
    let ws = workspace_root();
    let roots = floor_skip_source_roots();
    let index = build_multi_entry_index(&roots);
    let disc_entry = ws
        .join(SELECTION_CONTROL_NODE_PRECISE_REL)
        .to_string_lossy()
        .into_owned();

    let roster = vec![
        DiscoveryRow {
            label: "real".to_string(),
            entry: disc_entry.clone(),
            function: "floor_disc_witness_a_only_holds".to_string(),
            reads_live_tree: false,
        },
        DiscoveryRow {
            label: "unbindable".to_string(),
            entry: disc_entry.clone(),
            // No such declaration exists in this entry, so the root cannot bind and the producer
            // emits a PRESENT row carrying LiveReadSelectionRefused.
            function: "floor_disc_witness_no_such_declaration_exists".to_string(),
            reads_live_tree: false,
        },
    ];
    let live = LiveReadSelectionContext::build(&index, &roster)
        .expect("the manifest builds; an unbindable root is a refused ROW, not a build failure");

    let touched = vec![SELECTION_CONTROL_NODE_PRECISE_REL.to_string()];
    let err = live
        .runtime_dependency_touched_for_entry(&index, &disc_entry, &touched)
        .expect_err(
            "a present-but-refused row must stop the line, not answer `true` and read as a \
             decided runtime-read touch",
        );
    assert!(
        err.contains("LiveReadClassificationRefused"),
        "the refusal must name the modelled-refusal cause so an incomplete producer is counted \
         rather than absorbed into a superset run: {err}"
    );
}

/// A manifest may not be attributed to an index it was not built against.
///
/// The subject check is what makes the memo checkable rather than merely fast, so it needs an
/// executed control: a second index over byte-identical roots is a different subject, and serving
/// it the first index's classifications would license a skip derived from another tree.
fn live_read_subject_mismatch_refuses() {
    chdir_workspace();
    let ws = workspace_root();
    let roots = floor_skip_source_roots();
    let first = build_multi_entry_index(&roots);
    let second = build_multi_entry_index(&roots);
    let disc_entry = ws
        .join(SELECTION_CONTROL_NODE_PRECISE_REL)
        .to_string_lossy()
        .into_owned();
    let roster = vec![DiscoveryRow {
        label: "a".to_string(),
        entry: disc_entry.clone(),
        function: "floor_disc_witness_a_only_holds".to_string(),
        reads_live_tree: false,
    }];
    let live = LiveReadSelectionContext::build(&first, &roster).expect("context builds");
    let touched = vec![SELECTION_CONTROL_NODE_PRECISE_REL.to_string()];

    live.runtime_dependency_touched_for_entry(&first, &disc_entry, &touched)
        .expect("its own index decides");
    let err = live
        .runtime_dependency_touched_for_entry(&second, &disc_entry, &touched)
        .expect_err("a different index must refuse, never be served another tree's rows");
    assert!(
        err.contains("LiveReadContextSubjectMismatch"),
        "the refusal must name its cause: {err}"
    );
}

fn main() -> ExitCode {
    let _workspace_marker: PathBuf = workspace_root();

    let tests: Vec<(&str, fn())> = vec![
        (
            "budget_roster_resolves_after_frontier_warmup",
            budget_roster_resolves_after_frontier_warmup,
        ),
        ("budget_roster_resolves_cold", budget_roster_resolves_cold),
        (
            "discovery_corpus_skip_disabled_runs_without_panic",
            discovery_corpus_skip_disabled_runs_without_panic,
        ),
        (
            "discovery_corpus_skip_enabled_empty_diff_skips_unaffected",
            discovery_corpus_skip_enabled_empty_diff_skips_unaffected,
        ),
        (
            "discovery_corpus_skip_enabled_git_observation_refuses",
            discovery_corpus_skip_enabled_git_observation_refuses,
        ),
        (
            "predict_only_red_predicted_unaffected_is_divergence",
            predict_only_red_predicted_unaffected_is_divergence,
        ),
        (
            "predict_only_green_predicted_unaffected_no_divergence",
            predict_only_green_predicted_unaffected_no_divergence,
        ),
        (
            "predict_only_red_predicted_affected_is_not_divergence",
            predict_only_red_predicted_affected_is_not_divergence,
        ),
        (
            "declared_live_tree_row_runs_on_unrelated_diff",
            declared_live_tree_row_runs_on_unrelated_diff,
        ),
        (
            "declared_live_tree_row_never_predicted_unaffected",
            declared_live_tree_row_never_predicted_unaffected,
        ),
        (
            "doc_reachability_runs_on_docs_only_diff",
            doc_reachability_runs_on_docs_only_diff,
        ),
        (
            "doc_reachability_never_predicted_unaffected_on_docs_only_diff",
            doc_reachability_never_predicted_unaffected_on_docs_only_diff,
        ),
        (
            "node_precise_referenced_runs_orphan_skips_by_execution",
            node_precise_referenced_runs_orphan_skips_by_execution,
        ),
        (
            "node_precise_transitive_c_edit_runs_conj_witness",
            node_precise_transitive_c_edit_runs_conj_witness,
        ),
        (
            "node_precise_test_fn_body_edit_runs_only_that_witness",
            node_precise_test_fn_body_edit_runs_only_that_witness,
        ),
        (
            "entry_file_helper_fn_edit_scopes_runs_to_touched_entry_only",
            entry_file_helper_fn_edit_scopes_runs_to_touched_entry_only,
        ),
        (
            "import_closure_helper_fn_edit_runs_importer_entry_only",
            import_closure_helper_fn_edit_runs_importer_entry_only,
        ),
        (
            "frontier_warmup_does_not_poison_corpus_resolution",
            frontier_warmup_does_not_poison_corpus_resolution,
        ),
        (
            "live_read_selection_narrows_incident_subject_to_touched_entry",
            live_read_selection_narrows_incident_subject_to_touched_entry,
        ),
        (
            "stop_line_population_does_not_widen_incident_subject",
            stop_line_population_does_not_widen_incident_subject,
        ),
        (
            "live_read_missing_classification_refuses",
            live_read_missing_classification_refuses,
        ),
        (
            "live_read_subject_mismatch_refuses",
            live_read_subject_mismatch_refuses,
        ),
        (
            "live_read_present_but_refused_row_propagates_refusal",
            live_read_present_but_refused_row_propagates_refusal,
        ),
        (
            "live_read_classifies_a_declaration_outside_the_lens_closure",
            live_read_classifies_a_declaration_outside_the_lens_closure,
        ),
    ];

    for (name, test) in tests {
        let result = std::panic::catch_unwind(test);
        if result.is_err() {
            return fail(format!("{name} panicked"));
        }
    }

    // Exit WITHOUT running thread-local destructors. `cli_run`'s PROCESS_RESOLVE_INDEX
    // (`Rc<MultiEntryIndex>`, the whole-pool index) and PROCESS_RESOLVE_STORE (one
    // `Rc<ResolvedGraph>` per scenario resolved above) are thread-locals, and thread-locals
    // ARE dropped when the main thread ends — so returning normally spent 11.2s walking
    // Rc/HashMap destructors for a whole-corpus graph the process is about to abandon
    // (measured: run 30482171871 job 90679506428, last receipt line 19:30:55.34, step
    // completion 19:31:06.53, nothing logged in between). The OS reclaims the address space,
    // so that teardown buys nothing.
    //
    // Safe because no destructor here is load-bearing: every receipt this suite emits is
    // printed eagerly during the scenarios (visible in the CI log before the gap), the
    // scenarios are each wrapped in `catch_unwind` above so a panic is already handled, and
    // the process holds no buffers other than stdout/stderr — flushed explicitly first,
    // since `process::exit` does not run libstd's at-exit flush.
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let _ = std::io::Write::flush(&mut std::io::stderr());
    std::process::exit(0);
}
