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

## BATCH 1 RESULT: main @ 8ca9a3a4 (quick-newt-573, msg_603b3f87)
- PASS: condition_position_record_parse 15/15, nested_arm_pattern_position 2/2, service_operation_call 5/5, lex_prefix_door 10/10, lex_window_beyond 17/17, openai_text_completion_wire 3/3, fabric_terminal_contract 17/17, harness_shell_job_policy 8/8, optional_lowered_position_projection 8/8, github_effect_perform_redaction 12/12, heal_publication_refusal_redaction 3/3.
- REFUSED at load: harness_shell_job (+tool_route), because of `list_length` (from #16020); service_operation_unrealizable, because of `concat` (from #15894). Repair lane: adhoc-2968bab0-fd3.
- FALSE rows: host_effect_apply 3, closure_emission 2, service_realization_lowering 1, printable_assembly_selection 1, harness_tool_kind 2, occurrence_file_attribution 1. Repair lane: adhoc-2968bab0-fd3.
- UNOBSERVED (needs wet): durable_attempt_controller 16 rows (DirWithTemplate). NOT RUN: the four *_wet witnesses.
- LESSON: archived branches assumed names (list_length, concat) that main does not provide. Seed compile plus file-overlap checks cannot see this; only a claim run can.

## BATCH 1 RESULT: continuation stack tip 1f18312b (quick-newt-573, msg_f2bd22a1)
- PASS (all claims) in 11 modules: claude_code_dispatch 22, credential_snapshot_seal 7, supervisor_guidance 4, throughput 22, worker_turn_dispatch 4, append_log_store_seal 7, dispatch_replay 8, exit_spool 27, attempt_occurrence 8, dispatch_actuator 58, result_returned 20.
- Refused by MAIN's list_length (#16020): 8 modules, 267 claims unrun. Waits on adhoc-2968bab0-fd3.
- Stack-own load refusals (4): transport_retry (Int vs Second), roadmap_alignment (declared_roadmap_nodes identity), attempt_continuation (missing verification_retry), verification_evidence_addressing (wakes_at). Routed to smart-ferret-84.
- FALSE rows: belt_actuate 2 (fix 6ec056ba), component_dispatch_button 1, launch_admission 1.
- INCOMPLETE: durable_append_log 25/31; the_raw_events_initializer_is_gone hangs or runs very slowly at the 45-minute limit.
- UNOBSERVED/wet: append_log_srv1_host_shape 4, belt_actuate 9, and the four *_wet witnesses.
- LATENT on main (from the #15993 skip): v1 infer has two disagreeing inline copies of the fold-carrier rule. In the seed mirror, refined_call_fold_acc_type carries the generic test and refined_fold_acc_type does not. #15993's fixtures do not go red on main. Wanted: a fresh, fixture-first PR (a fixture red on main first, then the unification), with its claim run at a head that carries the heal-published mirror.

## CLAIM-CORPUS DRIFT on main (pending one lane, after #15363's reason read-out)
- statement_if_lowering: REFUSED at load, `:120` Node vs NodeOccurrence. Same class as #15363's `:231` (a.at -> a.at.node).
- parse_test_else_less_if_value_category: REFUSED at load (effect-summary/tokenize callee identity).
- return_tail_position: 3 PASS / 14 FAIL; fixtures resolve-refused upstream.
- let_match_early_return: 12/18 false; same resolve-refused pattern as #15363's accepted controls (the reason lies outside every resolve/infer reason symbol).
- #15868 CLOSED, not restored (2026-11-09). The archived head ffd4e4ec's claim files never compiled (inv 93657c61), and its own code turns main-green infer_declared_return_inhabitance_witness::declared_return_list_of_bool_with_an_int_list_body_refuses_holds red (inv 6d869732). The argument-path verdict machinery (~656 lines of production change) is unexecuted, scoring 4/15 at the call-shape-fixed head 8770fe01. The closed-sum reason naming is re-authoring, not a subtraction from that diff, and nothing is filed for it. Archive: pr-15868 plus branch v2/inhabitance-decision-reasons @ 8770fe01.

## CONTINUATION STACK LANDED: #15804 (39d43604), 2026-11-09
- Owed-wet: the real supervised session (verdict written, read back, acted on; needs a stub backend of scripted tool calls, which does not exist yet); 25 UNOBSERVED rows (belt_actuate 9, operator_close_retire 15, roadmap_serve 1); the *_wet stack modules, including harness_events_lost_update_wet_witness.
- COST DEFECT on main: durable_append_log_witness::the_raw_events_initializer_is_gone exceeds a 600 s CPU budget at about 4,100 eval steps, so the time is inside a builtin (the nested in-process compile of deleted_callable_route_is_gone).
- Declared frontier: 0b-2, the wake that fires at redeliver_at. Not built.

## MORE LOAD-RED ON MAIN (witty-wolf-241 handoff, 2026-11-09)
- compiler/callback_argument_formal and compiler/function_value_type: both refused at load by a non-exhaustive match in src/v2/test/claim/body_let_annotation.dag (missing the GroundingNoValue arm).
- compiler/staging: refused at load, unresolved LiveTreeDisposition.
- emit/bind_tail_is_an_expression: refused at load by the LocusAnchor equality wall (#15992, #16143).
- body_lowering/data_initializer_fn_value: 0/3 at every leg.
- The emitter third boundary (branch/match tail closure): lane dispatched 2026-11-09.
- Fleet flake: runs that die at ~50 s with "workflow file issue" and no steps are the fleet, not the diff; re-run them.

## EMITTER THIRD BOUNDARY (masked; recorded in concern row branch_match_arm_closure_declared_return_not_admitted, #16195)
- Becomes testable once any upstream blocker admits the shape. Blocker (ii), inhabitance, is being worked in lane adhoc-b66614dd-46c.
- The fix is mapped (cool-newt-294): thread the FunctionTypeAtItemReturn position through target_project_branch_bool_if_else and the match projector to target_project_realized_closure, at BOTH item-body routes. In target_serialize project_arrow_body_from_inferred_facts, add ComputationBody{Branch}/{Match} arms mirroring #16180's Bind arm. In target_model target_project_arrow_body_by_kind, pass Present(FunctionTypeAtItemReturn) at the Branch/Match arms. Fixture-first; rustc is the referee.

## FIELD-DEFAULT SCOPE (#16196 landed, class FoundBelowTheLadder, per-path)
- NEXT (priority 1): rest_exchange_selection in src/v1/stage0/src/v1_interpreter.rs turns RestReplayExchangeAbsent{invocation} into a bare TypeError and DROPS the bound invocation. Surface it (render invocation.target into the refusal, or return a typed refusal a witness can read). The zero-fixture op-route probe then shows path 2's bound value, after which #16185's explicit fields become removable. Do NOT hand-key a fixture (rest_bound_invocation_eq is exact on host-computed input_digest).
- Path 3: the type layer's parameter_default_emissions still types an omitted default in the call site's scope (silent). It needs its own claim.
- The four claim cells are not enrolled in any lane (path 1's limit).
- dag/test/claim/github_workflow_runs_witness.dag: 0/27, refused at load (LiveTreeDisposition, SubstrateInputsOnly, WorkflowRunStatus/Conclusion arm imports).
