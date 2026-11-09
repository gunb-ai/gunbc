# Owed executions: claims that landed on gunbc main without having run

Maintained by silent-bee-845, the daily integration manager. CI's required job (v2-bootstrap) only compiles the seed and runs the two emitted closures; it executes no `.dag` witness and no v1/stage0 unit test. Every item below landed on review plus a seed compile, and it is owed one real execution once an executor is named.

## Continuation stack (not landed; blocked on an executor)
- #15784 -> #15788 -> #15943 -> #15791 -> #15792 -> #15804: no witness has ever executed. #15791 holds REQUEST_CHANGES (review 92187) that only an executed run clears. #15784 still owes a supervisor inhabitance claim and a CLI verdict witness (reviews 91801, 91883).

## Landed restores (2026-11-09)
- #16125 (50f0644d): test.claim.host.host_effect_apply_witness. It is offline-local-only (excluded from CI enumeration). Its concern row host_effect_apply_witness_import_block_absent closes only when a scheduled route executes it.
- #16038 (9193e5fc): all 17 durable_attempt_controller claims, the attempt_lifecycle witnesses, and both terminal controls. attempt_lifecycle_wet_witness is excluded as LocalRepoWetLane, so the stale-owner-after-transfer CAS control has never run anywhere.
- #16020 (2f11e50b): harness_shell_job_tool_route, plus both wet witnesses (spawn/hold/reap/group-kill/backstop). The tool-route header cites "main seed refuses host_effect" as its NO_OBSERVATION trigger. That is STALE after #16099 and #16125: re-run before trusting or deleting it.
- #15710 (d65a1941): the stage0 Rust tests (record refusal before any request; URL credential-position refusal) and both redaction witnesses (heal_publication_refusal_redaction_witness; the six fake-secret claims in github_effect_perform_witness).
- #15648 (6498ee29): the pattern_binder_capture_native label (a plain-path regression guard only). The rc-ref branch rung is unclaimed; a recursive or shared-type fixture is owed once both legs can be built.
- #15687 (c5ec5901): the closure_emission and nested_arm_pattern_position claims (receipts were srv2 runs at the archived head and have not been re-executed).
- #15614 (b36c11f3): the touched claims and the witnesses of the repointed modules.
- #15567 (260cd3a1): printable_assembly_selection_witness, duct_card_catalog_witness, gpu_duct_witness, card_cooling_route_witness.
- #16042 (a2dddbe7): the substring seam is inert until heal publishes the primitive_substring HostRealizedSeam row from main.
- #15583 (6ea348f7): chunk-record optionality needs a real captured stream.
- #15894 (2e18b541): the service_operation_unrealizable claim.
- #15695 (7606bd82): the condition_position_record_parse claims.
- #15560 (e923d5c4): heal was RED at landing (two-generation retirement; operator default, msg_76ed4315). The next PR that touches generation inputs must heal green.

## Deferred by authors (recorded, not dropped)
- #16038: ControllerCapacityWaiting/Cancel silent overwrite; DurableHoldSubject's two wire forms; the witness lease() helper bypasses LeaseNotObserved. Remote-build PR 4/5 are tracked by concern controller_executor_fence_handoff.
- #16020: thread-bind admission (d080f31e) waits on #15804; hand-in coupling (main has no HandInTool).
- #15710: gh-api stderr passthrough; raw-body sites heal_publisher_provision::rest_refusal_text and printer_credential_migration (left alone).

## Added 2026-11-09 (after batch 1 was requested)
- #15876 (97138503): its claim modules (else-less-if statement lowering).
- #15894 (2e18b541): src/v2/test/claim/service_operation_unrealizable.dag (if batch 1 did not cover it).
- #16124 (9ab54645): dag/gunbc/live_deploy/user_work_slice_admission_witness.dag. CI typechecks only std.measure here, NOT slice_bounds, roadmap_belt_actuate or the witness, so a green run proves nothing about those types. HOST ACTION (the operator's): re-activate the srv1 belt so the gate goes live. The drain check plus `systemctl --user restart` are in the #16124 body; run them only with 0 review/audit workers in flight.
- Continuation stack: #16124 changed roadmap_belt_actuate.dag, which the stack's roadmap_belt_actuate_witness exercises. The stack's next main sync must re-run that witness against #16124.

## RED ON MAIN, found by execution (2026-11-09)
- v2.test.claim.namespace_xl0.let_match_early_return at main b36c11f3: 12 of 18 rows false, with no diagnostic (6 pass). It predates the restores; the guess is that expectations drifted as body lowering changed. Needs its own work item.
- branch_slot_readers.dag:268 refuses at LOAD (equality admission for generic LocusAnchor; v1.compiler.infer equality walk; concern equality_wall_three_site_measure_stopgap_interim; root #15992). It is the same at #16139's head and base. Suspected: every claim file that writes `== node_locus(...)` (at least 8 on main) is unloadable. A load-only check is pending on quick-newt-573.
- Class evidence for test_claim_module_is_not_compiled_by_required_ci: #15363's new claim module passed two CI cycles with a headless Cons, then with a Node/NodeOccurrence type error.
- SILENT MISS on main (found during the #15607 skip): seed keyed_decl_facts (src/v1/stage0/src/coproduct_reflection.rs, from #15930) uses `if let Some(rows) = decl_facts_of_source(...)`, so an unparseable derived file contributes nothing without any refusal. Archived #15607 carried a typed-refusal layer (DeclFactsKeyedRefusal) for exactly this, but no longer applies; it needs a new change.
