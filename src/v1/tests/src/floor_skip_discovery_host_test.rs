use std::path::PathBuf;
use std::sync::Mutex;

use v1_compiler::cli_run::{
    run_discovery_corpus_with_options, DiscoveryCorpusOptions, DiscoverySummary,
};
use v1_compiler::v1_interpreter::ExecutionMode;

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
        ..Default::default()
    }
}

fn run_explicit_roster(skip: bool) -> Result<DiscoverySummary, String> {
    chdir_workspace();
    let _overlay = EnvVarGuard::set("GUNBC_FLOOR_PROVENANCE_OVERLAY", "0");
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
        .join("src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag")
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
        .join("src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag")
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

fn run_injected_diff_roster(
    rel_path: &str,
    line: i64,
    roster: &[(String, String)],
) -> DiscoverySummary {
    let unified = format!("+++ b/{rel_path}\n@@ -{line},0 +{line},1 @@\n");
    let _overlay = EnvVarGuard::set("GUNBC_FLOOR_PROVENANCE_OVERLAY", "0");
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

#[test]
fn entry_file_helper_fn_edit_scopes_runs_to_touched_entry_only() {
    let _env = DIFF_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
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

#[test]
fn import_closure_helper_fn_edit_runs_importer_entry_only() {
    let _env = DIFF_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
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

fn run_injected_diff_roster_live_overlay(
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
    .expect("live provenance overlay skip path must not error")
}

#[test]
fn discovery_corpus_live_provenance_overlay_resolves_entry_root_off_ingest() {
    let _env = DIFF_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    chdir_workspace();
    let ws = workspace_root();
    let poisoner_rel = "src/v2/test/fixture/program_assembly/pa_ingest_subject.dag";
    let poisoner_abs = ws.join(poisoner_rel);
    let text = std::fs::read_to_string(&poisoner_abs).expect("pa_ingest_subject readable");
    let fn_line = fixture_line(&text, "fn add");
    let peer = ws
        .join("src/v2/test/fixture/program_assembly/pa_ingest_peer_test.dag")
        .to_string_lossy()
        .into_owned();
    let roster = vec![(peer, "pa_ingest_peer_witness_holds".to_string())];

    let summary = run_injected_diff_roster_live_overlay(poisoner_rel, fn_line, &roster);
    assert_eq!(summary.total, 1);
    assert!(
        summary.failures.is_empty(),
        "live provenance overlay must resolve witness entry root without ingest membership: {:?}",
        summary.failures
    );
    assert_eq!(
        summary.skipped, 1,
        "unrelated .dag edit must skip witness via live provenance (entry_root off-ingest; \
         ingest membership lookup would fail-closed and run)"
    );
    assert_eq!(
        summary.passed, 0,
        "live provenance path must not run witness when frontier misses"
    );
}

#[test]
fn frontier_warmup_does_not_poison_corpus_resolution() {
    let _env = DIFF_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    chdir_workspace();
    let ws = workspace_root();
    let poisoner_rel = "src/v2/workflow/ci_floor_plan.dag";
    let poisoner_abs = ws.join(poisoner_rel);
    let text = std::fs::read_to_string(&poisoner_abs).expect("ci_floor_plan readable");
    let data_line = fixture_line(&text, "data floor_corpus_node");
    let budget = ws
        .join("src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag")
        .to_string_lossy()
        .into_owned();
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
