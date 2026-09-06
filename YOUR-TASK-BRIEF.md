LANE L8 - emit ingest round-trip (python/typescript) + produced_decl_two_target + transport/source-store. 7 modules, 21 identities, 5254ms CPU.

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
   414  v2.test.emit.produced_decl_two_target.produced_decl_two_targets_render_own_order
   412  v2.test.emit.produced_decl_two_target.produced_decl_module_folds_declarations_in_order
   351  v2.test.emit.produced_decl_two_target.produced_decl_unwired_target_still_refuses
   330  v2.test.manual.emit_source_store.emit_source_store_mutated_emitter_misses_holds
   330  v2.test.manual.emit_source_store.emit_source_store_cold_then_warm_holds
   291  v2.test.execution.emit_ingest_python_same_language_round_trip.emit_ingest_python_same_language_round_trip_holds
   276  v2.test.execution.emit_ingest_typescript_same_language_round_trip.typescript_same_language_parse_bridge_holds
   272  v2.test.manual.rust_add_emit_translate.rust_add_emit_add_fn_accepts_holds
   266  v2.test.manual.rust_add_emit_translate.rust_add_compile_inferred_receipt_holds
   262  v2.test.execution.emit_ingest_typescript_same_language_round_trip.emit_ingest_typescript_same_language_round_trip_holds
   251  v2.test.execution.emit_ingest_typescript_same_language_round_trip.typescript_same_language_source_emit_round_trip_holds
   246  v2.test.execution.emit_ingest_python_same_language_round_trip.python_same_language_source_emit_round_trip_holds
   238  v2.test.execution.emit_ingest_python_same_language_round_trip.python_same_language_parse_bridge_holds
   182  v2.test.execution.emit_on_demand_family_crate_witness.family_crate_member_change_cold_rebuild_holds
   170  v2.test.execution.emit_on_demand_family_crate_witness.family_crate_dispatch_change_cold_rebuild_holds
   168  v2.test.execution.emit_on_demand_family_crate_witness.family_crate_member_change_key_sensitivity_holds
   165  v2.test.execution.emit_on_demand_family_crate_witness.family_crate_one_build_members_warm_holds
   161  v2.test.execution.emit_host_transport_wire.emit_host_transport_green_run_holds
   160  v2.test.execution.emit_host_transport_wire.emit_host_transport_byte_width_mismatch_refuses_holds
   159  v2.test.execution.emit_host_transport_wire.emit_host_transport_nonzero_exit_refuses_holds
   150  v2.test.execution.emit_host_transport_wire.emit_host_transport_outside_grant_refuses_holds
