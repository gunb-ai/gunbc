LANE L5 - v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract + v2.test.claim.wrap_decision_predicate. 2 modules, 29 identities, 5160ms CPU.

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
   201  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_btree_set_debug_contract_requires_ord_from_im_ord_set_authority
   200  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_vec_unmodeled_ord_contract_refuses_as_missing
   200  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_vec_serialize_contract_requires_clone_from_im_vector_authority
   200  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_btree_set_serialize_contract_requires_ord_and_clone_from_im_ord_set_authority
   200  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_btree_set_deserialize_contract_requires_ord_and_clone_from_im_ord_set_authority
   199  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_vec_partial_eq_contract_requires_clone_from_im_vector_authority
   199  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_vec_debug_contract_requires_clone_from_im_vector_authority
   199  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_hash_set_unrepresentable_hash_contracts_refuse_with_typed_blockers
   198  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_vec_target_bundle_round_trip_preserves_im_vector_policy
   198  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_vec_deserialize_contract_requires_clone_from_im_vector_authority
   198  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_vec_clone_contract_remains_explicit_empty_from_im_vector_authority
   198  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_set_target_bundle_round_trip_preserves_both_production_policies
   198  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_hash_set_serialize_contract_is_known_zero_after_bundle_decode
   198  v2.test.claim.emit.trait_derive_supplemental_generic_bound_contract.rust_btree_set_partial_eq_contract_requires_ord_from_im_ord_set_authority
   171  v2.test.claim.wrap_decision_predicate.wrap_decision_instantiation_arg_is_wrapped
   163  v2.test.claim.wrap_decision_predicate.wrap_decision_flow_agreeing_sites_accept
   163  v2.test.claim.wrap_decision_predicate.wrap_decision_diagnostics_return_is_rc
   162  v2.test.claim.wrap_decision_predicate.wrap_decision_flow_over_wrap_direction_refuses
   162  v2.test.claim.wrap_decision_predicate.wrap_decision_diagnostics_param_is_owned
   161  v2.test.claim.wrap_decision_predicate.wrap_decision_flow_under_wrap_direction_refuses
   161  v2.test.claim.wrap_decision_predicate.wrap_decision_flow_missing_row_refuses
   161  v2.test.claim.wrap_decision_predicate.wrap_decision_flow_distinct_reference_layers_refuse
   159  v2.test.claim.wrap_decision_predicate.wrap_decision_node_struct_field_is_box
   158  v2.test.claim.wrap_decision_predicate.wrap_decision_probe_heap_param_miss_rejects
   157  v2.test.claim.wrap_decision_predicate.wrap_decision_use_site_verdict_param_is_owned
   154  v2.test.claim.wrap_decision_predicate.wrap_decision_use_site_verdict_return_is_owned
   149  v2.test.claim.wrap_decision_predicate.wrap_decision_bundle_partial_rejects
   149  v2.test.claim.wrap_decision_predicate.wrap_decision_bundle_absent_inapplicable
   144  v2.test.claim.wrap_decision_predicate.wrap_decision_flow_bundle_absent_inapplicable
