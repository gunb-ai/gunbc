# §6b diagnosis skeleton — v17 rtp/ce FAILs (sleek-seal-226; sources at pins only, no runtime claims)

## Partition (new, head 495da688 return_tail_position.dag; identical roster in both legs)
The 17 rtp claims partition by EXPECTED verdict, and the partition is exactly the pass/fail split:
- PASS 6 (all rtp_refuses_as on the file-refusal lane): bug/nontail/lambda_nontail expect fatal ==
  ^body_lowering_reason_return_not_in_tail_position; guard_fallthrough/guard_last/guard_nontail expect
  fatal == ^body_lowering_reason_else_less_if_unlowered (lines 277-318).
- FAIL 11 = 7 acceptance claims (tail_ctl 319, let_tail_ctl 326, lambda_tail_ctl 333, arm_ctl 347,
  guard_ctl 354, match_ctl 375, match_nested_ctl 389; expect rtp_is_resolved => RtpResolved)
  + 4 unbound-read claims (tail_und 340, guard_arm_und 361, guard_rest_und 368, match_und 382;
  rtp_unbound_at: RtpRefused reason == ^resolve_reason_unbound_symbol and at_atom Present == atom).
No FAIL spans shapes, imports, or modules other than the expected-verdict class; bare Bool false,
no error lines (v17). Identical verdicts at f08a6ae4 (leg A) and 495da688 (leg B) except
an_else_less_if_that_falls_through_still_refuses_as_unlowered (FAIL A / PASS B) — the PR's own fix.

## Reading (bounded)
1. The typed file-refusal lane is ALIVE: six refusals carry the exact expected fatal symbols, so the
   ingest, native_test_context_from_ingest (source_root_eval.dag:1083; catch-all Refused => false at
   the claim), the refusal index and Symbol '==' on ^body_lowering_reason_* discriminate for those
   symbols in that lane.
2. EVERY expected ACCEPT shape fails to reach RtpResolved, and EVERY expected resolve-refusal-with-
   unbound-atom fails rtp_unbound_at — including the simplest fixture (rtp_tail_ctl: body `return i`,
   no if, no match, no undeclared name). So the failure is upstream of per-shape logic: the lane is
   refusing (or failing) everything that is not a body-lowering refusal, at BOTH legs.
3. Attribution NOT ESTABLISHED: the 13 FAILs occur identically at A (absorbed main) and B (head);
   a FAIL shared across both pins does not establish a shared mechanism and does not exonerate the
   PR. The fallthrough claim is the only observed verdict delta in this roster — not the PR's only
   effect, and not proof of its mechanism.
4. Positive control (brief item 1): NO acceptance claim accepts anything at this seed — that is the
   finding itself. The nearest live controls are the six refusal claims.

## ce pair (same attribution)
Both ce FAILs (a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds,
the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds, closure_emission.dag:336/380)
consume ONE producer, ce_emission_member_body_refuses(), over synthetic sources whose member body is
`if p { p }` (else-less-if), requiring Accepted + length(modules)==3 + refused arm reason
^body_lowering_reason_else_less_if_unlowered + first-refusal agreement. Fails identically in both
legs -> base, same window. NOTE: rtp refusal claims show ^body_lowering_reason_else_less_if_unlowered
IS produced and compared successfully in the rtp lane — the ce pair failing on a producer that DEMANDS
ACCEPTED is consistent with the same base-window change having flipped accepted->refused for shapes
that previously lowered.

## Earliest unjustified boundary — HYPOTHESIS only (not established, not assumed)
H-boundary (hypothesis): a base-window change between 9c15c98c and f08a6ae4 altered body lowering /
lane acceptance so non-refusal bodies no longer reach Resolved/Accepted, refusal production intact.
SOURCE FINDING (this turn, git-only): in 9c15c98c..f08a6ae4 (156 commits) NONE of
source_root_eval.dag, resolve.dag, name_resolve.dag, body_lowering.dag, closure_emission.dag, nor
the three claim files changed — so if H-boundary holds, the change sits in other window files
(candidate set = the merge's other paths; to be enumerated in the approved bisect, not before).
Alternatives remain open: the predictions may never have held at this seed (first readings here),
or the mechanism is not a window change at all. The predictions were
authored at 9c15c98c — PRED BEHAVIOUR UNOBSERVED (never executed there; do not write 'pass at PRED');
the roster's predictions are the first artifact that would need
re-basing at this seed IF they are shown not to hold — attribution of the FAILs remains NOT established. The wc probe chain (v7) settles a separate control (caret '=='
discrimination on ^resolve_reason_* and the wc 0/4 controls) BEFORE any repair is proposed; if v7's
ctl_ok returns X, the resolve-lane comparison joins the suspicion list; if it returns 0, the boundary
is the acceptance gate itself.

## Owner and next step
Owner: NOT ESTABLISHED (parent correction msg_55361203): 'the 13 FAILs occur identically at A
(absorbed main) and B (head); attribution is not established'. Do not use the two parent-forbidden formulations (msg_55361203 corrections 1 and 2).
Next: (a) await v7 grant/reading (caret discrimination + wc controls) FIRST. (b) The bisect (roster
at 9c15c98c vs f08a6ae4): prepare NOTHING until v7 ends and tidy-ant-251 says so; when proposed it
must pre-register the meaning of EVERY outcome (both FAIL; only f08a6ae4 FAIL; neither) under the
same exit 0/1/2 law, and must use the END-TO-END pre-fetch check (v7 pattern). (c) Only then a
repair proposal; landing #15695 still needs side-chat approval at exact head, CI green, heal rule.

## Appendix (append-only; parent guard msg_65539ba1)
'predictions authored at PRED; PRED behaviour unobserved' — nothing has ever shown them passing at
9c15c98c. Pre-registration (parent wording, msgs_65539ba1 + 27de8b01; released design: roster at PRED 9c15c98c vs
head 495da688, claims constant, outcome-join law, end-to-end pre-fetch check; release only after v9's
control reading and the parent's word):
  (1) BOTH FAIL: the roster fails on a lineage WITHOUT the mainline delta, so the delta is not NECESSARY
      for the PRED failure; whether the head failures share that mechanism is NOT established.
  (2) ONLY HEAD FAIL: the failure is carried by the 1,112-file mainline delta — a narrowing of the
      candidate set, NOT an identification of a cause.
  (3) NEITHER FAILS: v17's result does not reproduce; candidates are the instrument or host,
      run-to-run nondeterminism, or a difference between the v17 and bisect frames; not established.

## Appendix (append-only; candidate-set reduction, source-only, 2026-11-06T14:55Z)
Byte-identity verified PRED 9c15c98c vs head 495da688 for EVERY file in the rtp/ce claims' lane closure
that was ever suspected: claim files, extdeps/languages/dag.dag, std/grammar.dag, std/node.dag,
compiler/resolve.dag, compiler/name_resolve.dag, compiler/body_lowering.dag, compiler/closure_emission.dag,
compiler/dependency_demand.dag, src/v1/stage0/src/bin/claim_batch.rs. In-delta files reviewed and shown
lane-irrelevant: source_root_eval.dag (census-CLI family retirement + D13 producer move; lane producers
byte-preserved incl. demand_facts_unbound_locus_from_diagnostics), native_lane_runner.rs (serve/census-verb
paths; run_native_claim_program +1 descriptor hunk #15834), claim_executor.rs (+47 heal comments/
orchestrator), node_reflection (+1 import), decl_ref_disposition_roster (census rename + roster entry),
emit_orchestration (rename), compile_cli (bash cmdsubst spelling wrapper).
REMAINING CANDIDATE CLASSES (NOT established): (a) preparation/host inputs the lane consumes indirectly —
floor-phase built-by-preparation bare-reference-edge index (pool_acquire/cli_run Rust files changed in
delta; claim_batch host itself identical), (b) eval-host Rust semantics (std_*.rs hosts, main.rs −136),
(c) frame/nondeterminism. Settled by: v10 CONTROL (Symbol '==' discrimination + wc controls) then, on
parent release, the PRED-vs-head bisect with the pre-registered three outcomes.

## Appendix (append-only; landing-gate data point, 2026-11-06T14:48Z)
dashboard-ops reviews @ head 495da688d0ce: merge_criteria.ready=true, mergeable MERGEABLE, checks_state
passing, 1/1 dashboard approval (claude opus[1m] id 91223, verdict approve AT THIS HEAD), 0 request_changes,
open/not-draft, github_admissible. THIS IS READINESS DATA, NOT PERMISSION: #15695 stays not landable until
the parent confirms side-chat approval at the exact head, CI green, and the heal mirror rule. Diagnosis
unresolved; v10 control reading pending.

## Appendix (append-only; host-side reduction, source-only, 2026-11-06T15:00Z)
Symbol identity machinery at BOTH pins (not a delta): Symbol = (&'static str) with PTR equality; ALL
production minting flows through SymbolInterner::intern -> canonical_symbol_spelling -> the global
CANONICAL_SYMBOLS table (HashSet<&'static str>, dedup by spelling, 16 MiB retained cap -> so r=X/al
paths are canonical, cap-refusal panics loudly); raw Symbol("literal") constructions exist only inside
#[cfg(test)] blocks. Floor-phase / pool / execution host files (bin/claim_batch.rs, cli_run.rs,
cli_run/claim_execution.rs, cli_run/emit_host.rs) are BYTE-IDENTICAL PRED<->head.
NET: within the rtp/ce claim-execution closure (corpus lane + claim_batch host + floor-phase + symbol
interning) nothing in the 1,112-file delta is lane-relevant so far. Residual classes (NOT established):
generic v1_interpreter/v1_rt/std_primitive_projection churn (census-CLI + serve deletions, +11 accessor,
char_at typing refactor #? valiant-deer-27), eval-vm paths not yet traced, and frame/nondeterminism.
This SHARPENS v10's value: with canonical interning intact and pointer-eq pre-existing, the control
(CTL ok=0/out=X) is expected to show discrimination; if it instead shows collapse at the pins, the
comparison path itself is located as the boundary. Either way v10's reading is the deciding measurement;
the bisect remains unreleased/unprepared per parent.

## Appendix (append-only; LEAD MECHANISM CANDIDATE, 2026-11-06T15:05Z — still NOT established)
Commit 1355ef4fab66 'fix(v2 equality): judge alias operands of == by their expansion (srdk26 a4/a5)
(#15754)', gunbai-bot, Nov 5 05:06Z, is HEAD-ONLY (not an ancestor of PRED 9c15c98c) and accounts for the
WHOLE src/v1/04_resolve.dag PRED<->head delta. It adds ground_ungrounded_alias_leaf and calls it from the
else branch of expand_nominal_alias_rhs_by_name (04_resolve.dag:380-392): nominal aliases whose
TypeBinding.resolved is an ungrounded NoConnective leaf with no inferred payload now ADOPT their
alias_rhs structure instead of staying ungrounded shells; the authored-identity branch below
(brand != authored_name_at(structural)) is shape-dependent and its outcome changes accordingly.
stage0 mirror v1_compiler_infer_resolve.rs regenerated in the same commit. Every rtp/ce fixture imports
v2.std.integer { Int } — Int is a nominal alias — so parameter/member node SHAPES in the claim corpus
differ at head vs PRED, while all lane .dag, claim, and execution-host files are byte-identical.
FIT (hypothesis): verdict-level logic that branched on ungrounded-shell shapes (PRED era) flips at head —
acceptance never reaches Resolved (7), unbound-atom reads miss (4), closure-lowering Accepted demands fail
(2), while typed file-level refusals (6 PASS) precede this path. CE pair explicitly demands Accepted over
'if p { p }' — direct fit. §6b EARLIEST UNJUSTIFIED BOUNDARY (candidate): 1355ef4fab66 grounding shared
nominal-alias leaves WITHOUT an audit of consumers that treat the shell; the first such consumer is
among the rtp/ce verdict/lowering producers (sites pending the control reading + bisect).
Settling instruments: v10 control (comparison discriminates? independent of this candidate), then the
parent-released PRED-vs-head bisect (outcome (2) 'only head FAIL' expected under this candidate; (1)
both-FAIL would refute it as necessary; pre-registered wordings govern interpretation).

## Appendix (append-only; consumer hunt, 2026-11-06T15:07Z)
Negative result: the rtp/ce lane .dag (source_root_eval.dag at head) contains NO branch on alias/
NoConnective/authored-name shapes — so under the 1355ef4fab66 candidate, the first consumer is NOT a lane
.dag branch: it is downstream — the RESOLVED GRAPH CONTENT the lane reads (compile_to_resolved / infer /
resolve stages run the grounding change in-host), which the lane's census/verdict producers then consume.
Precise site identification therefore requires observation (v10 control + released bisect), not more
source reading; further static hunting is exhausted without execution (rules forbid local runs).

## Appendix (append-only; bisect discriminator pre-registration, parent msg_8b9a28cf, 2026-11-06T15:11Z)
When RELEASED (nothing prepared until then), prefer the sharper discriminator: head vs head-with-
1355ef4fab66-REVERTED (src/v1/04_resolve.dag AND the regenerated stage0 mirror v1_compiler_infer_resolve.rs
TOGETHER — the packet rebuilds the seed), claims constant. Pre-registered readings: FAILs VANISH on the
revert -> #15754 is NECESSARY (still not the full mechanism); FAILs PERSIST -> not necessary. Revert
applies cleanly: checked PRE-FETCH over the embedded patch. Alternative retained: PRED-vs-head (3 outcomes,
wordings per msgs_65539ba1+27de8b01). Propose whichever is cheaper at release.

## Appendix (append-only; lead-candidate FALSIFICATION, static, 2026-11-06T15:05Z)
The adoption path of 1355ef4fab66 (#15754) in ground_ungrounded_alias_leaf (src/v1/04_resolve.dag:358-376,
call site :390) can fire ONLY when lookup_binding_by_name yields a TypeBinding with alias_rhs=Present.
alias_rhs has EXACTLY two authoring sites (src/v1/04_infer.dag:19897,19916), both value =
alias_rhs_reference(pre) = transparent_alias_rhs(pre) (src/v1/04_env.dag:2429), which returns Present ONLY if
pre.connective==NoConnective && pre.params==[] && pre.type_annotation==none && pre.children==[] &&
pre.return_cardinality==Required && pre.inferred==Present{Resolved{rhs}} with rhs.connective==NoConnective,
rhs.return_cardinality==Required, rhs.type_annotation==none — i.e. `type A = B` bare-reference aliases.
rtp fixtures declare NO aliases; their imported/declared types fail the guard on shape:
- Int: 'type Int = GroupCompletion<Nat>' (src/v2/std/integer.dag:19) -> pre has children => none.
- Bool (std.types), RtpS = RtpA | RtpB (coproduct) => not transparent-alias shape => none.
Therefore no rtp fixture binding can trigger adoption => 1355ef4f is NOT the mechanism for the 11 rtp
FAILs via this path (STATIC, at head 495da688; source-only; no runs). Residual: other consumers of the
same commit unexamined; ce-pair linkage NOT tested by this argument (closure_emission.dag not yet
audited for transparent-alias bindings). Candidate status demoted: rtp-routed adoption mechanism RULED OUT by SOURCE READING (not executed); the interpreter could still take a path the reading missed — the bisect, not the reading, decides.
Also static (this session): expand_nominal_alias_rhs_by_name call graph — the new fn sits in the else arm
where the base returned 'resolved' unchanged; base behaviour is recovered whenever alias_rhs is none.

## Appendix (append-only; ce-side completion of the falsification, static, 2026-11-06T15:08Z)
closure_emission.dag embedded sources audited (head copy, lines 67-95): declared types are ONLY coproducts
('type Bool = True | False' at ce_logic_source:82; 'type CeBase = CeBaseA | CeBaseB' at ce_base_source:74) —
no 'type A = B' bare-reference alias anywhere. transparent_alias_rhs therefore returns none for every ce
binding too => the 1355ef4fab66 adoption path cannot fire for the 2 ce FAILs either (their fixture chain is
ce_refusing_entry -> ce_refusing_member 'if p { p }', Bool-only). COMBINED WITH PRIOR APPENDIX: no surviving
STATIC causal path (source reading, not executed) from 1355ef4fab66 to any of the 13 FAILs. Revert-discriminator loses its rationale.
Side observation (hypothesis, NOT evidence): the 2 ce FAILs are exactly the else-less-if unlowered-refusal
claims ('if p { p }'), the same construct #15695's parse fix touches, and the sole observed A-vs-B verdict
delta is the fallthrough claim — consistent with 'the PR's acceptance broadening reaches the positions whose
claims demand refusal', direction NOT yet separated from 'acceptance fails' on the rtp side. Discriminator
needed: probe reading (v11), then possibly a widened probe over the body-lowering refusal producer.

## Appendix (append-only; refusal-producer map + positive-control relocation, static, 2026-11-06T15:14Z)
At head 495da688, ^body_lowering_reason_else_less_if_unlowered has EXACTLY two emission sites in
src/v2/compiler/body_lowering_fold.dag: body_lower_else_less_if_reject (:12678-80, via body_lower_diagnostic)
called at :8903/:9165/:9560, and body_lower_exit_walk_if (:12946-50) refusing via body_lower_exit_refused.
Both key on body_lower_else_less_if_parts_optional (:12726), a POSITIONAL parse reader: requires
parse_production_emitted_identity ^dag_surface_if_expr, IfRowNoElse else-slot, and then-arm identity
^dag_surface_fn_body (:12735-48). So the refusal exists iff the PARSE emits that exact shape.
POSITIVE CONTROL (better than any rtp claim): the cpr lane (15/15 PASS at A and B) contains cav_else_less_if
claims (src/v2/test/claim/namespace_xl0/call_argument_value_resolve_refusal.dag:85,145,201,277...) consuming
the SAME producer — refusal for else-less-if IS observed and readable at both pins in the cav context; and
the sole A-vs-B delta claim (rtp_guard_fallthrough 'if c { i }' refusing) PASSES at B. The classifier is
therefore NOT globally broken at either pin.
PARTITION UPDATE (data, not conclusion): the 2 ce FAILs are closure-MEMBER-context demands for that refusal
(the 'if p { p }' member, via closure_emission) failing identically at A and B — context-specific, not
producer-wide. The 11 rtp FAILs (acceptance + unbound-read) also fail identically at A and B. Since
predictions were AUTHORED at PRED and PRED behaviour is UNOBSERVED, a live candidate consistent with all
statics: the 13 expectations encode PRED-era construct-position grammar; the MB..A window changed what the
parser emits for those shapes (candidate-grammar lineage NOT in A), and the head re-lands only part of it —
the ONE observed delta (fallthrough) is exactly within #15695's re-landed scope. Bisect-relevant; remains
HYPOTHESIS until a run separates it from instrument/claim-authoring defects. No probe widening is needed to
test THIS: the bisect roster itself discriminates. HOLDING for v11's control reading (parent deferral
msg_f21a482c).

## Appendix (append-only; parse-surface and lane-function statics, 2026-11-06T15:10Z; supersedes the 'partial re-land' phrasing in the 15:14Z appendix WHERE NOTED)
Statics (git-only, no runs):
- MB..A window (156 commits) touched NONE of src/v2/extdeps/languages/dag.dag, src/v2/std/grammar.dag,
  src/v2/std/node.dag, src/v1/02_parse.dag — all four byte-identical MB vs A.
- At head B: dag.dag, grammar.dag, node.dag, 02_parse.dag are BYTE-IDENTICAL PRED vs B. The PR's parse-surface
  re-land is COMPLETE on those files (A..B delta == MB..PRED lineage footprint on them: 4 files incl. the new
  condition_position_record_parse.dag claim file and floor_pure_producer_share.dag). 'Head re-lands only part'
  is NOT supported for the parse surface; the sole observed verdict delta (fallthrough) is then what the
  COMPLETE re-land changes over A, as expected of #15695's stated scope.
- Function-level check on the window-edited rtp host file src/v2/compiler/source_root_eval.dag (window commits
  c63e8b2129d3 #15858, ae71693e61e6 #15541): fn bodies native_census_module_resolution and
  native_test_context_from_ingest are BYTE-IDENTICAL PRED vs head. Strengthens the earlier census-CLI-only
  exclusion to function level; interpreter-path caveat applies (SOURCE READING, not executed).
CONSEQUENCE FOR THE STANDING PARTITION: with parse surface and the two lane functions unchanged PRED->head,
and claims/lane/host files byte-identical PRED<->head (prior appendix), any PRED-vs-head behavioural
difference for the 13 claims must sit OUTSIDE this audited file set (candidates: stage0 mirror regeneration,
std/ or other runtime files touched by the window; NOT yet enumerated) or in the claims' own unobserved-at-PRED
status. The bisect remains the decider; v11 control reading first (deferral msg_f21a482c).

## Appendix (append-only; CORRECTION to 'lane/claim/host byte-identical PRED<->head', 2026-11-06T15:15Z)
The earlier byte-identity claim covered only the enumerated files (claim files, extdeps/languages/dag.dag,
std/{grammar,node}.dag, src/v2/compiler/{resolve,name_resolve,body_lowering,closure_emission,dependency_demand}.dag,
stage0 claim_batch.rs/cli_run{.rs,/claim_execution.rs,/emit_host.rs}). Full PRED-vs-head enumeration over
src|std|extdeps: 123 files differ; 1 are PR files (4-file PR delta ∩ list: dag.dag, grammar.dag — the other two
PR files sit under test/workflow), 122 outside the PR, ALL attributable to the MB..A window (none MERGE/OTHER).
Composition: 41 src/v1/stage0/src + 8 cli_run (generated Rust mirrors), 8 src/v1/*.dag (00_core, 04_infer,
04_method, 04_resolve, 04_types, 05_emit, 05_emit_rust, runtime_rust), 17 workflow, ~24 test/claim, plus small
compiler/self_host/extdeps-formats/std/lens sets. CONSEQUENCE: (a) the claim_batch RUNTIME (stage0 Rust + mirrors)
DIFFERS PRED->head; window resolve/infer/emit deltas (incl. 1355ef4fab66 in 04_resolve.dag) are behavioural
deltas vs the authoring state; (b) the demotion of 1355ef4f excludes only the ALIAS-ADOPTION path by source
reading — it does NOT exclude the whole window-delta family, which is the live candidate class for the 13
identical-at-A-and-B FAILs. 'Attribution not established' unchanged. Bisect (PRED-vs-head or windowed) remains
the decider; proposal after v11's control reading per msg_f21a482c.

## Appendix (append-only; v11 END READING recorded, 2026-11-06T15:30Z; parent report msg_ab8e6cf6)
VALID OBSERVATION (exit 0; single EXIT3; rc 1; join held; 140B<=218; controls HELD: ctl_ok=r=0, ctl_out=r=X
=> caret '==' DISCRIMINATES). REASON: ctx=OK; arm/field/binder/body ALL = X{r=X,a=-,s=T,al=0}. Reader:
per-module WcRefused, reason outside the 24-name vocabulary (or comparison failure — controls exclude the
latter), atom ABSENT, sole_chain=True, len 0. All four claim expectations MISMATCH (arm/field/binder expect R;
body expects X{r=0,a=U,s=T}). FALSIFIER DID NOT FIRE. No boundary row (r=X maps to none). Receipts: log sha
29d72107; probe-reason.txt 141B sha c3f963c7 (my extraction 140B minus newline); claim_batch sha 14c09ed8;
peak 13.2 GiB, oom 0; disclosed SELF_ROSTER quirk line preserved. DISPOSITION: v5's all-X line now reproduced
IN SHAPE under held controls at A pin (v5 stays OBSERVED BYTES ONLY). Consistent with the standing
'refuse-everything via outside-table reasons' hypothesis at producer level; neither falsified nor proven.
Open: WHICH outside symbol; whether rtp/ce fixture modules share the shape. Bisect awaits parent release.

## Appendix (append-only; wording-label correction per parent msg_2d8d0262, 2026-11-06T15:35Z)
Per parent guard: BOTH of the following are labelled HYPOTHESIS — (i) 'the mechanism lives in the
resolve-reason vocabulary on the census path'; (ii) 'parse-surface excluded' — the exclusion is a SOURCE
READING ONLY (not executed), interpreter-path caveat applies. The v11 READING paragraph in the 15:30Z
appendix stands as written (observation only); these labels cover the NEXT-STEP framing in the report
'NEXT' paragraph.

## Appendix (append-only; v12 prepared, 2026-11-06T16:05Z)
Per parent release (msg_2d8d0262): v12 names the refusing reason via rs=<lexeme>; full spec/links in
POST_RUN_PROCEDURE v12 appendix. HYPOTHESIS under test by v12 (labelled, not established): the census-path
refusal reason is one of the 15 outside-table lane symbols (v11 showed r=X on all four with controls held;
'outside-table OR comparison failure' is collapsed by the controls to outside-table, HYPOTHESIS-grade).
v12's rs reading either locates it (a named symbol among the 37) or falsifies the candidate vocabulary
(unknown lexeme => outside both lists => new datum). Bisect remains unreleased.

## Appendix (append-only; v13 prepared, 2026-11-06T16:30Z)
v12 HELD (keen-otter msg_93520ef6 via sleek-owl-397, relayed msg_eef3db11): reader over-read on
index/name mismatch (parent's repro: r=0 with an in-table name -> MATCH + FALSIFIER + 'chain arm 1'
conclusion) and vacuous pv_line rs fixtures. v13 fixes reader-side ONLY (parent directive): consistency
gate in BOTH readers before any conclusion (raw data + VOID on mismatch), asserted rs fixtures replayed,
inverted-expectation proof, helper-arity audit (pv_line class only), SIGPIPE-safe buffered assertions.
Seed unchanged (b2c2c214); producer .dag byte-untouched; expectations FROZEN (the parent B1 rule
superseded v12's pre-registered 'CONTRADICTION DATA reading' line by contract directive, not by me
editing after output). Comments 6689901373 (bytes+argv) / 6689901812 (diff) / 6689902464 (seed gates 9/9)
/ 6689903146 (E2E + inversion proof + arity audit + 2795-form sweep HELD). Bisect still unreleased.
