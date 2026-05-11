//! **Layer:** integration
//!
//! R4.B prototype smoke for [`v3_compiler::affected_set_lens`] (tracking issue `#2699`, design companion PR `#2700`).
//! Assertions load `case_*_{before,after}.dag` via `include_str!` and use **`case_*_shared.dag`** files
//! as attribution paths (sparse module stubs beside each pair; hermetic `TESTING.md`).
//!
//! **Charter bookkeeping (`#2699` § upstream PR probes)** — human/tooling anchors only (**not**
//! exercised as `#[test]` per `TESTING.md` § hermetic behavior claims). Downstream tooling may
//! compare Dag snapshots keyed to these merges (REST `mergeCommit.oid` snapshots, authored 2026-05-11):
//! - `#2647` → `a091e1a2671efdfe50ee49bb4a2f7b5908e85f53`
//! - `#2679` → `6897445b874f1831468f27c871c00f5b23d7ded2`
//! - `#2693` → `39ba757288246f95bea187f81593ed75729507e0`

use v3_compiler::affected_set_lens::compute_affected_set_lens_report;
use v3_compiler::dag::Dag;
use v3_compiler::CompileError;

const CASE_A_ATTRIB: &str = "src/v3/compiler/tests/fixtures/affected_set/case_a_shared.dag";
const CASE_B_ATTRIB: &str = "src/v3/compiler/tests/fixtures/affected_set/case_b_shared.dag";
const CASE_C_ATTRIB: &str = "src/v3/compiler/tests/fixtures/affected_set/case_c_shared.dag";
const CASE_D_ATTRIB: &str = "src/v3/compiler/tests/fixtures/affected_set/case_d_shared.dag";
const CASE_E_ATTRIB: &str = "src/v3/compiler/tests/fixtures/affected_set/case_e_shared.dag";

fn compile_fixture(source: &str, attribution: &str) -> Dag {
    match v3_compiler::compile_parse_surface_std_authority_dag(source, attribution) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "fixture `{attribution}` emitted diagnostics: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(err) => panic!("fixture `{attribution}` failed structural compile: {err:?}"),
    }
}

fn assert_each_slice_within_transitive(
    report: &v3_compiler::affected_set_lens::AffectedSetLensReport,
) {
    assert_eq!(
        report.structural.affected_ids, report.transitive_downstream,
        "structural.affected_ids must track legacy transitive_downstream"
    );
    assert_report_slices_carry_receipts(report);
    let bound = report.transitive_downstream.len();
    assert!(
        report.value.affected_ids.len() <= bound,
        "value slice should never expand beyond structural downstream slice (same seeds)"
    );
    assert!(
        report.cost.affected_ids.len() <= bound,
        "cost slice should land inside structural downstream when seeds ⊆ structural edits"
    );
    assert!(
        report.effect.affected_ids.len() <= bound,
        "effect slice should stay inside structural downstream envelope"
    );
    assert!(
        report.refinement.affected_ids.len() <= bound,
        "refinement slice should stay inside structural downstream envelope"
    );
}

fn assert_report_slices_carry_receipts(
    report: &v3_compiler::affected_set_lens::AffectedSetLensReport,
) {
    let slices = [
        &report.structural,
        &report.value,
        &report.cost,
        &report.effect,
        &report.refinement,
    ];
    for s in slices {
        assert!(
            !s.receipts.is_empty(),
            "dimension '{}' must emit at least one discipline receipt",
            s.dimension,
        );
    }
}

#[test]
fn case_a_cost_or_effect_narrows_vs_transitive() {
    let before = compile_fixture(
        include_str!("../fixtures/affected_set/case_a_before.dag"),
        CASE_A_ATTRIB,
    );
    let after = compile_fixture(
        include_str!("../fixtures/affected_set/case_a_after.dag"),
        CASE_A_ATTRIB,
    );
    let report = compute_affected_set_lens_report(&before, &after);
    assert_eq!(
        report.span_key_behavior_orphan_after_count, 0,
        "case_a before/after fixtures must share byte-aligned preambles so SourceSpan pairing exercises real peer rows, not orphan fail-closed ({report:?})"
    );
    assert!(
        report.span_key_behavior_pair_count >= 2,
        "case_a should span-pair multiply_then_add + consumer_lane behaviors ({report:?})"
    );
    assert!(
        !report.transitive_downstream.is_empty(),
        "fixture should include downstream consumers of multiply_then_add"
    );
    assert_each_slice_within_transitive(&report);
    let strict_narrow = report.cost.affected_ids.len() < report.transitive_downstream.len()
        || report.effect.affected_ids.len() < report.transitive_downstream.len()
        || report.refinement.affected_ids.len() < report.transitive_downstream.len();
    assert!(
        strict_narrow,
        "expected at least one auxiliary dimension to narrow vs transitive downstream: {report:?}"
    );
}

#[test]
fn case_b_signature_change_surfaces_wide_structural_seed() {
    let before = compile_fixture(
        include_str!("../fixtures/affected_set/case_b_before.dag"),
        CASE_B_ATTRIB,
    );
    let after = compile_fixture(
        include_str!("../fixtures/affected_set/case_b_after.dag"),
        CASE_B_ATTRIB,
    );
    let report = compute_affected_set_lens_report(&before, &after);
    assert!(report.structural_seed_count >= 2, "{report:?}");
    assert_each_slice_within_transitive(&report);
}

#[test]
fn case_c_algebra_carrier_surrogate_changes_walker_dependency() {
    let before = compile_fixture(
        include_str!("../fixtures/affected_set/case_c_before.dag"),
        CASE_C_ATTRIB,
    );
    let after = compile_fixture(
        include_str!("../fixtures/affected_set/case_c_after.dag"),
        CASE_C_ATTRIB,
    );
    let report = compute_affected_set_lens_report(&before, &after);
    assert!(!report.transitive_downstream.is_empty());
    assert_each_slice_within_transitive(&report);
}

#[test]
fn case_d_test_only_surrogate_isolates_unreachable_node() {
    let before = compile_fixture(
        include_str!("../fixtures/affected_set/case_d_before.dag"),
        CASE_D_ATTRIB,
    );
    let after = compile_fixture(
        include_str!("../fixtures/affected_set/case_d_after.dag"),
        CASE_D_ATTRIB,
    );
    let report = compute_affected_set_lens_report(&before, &after);
    assert_each_slice_within_transitive(&report);
    assert!(
        report.transitive_downstream.len() <= 2,
        "expected surrogate test-only churn to remain tiny, got {:?}",
        report.transitive_downstream
    );
}

#[test]
fn case_e_port_tightening_surrogate_touches_every_call_site() {
    let before = compile_fixture(
        include_str!("../fixtures/affected_set/case_e_before.dag"),
        CASE_E_ATTRIB,
    );
    let after = compile_fixture(
        include_str!("../fixtures/affected_set/case_e_after.dag"),
        CASE_E_ATTRIB,
    );
    let report = compute_affected_set_lens_report(&before, &after);
    assert!(report.structural_seed_count >= 2, "{report:?}");
    assert_each_slice_within_transitive(&report);
}
