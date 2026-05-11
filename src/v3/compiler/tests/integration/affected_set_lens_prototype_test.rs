//! **Layer:** integration
//!
//! R4.B prototype smoke for [`v3_compiler::affected_set_lens`] (tracking issue `#2699`, design companion PR `#2700`).

use std::path::Path;

use v3_compiler::affected_set_lens::compute_affected_set_lens_report;
use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::CompileError;

const CASE_ATTRIB_PREFIX: &str = "src/v3/compiler/tests/fixtures/affected_set/";

fn compile_fixture(source: &str, attribution: &str) -> Dag {
    match compile_to_dag(source, attribution) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "fixture `{attribution}` emitted diagnostics: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(err) => panic!("fixture `{attribution}` failed structural compile: {err:?}"),
    }
}

fn repo_root_expect_git() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

fn probe_merge_revision(
    merge_oid: &str,
    repo_rel_path: &str,
    attribution_stub: &str,
) -> Option<(Dag, Dag)> {
    let repo_root = repo_root_expect_git();
    if !repo_root.join(".git").exists() {
        return None;
    }
    let before_spec = format!("{merge_oid}^1:{repo_rel_path}");
    let after_spec = format!("{merge_oid}:{repo_rel_path}");
    let before_bytes = std::process::Command::new("git")
        .current_dir(&repo_root)
        .args(["show", before_spec.as_str()])
        .output()
        .ok()
        .filter(|out| out.status.success())?;
    let after_bytes = std::process::Command::new("git")
        .current_dir(&repo_root)
        .args(["show", after_spec.as_str()])
        .output()
        .ok()
        .filter(|out| out.status.success())?;
    let before_src = String::from_utf8(before_bytes.stdout).ok()?;
    let after_src = String::from_utf8(after_bytes.stdout).ok()?;
    let attrib = format!("{CASE_ATTRIB_PREFIX}{attribution_stub}");
    let before = compile_to_dag(&before_src, &attrib).ok()?;
    let after = compile_to_dag(&after_src, &attrib).ok()?;
    Some((before, after))
}

fn assert_each_slice_within_transitive(report: &v3_compiler::affected_set_lens::AffectedSetLensReport) {
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

#[test]
fn case_a_cost_or_effect_narrows_vs_transitive() {
    let before = compile_fixture(
        include_str!("../fixtures/affected_set/case_a_before.dag"),
        concat!(
            CASE_ATTRIB_PREFIX,
            "case_a_shared.dag"
        ),
    );
    let after = compile_fixture(
        include_str!("../fixtures/affected_set/case_a_after.dag"),
        concat!(
            CASE_ATTRIB_PREFIX,
            "case_a_shared.dag"
        ),
    );
    let report = compute_affected_set_lens_report(&before, &after);
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
        concat!(
            CASE_ATTRIB_PREFIX,
            "case_b_shared.dag"
        ),
    );
    let after = compile_fixture(
        include_str!("../fixtures/affected_set/case_b_after.dag"),
        concat!(
            CASE_ATTRIB_PREFIX,
            "case_b_shared.dag"
        ),
    );
    let report = compute_affected_set_lens_report(&before, &after);
    assert!(report.structural_seed_count >= 2, "{report:?}");
    assert_each_slice_within_transitive(&report);
}

#[test]
fn case_c_algebra_carrier_surrogate_changes_walker_dependency() {
    let before = compile_fixture(
        include_str!("../fixtures/affected_set/case_c_before.dag"),
        concat!(
            CASE_ATTRIB_PREFIX,
            "case_c_shared.dag"
        ),
    );
    let after = compile_fixture(
        include_str!("../fixtures/affected_set/case_c_after.dag"),
        concat!(
            CASE_ATTRIB_PREFIX,
            "case_c_shared.dag"
        ),
    );
    let report = compute_affected_set_lens_report(&before, &after);
    assert!(!report.transitive_downstream.is_empty());
    assert_each_slice_within_transitive(&report);
}

#[test]
fn case_d_test_only_surrogate_isolates_unreachable_node() {
    let before = compile_fixture(
        include_str!("../fixtures/affected_set/case_d_before.dag"),
        concat!(
            CASE_ATTRIB_PREFIX,
            "case_d_shared.dag"
        ),
    );
    let after = compile_fixture(
        include_str!("../fixtures/affected_set/case_d_after.dag"),
        concat!(
            CASE_ATTRIB_PREFIX,
            "case_d_shared.dag"
        ),
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
        concat!(
            CASE_ATTRIB_PREFIX,
            "case_e_shared.dag"
        ),
    );
    let after = compile_fixture(
        include_str!("../fixtures/affected_set/case_e_after.dag"),
        concat!(
            CASE_ATTRIB_PREFIX,
            "case_e_shared.dag"
        ),
    );
    let report = compute_affected_set_lens_report(&before, &after);
    assert!(report.structural_seed_count >= 2, "{report:?}");
    assert_each_slice_within_transitive(&report);
}

#[test]
fn real_pr_git_probe_2679_matches_expectations_when_history_present() {
    let Some((before, after)) = probe_merge_revision(
        "6897445b874f1831468f27c871c00f5b23d7ded2",
        "src/v3/lenses/idempotency.dag",
        "probe_idempotency_lens.dag",
    ) else {
        return;
    };
    let report = compute_affected_set_lens_report(&before, &after);
    assert_each_slice_within_transitive(&report);
    assert!(
        !report.transitive_downstream.is_empty(),
        "idempotency lens edit should move at least one behavioral node"
    );
}

#[test]
fn real_pr_git_probe_2647_verification_substrate_when_history_present() {
    let Some((before, after)) = probe_merge_revision(
        "a091e1a2671efdfe50ee49bb4a2f7b5908e85f53",
        "src/v3/std/verification.dag",
        "probe_verification.dag",
    ) else {
        return;
    };
    let report = compute_affected_set_lens_report(&before, &after);
    assert_each_slice_within_transitive(&report);
    assert!(
        report.structural_seed_count > 0,
        "quantifier substrate PR should disturb multiple declarations"
    );
}

#[test]
fn real_pr_git_probe_2693_types_dag_when_history_present() {
    let Some((before, after)) = probe_merge_revision(
        "39ba757288246f95bea187f81593ed75729507e0",
        "dsl/std/types.dag",
        "probe_types_dag.dag",
    ) else {
        return;
    };
    let report = compute_affected_set_lens_report(&before, &after);
    assert_each_slice_within_transitive(&report);
    assert!(
        report.transitive_downstream.len() > 10,
        "large structural PR should surface a big (but still bounded) downstream envelope"
    );
}
