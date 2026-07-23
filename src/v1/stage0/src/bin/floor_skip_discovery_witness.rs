#![allow(clippy::disallowed_macros)]

use std::path::PathBuf;
use std::process::ExitCode;

use v1_compiler::cli_run::{
    build_multi_entry_index, resolve_entry_with_index, run_discovery_corpus_with_options,
    workspace_root, DiscoveryCorpusOptions, DiscoverySummary, DiscoveryWidthPolicy,
    NodeFrontierSelectionMode,
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
        .join("src/v2/workflow/affected_set_floor_runner_test.dag")
        .to_string_lossy()
        .into_owned();
    let function = "floor_runner_node_frontier_policy_holds".to_string();
    (floor_skip_source_roots(), vec![(entry, function)])
}

fn discovery_options(skip: bool) -> DiscoveryCorpusOptions {
    DiscoveryCorpusOptions {
        node_frontier_selection: if skip {
            NodeFrontierSelectionMode::Applied
        } else {
            NodeFrontierSelectionMode::Off
        },
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
            explicit_roster_only: true,
            ..Default::default()
        },
    )
    .expect("predict-only path must not error")
}

const FALSIFIER_CONTROL_REL: &str =
    "src/v2/test/fixture/floor_skip/falsifier_divergence_control_test.dag";

const LIVE_TREE_DECLARED_REL: &str = "src/v2/test/fixture/floor_skip/live_tree_declared_test.dag";

const DOC_REACHABILITY_WITNESS_REL: &str = "dag/test/claim/doc_reachability_witness_test.dag";

fn live_tree_declared_roster() -> Vec<(String, String)> {
    let ws = workspace_root();
    vec![(
        ws.join(LIVE_TREE_DECLARED_REL)
            .to_string_lossy()
            .into_owned(),
        "live_tree_declared_control_holds".to_string(),
    )]
}

fn doc_reachability_roster(function: &str) -> Vec<(String, String)> {
    let ws = workspace_root();
    vec![(
        ws.join(DOC_REACHABILITY_WITNESS_REL)
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
        "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag",
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
        "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag",
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
        ws.join(FALSIFIER_CONTROL_REL)
            .to_string_lossy()
            .into_owned(),
        function.to_string(),
    )]
}

fn predict_only_red_predicted_unaffected_is_divergence() {
    chdir_workspace();
    // Diff touches an UNRELATED fixture, so the red control is predicted-unaffected.
    let summary = run_injected_diff_roster_with_mode(
        "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag",
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
        "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag",
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
    let text = std::fs::read_to_string(ws.join(FALSIFIER_CONTROL_REL))
        .expect("falsifier control fixture readable");
    let line = fixture_line(&text, "test fn falsifier_red_control_holds");
    let summary = run_injected_diff_roster_with_mode(
        FALSIFIER_CONTROL_REL,
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
    ws.join("src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag")
        .to_string_lossy()
        .into_owned()
}

fn budget_roster_resolves_after_frontier_warmup() {
    chdir_workspace();
    let roots = floor_skip_source_roots();
    let index = build_multi_entry_index(&roots);
    for path in [
        "dag/std/realization_schedule.dag",
        "src/v2/workflow/affected_set_floor_runner.dag",
        "src/v2/workflow/affected_set_floor_runner_test.dag",
    ] {
        let _ = resolve_entry_with_index(&index, path);
    }
    let entry = budget_roster_completeness_entry(&workspace_root());
    resolve_entry_with_index(&index, &entry).expect("budget_roster should resolve");
}

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
    let rel = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
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

fn node_precise_test_fn_body_edit_runs_only_that_witness() {
    chdir_workspace();
    let ws = workspace_root();
    let rel = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
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
    let disc_rel = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
    let disc_abs = ws.join(disc_rel);
    let text = std::fs::read_to_string(&disc_abs).expect("discriminator fixture readable");
    let helper_line = fixture_line(&text, "fn floor_disc_helper_fn");
    let disc_entry = disc_abs.to_string_lossy().into_owned();
    let runner_entry = ws
        .join("src/v2/workflow/affected_set_floor_runner_test.dag")
        .to_string_lossy()
        .into_owned();
    let roster = vec![
        (
            disc_entry.clone(),
            "floor_disc_witness_a_only_holds".to_string(),
        ),
        (
            runner_entry,
            "floor_runner_node_frontier_policy_holds".to_string(),
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
    let helper_rel = "src/v2/test/fixture/floor_skip/floor_disc_shared_helper.dag";
    let helper_abs = ws.join(helper_rel);
    let text = std::fs::read_to_string(&helper_abs).expect("shared helper readable");
    let helper_line = fixture_line(&text, "fn floor_disc_shared_helper");
    let disc_entry = ws
        .join("src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag")
        .to_string_lossy()
        .into_owned();
    let runner_entry = ws
        .join("src/v2/workflow/affected_set_floor_runner_test.dag")
        .to_string_lossy()
        .into_owned();
    let roster = vec![
        (disc_entry, "floor_disc_witness_a_only_holds".to_string()),
        (
            runner_entry,
            "floor_runner_node_frontier_policy_holds".to_string(),
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
    let poisoner_rel = "src/v2/workflow/ci_floor_plan.dag";
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
    ];

    for (name, test) in tests {
        let result = std::panic::catch_unwind(test);
        if result.is_err() {
            return fail(format!("{name} panicked"));
        }
    }

    ExitCode::SUCCESS
}
