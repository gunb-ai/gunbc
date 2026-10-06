# Next-few-days notepad (from #12787 ROADMAP recut, 2026-10-02)

Source: `R.md` (ROADMAP.md @ roadmap/land-to-main). Landing state: neat-boar-16 reply 2026-10-02 (v2 Foundation tree). Raw: neat-boar-16.txt

## Tracks (roadmap umbrella rows → candidate milestones)

### A. v2 reads/names/types its own source  (R.md l.70–90)
- [ ] Literals elaborate (MQ track, #12740) → v2 parses+types its 7 native test modules
- [ ] Lets/match binders by the one relation → realization-per-target
- [ ] Generic record typed from expected type
- [ ] One definition each: Int/Nat/Bool/List/String (#12583, #12526, #12760, #12809)
- [ ] Every reference carries its declaration (#12612/#12635) → cross-module refs (#12726 PR2)
- landed 48h: one-relation walls #12770 #12896 #12902 #12766 #12841 #12919 + others; namespace provider #12735/#12726; occurrence #12790 (97/110 modules; 13 = resolve fails, String ambiguity), #12913/#12914; MQ literals #12862 #12879 #12926 #12910 #12928; kernel de-forks Float #12547, List carrier #12569/#12603, String #12760
- queue 1-3d: #12799 EdgeLabel -> #12942 MQ-5 cut; #12512 text wall; #12846 Nat (conflicts, mid regen); #12559 -> #12583 Bool; #12935 checking-mode plan; #12526 List cut; std.string_type deletion
- NOT landing / UNOWNED: native route passes 0 tests (5121 refused, 4870 at prepare, mostly resolve_reason_unbound_symbol) -> blocks "7 native test modules" + native gating; type_env PR-2 6.3x slowdown unexplained; silent wrongs: record field admits distinct product (RFM), Int->Nat implicit crossing; parse_grammar_choice_overlap_residue (ruling: v2 grammar authoritative)
- dispatch candidates: ** native-route unbound_symbol census+cut ** ; RFM silent wrong ; Int->Nat crossing ; type_env PR-2 slowdown derivation

### B. v2 emits Rust, then rebuilds itself  (l.21–27, 91–95)
- [ ] One real module emitted by v2 passes its own tests
- [ ] Crate split decision → built v2 CLI emits v2.compiler.compile → gen2/gen3
- [ ] Emitter produces main.rs
- [ ] Native-frontier ratchet on every landing → production admission
- landing status: ?
- dispatch candidates: ?

### C. Namespace cut  (l.13–16, 57, 84–88, 133)
- [ ] Per-file visibility → reference-derived deps (XL-4) → ambiguity settlement → rewrite waves (ACT-0/XL-5)
- [ ] Import list binds? (duplicated at l.85 and l.133 — check)
- landing status: ?

### D. Demand engine  (l.74, 96–104)
- [x?] D13 b1 DependencyDemand reducer — landed #12985
- [ ] M1.b keys → M1.c cross-run hits (#12596) → M2 …
- landing status: ?

### E. CI / floor cost & honesty  (l.18–20, 105–111, 127–132, 168–171)
- landed: #12890 (-6GB peak) #12916 #12861 #12944 #12978 + floor/CI batch; DP-M4 #12610 (no roster)
- BOTTLENECK: #12996 (queue pos 1) -> #12980 (NFR roster) unblocks #12799 #12559 #12512 #12846 #12951
- #12885 41G floor slot class: DIRTY vs #12891, needs manual merge + operator converge srv1/3/4 (std-root PRs need ~34GB vs 25GiB)
- #12951 deep-ferret-305 43-PR integration: needs #12980, #12514 srv1 replays, regen
- WATCH: #12969 merged 13:25Z with floor FAILING — check main
- [ ] Floor memory peak / pool_parse attribution / floor affordable as gate
- [ ] Required run with failed phase must not report success / exit zero (two rows — likely one)
- [ ] Cheap deletions: 21 dead expected-red rows, 2 no-op exclusions, measurement_bankruptcy drop, grandfathered removals
- [ ] v1-compiler unit tests block again (#12753)
- landing status: ?

### F. Factory / harness  (l.52–56, 114–125)
- [ ] Lands with #12787 itself (checkpoints, prompt context, conversation log)
- [ ] F1–F6 are DRAFT pending contracts doc — not dispatchable yet?

### G. Seed deletion / shell-to-typed  (l.28–47, 63–68)
- [ ] DP-M1..M6
- [ ] small typed-argv rows (cron, fmt gate, ShowToplevel x2, systemd install, live_deploy…) — good small dispatches

### H. Fleet / crypto / encoding  (l.135–164)
- parked unless prioritized?

## Days
- Day 1 (10-02/03):
- Day 2:
- Day 3:

## Dispatch queue (draft)
| # | roadmap row | brief summary | intricacy/volume | depends on |
|---|---|---|---|---|

## Native 7 (//v2/test/parse/expression_bodied_fn_decl_parse:all, native) — neat-boar-16 16:3xZ
- No current record. Last (#12628 body, eager-newt-412, DO NOT MERGE): all 7 RED at resolve, `resolve_reason_unbound_symbol @ tokenize`; later stop at EMISSION: if-branches Primitive(Token) vs Coproduct(Optional) in src/v2/compiler/02_parse.dag
- Fixed since: 01_tokenize ingest (#12714), caret refusal (#12420/#12421)
- Fresh srv1 run at current main started (~40 min) — per-test stage+cause pending
- Movers: #12799 + declared-scope cut (quiet-hawk-702 plan, UNSTARTED, owner archived) for the tokenize unbound; #12726 PR2 (gated on #12506)
- Unowned: declared-scope cut; #12506 landing (conflicts w/ infer); emission if-branch Optional-vs-Token typing

### Native 7 — research agent (cited from #12628 comments; nothing run on main 1a013596270)
- Subject = ONE module v2.test.parse.expression_bodied_fn_decl_parse, now 8 test fns (#12686 added one). Roadmap row text not on main yet.
- Walls: W1 tokenize ingest CLOSED #12714 | W2 artifact.tree projection on match binder: fix on #12506 (OPEN) | W3a `found` unbound: closed on lane branch? | W3b `e` unbound CLOSED #12550 | W3c binder_hides_visible_value @ synthetic `found`: LIVE (fold carrier reuses authored step-param name) | run9 import_target_file_refused on 02_parse.dag: unknown if current | NEXT infer: Outcome<ParseArtifact> not admitted by canonical_grounding_admits_infer_facts (#12506 handoff)
- CONFLICT with neat-boar-16's "stopped at EMISSION Token vs Optional" — srv1 run settles it
- 1-3d PRs: none target live walls. #12583 Bool may perturb (module imports v2.std.logic Bool). #12935 = checking-mode (infer, later).
- Work: (1) fold carrier identity in body_lowering_fold — mint unauthorable carrier, relate by role (2) land #12506 (3) infer grounding of instantiated generic (4) checking mode impl (5) recheck ingest of closure (02_parse, std/types map literals)
- No rung drops (#12628 rule: green through frontier ≠ coverage)
- Repro: gunbc test //v2/test/parse/expression_bodied_fn_decl_parse:all native ≈19 min (+7 rebuild); 95% is the 134-module ingest context

### Native 7 — CORRECTED by operator review (5 runs today on #12506 lane merged with main) — supersedes the two blocks above
- All 8 refuse at PREPARE: infer_reason_projection_receiver_declares_no_fields on `e.target` (e = fold step binder). Root: nothing types the step member from the Loop domain element; ^loop_domain_edge has a producer in fold_lowering and no reader. RFM infer_child_context_cannot_depend_on_a_sibling_result. Resolve exonerated.
- W3c already fixed: fresh_fold_carrier 2bd1f0ded41 on #12506 (not on main). #12506 is ACTIVELY owned (not unowned).
- Regression: W2 artifact.tree no longer infers — #12766 froze infer_parameter_scope_search to lambda params; match-arm binder is neither.
- Eval: arrow_body_admits_eval_entry ParameterReferenceBody => false refuses before binding (two producers of ^eval_rejected_parameter_reference_unbound). On main too.
- 6 file refusals (constant, driver-rows.jsonl): filesystem_io, uri_path, machine_constraints, occurrence_identity, std/types, body_lowering_fold (2 parse_g0_tokens_remain); admission: unattributed_exclusions_present.
- Cost: 19 min correct; loading ~18% (context_nanos 201–216s), emit ~5 min + crate cargo build ~7–8 min.
- Actions: loyal-lark-119 (W3c) CANCELLED; tidy-raven-393 REDIRECTED → six file refusals.
- OPEN: owner for Loop domain-element typing + match-binder projection + eval entry gate — #12506 owner or new lane?

## #13018 flip recipe (swift-koi-30 handoff)
Trigger: advisory steps 2-4 landed. Then: merge main into #13018, regenerate witnesses.yml:
  gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=.github/workflows/witnesses.yml
then mark ready, side-chat, queue. Notes: census row for emit-build restates 2 entry paths from Rust constants in native_lane_runner (second spelling; move together). Next for v2_native_route_off_the_merge_path: emit-build runs //gunbc/instruments:v2-native-frontier.

## XL-2 3b-ii binding ruling (quiet-seal-543), if picked up
Transport templates in the realization sibling become ArgvTemplate = Template<OperationInputName> (#12987); each hole is resolved against the op's declared inputs AT LOWERING; an unknown name refuses, located. NO value-expression holes / ^dag_string_template in the sibling. This OVERRIDES eager-heron-413's handoff plan (keep template nodes, resolve treats them as structure). Raise before building if unworkable.
Order: interp infer (eager-koi WIP session/eager-koi-296-infer-self) -> hole-type census (~90 files) -> PR2b lexer+template+lowering -> 3b-ii -> 3c. Prereqs: #12949, #12974, 3b-i.

## N7 closure file refusals (tidy-raven census, durable table on #13051 comment 5969138214)
Parse refusals in the N7 closure: body_lowering_fold (ROOT E, keyword binders; tidy-raven after #13099/#13056) and machine_constraints (ROOT B, #13038). filesystem_io = ROOT D (XL-2, operator). node/uri_path fixed (#13010). 00_compile NOT in the N7 closure.
Not N7: 23 argv [..] in service transport blocks (XL-2), 1 index expr, about 22 service string paths, where range (refinement deletion), 7 let positions (eager-crab-610), match guard (no guard slot in v2 match_arm; grammar lanes), 9 unattributed.

## Gotchas (tidy-wolf-843 handoff)
- Stage0 regen can't run on BuildBuddy (HostBudgetUnreadable, no memory cgroup); CI auto-heal skips stage0 mirrors by design. Needs srv1 under systemd-run MemoryMax from a private clone (or git merge-file over the 3 stages, then let the generated lane confirm). Root fix: child B of silent-stag-648's lane.
- Editing any file with ActiveDebt rows in floor_unimported_bare_provider_debt_roster trips the touched-file gate: add the named imports, retire the stale rows as Retired{ImportsFixed}.
- #13017 behavior note: discovery-corpus resolves now count in judged-module-identities (outside-subject count drops); no refusal change.

## Root F landed (#13118, nimble-moth-144 handoff) — open items
- 4 warm floor rows (lme_outcomes, lme_shape, lmee_exit_path, lmee_binding_path): royal-deer drops them at #13043; lmee needs one SingleClaimFillDebtModule row (royal-deer).
- lmee eval rows assert today's refusal; flip to lmee_both_execute_to(...,7/6) when infer grounds a branch over a parameter (= LATER WALL). Coproduct scrutinee waits on infer_match_scrutinee_not_bool.
- 2+ binding arms refuse: join point blocked on lambda_value_has_no_derived_type_in_v2_infer.
- uri_path whole-corpus N7 re-measure: tidy-raven-393.

## N7 re-measure (neat-raven-383, #13151 head, 2026-10-04 ~02:30Z)
0/8, but the wall MOVED: all 8 refuse at PREPARE, cause=infer_reason_projection_receiver_declaration_unavailable <- infer_grounding_not_derived <- inhabitance_undecidable_argument_type. The text link is gone. The refused files are unrelated: filesystem_io (D), uri_path (F; head predates #13118?), machine_constraints (B #13038), body_lowering_fold (E #13168), service_realization (parse_g0). -> Routed to stern-swift-290's generic-formal infer wall to confirm whether it is the same arm.

## HELD: swift-lynx-62's 24-witness restructure (#12506 claims, §3 rule): pushed at session/swift-lynx-62 1067a42aa2 (6 files). Dispatch a new child to open its PR when #13043 lands; check the child is NOT on gmail1.
  Handoff (swift-lynx-62): verify branch swift-lynx-62-verify @ bea297a7e8 = branch + #13043 head + 22 retirements. After #13043: merge main; port retirements (RestructuredPerWitnessRule: cref 6, fps 10, dre bool_fn/same_leaf/invalid_arg/unresolved_sig; BecameSharedByDemand: mbt underived_arm_body, sevens_call; KEEP ActiveFillDebt mbt_a_binder_hiding..., dre_an_imported_reference_grounds...). PR body: vacuous claims made real; 3 new resolve-half claims; unique steps 12.6M->7.1M; CI runs 37149874428 / 37153453947 / 37153455406. Gotcha: main's hand roster names 10 removed fns, so CI refuses until #13043. Probe branches are throwaway.

## Stamp coverage (loyal-tern-472, #13164): no mirror moved outside its cut. Broker census: most fn_signature_param/return rows still resolve by declaring_span (1,538+1,498) or leaf_spelling (423+219), not by the Node.declaration stamp. Either infer replaces the stamped signature node, or the walk skips a declaration node. UNCHASED: a coverage question for text_carrier_render_not_keyed_on_identity_stall program 1 (stamp on every type reference). Route when that program is picked up.

## OPEN: program_assembly fixed cost ~71k eval steps per assemble_program_from_ingest, independent of source text (neat-seal-675, #13185). Needs an owner (§6b chain re-derivation: what fixed work repeats per call). The program_assembly contract is off the floor until then.

## PARKED: (b) cargo download-EXHAUSTION infra signature (CargoNetworkDownloadFailed), keyed on cargo's terminal download error text, not the retry text. Waits on a real log specimen. From swift-lynx-592 (closed after #13181).
  swift-lynx-592 handoff: landed #13002 #13005 #13094 #13159 #13007 #13181. Parked: (b) above; offered: run_cargo --message-format=json (replaces stderr scraping); unmeasured: positioned file_refusal count over the full native corpus. Gotchas: new native_test_context_absorb callers must pass spans (span_index_empty() on tokenize/parse refusal); bold-lynx-438 owns dropping NativeTestFileRefusal head_reason/fatal_reason; its locus tests use ^parse_grammar_choice_overlap_residue as a fixture head (swap to ^tokenize_lex_e1_unrecognized_char if calm-tern-13's deletion lands).

## valiant-stag-606 handoff (closed): landed #13170 #13201 #13216.
UNOWNED:
- 4 resolve tests RED ON MAIN, not blocking: admission_fail_closed namespace_provider_* (3) + resolve_test spine_resolve_accepts_normalized_tree. Why do they block nothing (floor disposition)? Same question as the text wall witness.
- Cons in extdeps.uri still R4 (untraced).
- #13216 known gap: a root FN head reads as a projection; infer refuses (loud).
- #13201's confirming census run 37182742735: result unread.
- 20 R4 modules parked on the std.optional re-home (list in #13170's body).
Lessons: a dag/ module on the seed/grammar route cannot import src/v2. Local stage0 regen: claim_executor --required-regen with CARGO_TARGET_DIR=./target RUSTC_WRAPPER= .

## sunny-lynx-759 handoff (closed; #13038 merged)
- 9 tak_* warm rows: royal-deer drops them at #13043 -> SingleClaimFillDebtModule for v2.test.claim.type_application_kind.
- UNOWNED WORKAROUND: resolve_path_declares_node compares index nodes because symbol_index_fill_unique_variant_aliases files a unique variant as a second ENTRY (p.Pointer) instead of a binding to p.Resolution.Pointer. Repair is on RFM a_binding_position_recorded_as_a_declaring_identity.
- Kind check heads cover corpus declaration refs only; kernel-atom and type-parameter heads are outside (stated in resolve_type_application's note).

## #13038 re-land: the mechanism (my bisect agent, code-read; probe values confirmed)
- `let g = fn(y) { y }` and `fold(x, 0, fn(acc, _) { acc })`: ACCEPTED at 9db1a8bbaa, refused normalize_reason_post_normalize_not_well_formed at a21a9014da.
- #13038 added type_binder_order_conforms / type_binder_set_edge_conforms (v2.std.node), via type_binder_conj_conforms: a type-binder Conj must carry ONE ArrowSignatureOrderEdge whose labels are exactly its binder set; arrow_signature_edges_conform applies it to every binder edge on an Arrow. body_lower_function_value_arrow builds the fresh-type-variable binder set (incl. the elided return tyvar) via type_binder_set_children + close_lowered_image -> the order edge is missing or mismatched -> post-normalize refuses.
- Re-land: the lowering must mint the order edge for synthesized binder sets (or the rule must hold for them); red-first controls = wise-ant-549's 4 claims + an annotated fn(y: Int) -> Int positive.
- Revert: #13293.

## OPEN (mine to route): comparisons are UNTYPED in v2 infer: infer_binary_algebra_field answers Absent for EqualityComparison/OrderingComparison, so the Transform stays GroundingNotDerived and operands are never judged ('n == (0 + 1)' with n: Nat accepted). Below-floor. Fix = #13060's operator route extended to Eq/Ord through the AlgebraInhabitanceDecl rows. RFM being filed by stern-swift-290.
  Comparison derivation (my fork): EqualityComparison {^equivalence_field_eq} and OrderingComparison {^ordered_ring_field_compare} decode but answer Absent in infer_canonical_operation_field. equivalence_type_node exists, with no rows; only Int declares compare; the rows are Int ordered ring + Bool boolean algebra only. Plan, staged: (1) ordering via the rows (Int only) + same-carrier unify, result Bool; (2) equality via std.algebra algebra_profile_equality_extensional + unify; needs a measured census FIRST and needs the literal-elaboration PR landed. BLOCKER: there is no corpus-wide v2-infer census instrument (the native census stops at resolve). Syntactic: ~52k comparison sites. Distinct-carrier refusals (Symbol vs host String, two Bool homes) are real bugs, fixed in the same motion.
  + SUBTRACTION (AlgebraInverseCompose) is also untyped: the same class; specimen node.dag count(children) - positional_child_count(children) (Nat - Int, silent). Added to #13311's row.

## OPEN: v2 infer does not judge a CALL RESULT when the callee returns a declared (non-kernel) type (fn mk() -> Nat used where Bool is declared: ACCEPTED; -> Int: refused). count stays the stated Int until this is fixed. HYPOTHESIS: this is N7's wall (receiver declaration unavailable / argument type not derived); stern-swift-290 is checking.
  DERIVED (code-read at 623eb3b32f): the earliest unjustified boundary is 04_infer infer_arrow_declared_return_type: it resolves ret only via dag_binding_denotation (KERNEL value-type rows only) or infer_established_value_type_optional (Bool only), and answers Absent for every declared or applied return. Second gap: infer_application_argument_inhabitance DISCARDS the TypeVariableInstance list from infer_judge_formal_args (only a Bool flows on). Fix: (a) thread the instances to the result step; (b) after the kernel lookup, return the declared node, alias-normalized (#13187) then instantiated; a residual own type parameter stays underived. Census risk unmeasured.

## Instrument gap: v2-native-census (census-resolve) prints NOTHING to the job log (output goes only to artifact files), and uploads no artifact on cancel/timeout -> a hang is indistinguishable from a slow run (run 37229309848: 3h48m silent). Needs a heartbeat/progress line to the log + an upload-on-cancel. Owner: whoever owns //gunbc/instruments:v2-native-census (bold-owl-731 is touching census output).

## neat-seal-675 handoff (closed): landed #13057 #13098 #13185 #13240 #13316.
OPEN:
- floor repair (annotation-only edit selects a changed witness; RFM annotation_only_edit_selects_a_changed_witness; work item adhoc-1099d5a3-cb0): that PR must also fix the 11 stale driver_rel notes listed in the RFM row.
- program_assembly ~71k fixed steps per assembly (stall self_host_program_assembly_contract_floor_enrollment_stall): MINE TO ROUTE.
- whether the nine stage0_crate_layout has_pub_mod flips can be unflipped: needs a two-generation regenerated-seed receipt.
- its old ask about the #13104 dispatch-group rename may be stale.

## neat-raven-383 handoff (closed): landed #13143 #13150 #13151 #13172 #13306.
OPEN:
- the generic-formal hole is NOT closed (fixture on floor_expected_red; trigger: resolver identity on type references).
- the KERNEL-CROSSING WALL (refuse kernel chars/chars_to_string outside the route functions) was ruled the LAST step and is UNOWNED: branch text-wall-switch-rest has a 58-file switch of the remaining production callers (builds; unverified; stale; needs a rebase plus a floor run); the v1 seed's own callers need a typed roster.
- only 3 wall claims floor-priced so far; if one is over budget -> royal-deer-478.
Notes: after editing a v1-closure .dag, regen the mirrors from a seed built on the EXACT main merge-base; body // comments don't parse; check git diff --diff-filter=U after each merge.

## OWED (mine): restructure type_application_kind under the §3 witness rule (one real-path claim; the rest supplied at resolve_type_application / type_application_conformance) to retire the 9 interim tak_* ActiveFillDebt members from #13309. Immediate follow-up once #13309 lands.

## calm-pike-525 handoff (closed): #13009 #13173 #13116 #13155 #13163 #13231 #13239 #13245.
UNOWNED:
- witness_admission_explicit_consumer_keys is dead but still bound by the cli_run_witness_admission_source_scan_scaffold row.
- dormant but still read: index_arm_schedule_retention (no production route arms it); entry_eligible_for_discovery_skip_before_resolve with its 2 callers (census in #13231's body).
Gotchas: remote regen needs the EstimatedMemory=60GB pool plus a fresh cgroup; git clean src/v1/stage0/src before a regen in a reused BuildBuddy workspace; deleting a stage0 module needs TWO regen rounds; never install main.rs (declared-divergent).

## filter/any (B'), stern-swift-290: needs new CallbackRealization arms (a conditional-cons for filter; an OR for any), roster rows, delete v2.std.algebra filter/any, drop the imports via bright-fox-661's instrument. Starts after #13207 plus the instrument. GAP for later: v2 eval does NOT execute a synthesized callback step's Loop (map/all/filter/any): a future N7 eval wall.

## MINE: the Cardinality meaning fork (std.constructors Cardinality vs the v2.std.node Connective arm). Unify after the Optional de-fork's transition 2. RFM filed by bright-fox-380.

## tidy-raven-393 handoff (closed): landed #13168 (root E), #13029 (root C), #13010, #13105, #13118 (root F), #12506.
N7 on main 59c3845950: 0/8 at PREPARE (infer: receiver declaration unavailable); 1 file refusal: filesystem_io 749:5 (XL-2 template chain).
OPEN: the bare 'NAME =' keyword binder refuses only as generic parse_g0_tokens_remain (next-rung trigger: a FIRST-set computation that sees through a refusal guard; in the RFM row).
KNOW: the floor's enrolment margin judges ms, not steps. A margin refusal at unchanged steps = a steps-invisible cost: diff [cross-claim-demand] and [floor-shared-fill] between a green and a red run before restructuring claims.

## Census instrument (mine): NOT a hang. Per-file NORMALIZE is superlinear and content-dependent (ampere_altra_package_table4_raw 54k tokens: 170s normalize; app_attest_sample_ios_14_4 11k tokens: 33s, about 5x ci_spec 16k). #13457 adds progress lines (hand-edited mirror: regen if the generated lane drifts). Next: localize the superlinear normalize pass.

## 2026-10-06 ~12:20Z, owed from valiant-swift-469 (#13428 alias carry, waiting on the freeze)
- symbol_index_fill symbol_index_alias_rows_of: a second reader of alias rows from raw roots, reached only by hand-built fixtures. Delete or consolidate it once #13428 lands.
- An unreferenced alias whose target is unresolved is accepted silently (same as an unreferenced import). §5 gap; recorded in RFM alias_declaration_binding_row_not_carried.

## 2026-10-06 15:20Z, bright-fox-661 handoff (Optional de-fork)
LANDED: #13178 (kernel Optional mint bound), #13388 (v2.std.optional -> std.optional, a453be995a), carrying:
- tools.source_reference_repoint (dag/gunbc/instruments/source_reference_repoint.dag): rows ReferenceRepoint{from_path,to_path}; fn repoint, check repoint_pending; whole module paths only.
- the std_optional partition row; std.optional in the 19 parked R4 modules; the floor fix closure_members_carried_at (UNREVIEWED; #13408 replaces it).
NEXT, in order:
1. #13408 (bright-fox-380) rebases and replaces closure_members_carried_at, delete-first.
2. stern-swift-290: member-grain import producer on the repoint instrument (filter/any out of v2.std.algebra).
3. PR B (MINE now, no new children): delete none inside the N7 closure (23 modules) -> Absent, with typed Present/Absent, counted. Refusal NoneLiteralOutsideAdmittedDebt over a generated shrink-only roster at (module, enclosing decl) grain. Measure the roster seed cost first. The body names the fork outside the closure and its trigger.
4. Transition 2: rest of the corpus; delete the roster, wall, none/null rows, LitNull and the 2 join arms; unify std.constructors Cardinality with the v2.std.node Connective Cardinality arm; retire bright-fox-380's node_query qualification.
Residue: v2.std.optional in comments/strings plus 2 docs/plans files. Scratch: bright-fox-661's /tmp plan_tail.txt and deriv.txt.
