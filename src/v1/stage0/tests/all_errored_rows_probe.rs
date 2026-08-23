//! LOCAL DIAGNOSTIC PROBE — not for commit. All 170 RUNTIME-ERRORED enrolled rows through the
//! real floor path, reporting scope size beside outcome.

use v1_compiler::cli_run::{
    claim_scope_for, evaluation_frame, floor_prepared_subject_exclusions, prepare_repository_once,
    run_claim_measured,
};
use v1_compiler::v1_interpreter::ExecutionMode;

const ROWS: &[(&str, &str)] = &[
    ("test.claim.accelerator_demo_execution_witness", "test.claim.accelerator_demo_execution_witness.accelerator_demo_execution_lane_witnesses"),
    ("test.claim.accelerator_demo_model_witness", "test.claim.accelerator_demo_model_witness.accelerator_demo_modeling_lane_witnesses"),
    ("test.claim.ci_deploy_target_host_witness", "test.claim.ci_deploy_target_host_witness.witness_deploy_stage_pins_srv1_runner_label"),
    ("test.claim.ci_deploy_target_host_witness", "test.claim.ci_deploy_target_host_witness.witness_deploy_job_not_on_ubicloud_runner"),
    ("test.claim.config_record_emit_witness", "test.claim.config_record_emit_witness.witness_manifest_knobs_space_separated"),
    ("test.claim.config_record_emit_witness", "test.claim.config_record_emit_witness.witness_same_fields_differ_across_knobs"),
    ("test.claim.design_argument_witness", "test.claim.design_argument_witness.design_s1_argument_form_is_valid"),
    ("test.claim.design_argument_witness", "test.claim.design_argument_witness.design_s1_argument_is_non_vacuous"),
    ("test.claim.design_argument_witness", "test.claim.design_argument_witness.design_full_argument_form_is_valid"),
    ("test.claim.design_argument_witness", "test.claim.design_argument_witness.design_full_argument_has_no_orphan"),
    ("test.claim.design_argument_witness", "test.claim.design_argument_witness.design_full_argument_is_acyclic"),
    ("test.claim.design_argument_witness", "test.claim.design_argument_witness.design_full_argument_axiom_set_is_closed"),
    ("test.claim.design_argument_witness", "test.claim.design_argument_witness.orphan_is_detected"),
    ("test.claim.design_argument_witness", "test.claim.design_argument_witness.cycle_is_detected"),
    ("test.claim.design_argument_witness", "test.claim.design_argument_witness.smuggled_axiom_is_detected"),
    ("test.claim.ebay_listing_witness", "test.claim.ebay_listing_witness.witness_default_listing_policy_targets_us_marketplace"),
    ("test.claim.emit_host_gate_witness", "test.claim.emit_host_gate_witness.w_rust_fixture_nonempty"),
    ("test.claim.emit_host_gate_witness", "test.claim.emit_host_gate_witness.w_python_fixture_nonempty"),
    ("test.claim.emit_host_gate_witness", "test.claim.emit_host_gate_witness.w_go_fixture_nonempty"),
    ("test.claim.emit_host_gate_witness", "test.claim.emit_host_gate_witness.w_ts_fixture_nonempty"),
    ("test.claim.emit_host_gate_witness", "test.claim.emit_host_gate_witness.w_byte_count_is_five"),
    ("test.claim.emit_host_gate_witness", "test.claim.emit_host_gate_witness.w_ts_byte_count_is_four"),
    ("test.claim.fleet_convergence_verdict_witness", "test.claim.fleet_convergence_verdict_witness.live_fleet_is_not_converged_and_unobserved_cells_are_counted"),
    ("test.claim.fleet_convergence_verdict_witness", "test.claim.fleet_convergence_verdict_witness.knob_receipts_alone_cannot_converge_the_fleet"),
    ("test.claim.fleet_receipt_collector_witness", "test.claim.fleet_receipt_collector_witness.witness_partial_collection_never_yields_fleet_converged"),
    ("test.claim.go_module_versioning_witness", "test.claim.go_module_versioning_witness.w_go_manifest_file"),
    ("test.claim.hand_lens_host_bridge_scaffold_watchdog", "test.claim.hand_lens_host_bridge_scaffold_watchdog.hand_lens_host_bridge_scaffold_roster_tracks_5364_dissolution"),
    ("test.claim.hetzner_cost_quote_witness", "test.claim.hetzner_cost_quote_witness.hetzner_cax41_cost_class_is_per_second_billed"),
    ("test.claim.hetzner_cost_quote_witness", "test.claim.hetzner_cost_quote_witness.hetzner_cax41_currency_is_eur"),
    ("test.claim.hetzner_cost_quote_witness", "test.claim.hetzner_cost_quote_witness.hetzner_cax41_unit_price_matches_catalog"),
    ("test.claim.host_phase_status", "test.claim.host_phase_status.witness_reconcile_covers_upsert_teardown_and_noop"),
    ("test.claim.host_phase_status", "test.claim.host_phase_status.witness_matrix_is_exact_join_over_participation_programs"),
    ("test.claim.host_phase_status", "test.claim.host_phase_status.witness_live_observation_frontier_is_one_cell"),
    ("test.claim.host_phase_status", "test.claim.host_phase_status.witness_unmodeled_cells_track_spine_gaps"),
    ("test.claim.host_phase_status", "test.claim.host_phase_status.witness_remaining_work_counts_every_unfinished_cell"),
    ("test.claim.host_phase_status", "test.claim.host_phase_status.witness_program_scoping_preserves_the_runner_fleet_matrix"),
    ("test.claim.host_phase_status", "test.claim.host_phase_status.witness_fleet_matrix_has_no_duplicated_cells"),
    ("test.claim.host_phase_status", "test.claim.host_phase_status.witness_matrix_hosts_are_exactly_the_enrolled_roster"),
    ("test.claim.host_phase_status", "test.claim.host_phase_status.witness_enrolled_sparks_carry_exactly_the_shared_obligations"),
    ("test.claim.host_phase_status", "test.claim.host_phase_status.witness_enrolled_spark_cells_are_all_unobserved"),
    ("test.claim.host_phase_status", "test.claim.host_phase_status.witness_runner_only_obligations_exist_on_runner_hosts"),
    ("test.claim.host_phase_status", "test.claim.host_phase_status.witness_every_covered_cell_today_is_the_authored_one"),
    ("test.claim.host_reach_identity_probe_witness", "test.claim.host_reach_identity_probe_witness.committed_probe_receipts_feed_producers_and_grow_coverage"),
    ("test.claim.host_reach_identity_probe_witness", "test.claim.host_reach_identity_probe_witness.observe_host_phase_routes_by_label_and_stays_fail_closed"),
    ("test.claim.host_standup_assimilation_deduction", "test.claim.host_standup_assimilation_deduction.pre_install_within_budget_is_absent_not_converged"),
    ("test.claim.host_standup_assimilation_deduction", "test.claim.host_standup_assimilation_deduction.post_install_lease_row_deduces_converged_noop"),
    ("test.claim.host_standup_assimilation_deduction", "test.claim.host_standup_assimilation_deduction.unreadable_lease_is_unknown_refused_not_pending"),
    ("test.claim.host_standup_assimilation_deduction", "test.claim.host_standup_assimilation_deduction.no_lease_past_budget_is_drifted_not_fabricated_success"),
    ("test.claim.host_standup_assimilation_deduction", "test.claim.host_standup_assimilation_deduction.upsert_pending_applies_assert_pending_plan"),
    ("test.claim.host_standup_spine", "test.claim.host_standup_spine.post_install_dhcp_subsumption_reduces_gap_count_by_one"),
    ("test.claim.materialized_secret", "test.claim.materialized_secret.witness_enabled_but_unprobed_refuses"),
    ("test.claim.materialized_secret", "test.claim.materialized_secret.witness_enabled_but_revoked_refuses"),
    ("test.claim.materialized_secret", "test.claim.materialized_secret.witness_live_secret_materializes_and_runs_the_use"),
    ("test.claim.materialized_secret", "test.claim.materialized_secret.witness_destroyed_version_refuses_even_when_probe_says_live"),
    ("test.claim.materialized_secret", "test.claim.materialized_secret.witness_self_repairing_probe_is_refused_by_classification"),
    ("test.claim.materialized_secret", "test.claim.materialized_secret.witness_red_control_existence_only_rejected_by_gate"),
    ("test.claim.realization_reconcile_witness", "test.claim.realization_reconcile_witness.witness_same_keysource_collapses_to_share"),
    ("test.claim.realization_reconcile_witness", "test.claim.realization_reconcile_witness.witness_different_keysource_does_not_collapse"),
    ("test.claim.realization_reconcile_witness", "test.claim.realization_reconcile_witness.witness_post_always_never_collapses"),
    ("test.claim.samsung_dram_module", "test.claim.samsung_dram_module.four_rank_by_four_is_realizable_on_ddr4"),
    ("test.claim.samsung_dram_module", "test.claim.samsung_dram_module.same_capacity_at_one_rank_by_eight_is_not_realizable"),
    ("test.claim.samsung_dram_module", "test.claim.samsung_dram_module.generation_mismatch_fails_closed"),
    ("test.claim.srv3_subsumption", "test.claim.srv3_subsumption.srv3_reconcile_adds_exactly_the_missing_fleet_key"),
    ("test.claim.srv3_subsumption", "test.claim.srv3_subsumption.srv3_reconcile_refuses_both_unowned_keys_never_teardown"),
    ("test.claim.srv3_subsumption", "test.claim.srv3_subsumption.srv4_reconcile_is_converged_empty_plan"),
    ("test.claim.srv3_subsumption", "test.claim.srv3_subsumption.srv3_and_srv4_disagree_on_convergence"),
    ("test.claim.srv3_subsumption", "test.claim.srv3_subsumption.reconcile_would_teardown_only_if_ownership_claimed_it"),
    ("test.claim.srv3_subsumption", "test.claim.srv3_subsumption.comment_only_drift_is_one_change_not_remove_add"),
    ("v2.test.lens_mutation_adequacy.mutation_adequacy_test", "v2.test.lens_mutation_adequacy.mutation_adequacy_test.adequate_when_every_mutation_has_a_discriminating_witness"),
    ("v2.test.lens_mutation_adequacy.mutation_adequacy_test", "v2.test.lens_mutation_adequacy.mutation_adequacy_test.surviving_mutation_is_inadequate"),
    ("v2.test.lens_vacuity.vacuity_test", "v2.test.lens_vacuity.vacuity_test.vacuity_rung5_emit_vs_eval_classifies_proven_independent"),
    ("v2.test.lens_vacuity.vacuity_test", "v2.test.lens_vacuity.vacuity_test.vacuity_unified_node_corpus_matches_transport_classifier"),
    ("v2.test.lens_vacuity.vacuity_test", "v2.test.lens_vacuity.vacuity_test.vacuity_falsification_cases_each_match_expected_evidence"),
    ("v2.test.lens_vacuity.vacuity_test", "v2.test.lens_vacuity.vacuity_test.vacuity_advisory_census_partitions_falsification_suite"),
    ("v2.test.lens_vacuity.vacuity_test", "v2.test.lens_vacuity.vacuity_test.vacuity_falsification_suite_nonempty"),
    ("v2.test.claim.compiler_materialization_witness", "v2.test.claim.compiler_materialization_witness.parse_table_carrier_inhabits_realization"),
    ("v2.test.claim.compiler_materialization_witness", "v2.test.claim.compiler_materialization_witness.compile_stage_carrier_inhabits_realization"),
    ("v2.test.emit.produced_decl_support_preserved", "v2.test.emit.produced_decl_support_preserved.produced_decl_support_survives_augmentation"),
    ("v2.test.extdeps.spice_rc_passive_deck_claims", "v2.test.extdeps.spice_rc_passive_deck_claims.spice_rc_passive_deck_authority_holds"),
    ("v2.test.extdeps_shape_transport_policy.corpus.cargo_build_policy_leak", "v2.test.extdeps_shape_transport_policy.corpus.cargo_build_policy_leak.corpus_cargo_build_defused_holds"),
    ("v2.test.extdeps_shape_transport_policy.corpus.cargo_clippy_dead_param", "v2.test.extdeps_shape_transport_policy.corpus.cargo_clippy_dead_param.corpus_cargo_clippy_dead_param_defused_holds"),
    ("v2.test.extdeps_shape_transport_policy.corpus.cargo_doc_dead_param", "v2.test.extdeps_shape_transport_policy.corpus.cargo_doc_dead_param.corpus_cargo_doc_dead_param_defused_holds"),
    ("v2.test.extdeps_shape_transport_policy.corpus.cargo_doc_policy_leak", "v2.test.extdeps_shape_transport_policy.corpus.cargo_doc_policy_leak.corpus_cargo_doc_defused_holds"),
    ("v2.test.extdeps_shape_transport_policy.corpus.cargo_fmt_dead_param", "v2.test.extdeps_shape_transport_policy.corpus.cargo_fmt_dead_param.corpus_cargo_fmt_dead_param_defused_holds"),
    ("v2.test.extdeps_shape_transport_policy.corpus.cargo_fmt_policy_leak", "v2.test.extdeps_shape_transport_policy.corpus.cargo_fmt_policy_leak.corpus_cargo_fmt_defused_holds"),
    ("v2.test.extdeps_shape_transport_policy.corpus.cargo_run_dead_param", "v2.test.extdeps_shape_transport_policy.corpus.cargo_run_dead_param.corpus_cargo_run_dead_param_defused_holds"),
    ("v2.test.extdeps_shape_transport_policy.corpus.gcp_login_dead_param", "v2.test.extdeps_shape_transport_policy.corpus.gcp_login_dead_param.corpus_gcp_login_dead_param_defused_holds"),
    ("v2.test.extdeps_shape_transport_policy.corpus.gcp_oauth_fusion_fork", "v2.test.extdeps_shape_transport_policy.corpus.gcp_oauth_fusion_fork.corpus_gcp_oauth_defused_holds"),
    ("v2.test.extdeps_shape_transport_policy.corpus.gist_create_policy_leak", "v2.test.extdeps_shape_transport_policy.corpus.gist_create_policy_leak.corpus_gist_create_defused_holds"),
    ("v2.test.extdeps_shape_transport_policy.corpus.git_policy_leak", "v2.test.extdeps_shape_transport_policy.corpus.git_policy_leak.corpus_git_policy_leak_defused_holds"),
    ("v2.test.extdeps_shape_transport_policy.corpus.runtime_local_embedded_policy", "v2.test.extdeps_shape_transport_policy.corpus.runtime_local_embedded_policy.corpus_runtime_local_embedded_policy_defused_holds"),
    ("v2.test.extdeps_shape_transport_policy.coverage_domain_equivalence_test", "v2.test.extdeps_shape_transport_policy.coverage_domain_equivalence_test.extdeps_shape_transport_policy_coverage_domain_equivalence_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.clean_gist_create", "v2.test.extdeps_shape_transport_policy.lens_unit.clean_gist_create.clean_gist_create_is_green_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.clean_git_diff", "v2.test.extdeps_shape_transport_policy.lens_unit.clean_git_diff.clean_git_diff_is_green_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.dead_param_cargo_build", "v2.test.extdeps_shape_transport_policy.lens_unit.dead_param_cargo_build.dead_param_cargo_build_is_red_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.dead_param_cargo_clippy", "v2.test.extdeps_shape_transport_policy.lens_unit.dead_param_cargo_clippy.dead_param_cargo_clippy_is_red_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.dead_param_gcp_login", "v2.test.extdeps_shape_transport_policy.lens_unit.dead_param_gcp_login.dead_param_gcp_login_is_red_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.embedded_policy_literal_local", "v2.test.extdeps_shape_transport_policy.lens_unit.embedded_policy_literal_local.embedded_policy_literal_local_is_red_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.module_path_rename", "v2.test.extdeps_shape_transport_policy.lens_unit.module_path_rename.module_path_rename_resolves_by_qn_not_filepath_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.module_path_rename", "v2.test.extdeps_shape_transport_policy.lens_unit.module_path_rename.module_path_rename_unknown_qn_does_not_resolve_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.module_source_nickname_literal_green", "v2.test.extdeps_shape_transport_policy.lens_unit.module_source_nickname_literal_green.module_source_nickname_literal_exempt_literals_is_green_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.module_source_nickname_literal_coverage_domain_green", "v2.test.extdeps_shape_transport_policy.lens_unit.module_source_nickname_literal_coverage_domain_green.module_source_nickname_literal_coverage_domain_is_green_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.module_source_nickname_literal_local_red", "v2.test.extdeps_shape_transport_policy.lens_unit.module_source_nickname_literal_local_red.module_source_nickname_literal_local_red_is_red_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.policy_leak_cargo_build", "v2.test.extdeps_shape_transport_policy.lens_unit.policy_leak_cargo_build.policy_leak_cargo_build_is_red_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.policy_leak_cargo_doc", "v2.test.extdeps_shape_transport_policy.lens_unit.policy_leak_cargo_doc.policy_leak_cargo_doc_is_red_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.policy_leak_cargo_fmt", "v2.test.extdeps_shape_transport_policy.lens_unit.policy_leak_cargo_fmt.policy_leak_cargo_fmt_is_red_holds"),
    ("v2.test.extdeps_shape_transport_policy.lens_unit.transport_fusion_gcp_oauth", "v2.test.extdeps_shape_transport_policy.lens_unit.transport_fusion_gcp_oauth.transport_fusion_gcp_oauth_is_red_holds"),
    ("v2.test.fact_cardinality.lens_unit.synthetic_fork_red", "v2.test.fact_cardinality.lens_unit.synthetic_fork_red.synthetic_fork_red_holds"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_coproduct_exhaustiveness_is_diagnostic"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_coproduct_exhaustiveness_anchor_holds"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_coproduct_exhaustiveness_roster_covers_testclaim"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_coproduct_exhaustiveness_covers_every_coproduct"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_cross_representation_equality_is_equals"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_cross_representation_equality_anchor_holds"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_cross_representation_equality_covers_bool"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_cross_representation_equality_covers_optional_null"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_refinement_preserves_nonempty_list_base"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_refinement_subject_anchor_holds"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_refinement_scheduler_emits_one_generator"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_refinement_claim_emits"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_lbe_conj_snapshot_passes"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_lbe_disj_snapshot_passes"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_lbe_transform_snapshot_passes"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_lbe_schedules_three_generators"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_lbe_dag_surface_language_identity"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_witness_validity_constraint_satisfaction_emits"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_witness_validity_property_symbol_mismatch_emits"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_witness_validity_evidence_node_mismatch_emits"),
    ("v2.test.claim.generated_conformance_floor", "v2.test.claim.generated_conformance_floor.generated_witness_validity_rule_not_realized_emits"),
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
fn all_errored_rows_through_floor_scope() {
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let (prepared, _views) =
        prepare_repository_once(&roots, &floor_prepared_subject_exclusions()).expect("prepare");
    let skip: Vec<String> = std::env::var("PROBE_SKIP")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    let lo: usize = std::env::var("PROBE_LO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let hi: usize = std::env::var("PROBE_HI")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(usize::MAX);
    for (idx, (module, qualified)) in ROWS.iter().enumerate() {
        if idx < lo || idx >= hi || skip.iter().any(|s| qualified.contains(s.as_str())) {
            continue;
        }
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
