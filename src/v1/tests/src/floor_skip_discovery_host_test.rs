//! §5 execution receipts for Phase 1.5a REDO floor skip host transport (`cli_run`).
//!
//! Proves skip-disabled path runs, skip-enabled with empty/non-applicable diff runs all,
//! git diff observation failure fail-closes to running witnesses, and same-file node-precision
//! discrimination (edit node A → run, edit node B → skip).

use std::path::PathBuf;

use v1_compiler::cli_run::{
    build_multi_entry_index, floor_skip_entry_touches_unified_diff, resolve_entry_with_index,
    run_discovery_corpus_with_options, DiscoveryCorpusOptions, DiscoverySummary,
};
use v1_compiler::v1_compiler_infer_items::{item_kind, ItemKind};
use v1_compiler::v1_interpreter::ExecutionMode;
use v1_compiler::v1_std_core::{authored_name_at, byte_to_line_col};

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

const FLOOR_SKIP_PRECISION_FIXTURE: &str = "dsl/test/claim/floor_skip_node_precision_fixture.dag";

fn floor_skip_source_roots() -> Vec<String> {
    let ws = workspace_root();
    vec![
        ws.join("src/v2").to_string_lossy().into_owned(),
        ws.join("dsl").to_string_lossy().into_owned(),
    ]
}

fn fixture_entry_abs() -> String {
    workspace_root()
        .join(FLOOR_SKIP_PRECISION_FIXTURE)
        .to_string_lossy()
        .into_owned()
}

fn data_decl_mid_line(data_name: &str) -> i64 {
    let roots = floor_skip_source_roots();
    let index = build_multi_entry_index(&roots);
    let entry_abs = fixture_entry_abs();
    let (graph, si) = resolve_entry_with_index(&index, &entry_abs)
        .unwrap_or_else(|e| panic!("resolve {entry_abs}: {e}"));
    for module in graph.modules.iter() {
        for item in module.items.iter() {
            if item_kind(item.clone()) != ItemKind::DataItem {
                continue;
            }
            let name = authored_name_at(si.clone(), item.clone());
            if name != data_name {
                continue;
            }
            let span = &*item.span;
            let file_key = span.file.clone();
            let nl = si
                .get(&file_key)
                .or_else(|| {
                    si.iter()
                        .find(|(path, _)| {
                            path.ends_with(&file_key) || file_key.ends_with(path.as_str())
                        })
                        .map(|(_, idx)| idx)
                })
                .unwrap_or_else(|| panic!("newline index missing for {}", span.file));
            let start = byte_to_line_col(nl.clone(), span.start).line;
            let end = byte_to_line_col(nl.clone(), span.end).line;
            return (start + end) / 2;
        }
    }
    panic!("data item {data_name} not found in {entry_abs}");
}

fn unified_diff_hunk_for_line(file: &str, line: i64) -> String {
    format!(
        "\
diff --git a/{file} b/{file}
--- a/{file}
+++ b/{file}
@@ -{line},1 +{line},1 @@
 x
"
    )
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

/// §5 discriminating witness: same file, independent nodes A/B; witness touches A only.
/// File-level skip would run the witness for both edits; node-precise skip must not.
#[test]
fn floor_skip_same_file_node_precision_holds() {
    let roots = floor_skip_source_roots();
    let entry = fixture_entry_abs();
    let line_a = data_decl_mid_line("floor_skip_precision_node_a");
    let line_b = data_decl_mid_line("floor_skip_precision_node_b");
    assert_ne!(
        line_a, line_b,
        "fixture must place A and B on distinct declaration lines"
    );

    let diff_a = unified_diff_hunk_for_line(FLOOR_SKIP_PRECISION_FIXTURE, line_a);
    let touches_a = floor_skip_entry_touches_unified_diff(
        &roots,
        &entry,
        &diff_a,
        ExecutionMode::Wet,
    )
    .expect("node A edit must produce a frontier")
    .expect("node A edit must not fail-closed to run-all");
    assert!(
        touches_a,
        "edit inside node A span → witness entry must RUN (touch frontier)"
    );

    let diff_b = unified_diff_hunk_for_line(FLOOR_SKIP_PRECISION_FIXTURE, line_b);
    let touches_b = floor_skip_entry_touches_unified_diff(
        &roots,
        &entry,
        &diff_b,
        ExecutionMode::Wet,
    )
    .expect("node B edit must produce a frontier")
    .expect("node B edit must not fail-closed to run-all");
    assert!(
        !touches_b,
        "edit inside node B span only → witness entry must SKIP (assumed-green)"
    );
}
