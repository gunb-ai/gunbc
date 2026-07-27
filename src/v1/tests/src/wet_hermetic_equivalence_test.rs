use v1_compiler::cli_run::{
    discover_floor_witness_roster, is_governed_service_representative_row,
    run_discovery_corpus_with_options, wet_hermetic_discovery_outcome_divergences,
    wet_hermetic_scaffold_roster_entry_prefix, witness_exclusion_substrings, ClaimOutcome,
    DiscoveryCorpusOptions, DiscoveryRow, DiscoveryWidthPolicy, DiscoveryWitnessOutcome,
};
use v1_compiler::v1_interpreter::ExecutionMode;

use crate::helpers::workspace_root;

fn sample_outcome(entry: &str, function: &str, outcome: ClaimOutcome) -> DiscoveryWitnessOutcome {
    DiscoveryWitnessOutcome {
        entry: entry.to_string(),
        module_path: "test.wet_hermetic_equivalence_fixture".to_string(),
        function: function.to_string(),
        outcome,
        execution_leg: "InterpretedLeg".to_string(),
    }
}

#[test]
fn wet_hermetic_comparator_reports_outcome_divergence() {
    let wet = [sample_outcome(
        "dag/test/claim/example_witness_test.dag",
        "witness_holds",
        ClaimOutcome::Pass,
    )];
    let hermetic = [sample_outcome(
        "dag/test/claim/example_witness_test.dag",
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
        ws.join("dag").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ]
}

fn ci_witness_scan_dirs() -> Vec<String> {
    let ws = workspace_root();
    vec![
        ws.join("dag/test/claim").to_string_lossy().into_owned(),
        ws.join("src/v2/test/claim/manual")
            .to_string_lossy()
            .into_owned(),
    ]
}

fn governed_service_representative_explicit_entries() -> Vec<(String, String)> {
    let roots = ci_witness_layer_roots();
    let prefix = wet_hermetic_scaffold_roster_entry_prefix(&roots)
        .expect("load scaffold roster prefix from witness .dag authority");
    let scan_dirs = ci_witness_scan_dirs();
    let excludes = witness_exclusion_substrings();
    let rows = discover_floor_witness_roster(&roots, &scan_dirs, &excludes, &[])
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
    let excludes = witness_exclusion_substrings();
    let rows = discover_floor_witness_roster(&roots, &scan_dirs, &excludes, &[])
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
        entry: "dag/test/claim/unrelated_witness_test.dag".into(),
        function: "witness_holds".into(),
        reads_live_tree: true,
    };
    assert!(
        !is_governed_service_representative_row(&outsider, &prefix),
        "rows outside the dag prefix must not match the scaffold filter"
    );
}

#[test]
#[ignore = "flaky under full-suite parallel contention, surfaced by the run-all widening (#5427): passes isolated (~110s) but fails under the 766-test parallel load (~335s). nextest process-isolates each test, so this is resource/timing contention on the wet-execution path (real subprocess dispatch), not a logic bug — a non-deterministic test can't gate a merge (DESIGN §5 fail-open-by-noise). Pre-existing (#5276 wet==hermetic gate), never run under the old 3-filter allowlist. Route to the wet==hermetic-equivalence / hermetic-testing owner to make the wet path contention-robust (resource-bound or serialize via a nextest test-group) before re-enabling in the run-all gate. FLAG-DON'T-FIX."]
fn wet_hermetic_scaffold_roster_outcomes_agree() {
    let roots = ci_witness_layer_roots();
    let explicit = governed_service_representative_explicit_entries();
    let wet = run_discovery_corpus_with_options(
        &roots,
        &[],
        &explicit,
        ExecutionMode::Wet,
        DiscoveryWidthPolicy::Serial,
        DiscoveryCorpusOptions::default(),
    )
    .expect("wet discovery run for scaffold roster");
    let hermetic = run_discovery_corpus_with_options(
        &roots,
        &[],
        &explicit,
        ExecutionMode::Hermetic,
        DiscoveryWidthPolicy::Serial,
        DiscoveryCorpusOptions::default(),
    )
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
