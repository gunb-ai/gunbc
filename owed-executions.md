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

## eval_cast class (lane adhoc-e72164c8-96a)
- LIVE: router_proxy_config_witness 1/9 at main (cannot cast Int to Port, invocation 3d6e6fb0). The production 503 as HttpStatus / 2000 as Milliseconds are masked. The EpochSecs cast is a latent outage on the operator-dispatched RLM launch-receipt job.
- UNOWNED: commit_writer_admission_witness 0/13, refused at LOAD (expected Product(PositiveInt), got Primitive(Int), at 8 sites). Pre-existing, outside the cast class.

## From fierce-lark-531's final report (2026-11-10)
- runner_host_deploy_witness 0/9: its own effect-summary errors (inv 918ff157). OWNED by gentle-ram-548 (adhoc-29d1e3bf) since 2026-11-10.
- commit_writer_admission_witness 0/13, refused at LOAD: nine Product(PositiveInt) mismatches (:202 through :595). OWNED by gentle-ram-548 (#16218). Once it loads, line 103 is expected red until zesty's #16224 drops the cast; either order is fine.
- Sweep quality: the v1_src_dag_parse test does not count a `| Arm` head, an `operation` or a `service` as a declaration. About 31 of the 312 reference rows are spurious for that reason; the rest is real work. The 398-line list is attached to #16200 (comments 6756695262, 6756696281).
- seed_builder_free_route_probe_witness is non-discriminating (2/2 at both legs).

## No detectors (sunny-bat-59, #16214)
- Lock staleness: `cargo metadata --locked` returns rc 0 even with a stale package. Only an unlocked step followed by `git diff --exit-code Cargo.lock` sees it.
- Witness load drift: no required job loads dag/test/claim/*.dag (v1_required_lanes_withdrawn). The cheap sweep shape is claim_batch --hermetic --wet --source-root dag --source-root src/v2 --entry <file> --functions <all>, one file per invocation. A corpus pass would find the rest of the load-refused witnesses.
- rustfmt drift: already repaired on main by #16183 (fmt --check rc 0 at a38e6eb9, invocation 7e9667fb). Hooks failing on older checkouts just need a main merge.

## fierce-lark-531 handoff (2026-11-10, lane closed)
- Landed: #16200 #16205 #16208 #16209 #16211 #16216. Post-merge composition 47/47 at fd28ef2f (BuildBuddy a5408368) and eefe616d (775fa819).
- UNOWNED: disposition_redundancy 0/25, refused at load on main.
- UNOWNED: the 398-line sweep list is a FLOOR (the sweep hit a 38-min timeout while emitting; ask quick-newt-573 for the tail). The list is form-blind, so triage before budgeting.
- NOT CLEARED: HttpStatus / Milliseconds casts were never observed red, because the files carrying them do not load.
- RISK for #16224: if the ARM-REFINE pre-empts assignment conformance on the same field, fierce-lark's 2 TypeMismatch rows drop to 0. The identity pin a_value_already_on_the_carrier_casts_by_identity must stay ADMITTED.

## tidy-otter-373 handoff (2026-11-10, #16212 merged as 06959167)
- Concern row branch_match_arm_closure_declared_return_not_admitted stays OPEN. Blocker (i): typed arm parameters do not parse (grammar). Blocker (iii): a wholly unannotated arm closure `fn(q) { p }` still settles open. Both UNOWNED.
- TRIGGER SATISFIED, UNOWNED: a return-annotated arm closure now reaches the emitter, so the fixture for the emitter's branch/match tail-closure position (move / Box::new) is writable. This may be the emitter third boundary already on this ledger; check before dispatching.

## Closed without a PR (2026-11-10)
- disposition_redundancy: already 22/22 at main 00cacc14 (BuildBuddy 18cd136c). #16205 and #16200 fixed it after the sweep's reading. The lens is live: registry, contract, and cli_run_floor_lens_oracle_witness. Closed by deep-swift-104.
- LOW, UNOWNED: v2.workflow.floor_grandfathered_roster still lists 6 deleted rows (redundancy_region_bmc_* and redundancy_region_rust_gates_*). The roadmap's removal protocol is a WitnessDeleted row in floor_grandfathered_removals. The roster's consumer is required_floor, which is v1-floor machinery that no required job runs, so this is low value.
- WATCH: where_refinement_enforcement_witness fails 41/48 at both main and #16224's head (7 rows return false with no diagnostic). Where-refinements are retired by ruling; the witness may be obsolete. Unowned.

## Field defaults (vivid-carp-423, 2026-11-10; #16232 4a4f790d + #16235 e1dd6b50 merged)
- Path 2 DONE: the STS exchange binds the declaring-scope Jwt/RequestAccessToken defaults; present vs omitted gives a byte-identical refusal (digest efb5659d72073e77).
- PENDING: a live non-vacuous heal-publish after e1dd6b50 must reach sts/iamcredentials/secretmanager (watcher hp-watch.sh).
- PARKED, UNMERGED: branch field-default-path2-jwt-default-binds @ 378a917a790 (off 4a4f790d). It holds (a) input_digest in the RestReplayExchangeAbsent reason and (b) field_default_jwt_path_binds_default.dag, a permanent hermetic witness for the default bind; (b) depends on (a). Ship it if a regression witness is wanted. If the dashboard auto-opens a PR for it, close it with a pointer here.
- UNOWNED: path 3 (the type layer's parameter_default_emissions types an omitted default in the call site's scope).
- gunbc run cannot resolve rest_exchange_resolution ("has no body"); these claims run only via claim_batch.

## Cast class closed (zesty-ferret-621; #16215 fe53b150 + #16224 9443e64f)
- DONE: v1 validate_cast refuses Int/Nat into numeric int-predicate refinements with the typed RefinementCastRefused, identity keyed on DeclarationRef. Fork 4/4; router_proxy_config 1/9 to 9/9.
- UNOWNED, v2 sibling: src/v2/compiler/infer.dag validate_cast may carry the same fork, UNTESTED. Trigger: a v2 witness shows it red.
- RESIDUE (class rung stays Mitigatable/BelowCeiling): non-int-predicate refinements (NonEmptyStr, PciDeviceId, Sha256DigestHex) are eval-decided; casts outside dag_cast_rules are eval-decided; eval_cast's one-table std.coercion trigger is unbuilt.
- BOOTSTRAP RECIPE (reusable): a new CompilerDiagnostic variant consumed by hand-Rust (compile_clean.rs) deadlocks heal with E0599. Break it with ONE operator-granted regen push: quick-newt-573 regenerates, blob shas are verified after base64 rc0 + gunzip rc0, and heal must reproduce it byte-identically (no bot commit).
- HEAL-PUBLISH DIAGNOSTIC DEFECT (low, unowned): run 42795658738 (10:01Z) refused on PR #16238, an auto-opened notes PR that was already CLOSED, with "HealPublishPreconditionUnmet: the pull request could not be read; status 200 undecodable". Refusing is right; the reason is wrong. The likely cause is the publisher's PR decoder not modelling a closed-PR response (unconfirmed), so a closed PR reads as an undecodable payload instead of a typed PrClosed decline. #16235 is still unproven live: the post-merge publish attempts were HeadMoved (42785725347) and this one; the last successes were 08:22 and 08:31, before #16235.

## Heal-publish outage (2026-11-10 08:45Z to 14:43Z): #16241 MERGED 4aab999d
- Root: #16156 (5e315131) dropped the wire-contract rows from the demanded realization, so every PR read refused 'status 200 undecodable'. Fixed by a declared reader family in realization_names/realization_row_edges. Granted mirror push msg_8e2130c3; heal reproduced it.
- FALSIFIABLE: the first non-vacuous heal-publish after 4aab999d must read a PR successfully; otherwise revert #16156 the same hour (watcher hp-watch2 since 14:43:30Z).
- Follow-ups: (B) emitter arms in 05_emit_rust (cool-tern-790, dispatched). (C) the redaction witness should assert against the real renderer, not a recorded string. (D) stale field_path sentence at dag/gunbc/declared_side_refusal_cause_seed_growth.dag:24-25 (the wording is in the PR comment). (E) a name-resolved shape test for the contract-row recognition rule (review 93104 note). (C/D/E UNOWNED.) #16156's second instance (service-op registry): quiet-wren-507.
- Lesson: prose receipts inside .dag string rows broke the parse twice (an unescaped double quote). Check-parse a single file before pushing (gunbc compile --source-dir, 20 ms).
- #16249 (b4588c88) restored integration_concern_witness to 13/13. UNOWNED: SightingEvidence has no digest-less / run-tree evidence variant (capability gap, review note). The class: no CI job loads the ledger witness, so #16212 broke it silently; a gate would need operator sign-off (job roster closed to growth).
- OUTAGE CLOSED, live: heal-publish 42844365342 decoded PR #16250 (PullRequestNotOpen comes only from PublishPrObserved). No #16156 revert. A full PullRequestOpenCheckPassed publish is not yet observed (next: a live candidate on #16122).
- (E) detail: realization_wire_contract_read_row recognises rows by the leaf 'CoproductWireContract' plus field-init names, so an aliased or renamed row would reproduce the outage; the audit edge is the backstop, and name-resolved matching is the next rung.
- MEASURED, do not re-derive: the scoped gunbc run route has NO REST-replay realization at any commit (BB 3d8b1298), so scoped entries cannot RED/GREEN reader-seeded gaps. Only the demanded grain (realization_of/OnlyDeclarations, cli_run/demand_acquire.rs) shows RED; claim_batch is green everywhere.

## Emitter third boundary (valiant-carp-593; #16240 495211eb)
- DONE: the branch leg; TargetConditionalShape gains arm delimiters (Rust no longer prints `if c { a else b }`).
- UNOWNED (on concern row branch_match_arm_closure_declared_return_not_admitted): (1) production Rust match token wiring (flips the closure_emission match pins); (2) function-type realization SELECTION at an item return with capturing arms (boxed vs impl; E0308); (3) a branch/match inside a returned closure BODY refuses upstream.
- UNOWNED, small: rust_emit_small_conditional_target lacks a ^rust_token_kw_else lexeme, so production_rust_conditional_declares_a_close can never pass; emit/conditional_delimiting also refuses to load at main (dag/std/determinism.dag:6 unresolved DeclarationRef).
- ENV: ctrl worktrees are dangling (.git -> /home/briansrls/ctrl/.git/worktrees/<id>, which does not exist). Seen by tidy-otter, fierce-lark and valiant-carp.

## 2026-11-11
- #16273 (v1-compiler-tests compiles again after #16069's Rc signatures). The suite's first run in a while: 79 tests, 68 ok, 5 FAILED (pre-existing, not Rc-related; listed in the PR body), 2 time out at >35 min solo, 4 unrun (BuildBuddy outage). The suite hangs under default threads (shared-state contention). UNOWNED: the 5 failures, the hang, and NO CI job builds v1-compiler-tests (a gate needs the operator).
- #16143 CLOSED as superseded by #16122 (caeac322). Residuals: the 54/54 reading at main (bold-pike-241, pending BuildBuddy); concern row qualified_corpus_reference_minted_as_kernel (#16290, awaiting a load check).
- v1-compiler-tests, the 5 pre-existing FAILs (calm-seal-225 handoff): ambiguous_prefix_infix_newline_test::explicit_subtraction_and_unary_statement_keep_their_values; helpers::tests::resolver_imports_ephemeral_generated_source_root (possibly the /tmp checkout environment); native_driver_cost_refusal_test::emitted_driver_refuses_over_attribution_instead_of_clamping_and_fails_the_process and ::the_preparation_span_closes_before_any_declaration_is_evaluated (emitter-output expectation); whole_tree_wiring_enum_test::the_named_subject_decides_the_population_and_the_context_does_not (eval_fn_arrow_decl_facts_live now takes pool_files). Timeouts (>35 min solo): field_of_fractions_pair_stays_boxed_record_not_native_collapse; record_shaped_map_insert_is_located_type_error. UNOWNED.

## v2 cast fork (witty-moth-273; #16259 e4722ea7)
- DONE: the evaluator admits a refined value via its recorded carrier (InferredFacts.refinement_carrier) and coercion_cast_exact_witness. 11/11.
- UNOWNED: (1) the identity grounding leg: `cecf_src as Port` refused at the cast root with eval_rejected_grounding_not_derived, while the deep probe reads it as derived. Two reads of one tree disagree; first find which read the production claim uses. (2) A typed v2 refusal reason for the population the checker already refuses (the #16224 analogue).
- RESIDUAL RISK, worth a look: the evaluator gate now admits ANY value stamped with the refinement's carrier. It cannot tell a discharged Port from a raw Int, so predicate enforcement rests entirely on the checker's discharge_refinement_obligations. Before, the gate refused every refined value, by accident. If any route reaches a refinement position unchecked, it is now silent. Candidate climb: carry a discharge witness on the value.

## main_wet_one consumer break (2026-11-11, bold-wren-111's srv1 probes GATE_WET4/5 + quiet-wren-507)
- Root: ledger row a_field_default_is_evaluated_in_the_callers_scope.dag:36 had an UNESCAPED `{ service: "github.Pulls" }` inside a receipts string, so string interpolation evaluated github.Pulls as a variable (NoSuchVariable). #16217 had escaped the OTHER brace and missed this one. Fixed as a data commit in #16279.
- CLASS (record as an RFM row): prose inside .dag string rows breaks far from its cause. Twice now: #16241's rung-drop quote (a parse error) and this interpolation brace (a runtime throw in an unrelated job). Candidate climb: a lint that refuses an unescaped { in a String row whose interior cannot form an expression, or a typed carrier for receipts.
- UNOWNED (dispatch once BuildBuddy recovers): build_initial_env (v1_interpreter.rs ~8100) eagerly evaluates EVERY realized DataItem, with no demand filter. Under the declaration grain that pulls in ledger rows the entry never reads (cost plus an error surface). Fix: evaluate only demanded rows, or make DataItem bodies lazy.
