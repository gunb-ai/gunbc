//! LOCAL DIAGNOSTIC PROBE — not for commit. The RUNTIME-ERRORED rows not yet measured,
//! excluding v2.test.claim.generated_conformance_floor: several of its rows OOM-kill a 31 GiB
//! container on their own, on the unmodified tree as well as the fixed one.

use v1_compiler::cli_run::{
    claim_scope_for, evaluation_frame, floor_prepared_subject_exclusions, prepare_repository_once,
    run_claim_measured,
};
use v1_compiler::v1_interpreter::ExecutionMode;

const ROWS: &[(&str, &str)] = &[
    ("v2.test.claim.gha_workflow_yaml_fold_structural", "v2.test.claim.gha_workflow_yaml_fold_structural.gha_fold_structural_tier_holds"),
    ("v2.test.intent_linearity.lens_unit.runtime_axis", "v2.test.intent_linearity.lens_unit.runtime_axis.nested_loop_cost_is_lowerable"),
    ("v2.test.intent_linearity.lens_unit.runtime_axis", "v2.test.intent_linearity.lens_unit.runtime_axis.flat_loop_cost_not_lowerable"),
    ("v2.test.intent_linearity.lens_unit.runtime_axis", "v2.test.intent_linearity.lens_unit.runtime_axis.nested_loop_is_runtime_candidate"),
    ("v2.test.intent_linearity.lens_unit.runtime_axis", "v2.test.intent_linearity.lens_unit.runtime_axis.flat_loop_not_runtime_candidate"),
    ("v2.test.intent_linearity.lens_unit.runtime_axis", "v2.test.intent_linearity.lens_unit.runtime_axis.nested_loop_not_changetime_redundant"),
    ("v2.test.intent_linearity.lens_unit.runtime_axis", "v2.test.intent_linearity.lens_unit.runtime_axis.flat_loop_is_linear"),
    ("v2.test.manual.bootstrap_footprint_anchor", "v2.test.manual.bootstrap_footprint_anchor.bootstrap_footprint_canonical_holds"),
    ("v2.test.manual.bootstrap_footprint_anchor", "v2.test.manual.bootstrap_footprint_anchor.bootstrap_footprint_non_canonical_violates"),
    ("v2.test.manual.bootstrap_footprint_anchor", "v2.test.manual.bootstrap_footprint_anchor.bootstrap_footprint_bad_content_hash_violates"),
    ("v2.test.manual.bootstrap_footprint_anchor", "v2.test.manual.bootstrap_footprint_anchor.bootstrap_footprint_bad_runtime_same_identity_violates"),
    ("v2.test.manual.medium_node_instantiation", "v2.test.manual.medium_node_instantiation.medium_node_carried_is_expected"),
    ("v2.test.manual.medium_node_instantiation", "v2.test.manual.medium_node_instantiation.medium_node_fidelity_is_lossless"),
    ("v2.test.manual.medium_node_instantiation", "v2.test.manual.medium_node_instantiation.medium_node_lossy_tag_not_lossless"),
    ("v2.test.manual.medium_source_text_instantiation", "v2.test.manual.medium_source_text_instantiation.witness_source_field_is_medium"),
    ("v2.test.manual.value_null_split_witness", "v2.test.manual.value_null_split_witness.optional_null_straddle_rostered_until_phase_e"),
    ("v2.test.claim.source_root_tagging", "v2.test.claim.source_root_tagging.source_root_tag_carried_and_folded_to_grounded_set"),
    ("v2.test.claim.staging", "v2.test.claim.staging.cached_stage_hit_skips_stage"),
    ("v2.test.claim.staging", "v2.test.claim.staging.cached_stage_miss_runs_stage"),
    ("v2.test.fixture.walk_plan_stage.recursion_refusal_member", "v2.test.fixture.walk_plan_stage.recursion_refusal_member.walk_plan_stage_recursion_refusal_member_holds"),
    ("v2.test.lens_mock_totality.cron_mock_totality", "v2.test.lens_mock_totality.cron_mock_totality.cron_mock_consumer_is_total_holds"),
    ("v2.test.lens_mock_totality.cron_mock_totality", "v2.test.lens_mock_totality.cron_mock_totality.cron_mock_omitted_member_is_red_holds"),
    ("v2.test.lens_mock_totality.diagnostic_mock_totality", "v2.test.lens_mock_totality.diagnostic_mock_totality.diagnostic_mock_consumer_is_total_holds"),
    ("v2.test.lens_mock_totality.diagnostic_mock_totality", "v2.test.lens_mock_totality.diagnostic_mock_totality.diagnostic_mock_omitted_member_is_red_holds"),
    ("v2.test.lens_mock_totality.filesystem_mock_totality", "v2.test.lens_mock_totality.filesystem_mock_totality.filesystem_mock_consumer_is_total_holds"),
    ("v2.test.lens_mock_totality.filesystem_mock_totality", "v2.test.lens_mock_totality.filesystem_mock_totality.filesystem_mock_omitted_member_is_red_holds"),
    ("v2.test.lens_mock_totality.gcp_mock_totality", "v2.test.lens_mock_totality.gcp_mock_totality.gcp_mock_consumer_is_total_holds"),
    ("v2.test.lens_mock_totality.gcp_mock_totality", "v2.test.lens_mock_totality.gcp_mock_totality.gcp_mock_omitted_member_is_red_holds"),
    ("v2.test.lens_mock_totality.git_mock_totality", "v2.test.lens_mock_totality.git_mock_totality.git_mock_consumer_is_total_holds"),
    ("v2.test.lens_mock_totality.git_mock_totality", "v2.test.lens_mock_totality.git_mock_totality.git_mock_omitted_member_is_red_holds"),
    ("v2.test.lens_mock_totality.git_mock_totality", "v2.test.lens_mock_totality.git_mock_totality.git_mock_omitted_show_tree_is_red_holds"),
    ("v2.test.lens_mock_totality.github_mock_totality", "v2.test.lens_mock_totality.github_mock_totality.github_mock_consumer_is_total_holds"),
    ("v2.test.lens_mock_totality.github_mock_totality", "v2.test.lens_mock_totality.github_mock_totality.github_mock_omitted_member_is_red_holds"),
    ("v2.test.lens_mock_totality.http_pilot_mock_totality", "v2.test.lens_mock_totality.http_pilot_mock_totality.http_pilot_mock_consumer_is_total_holds"),
    ("v2.test.lens_mock_totality.http_pilot_mock_totality", "v2.test.lens_mock_totality.http_pilot_mock_totality.http_pilot_mock_omitted_member_is_red_holds"),
    ("v2.test.lens_mock_totality.llm_mock_totality", "v2.test.lens_mock_totality.llm_mock_totality.llm_mock_consumer_is_total_holds"),
    ("v2.test.lens_mock_totality.llm_mock_totality", "v2.test.lens_mock_totality.llm_mock_totality.llm_mock_omitted_member_is_red_holds"),
    ("v2.test.lens_mock_totality.sec_edgar_mock_totality", "v2.test.lens_mock_totality.sec_edgar_mock_totality.sec_edgar_mock_consumer_is_total_holds"),
    ("v2.test.lens_mock_totality.sec_edgar_mock_totality", "v2.test.lens_mock_totality.sec_edgar_mock_totality.sec_edgar_mock_omitted_member_is_red_holds"),
    ("v2.test.lens_mock_totality.shell_mock_totality", "v2.test.lens_mock_totality.shell_mock_totality.shell_mock_consumer_is_total_holds"),
    ("v2.test.lens_mock_totality.shell_mock_totality", "v2.test.lens_mock_totality.shell_mock_totality.shell_mock_omitted_member_is_red_holds")
];

#[test]
#[ignore = "whole-corpus prepare; local diagnosis only"]
fn rest_errored_rows_through_floor_scope() {
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let (prepared, _views) =
        prepare_repository_once(&roots, &floor_prepared_subject_exclusions()).expect("prepare");
    for (module, qualified) in ROWS {
        match claim_scope_for(&prepared, module) {
            Ok(scope) => {
                let frame = evaluation_frame(&scope, ExecutionMode::Hermetic, None, None);
                let (outcome, _) = run_claim_measured(&frame, &prepared.subject_digest, qualified);
                println!(
                    "ROW\t{qualified}\t{}\t{}\t{outcome:?}",
                    scope.module_count, scope.ambiguous_bare_names
                );
            }
            Err(e) => println!("ROW\t{qualified}\tSCOPE-REFUSED\t0\t{e}"),
        }
    }
}
