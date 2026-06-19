//! Wet==hermetic equivalence gate for governed-service witnesses (P3c).
//!
//! Before flipping CI's default `ExecutionMode` to Hermetic, prove that the
//! representative governed-service roster — one discoverable `test fn` pair per
//! extdeps layer under `src/v2/test/lens_mock_totality/` — yields identical
//! per-witness outcomes under `--wet` and `--hermetic` on the real discovery
//! corpus runner (`cli_run::run_discovery_corpus`). RED on any divergence.

use v1_compiler::cli_run::{
    discover_floor_corpus_rows, is_governed_service_representative_row, run_discovery_corpus,
    wet_hermetic_discovery_outcome_divergences,
};
use v1_compiler::v1_interpreter::ExecutionMode;

use crate::helpers::workspace_root;

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
    let scan_dirs = ci_witness_scan_dirs();
    let rows = discover_floor_corpus_rows(&roots, &scan_dirs)
        .expect("discover floor corpus for governed-service representative roster");
    let rep: Vec<(String, String)> = rows
        .iter()
        .filter(|r| is_governed_service_representative_row(r))
        .map(|r| (r.entry.clone(), r.function.clone()))
        .collect();
    assert!(
        !rep.is_empty(),
        "governed-service representative roster must be non-empty (lens_mock_totality witnesses)"
    );
    rep
}

#[test]
fn wet_hermetic_governed_service_representative_equivalence_holds() {
    let roots = ci_witness_layer_roots();
    let explicit = governed_service_representative_explicit_entries();
    let wet = run_discovery_corpus(&roots, &[], &explicit, ExecutionMode::Wet)
        .expect("wet discovery run for governed-service representative roster");
    let hermetic = run_discovery_corpus(&roots, &[], &explicit, ExecutionMode::Hermetic)
        .expect("hermetic discovery run for governed-service representative roster");
    let divergences = wet_hermetic_discovery_outcome_divergences(
        &wet.witness_outcomes,
        &hermetic.witness_outcomes,
    );
    assert!(
        divergences.is_empty(),
        "wet and hermetic must agree on every governed-service representative witness; divergences:\n{}",
        divergences.join("\n")
    );
}
