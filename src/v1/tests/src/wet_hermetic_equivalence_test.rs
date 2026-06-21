//! §6 SCAFFOLD — P3c wet/hermetic equivalence gate (comparator + vacuous roster).
//!
//! **Dissolution-on-arrival:** gains real faithfulness teeth when the first
//! live-transport + published-mock witness lands (wet = live call, hermetic =
//! published mock). Until then the `lens_mock_totality/*` roster makes no
//! `eval_service_call` dispatch under either mode, so the roster integration
//! check is vacuous-by-construction — it cannot observe mock-vs-real divergence.
//!
//! **Comparator teeth (unit):** `wet_hermetic_discovery_outcome_divergences`
//! is tested with synthetic outcome vectors so divergence reporting is RED when
//! the comparator itself regresses, independent of the roster.

use v1_compiler::cli_run::{
    discover_floor_corpus_rows, is_governed_service_representative_row, run_discovery_corpus,
    wet_hermetic_discovery_outcome_divergences, wet_hermetic_scaffold_roster_entry_prefix,
    ClaimOutcome, DiscoveryRow, DiscoveryWitnessOutcome,
};
use v1_compiler::v1_interpreter::ExecutionMode;

use crate::helpers::workspace_root;

fn sample_outcome(entry: &str, function: &str, outcome: ClaimOutcome) -> DiscoveryWitnessOutcome {
    DiscoveryWitnessOutcome {
        entry: entry.to_string(),
        function: function.to_string(),
        outcome,
    }
}

#[test]
fn wet_hermetic_comparator_reports_outcome_divergence() {
    let wet = [sample_outcome(
        "dsl/test/claim/example_witness_test.dag",
        "witness_holds",
        ClaimOutcome::Pass,
    )];
    let hermetic = [sample_outcome(
        "dsl/test/claim/example_witness_test.dag",
        "witness_holds",
        ClaimOutcome::Fail,
    )];
    let divergences = wet_hermetic_discovery_outcome_divergences(&wet, &hermetic);
    assert_eq!(
        divergences.len(),
        1,
        "comparator must report Pass vs Fail on the same witness row"
    );
    assert!(
        divergences[0].contains("witness_holds"),
        "divergence line must name the witness: {}",
        divergences[0]
    );
}

#[test]
fn wet_hermetic_comparator_empty_when_outcomes_match() {
    let wet = [
        sample_outcome("a.dag", "one", ClaimOutcome::Pass),
        sample_outcome("b.dag", "two", ClaimOutcome::Fail),
    ];
    let hermetic = [
        sample_outcome("a.dag", "one", ClaimOutcome::Pass),
        sample_outcome("b.dag", "two", ClaimOutcome::Fail),
    ];
    let divergences = wet_hermetic_discovery_outcome_divergences(&wet, &hermetic);
    assert!(
        divergences.is_empty(),
        "matching outcome vectors must yield no divergences: {:?}",
        divergences
    );
}

fn ci_witness_layer_roots() -> Vec<String> {
    let ws = workspace_root();
    vec![
        ws.join("dsl").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ]
}

fn ci_witness_scan_dirs() -> Vec<String> {
    let ws = workspace_root();
    vec![
        ws.join("dsl/test/claim").to_string_lossy().into_owned(),
        ws.join("src/v2/compiler/manual")
            .to_string_lossy()
            .into_owned(),
    ]
}

fn governed_service_representative_explicit_entries() -> Vec<(String, String)> {
    let roots = ci_witness_layer_roots();
    let prefix = wet_hermetic_scaffold_roster_entry_prefix(&roots)
        .expect("load scaffold roster prefix from witness .dag authority");
    let scan_dirs = ci_witness_scan_dirs();
    let rows = discover_floor_corpus_rows(&roots, &scan_dirs)
        .expect("discover floor corpus for governed-service representative roster");
    let rep: Vec<(String, String)> = rows
        .iter()
        .filter(|r| is_governed_service_representative_row(r, &prefix))
        .map(|r| (r.entry.clone(), r.function.clone()))
        .collect();
    assert!(
        !rep.is_empty(),
        "governed-service representative roster must be non-empty (lens_mock_totality witnesses)"
    );
    rep
}

#[test]
fn wet_hermetic_scaffold_roster_filter_uses_dag_prefix_authority() {
    let roots = ci_witness_layer_roots();
    let prefix = wet_hermetic_scaffold_roster_entry_prefix(&roots)
        .expect("load scaffold roster prefix from witness .dag authority");
    assert!(
        prefix.contains("lens_mock_totality/"),
        "dag authority prefix must select lens_mock_totality tree: {prefix}"
    );
    let scan_dirs = ci_witness_scan_dirs();
    let rows = discover_floor_corpus_rows(&roots, &scan_dirs)
        .expect("discover floor corpus for prefix authority check");
    let rep: Vec<&DiscoveryRow> = rows
        .iter()
        .filter(|r| is_governed_service_representative_row(r, &prefix))
        .collect();
    assert!(
        !rep.is_empty(),
        "dag authority prefix must match at least one discovered witness row"
    );
    for row in &rep {
        assert!(
            row.entry.contains(&prefix),
            "filtered row must contain dag authority prefix (got entry={} prefix={prefix})",
            row.entry
        );
    }
    let outsider = DiscoveryRow {
        label: "outsider".into(),
        entry: "dsl/test/claim/unrelated_witness_test.dag".into(),
        function: "witness_holds".into(),
    };
    assert!(
        !is_governed_service_representative_row(&outsider, &prefix),
        "rows outside the dag prefix must not match the scaffold filter"
    );
}

/// Vacuous-by-construction scaffold: lens_mock_totality witnesses do not
/// dispatch live service calls under either mode; this only guards roster wiring.
#[ignore = "failing: wet_hermetic whole-tree mock-corpus precompute (run_discovery_corpus over ci_witness_layer_roots) fails resolving dsl/test/claim/doc_reachability_witness_test.dag (from #5484 doc-graph reachability-completeness lens) — functions doc_graph_orphan_count / doc_graph_dangling_link_count / doc_graph_doc_count 'not found in scope'. The witness either references undefined fns or omits the import of the doc_graph module; the normal 647-witness discovery corpus passes, so the whole-tree precompute resolves it in a different (mock-corpus) context. Pre-existing on origin/main (never ran under the old 3-test allowlist), surfaced by the run-all widening (#5427), NOT caused by it (file untouched by this PR). Route to the #5484 doc-graph reachability-lens owner. bucket=doc-graph-wholetree-resolve"]
#[test]
fn wet_hermetic_scaffold_roster_outcomes_agree() {
    let roots = ci_witness_layer_roots();
    let explicit = governed_service_representative_explicit_entries();
    let wet = run_discovery_corpus(&roots, &[], &explicit, ExecutionMode::Wet, 1)
        .expect("wet discovery run for scaffold roster");
    let hermetic = run_discovery_corpus(&roots, &[], &explicit, ExecutionMode::Hermetic, 1)
        .expect("hermetic discovery run for scaffold roster");
    let divergences = wet_hermetic_discovery_outcome_divergences(
        &wet.witness_outcomes,
        &hermetic.witness_outcomes,
    );
    assert!(
        divergences.is_empty(),
        "scaffold roster outcomes agree (vacuous — no service dispatch in roster); divergences:\n{}",
        divergences.join("\n")
    );
}
