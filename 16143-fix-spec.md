# #16143 E0369 fix — implementation spec (manager-approved msg_672c1022 / msg_c7f71036)

SITE (from D3 probe, BB b3f718d3): equality plan for NativeDemandRun assigns field `engine`
(type Rc<DemandEngine<Rc<NativeDemandValue>>>, a generic USER record) step=EqualityByTarget, and
DemandEngine never enters plan.declarations -> synthesizable=true -> emitter emits
`a.engine == b.engine` at emitted v2_compiler_source_root_eval.rs:5143 -> E0369 (DemandEngine no PartialEq).

ROOT: src/v1/04_infer.dag:3789 (equality_structure_walk) folds a record to EqualityByTarget whenever
`equality_structure_is_by_target` (:3760) says all member STEPS are by-target. A member whose RESOLVED
type is a user product/coproduct (or a kernel container holding one) must NOT be by-target.

FIX (manager predicate): a structure is by-target only if EVERY member's resolved TYPE is target-native.
  target_native(T): kernel(T)  OR  container(T) with EVERY type argument target_native.
  A user nominal product/coproduct is NOT target-native. A container of a user record is NOT target-native.
Compose ONLY from existing authorities (already imported at 04_infer.dag:9):
  is_kernel_type, is_container_type, container_expected_arity, kernel_type_set  (from std.types / 00_core / 04_env).
  Do NOT mint a new kernel list (DESIGN 3: DFS before you mint).
CAUTION (PR body must say): is_kernel_type keys on SPELLING -> apply it to the RESOLVED type so a user
  type shadowing a kernel spelling cannot pass.

HOW TO THREAD THE TYPE: EqualityPlanField (04_infer.dag) carries only {field: String, step}. The member
  TYPE node is available at the field-build site `equality_fields_walk` fold :3747-3750 (ch = member type
  node; w = equality_field_walk(...)) and the arm build site (~:3856). Add a Bool `target_native` to
  EqualityPlanField (and EqualityVariantArm fields), computed via a new fn `equality_type_is_target_native(node, scope, ...) -> Bool`
  applied to the RESOLVED member type; set it false when the resolved type is a user product/coproduct or a
  container whose argument resolves to a user product/coproduct. Then equality_structure_is_by_target (:3760-3764)
  additionally requires f.target_native for every field/arm-member. (Threading a Bool through the two field
  builders + the two *_is_by_target fns is the whole edit; keeps step enum untouched.)
  Alternative if adding a field is awkward in the .dag data-decl: compute target-native at :3750 and, when
  the member step is by-target but not target-native, do NOT count it (fold returns not-by-target).

MUST NOT REGRESS (this is the whole risk): LocusAnchor<Symbol> (diagnostic.dag:12 generic record, members
  Symbol/Int kernel) has MEMBERS that are target-native -> LocusAnchor's OWN structure stays by-target ->
  claim files keep passing (arrow_scope_atom_refuses_unresolved 11/11, match_binder_typing, branch_slot_readers).
  DemandEngine's OWN structure has a Map/List arg reaching a user record -> DemandEngine NOT by-target ->
  NativeDemandRun.engine not by-target -> routes to plan (generic, non-synthesizable) -> derived fn omitted.
  VERIFY with quick-newt arbiter: self-host cargo-emitted GREEN + `equality_derived_native_demand_run` no longer
  emitted, AND the 3 claim files still 54/54 green.

DROP: the :4187 equality_argument_by_target unbound-type-var change (manager cond1 msg_c7f71036: no control
  exercises it; revert to `true` to keep footprint minimal). Keep footprint = integration_concern.dag +
  self_host_equality_admission_witness.dag + 04_infer.dag (now the target-native fold change).

ADDITIONS required (fold into the SAME push):
  1. Refusal control in dag/test/claim/self_host_equality_admission_witness.dag: a NON-generic user record that
     nests another user record must NOT get a raw == ; assert the EMITTED plan step is routed/refused (not just
     self-host green) + a refusing case asserting its reason (per DESIGN 3c, test fns inside the executed witness,
     not a dangling file).
  2. match_binder_typing.dag rows :586 (seven) & :630 (constructor_body): replace the
     reports_within(cell, ^infer_match_arm_body_type_underived) conjunct with an assertion of the arm body's NEW
     inferred type ONLY after an executed reading of that type (ask quick-newt for the inferred type of `seven`
     / `constructor_body` arm bodies). If wrong/dropped = regression, report it. Do NOT just delete the conjunct.
  3. Appended ConcernSighting (review-92857 item 2) with the final green self-host + claim invocation ids.
     (Concern data block for equality_synthesis_helper_binding_is_import_authority is around :1076 / :1150;
      sightings appended near :568; status lives in the typed row, not a // comment.)

FLOW AFTER EDIT: re-merge LATEST origin/main (was 71a24718 at last merge; main >=4a4f790d; take MAIN's mirror
  src/v1/stage0/src/v1_compiler_infer.rs on conflict) -> quick-newt arbiter on merged tree (regen mirror from
  fixed .dag via claim_executor --regen-round-cost --base-mirror HEAL_BASE_MIRROR_COMMIT=<main>, seed build,
  gunbc test //gunbc/instruments:self-host, cargo build emitted crate) GREEN + derived fn gone + claims 54/54 ->
  push PR branch (merge commit, NO force, --no-verify) -> heal-publish -> contents-API verify src/v1/stage0
  carries change (not stale fetch) -> PAIRED CLAIMS base 5001d054 vs final bot head (arrow 11, match_binder 20,
  branch_slot 23, + self_host witness + refusal control) base refused@load head pass -> PR body (invocation ids,
  footprint note, mirror note, is_kernel_type-spelling caution, 7/7->23/23 correction) -> dashboard approve at
  head, no REQUEST_CHANGES -> report PR 16143 + full bot sha to silent-bee-845. NEVER merge.

EXEC RULE: no exec in my tree; every build/probe/claim via dashboard-message send --to quick-newt-573 citing
  msg_99822b94 + silent-bee-845. quick-newt BuildBuddy dispatch ~15 min each.

## SUPERSEDED predicate + B-first (manager msg_89d0a1d5 / my msg_6f5cc530) — READ THIS FIRST
msg_672c1022 predicate (kernel OR container-of-target-native; user nominal NOT target-native) is WRONG:
  emitter derives PartialEq CONDITIONALLY per record, and Symbol (type Symbol, node.dag:26) is an opaque
  nominal WITH derived PartialEq; LocusAnchor<T>{at:T} (diagnostic.dag:12) sole member=T; claims compare
  LocusAnchor<Symbol> PASS => Symbol must stay by-target. A plain user-record rejection regresses claims.
has_eq must be RECURSIVE + INSTANTIATION-AWARE, reading the SAME rule the emitter uses (one authority).

EMITTER AUTHORITY (single source of truth for "does this type have ==") — located, NOT yet fully read:
  src/v1/trait_derive_emit.dag:195 rust_nominal_identity_carrier_shape_eligible(n, source_indices) -> Bool
  src/v1/trait_derive_emit.dag:200 rust_symbol_wrapped_ord_carrier_shape_eligible(children, source_indices) -> Bool
  -> used by 05_emit_rust.dag:2525 rust_nominal_ord_derives_for_shape: emits "#[derive(... PartialEq ...)]"
     (:2519 rust_ord_derives_text) ONLY IF eligible, else "" (=> no PartialEq => E0369 on that record).
  Generic param `: PartialEq` bound: 05_emit_rust.dag:10311 v1_fn_body_equality_bound_param_names -> eq_param_names
     (:10323 union with call_forwarding_eq_param_names) -> :9258 v1_fn_fn_bounds_by_param adds PartialEq.
  NEXT: read trait_derive_emit.dag fully to extract the EXACT eligibility predicate + how eq_param_names is
     computed (which structural features make a record/param NOT earn PartialEq). has_eq must call/read THAT.

B IN FLIGHT: quick-newt msg_f80459a9 (rebuild 15da29b+mirror 5b56d0d9, dump emit derive lines for
  DemandEngine/DemandEntry/DemandState/DemandIdentity/NativeDemandValue + LocusAnchor/Symbol control,
  field types, FULL rustc E0369 note, which closure member lacks PartialEq + why). A waits on B.
DECISION SHAPE (manager msg_89d0a1d5): instantiation-aware recursive has_eq over the SAME emit fact,
  in the generic-instantiation path of equality_resolved_declaration_structure(_core) 04_infer.dag:4067/:4309
  (NOT a boolean gate at :3789). MUST keep LocusAnchor<Symbol>+Symbol by-target (claims 54/54); mark
  DemandEngine non-by-target -> plan-routed (generic non-synthesizable) -> equality_derived_native_demand_run omitted.
Tree HEAD 4a43baa88572, clean, :4187 reverted, NOT pushed.

## EMITTER PartialEq decision is capability-table + fixpoint, NOT a clean recursion (my trait_derive_emit read, this turn)
src/v1/trait_derive_emit.dag (2961 lines). Key fns:
  rust_nominal_identity_carrier_shape_eligible(n,si) TRUE ONLY for name=="Symbol" & no children/params & NoConnective.
  rust_symbol_wrapped_ord_carrier_shape_eligible(children,si) = exactly ONE child whose child_type is Symbol-eligible.
    => LocusAnchor<Symbol>{at:Symbol} matches (1 field, type Symbol) => gets ORD derive incl PartialEq. GOOD (claims ok).
  rust_nominal_ord_derives_for_shape (05_emit_rust:2525) uses these -> derive text or "".
  GENERAL record PartialEq is NOT shape-recursion: usage-derived capability table + fixpoints:
    v1_emit_struct_from_capability_table, v1_repr_grounding_derive_elem_shape*, v1_with_map_key_requirement +
    v1_map_key_required_type_names (LEAST FIXPOINT over declared-type graph, adds Eq+Hash for map-key types),
    Clone-bound fixpoint, keyed-carrier module allowlist [std.keyed_row, std.keyed_roster]. Comment explicitly:
    "a record's structure says what it CAN derive; it cannot say what its USE demands". So PartialEq for a plain
    record like DemandEngine is granted only if it lands in the capability table/roster; DemandEngine evidently does
    not => no PartialEq => the E0369.
IMPLICATION for A: manager's "has_eq reads the SAME rule (one authority)" means the CHECKER must ask the EMITTER's
  capability-table question "will this type be derived PartialEq?" — a cross-subsystem call (04_infer -> trait_derive_emit/
  capability table), NOT a local recursive predicate. Feasibility/shape must be re-confirmed with manager after D4 shows:
  (a) checker per-field EqualityStep for DemandEngine closure (what has_eq must reproduce), (b) which member lacks ==,
  (c) whether a clean "type is derive-PartialEq-eligible(type)" fn exists for the checker to call.
DO NOT author blind; D4 decides. quick-newt D4 IN FLIGHT (msg_808372aa).

## REAL edit site = plan NORMALIZATION collapse-to-by-target (my read, D4-independent)
04_infer.dag:4346 fn equality_step_normalized(s, live): if a step reaches NO routed step -> it is rewritten
  to EqualityByTarget. Liveness via equality_step_reaches_route (:4321) + equality_live_plans (:4340, least-fixpoint):
  a DeclarationEqualityPlan is LIVE only if some member step reaches a route (EqualityRouted, or ThroughPlan naming a
  live declaration). EqualityByTarget => not a route.
MECHANISM of the bug: NativeDemandRun.engine step = EqualityThroughPlan{DemandEngine}. DemandEngine's OWN plan
  reaches no route (its members all fold by-target, instantiation-blind on its generic args) => DemandEngine NOT live
  => engine's ThroughPlan{DemandEngine} reaches no route => equality_step_normalized COLLAPSES it to EqualityByTarget
  => emitter renders a.engine == b.engine => E0369 (DemandEngine has no Rust PartialEq).
=> THE FIX: the collapse at :4346 (and liveness) must NOT collapse a dead ThroughPlan{d} to EqualityByTarget unless
  the type named by d actually HAS Rust ==. has_eq(d's concrete instantiated type, via the emitter capability oracle)
  gates the collapse: has_eq -> ByTarget (sound); !has_eq -> leave routed (=> needs synthesized fn; for a generic
  non-synthesizable record => refused at use, the correct concern-(d) behaviour), never silently by-target.
Edit-site candidates: equality_step_normalized :4346 collapse arm; possibly equality_live_plans/equality_plan_reaches_route.
OPEN (D4 decides): (c) is there a clean checker-callable "type is derive-PartialEq-eligible" fn (emitter capability
  table), else has_eq must be minted from the same fixpoint inputs. Manager framed A as generic-instantiation path
  :4067/:4309; my read refines the site to the normalization collapse — flag to manager with D4 report.

## MANAGER FINAL DIRECTION (msg_357afb59) — LOCKS A's implementation shape
(1) Collapse gate at equality_step_normalized (04_infer.dag:4346) = RIGHT site. A dead ThroughPlan{d} may become
    EqualityByTarget ONLY IF d has Rust ==; else stays routed -> synthesized or refused at use (concern (d)).
(2) DO NOT MIRROR the emitter fact (second authority; DESIGN 3 forbids; copies drift). Instead:
    - If the derive decision in trait_derive_emit / 04_emit_info (base traits, has_fn_fields, param bounds) is ALREADY
      a function -> CALL it from 04_infer. Acyclicity is the only import law -> VERIFY the new import forms NO cycle
      (trait_derive_emit / 04_emit_info must not import 04_infer, directly or transitively).
    - If it is INLINE -> EXTRACT it into ONE shared fn that BOTH the derive emission AND has_eq call, in THIS same PR.
    - Refusal control: assert a record with a fn field NESTED UNDER A GENERIC does NOT collapse to ByTarget (i.e. the
      claim leg asserts the emitted plan step for such a record is routed/refused, not ByTarget).
(3) WAIT for D4 (quick-newt msg_808372aa) to CONFIRM the fn-field hypothesis before authoring.
NEXT after D4 confirms: locate/confirm the derive-decision fn (base traits + has_fn_fields + param bounds) in
  trait_derive_emit / 04_emit_info; check it is a function + import acyclicity 04_infer->that module; if inline, extract
  shared fn (both callers) in-PR; wire has_eq = call that fn, gate the collapse at :4346. Keep LocusAnchor<Symbol>/Symbol
  by-target (claims 54/54); DemandEngine stays routed -> equality_derived_native_demand_run omitted.

## IMPORT PLACEMENT for the shared derive fn (my check, D4-independent; satisfies manager acyclicity law)
Modules: 04_emit_info.dag = `module v1.compiler.infer_emit_info`. trait_derive_emit.dag imports v1.compiler.infer_types
  (:11) + v1.compiler.infer_emit_info (:14). 04_infer.dag imports v1.compiler.infer_emit_info (:211) but NOT trait_derive_emit.
  Neither infer_emit_info nor trait_derive_emit imports 04_infer (v1.compiler.infer) -> 04_infer -> {infer_emit_info} is acyclic.
=> The ONE shared derive-decision fn (base traits + has_fn_fields + param-bounds "does this type have Rust ==") should live
   in v1.compiler.infer_emit_info (04_emit_info.dag), which BOTH 04_infer (:211) and trait_derive_emit (:14) already import.
   Guaranteed acyclic, single authority (DESIGN 3). Extract the derive decision there; derive emission + has_eq both call it.
STILL GATED on D4 (fn-field hypothesis) + confirm the derive decision's current location/inline-ness in trait_derive_emit
  before extraction. Then gate collapse at 04_infer.dag:4346.

## DERIVE DECISION IS FUNCTION-BASED (D4-independent; decides manager branch-2 = extract)
trait_derive_emit.dag: v1_emit_struct_from_capability_table (:2828) + v1_emit_struct_derives (:1240) both take
  has_fn_fields: Bool + map_key_required + generic_param_names + bounds. When has_fn_fields true (or map_key_required
  / symbol-wrapped / group-completion), fm_params/ord_params=[] -> hand-written Debug/PartialEq impls
  (v1_freemonoid_struct_partial_eq_impl / set equivalents) with REQUIRED PARAM BOUNDS => == not usable at arbitrary
  param instantiation. has_fn_fields computed by a walk in 04_emit_info.dag (:52 field; :1047/:1060 set; summary).
=> The "does record d (instantiated) have usable Rust ==" verdict is spread INLINE across the capability table + base
  traits (v1_record_derive_traits_heap/copy_for_construction) + param-bound joins (v1_fn_bounds_by_param / eq_param_names).
  It is NOT one named predicate today. => MANAGER BRANCH-2: EXTRACT a single shared fn (e.g. type_has_rust_equality(
  children, generic_param_names, bounds, has_fn_fields, map_key_required, declared_shared, ...) -> Bool) into
  v1.compiler.infer_emit_info (04_emit_info.dag, already imported by BOTH 04_infer:211 and trait_derive_emit:14 => acyclic),
  called by BOTH the capability table (derivation) AND has_eq (checker collapse gate at 04_infer:4346). This is the ONE
  authority; no mirror. Exact inputs to fold: base derive traits membership + has_fn_fields + whether the generic params
  used by the == receive a :PartialEq bound (from the SAME eq_param_names / bounds the emitter renders).
D4 still gates the fn-field confirmation (which member lacks ==). Author only after D4.

## BYTE-IDENTICAL EXTRACTION CONTROL (manager msg_03d87b80) — HARD GATE
The extraction touches the EMITTER, so the derive OUTPUT must be byte-identical before vs after the refactor.
  Capture emitted v1 mirrors + the self-host emitted crate at BASE and at HEAD (final), diff them.
  ONLY allowed difference = the NativeDemandRun equality site (v2_compiler_source_root_eval.rs:~5143, where the raw
  a.engine==b.engine goes away). ANY other byte change => the extraction altered a derive decision => STOP (do not push).
TWO logical changes to keep separable: (a) pure refactor: extract type_has_rust_equality into 04_emit_info + call from
  capability table (must be byte-IDENTICAL emit); (b) behavior change: collapse gate at 04_infer:4346 consults that fn
  (the ONLY intended emit diff = NativeDemandRun equality site). Verify (a) is inert on its own, then apply (b).
BASELINE to reuse: 15da29b92e8d + regenerated mirror 5b56d0d9 (D4 is already building this) -> persist emitted
  self-host crate + regenerated v1 mirror DURABLY (survives BuildBuddy VM exit; NOT per-VM /tmp) for the post-change diff.

## D4 RESULT (quick-newt msg_b12d100b, EXECUTED, BB 4e82358f) — fn-field hypothesis DISCONFIRMED
CULPRIT (rustc, executed): NativeDemandValue — emitted #[derive(Clone)] ONLY (NO PartialEq, NO Debug) at
  v2_compiler_source_root_eval.rs:3488. Arms hold Rc<NativeLaneModuleResolution>, Rc<NativeLaneModulePreparation>,
  Rc<NativeLaneOccurrenceCensusRow>, Rc<NativeRouteMemberRow>. NO fn/Box/HashMap/closure fields => has_fn_fields is
  NOT the discriminator. Full E0369: "== cannot be applied to Rc<DemandEngine<Rc<NativeDemandValue>>>" :5143:26;
  note "implementation of PartialEq might be missing for NativeDemandValue" :3488; due to 1 previous error.
DERIVES (executed, retained crate): DemandEngine<V> (:248) derive(Debug,Clone,PartialEq,Serialize,Deserialize);
  DemandEntry<V> (:225) same; DemandState<V> (:64) same, arm DemandAvailable{value:V}; DemandIdentity (std_judgment_
  contract:324) PartialEq+Eq+Hash; LocusAnchor<T:Clone> (v2_std_diagnostic:48) derive(...PartialEq...) [POSITIVE CTRL,
  passes only because T always instantiated at PartialEq types — quick-newt reading, NOT executed]; NativeDemandValue
  (:3488) Clone ONLY; NativeDemandRun (:3656) Clone ONLY (equality synthesized as fn at :5143).
MECHANISM: derive(PartialEq) on DemandEngine<V> adds V:PartialEq bound; at V=Rc<NativeDemandValue> needs
  NativeDemandValue:PartialEq (absent). CHECKER DEFECT (executed trace): field-walk TYPE-PARAMETER arm accepts a bare
  UNSUBSTITUTED V as EqualityByTarget (owner=DemandState.DemandAvailable field=value type_head=V step=EqualityByTarget,
  depths 1,2,4,5,6). Walk never substitutes V:=Rc<NativeDemandValue>.
=> REVISED FIX = TWO parts: (i) CHECKER: the by-target/collapse must NOT credit a bare unsubstituted type parameter;
   substitute the instantiation args / require ARG eligibility when a field type is a generic application like
   DemandEngine<Rc<NativeDemandValue>> (this is the original #15992 instantiation-blindness; my earlier :4187 was the
   same arm — likely must be REINSTATED, not dropped, but instantiation-aware). (ii) has_eq must read the emitter's ACTUAL
   derive-narrowing rule (WHY NativeDemandValue is Clone-only) — NOT has_fn_fields (ruled out). Manager branch-2 premise
   (extract has_fn_fields decision) needs REVISION.
OPEN/asked quick-newt: WHY does NativeDemandValue derive Clone-only (which emitter rule; not fn-field)? Full trace / QN_STRUCT.
CAVEATS: only 150/23474 QN_FIELD lines printed; QN_STRUCT emitted 0 lines. Trace incomplete; no NativeDemandValue QN_FIELD
  among the 150 but cannot claim none exist.

## MANAGER CONFIRMED REVISED SHAPE (msg_a4dab543) — AUTHOR GATE
(a) At the field-walk TYPE-PARAMETER arm: substitute the INSTANTIATED argument and require IT to have ==, RECURSIVELY;
    a bare UNSUBSTITUTED param is NEVER credited by-target (fail closed). has_eq reads the emitter's REAL derive rule for
    NativeDemandValue (whatever narrows it to Clone-only), NOT has_fn_fields.
(b) The instantiation-aware :4187-style change IS IN SCOPE and now HAS its executed control: this NativeDemandRun site
    (by-target at base 5001d054 -> routed at head) + LocusAnchor<Symbol> staying by-target BECAUSE Symbol has ==.
    Footprint = 04_infer + the ONE shared 04_emit_info fn only. Byte-identical-except-NativeDemandRun control applies.
GATE: AUTHOR once quick-newt names the Clone-only rule (msg_084ee905). If that rule is NOT a callable fact (e.g. a
    per-type LIST), STOP and report before restating it (DESIGN 3 single authority).

## PROGRESS: commit 179bd7c1d47a on work-16143-v2 (HEAD) — additive accessor, byte-identical
Added to src/v1/04_emit_info.dag (before close_fn_fields ~:1208):
  fn type_has_rust_equality(index: TypeSummaryIndex, key: String) -> Bool {
    match type_summary_lookup(index: index, key: key) {
      TypeSummaryFound { summary: summary } => !summary.has_fn_fields
      _ => false
    }
  }
No caller yet => emitter derive output UNCHANGED (byte-identical). Reads CLOSED TypeSummary.has_fn_fields.

## REMAINING AUTHORING RECIPE (do next, small verifiable edits, on work-16143-v2 or a scratch branch)
Checker 04_infer.dag — the by-target for a bare/param type is produced in equality_operand_walk(:3915) →
  equality_leaf_walk(:3985) → equality_resolved_declaration_walk/_structure/_core(:4067/:4300) and the table
  arms equality_applied_argument_is_structural(:4134) / equality_argument_by_target(:4184/:4187 arm) which
  currently return `true` for equality_operand_is_type_variable. Per manager msg_a4dab543:
  (a) At the type-parameter arm: substitute the instantiated argument and require IT to have ==, RECURSIVELY
      via type_has_rust_equality(closed_index, arg_identity) + recurse into arg's own args; a BARE unsubstituted
      param => FAIL CLOSED (never by-target). Need the CLOSED TypeSummaryIndex in scope at the walk — 04_infer
      builds it at :21359 (EmitInfoBuildState.type_summaries = close_fn_fields(...)); thread it into the
      equality walk scope (InferScope gains/uses it) OR pass as a param to the equality_* walk fns. Import
      type_has_rust_equality at :211 (add to the v1.compiler.infer_emit_info import list).
  (b) Gate the collapse at equality_step_normalized(:4346 / the `if !reaches_route { EqualityByTarget }` at ~:4354):
      only collapse a dead ThroughPlan{d} to EqualityByTarget when type_has_rust_equality(closed_index, d); else
      stay routed (synthesized-or-refused at use). 
  (c) KEEP byte-identical except the NativeDemandRun equality site. LocusAnchor<Symbol>: field at:Symbol, Symbol is
      a concrete nominal (not a type var) => unaffected, stays by-target (Symbol closed has_fn_fields=false). Symbol's
      positive control must hold; claims 54/54 (arrow 11, match_binder 20, branch_slot 23).
  (d) Refusal control: non-generic user record nesting another user record / fn-reaching type under a generic must
      NOT collapse to ByTarget; assert the EMITTED plan step (claim in self_host_equality_admission_witness).
VERIFY (no exec in my tree): quick-newt — (1) regenerate mirror from fixed .dag (claim_executor --regen-round-cost
  --base-mirror HEAL_BASE_MIRROR_COMMIT=<main>), seed build, self-host, cargo build emitted crate GREEN; (2) BYTE-DIFF
  emitted crate vs quick-newt baseline-16143 (tgz sha256 f30cf1cd…, manifest de8412e5…, v2_compiler_source_root_eval.rs
  4b529ee7…): ONLY NativeDemandRun equality site (~:5143) may differ, else STOP; (3) equality_derived_native_demand_run
  no longer emitted; (4) claims 54/54 + base refused@load (BB 97b6d359).
THEN additions (refusal control test fn; match_binder :586 seven/:630 constructor_body replace
  ^infer_match_arm_body_type_underived conjunct with executed arm-body TYPE assertion — get the type from a quick-newt
  probe first; appended ConcernSighting review-92857 item2), re-merge latest origin/main (take main's mirror d7100787),
  push PR branch (merge commit, NO force, --no-verify), heal-publish, contents-API verify, paired claims, PR body,
  dashboard approve at head, report PR 16143 + full bot sha to silent-bee-845. NEVER merge.
NOTE: :4187 was reverted at 4a43baa (cond-1). The instantiation-aware reinstate is now folded into (a) above.

## CONSOLIDATED KEY IDS + SITES + BASELINE (self-contained handoff; manager msg_6927721d)
BuildBuddy invocations: D4 checker trace+derives BB 4e82358f-6e48-495a-adeb-0c892e389eb5; D5 clean baseline BB a61e7667-7302-4a90-b305-3e41fe37b093; D2 pre-fix self-host repro BB 57d84f8a-6a0a-478f-88f8-a376ca7a03a9; E0369 first repro BB cd7c7c9f-199b-4de2-a32a-2b645e6c2c59; D1' regen BB d9b6232f-9e60-4769-b0bd-d44a7d1efa26; D3 plan probe BB b3f718d3-ae21-474b-9d5c-e4cf38160a38. Claim legs: base refused@load BB 97b6d359-f8ee-43ff-a1b8-a9aef2c0211d (claim_batch 04928c9d…); HEAD@61880ea 52/54 BB c42456e7-0628-4632-aa7c-add225a3d0c5 (claim_batch 88930754…).
BASELINE (quick-newt session scratchpad /tmp/claude-1000/-home-briansrls--worktrees-quick-newt-573/a8193b2c-ef09-4c93-80e2-c320a8930d25/scratchpad/baseline-16143/): crate tgz sha256 f30cf1cd1dc0f5f3aef50c7c7fa47443e5e13b2f8988777bb34ae335d9daca2c (1085384 B, 271 files); manifest (per-file sha256) de8412e581ed71fed1d96efb1410af5c64686faa83d3c37825d3f668e25a8248; v2_compiler_source_root_eval.rs sha256 4b529ee7dd67050a43afc18c3e42c832f631749ec477c3b13c3b1956b1473f14 (5219 lines); regen mirror v1_compiler_infer.rs blob 5b56d0d98cfc0ac83b04e984543d18d90f5c1efd sha256 118531f41dfc7341a1574e36a174fa1db91597f4997a85b2fe6e088af9e725a8; std_types.rs 242a8521bb36bed1e2477f64eb73bbd8a1cd0885; regen patch qn-ecrg3 sha256 6f06d014…. BYTE-DIFF GATE: only NativeDemandRun equality site (v2_compiler_source_root_eval.rs ~:5143 a.engine==b.engine) may differ vs baseline; equality_derived_native_demand_run must be GONE; any other byte diff => STOP (extraction altered a derive decision).
SITES (verified at head 179bd7c): 04_emit_info.dag type_has_rust_equality (added, before close_fn_fields ~:1208); close_fn_fields:1208; fn_reaching_names:1198; fn_reaching_names_fixpoint:1184; TypeSummary.has_fn_fields field :52; type_summary_lookup:113. trait_derive_emit.dag v1_emit_struct_derives:1242 (if has_fn_fields :1251 -> fn_field_derive_traits() :1255); v1_emit_enum_derives:1291 (:1301/:1305). capabilities.dag:338 fn_field_derive_traits=[RustClone]. 04_infer.dag: import v1.compiler.infer_emit_info :211 (add type_has_rust_equality); EmitInfoBuildState closed index built :21359 (type_summaries=close_fn_fields(...)); equality_operand_walk:3915; equality_leaf_walk:3985; equality_resolved_declaration_walk/_structure/_core:4067/:4300; equality_applied_argument_is_structural:4134 (type-var arm returns true); equality_argument_by_target type-var arm:4187 (returns true; was :4187, reverted to true at 4a43baa); equality_step_normalized:4346 (collapse `if !reaches_route { EqualityByTarget }` ~:4354); equality_operand_is_type_variable:4631; structure by-target fold equality_structure_walk:3786/equality_structure_is_by_target:3760. :4770 has EqualityInstantiationBlindGenericSite precedent.
CONTROLLED FILES: work-16143-v2 @ 179bd7c1d47a on top of 4a43baa (:4187 revert, cond-1) + 95d3712d merge. Footprint vs main: dag/gunbc/integration_concern.dag, dag/test/claim/self_host_equality_admission_witness.dag, src/v1/04_infer.dag, src/v1/04_emit_info.dag (4th, the shared fn) — mirror src/v1/stage0/src/v1_compiler_infer.rs resolved to main's blob d7100787 (heal regenerates). origin/main was 71a24718 at last merge, has moved (>=4a4f790d): re-merge LATEST before PR push, take main's mirror on conflict.
