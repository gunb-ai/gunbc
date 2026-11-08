# ce-2-FAILs — SOURCE-ONLY NOTE (sleek-seal-226; source reading, no run, no attribution established)

Scope: the two persistent ce FAILs at both pins (matrix A f08a6ae4 and matrix B/v17 495da688: 11 PASS / 2 FAIL):
`a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds` (C1) and
`the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds` (C2),
`src/v2/test/claim/self_host/closure_emission.dag` (UNCHANGED by #15999 — ce already used the authority).

## 1. The parent's arm-order question — answered: NO arm-order defect (source)
- `closure_emission_refused_count` (compiler/self_host/closure_emission.dag:446) and
  `closure_emission_first_refusal` (:581) both fold `emission.modules` with `fold_list`.
- `fold_list` is a LEFT fold over the head-first list: `src/v1/stage0/src/v1_rt.rs:1577` and the generated
  runtime string (`v1_compiler_runtime_rust.rs`, `for item in xs.iter() { acc = f(acc, item.clone()) }`);
  a separate `fold_list_right` exists and is NOT used here.
- So `closure_emission_first_refusal` keeps the FIRST Refused arm in module order (Absent is overwritten only
  while Absent), extensionally EQUAL to the claim's own `ce_first_refused_reason` selection
  (`list_find_first` over `!ce_arm_is_emitted`, claim :371). Both then read the cause via
  `diagnostics_fatal_reason` (claim :360/:366, arm helper :258-267). C2's two derivations agree on every
  input; C2 cannot fail by arm-order or by head-vs-fatal disagreement.

## 2. What C2=false can mean (source-forced)
C2 = `Accepted && refused_count >= 1 && ce_reasons_agree(...)`; the agree-conjunct is a source tautology,
so C2=false implies, at the fixture `ce_emission_member_body_refuses()` (:159):
 (a) outcome is Rejected, OR (b) refused_count == 0 (every arm Emitted).
C1=false additionally allows (c) modules length != 3 / member arm count != 1 / (d) fatal reason !=
`^body_lowering_reason_else_less_if_unlowered`. But (c)/(d) alone keep C2 TRUE (a refused arm exists), so the
OBSERVED pair (both false) forces (a) or (b): the emission either refused WHOLE, or contains NO refused arm.

## 3. Where the boundary most probably is (HYPOTHESIS, source reading)
- Whole-Rejection candidates are narrow (parse :426/:182; normalize loop :219-244; `validate_module_roots` :353;
  entry-not-in-roster :379; ownerless-reference :305-307 is arm-carried via the :325 match, not whole). The other 11 ce claims PASS, including typed-refusal claims whose refusals ride the
  arm path, and the Accepted-path DIRECTION-1 claim — so door plumbing and arm-carrying are not broken in general.
- The most consistent single boundary: **the fixture member's body LOWERS (is Accepted) at this seed**, i.e.
  `fn ce_refusing_fn(p: Bool) -> Bool { if p { p } }` (entry calls it) — an ELSE-LESS `if` as the ONLY/LAST
  statement with a NON-returning arm, in value-owed position — is being ACCEPTED instead of refused.
  Then both members are Emitted: C1 arm-not-refused false; C2 count==0 false. Both bare Bool false. Coherent.
- Why rtp's PASSes don't contradict it: the rtp refusal fixtures that PASS cover DIFFERENT shapes —
  `rtp_guard_fallthrough_source` (`if c { i }` FOLLOWED by a statement) and `rtp_guard_last_source`
  (`if c { return j }` — arm ENDS IN RETURN, preceded by a `let`). No rtp fixture covers
  else-less-if-as-last-statement with a NON-returning arm and no following statement — exactly the ce shape.
  The contract comment (body_lowering_fold.dag:12672-12677, "AN ELSE-LESS if IS LOWERED ONLY AS A GUARD,
  AND REFUSED EVERYWHERE ELSE") demands refusal here via 12948-12949 / 9559-9560; the merge of 1,112 files is
  the only known movement since these predictions were authored (at 9c15c98c, pre-merge). Attribution NOT
  established; this is the candidate EARLIEST UNJUSTIFIED BOUNDARY (§6b) for the ce pair.

## 4. Positive control statement
The ce file's own PASSing refusal claims (entry-not-acquired, unbound-name, absent-import: all arm-carried,
all read via `diagnostics_fatal_reason`) show the refusal lane the claims consume works; the failing pair is
local to THIS fixture's shape. The rtp PASSing refusal claims show the exit-scan refuses the shapes they
fixture. Neither lane's PASS is for-the-wrong-reason evidence against this candidate; they simply do not
cover the ce shape.

## 5. Next step (no action taken; for release decision)
A cheap discriminator later (single granted entry, NOT in v15's frozen roster): run the two ce fns plus one
probe that reports, for `ce_emission_member_body_refuses()`, the outcome kind and, per arm, Emitted/Refused
+ fatal reason (same instruments; no new vocabulary). (a) both arms Emitted => the acceptance-path candidate
is located at the guard/last-statement acceptance arms of body_lowering_fold; (b) Rejected => name the door
reason; (c) a refused arm with a foreign fatal => reason-mismatch, and C1 alone is explained. Until then the
ce 2 FAILs stay OPEN, tracked separately from #15999 (which does not touch ce or compiler code).
