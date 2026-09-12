LANE L7 - test.claim.compiler_frontend_program_status_witness + lens gate/vacuity/registry witnesses. 9 modules, 29 identities, 5254ms CPU.

SUBJECT. The floor's per-claim cost instrument reports 302 of 3637 claim identities above the
100ms cost line, carrying 58% of floor CPU. Operator rule, verbatim: "do not raise the ceiling -
everything over 100ms needs to be broken down". Your lane is one family of that population.

THE LINE ALREADY EXISTS AND ALREADY COUNTS. required_floor_claim_cost_line_ms is 100 and each run
emits over_cost_line_diagnostic (302 last run). required_floor states it is DIAGNOSTIC ONLY with no
admission-path consumer - an inert lens. Making it gate is a policy change the operator owns; it is
NOT your lane. Do not add a gate, a ratchet, or a budget row.

WHAT "BREAK DOWN" MEANS. Get each identity's own marginal cost under 100ms without losing evidence.
Legitimate: split one fat identity into several smaller ones that each still reach a discriminating
verdict; hoist genuinely shared setup into the shared-artifact fill; remove authored duplication
(DESIGN.md 2 - several identities recomputing one fact should join at their least common ancestor,
not each recompute). Illegitimate and a hard reject: deleting or weakening a discriminating RED,
softening a fixture so it can no longer go red, folding a refusal probe into a holds probe. 4b(4) -
a climb deletes lower-rung PRODUCTION machinery, never the evidence.

ROOT-CAUSE FIRST (DESIGN.md 6). Your rows cluster because they are ONE SHAPE repeated N times.
Find the single shared cost defect in the family's host - a fixture recompiled per identity, a
copied accumulator, a quadratic fold - before editing identities one at a time. Thirty per-identity
nibbles is the forked-logic trap and will be reviewed as one.

MEASURE, DO NOT GUESS. Re-derive with the floor's own instrument:
  gh run download <run-id> -n required-floor-claim-cost -D /tmp/floor-cost/<run>/
(gh api .../zip returns a truncated archive; unzip is not in the container). The repo already has
the reader: gunbc.floor_cost_distribution (cost_bands / rows_at_or_above), driven by
floor_cost_distribution_report, which expects the TSVs under /tmp/floor-cost/<run>/. Cite the
instrument, never transcribe its numbers into prose (DESIGN.md 6).

TWO CAUTIONS. (1) Costs are MARGINAL - shared-artifact fill has been subtracted since gunbc#9477.
So hoisting into fill really does lower marginal cost, and you must say that plainly rather than
claim the work vanished. (2) Nothing appears above 500ms because the safety ceiling preempts there;
a preempted row's figure is a LOWER BOUND, which is exactly the class gunbc#10303 files as
fabricated plausible output. Every row in run 33818908502 has verdict_reached=true, so the figures
below are completions - but if a row of yours lands near the ceiling on a re-run, report it as a
bound, not a measurement.

SCOPE DISCIPLINE. Land the structural win. Do not chase a 105ms row to 98ms with a hack - the
near-100ms tail moves between runs. A row you cannot get under 100 is a FINDING with its reason,
not a failure: report it. No silent widening, no ceiling raise, no escape hatch.

DELIVERABLE. Read DESIGN.md first. One PR for the lane. Report: the root cause you found,
before/after per identity from the instrument, and any row left above the line with why. Do not
merge yourself - hand back to me (sunny-dove-686) and the operator merges.

YOUR IDENTITIES (cpu_ms, worst first; run 33818908502):
   352  v2.test.lens_testgen.shadow_ci_receipt.lens_testgen_shadow_ci_run_receipt_holds
   307  v2.test.lens_testgen.generator_provenance.lens_testgen_scheduled_generators_carry_provenance_holds
   262  v2.test.lens_test_migration_debt.test_migration_debt_test.live_populations_are_nonempty
   262  v2.test.claim.enforcement.lens_module_gate_witness.lens_module_gate_holds_live
   261  v2.test.claim.enforcement.lens_module_gate_witness.lens_closure_question_zero_live_matches_gate
   261  v2.test.claim.enforcement.lens_module_gate_witness.lens_closure_question_zero_holds_live
   252  v2.test.lens_registry.sg_claims.lens_registry_required_ids_resolve_holds
   230  v2.test.claim.rust_crate_partition_witness.witness_partition_perturbation_member_coverage_holds
   227  v2.test.lens_doc_reachability.doc_reachability_test.doc_graph_has_no_orphan_docs
   210  v2.test.lens_doc_reachability.doc_reachability_test.doc_graph_is_clean
   160  v2.test.claim.rust_crate_partition_witness.witness_v1_infer_unit_members_holds
   160  v2.test.claim.rust_crate_partition_witness.witness_v1_artifact_unit_members_holds
   159  v2.test.claim.rust_crate_partition_witness.witness_runtime_unit_members_holds
   159  v2.test.claim.rust_crate_partition_witness.witness_partition_fold_unit_count_holds
   151  v2.test.claim.enforcement.lens_module_gate_witness.question_zero_verdict_live_holds
   144  v2.test.lens_vacuity.vacuity_test.vacuity_falsification_suite_nonempty
   144  v2.test.lens_vacuity.vacuity_test.vacuity_falsification_cases_each_match_expected_evidence
   144  v2.test.lens_vacuity.vacuity_test.vacuity_advisory_census_partitions_falsification_suite
   141  v2.test.lens_vacuity.vacuity_test.vacuity_unified_node_corpus_matches_transport_classifier
   141  v2.test.lens_vacuity.vacuity_test.vacuity_rung5_emit_vs_eval_classifies_proven_independent
   135  test.claim.compiler_frontend_program_status_witness.the_report_renders
   127  test.claim.compiler_frontend_program_status_witness.xl0_is_not_derivable_and_awaits_the_repair_input_origin_producer
   127  test.claim.compiler_frontend_program_status_witness.every_milestone_startability_agrees_with_its_prerequisite_standings
   126  test.claim.compiler_frontend_program_status_witness.every_declared_instrument_is_distinct_and_awaited
   126  test.claim.compiler_frontend_program_status_witness.a_live_milestone_row_agrees_with_its_own_folds
   124  test.claim.compiler_frontend_program_status_witness.the_two_new_instruments_are_awaited_at_the_right_grain
   124  test.claim.compiler_frontend_program_status_witness.the_live_census_partitions_on_both_axes
   123  test.claim.compiler_frontend_program_status_witness.no_subject_is_scheduled_for_the_waves_and_xl5_is_not_derivable_from_the_empty_set
   115  v2.test.lens_application.serialize_applied_diff_swap_operands.serialize_applied_diff_loop_body_holds
