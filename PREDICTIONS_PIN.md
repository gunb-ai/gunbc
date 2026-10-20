# Pre-registered predictions — PR-1 as the pin (ruling (B)+(C)), head af4fcc078a1

Written BEFORE any run at this head. Measured tree = `af4fcc078a1` (pr1-producer-repair after the
mint is removed), pushed as `refs/heads/probe/pr1-pin-tree`. Compared against `origin/main` at the
tip printed by each script's own `MEASURED_AT` line. The old mint-era predictions live in
`PREDICTIONS_NARROWING.md` (N1–N6) and are no longer what ships.

The tree: `src/v1/04_infer.dag` and `src/v1/stage0/src/v1_compiler_infer.rs` are byte-identical to
main's (`DAG_IDENTICAL_TO_MAIN`, `MIRROR_IS_MAIN_PROJECTION`, both verified locally before the
commit). The only diff vs main is 3 files: the new pin row, its `lib.rs` registration, and the
rewritten failure-mode row.

| # | Prediction | If wrong |
|---|---|---|
| B1 | `pr1_row.sh <head> pin` prints `GUARD acc=0 mirror_fresh_marks=0 dag_namekeyed=0 dag_span_keyed=0` and `ROW_RC=0`. A `VOID` here means the guard is measuring the wrong tree, not that the row failed. | fix the guard, re-run; never report a VOID as a verdict |
| B2 | The row's own output is `MINT-BOUNDARY-INVENTORY positions=3 distinct=1 minted=0 calls=1 sample=["S"]` — the same `positions/distinct/calls` the mint-era narrowing inventory measured at this fixture (`positions=3 distinct=1 calls=1`), with `minted=0` and the callee's declared parameter `S` still present. `positions=0` or `minted>0` = the row is vacuous or the tree mints; both are failures, not passes. | treat as VOID, fix the fixture |
| B3 | `gate_emit_build_14152.sh <head> pin` gives `SELF_HOST_RC=0` and `EMITTED_BUILD_FAILED_LINES=0`. This is the claim the whole (B) ruling rests on: removing the mint removes the emitted defect. A main control at the same tip must also give 0; if head≠main here the defect is mine and the head is NOT pushed. | do not push; report the blamed emitted line via `emitted_error_site_14152.sh` |
| B4 | `gate_two_closures_14152.sh <head>` shows NO new hard-diagnostic identity versus main in either closure. With the infer authority byte-identical to main, any new identity is a false alarm from the harness, not from the change. | stop, name the identity, re-derive |
| B5 | `lib.rs` vs main has no `<` lines (purely additive) and every `mod *_test;` is still preceded by `#[cfg(test)]` — the defect review 82875 blocked. | restore before push |
| B6 | The failure-mode row still parses (`parsecheck.py` rc=0), carries no transcribed count (`\b(43|45|76|19|11|39)\b` → 0), and names no artifact that does not exist in the tree (`minted_variables_are_fresh_per_call_site`, `generic_call_variable_freshness_test`, `arguments_present_do_not_trigger_the_call_site_mint` → all 0). review 82881 blocked exactly this class. | rewrite before push |

B3 is the gate for pushing the PR head; CI on the head then remains the final self-host word
(ruling (2)=(A) — the gate costs one CI-time build and stops red pushes that cost a review round).

## Outcome of B2 — FALSIFIED, and what that taught

B2 as written is wrong and was deleted from the row rather than adjusted after the fact. Measured at
`af4fcc078a1` (kind=pin, ROW_RC=101): `MINT-BOUNDARY-INVENTORY positions=0 distinct=0 minted=0
calls=1`. Main records **no** type variable at the pinned position at all — the callee's declared
parameter is handed over without instantiation and nothing notes it, which is the silent class the
failure-mode row documents. So `minted.is_empty()` is satisfied on main by there being nothing to
satisfy, and the predicted `sample=["S"]` was a guess about main: the `positions=3 distinct=1
calls=1` half was copied from the mint-era narrowing inventory, which was measured **on a minting
tree** (`probe/pr1-narrowing-red`: `positions=3 distinct=1 minted=1 sample=["S@test.dag:355-367"]`),
and the `["S"]` half was inference about how main would represent the uninstantiated parameter.
Neither was a measurement of main.

The row now asserts the mint boundary plus the two fatal non-vacuity guards and prints the
inventory; a comment in the file states that main records nothing at this position, so green means
"no mint crossed the boundary", not "main types the call". Its discriminating power is the measured
minting-tree inventory above, which is what makes it red if a later mint widens past the boundary.

Lesson recorded where it will be read again: a prediction copied from a measurement of a DIFFERENT
tree (here, a minting tree) is not a prediction about the tree under test. Predictions that have not
been measured against the base are guesses and must be labelled as guesses.

## Outcome of B1-B6 (measured at the merged head, written after the runs)

Head measured: 5cc80be8e59141785fc32250d0d95b7a53d2841f (`pr1-producer-repair`, a merge commit whose
second parent is origin/main 7d4cadbe844; pushed as refs/heads/probe/pr1-pin-tree). Earlier heads in
this section (af4fcc078a1, ea5e0d4951f, be322cacc97, 0b1f8af7c68) are superseded by main moving
four commits; each was re-measured at the final head rather than carried forward.

- B1 PASS: `pr1_row.sh <head> pin_merged2 pin` printed `MEASURED_AT 5cc80be8e59…` and the tree-shape
  guard `GUARD acc=0 mint_markers=0 dag_span_keyed=0`.
- B2 PASS with the correction recorded above: `SYNTAX_RC=0`, and the row verdict line is
  `test result: ok. 1 passed; 0 failed; 60 filtered out` (the script's own header says the verdict is
  the `SUMMARY`/`test result:` line; `ROW_RC` is not printed by this script -- an earlier reading of
  mine claimed it was, and that reading was wrong). Inventory at the fixture on a tree with no mint:
  `MINT-BOUNDARY-INVENTORY positions=0 distinct=0 minted=0 calls=1 sample=[]` -- the call reaches the
  arm (`calls=1`, both fatal guards satisfied) and main records no type variable there.
- B3 PASS: `gate_emit_build_14152.sh <head> pin_merged2 pin` gave `build rc=0`, then
  `SELF_HOST_RC=0 lines=25` with `EMITTED_BUILD_FAILED_LINES=0` and `LOCATED_BY_GLOB=NONE`. Controls
  on main tips gave the same `SELF_HOST_RC=0 lines=25`, so the metric is not merely green but
  unchanged. The mint-era head at the same instrument was `SELF_HOST_RC=2` with
  `EmittedCompilerBuildFailed`; withdrawing the mint is what closed it.
- B4 IN FLIGHT at the time of writing (see the two-closure log); recorded separately when read.
- B5 PASS: `git diff origin/main...HEAD -- src/v1/tests/src/lib.rs` has 0 deletion lines (purely
  additive), and an awk scan of `lib.rs` finds no `mod *_test;` line whose predecessor lacks
  `#[cfg(test)]` -- the defect review 82875 blocked, checked again at the merged head.
- B6 PASS: `parsecheck.py <row>` rc=0; the transcribed-count regex `\b(43|45|76|19|11|39)\b` matches
  0 lines; the three withdrawn artifact names each appear 0 times; and the row's three emitter
  citations still resolve at this head after main's commit 6215e222ba8 added 132 lines to
  `src/v1/05_emit_rust.dag` -- the line numbers were re-checked, not assumed, because that commit
  touched one of the cited files.

Merge verification at the same head, in the form stern-bat-401 asked for (msg_1c6d0180): the merge is
a true merge commit (two parents), the stage0 projection needed no regeneration (268 files under
`src/v1/stage0/src` byte-identical to origin/main, `DRIFT=0`; no hand-written module under
`src/v1/tests/src` has a stage0 projection, and no `*.generated.*` artifact names any
`recurring_failure_mode` row, so a new row cannot create projection drift), the diff against main is
exactly three files and all additions, and the deleted-symbol grep over
`4e32f5a1007..7d4cadbe844` found the only declaration-level removals to be a `pub use` of
`function_value_params` / `is_host_text_carrier_type`, both of which are absent from my merged tree
(main's deletion taken, not silently retained).

## B4 re-run, pre-registered BEFORE the run (the first attempt's verdict was unreadable)

The first B4 dispatch (job jefbf6d42, exit 0, 42 min) is VOID as evidence, for a reason of my own making:
the script ends by tar-gzipping its logs and base64-encoding them inside one 200 KB line, and my dispatch
printed the verdict with `tail -c 4000` -- which landed inside that base64 blob. No number from it is read,
and I am not treating its exit 0 as a pass. Fix: capture the band between `PINNED head=` and `GATE DONE`
with awk, not a byte tail.

PREDICTION at head 5cc80be8e59 (pin tree) vs main 7d4cadbe844, per closure, before the comparator runs:
- Both sides' binaries are built from the SAME compiler sources: `src/v1/04_infer.dag` and all 268 stage0
  projections are byte-identical to origin/main at this head, and nothing else in the diff is a compiler
  input (the third file is a v1 Rust unit test, and the RFM row is imported by no closure -- verified:
  RFM rows reach a closure only through an explicit `import gunbc.recurring_failure_mode.<name>`, and no
  file outside `dag/gunbc/recurring_failure_mode/` names my row). So the two sides must AGREE exactly.
- self-host: `type-mismatch=0`, and the comparator's two identity sections are both EMPTY (0 identities at
  HEAD that MAIN does not have; 0 identities MAIN has that HEAD lost). `unparsed` must be 0 on both sides,
  else the counts are incomplete and the run is not usable.
- v2-native-cli: same expectation -- 0 hard diagnostics beyond main's, and both identity sections empty.
- The Acc-era numbers 43 and 45 must NOT appear. If either closure reports 43 or 45 conflict rows, that is
  a stale-log artifact of this instrument (it reuses `$LOGS` and a shared target dir across dispatches), not
  a property of this head, and the run is void rather than a red.
- Each side's `build rc=0` with a non-empty `binary=`, and `emitted>0` per closure per side; `emitted=0`
  means the log never reached an emit and is NOT a green.

## Outcome of the B4 re-run (job ja93372e7, exit 0, 47m) — every pre-registered prediction held

Head 5cc80be8e59 (refs/heads/probe/pr1-pin-tree) vs main 7d4cadbe844, both sides built from pinned refs
with a private CARGO_TARGET_DIR, in one dispatch. The verdict band was captured this time by
`awk '/^PINNED head=/{f=1} f{print} /^GATE DONE/{f=0}'` between the BEGIN/END markers, not by a byte tail,
so it is readable.

- self-host, head:  `rc=0 lines=25 type-mismatch=0 emitted=2`;  main: `rc=0 lines=25 type-mismatch=0 emitted=2`
- v2-native-cli, head: `rc=0 lines=32 type-mismatch=0 emitted=2`; main: `rc=0 lines=32 type-mismatch=0 emitted=2`
- both closures: `0 hard diagnostics, 0 distinct identities, 0 unparsed` on EACH side; the two identity
  sections are both `0 ... ` (empty) and `comparator rc=0`.
- PREDICTIONS CONFIRMED as pre-registered: type-mismatch 0 per closure, both identity sections empty,
  unparsed 0 on both sides, both sides `rc=0` with a real `binary=`, `emitted>0` per closure per side (so no
  side is a false green from a log that never reached an emit), and the Acc-era counts 43/45 do NOT appear.

This is the push gate stern-bat-401 required (msg_79410475, msg_1c6d0180): the pin tree is byte-identical to
main on every compiler input, so both closures refuse nothing that main does not already refuse, and lose
nothing main refuses. Combined with B3 (emit-and-build of the self-host product, SELF_HOST_RC=0 lines=25,
matching main's control) this head is green on every gate I can run; CI on the head is the final word.

## Gate re-run at the documentation-only head 7c32c1359f3 (job jf111355c)

NOT pre-registered in the strict sense: I dispatched the run and am writing this while it is still
running, so these are expectations stated after dispatch but before reading any result. Stating them
anyway is better than reading the numbers first and agreeing afterwards; the reader should weigh them
as weaker than B1-B6, which were written before their runs.

Head 7c32c1359f3 = merge commit of origin/main 9de8941f73f (ten commits, incl. #14354 and #14524) + the
row-only change. The diff against origin/main is ONE file, a `dag/gunbc/recurring_failure_mode/` row:
  - corpus parse: expect CORPUS_PARSE_RC=0 / VERDICT=CLEAN / ERROR_LINES=0 with a real binary under the
    private target dir (not a ctrl-build shim). The previous head at this gate gave exactly that. A
    non-zero here can only come from my row's text (the quote bug class), since no code changed.
  - emit-and-build: expect build rc=0 and SELF_HOST_RC=0 lines=25 with EMITTED_BUILD_FAILED_LINES=0,
    matching the controls on main tips. A red here would be the MERGE's fault, not the row's -- the base
    moved ten commits since the last emit-build measurement -- so if it is red I must re-run it against
    plain origin/main 9de8941f73f before attributing anything to this PR.

## Prereq (b) executed A/B — pre-registered BEFORE dispatch (job to follow)

Question from stern-bat-401 (msg_11dcdef2): "does substituting in transparent_alias_rhs move the
diagnostic produced at variant_reference_inferred_node? If they are different seams, prereq (b) is
re-scoped to the owner-identity carry at variant_reference_inferred_node."

Both trees are built from origin/main (9de8941f73f) + one probe test module; the ONLY difference between
control and mutant is the removal of the single guard term
`|| ((pre.params.clone().len() as i64) > 0)` from transparent_alias_rhs (mutated in the compiled mirror
src/v1/stage0/src/v1_compiler_infer_env.rs AND in the authority src/v1/04_env.dag, so the verdict cannot
depend on which of the two the build honours). The four programs are read out of the authority CI already
runs, dag/test/claim/algebra_receiver_alias_witness.dag: nominal_return_source (-> FreeMonoid<Int>,
accepts), alias_return_source (-> List<Int>), alias_field_source (type Box { values: List<Int> }),
alias_initializer_source (let initial: FreeMonoid<Int> = Empty, accepts).

PREDICTED CONTROL (guard intact), matching what the existing CI-run witness already asserts:
  ALIAS-PROBE TREE guard_present_in_compiled_mirror=true
  nominal_return_accept      hard=0
  alias_return_refuse        hard>=1, every diagnostic a MethodNotFound whose receiver_type is Coproduct(FreeMonoid)
  alias_field_refuse         hard>=1, same class and subject
  alias_initializer_accept   hard=0
If control shows the two refusing rows CLEAN, the whole experiment is VOID (it would mean the tree I built
does not reproduce the CI-run witness) and no conclusion about the seam may be drawn.

PREDICTED MUTANT -- three outcomes, named in advance so I cannot retrofit to whichever appears:
  (i)  the two refusing programs go hard=0  => transparent_alias_rhs IS the boundary; prereq (b) stands as
       TypeBinding-gains-declared-params + substitution at that guard.
  (ii) the two refusing programs still report the SAME class+subject (MethodNotFound / Coproduct(FreeMonoid))
       => the guard is NOT the boundary; prereq (b) re-scopes to the owner-identity carry at
       variant_reference_inferred_node (src/v1/04_infer.dag:741), which the search found is already owned by
       gunbc.guarantee_stall.algebra_application_owner_resolution_stall (blocker.grounding names exactly that
       carry) and enrolled in gunbc.explicit_witness_admission.
  (iii) a DIFFERENT class appears (e.g. an unresolved-declaration complaint about the alias's own parameter
       `element`, because substituting a parameterised RHS hands the caller an uninstantiated parameter) =>
       the guard participates but is insufficient; the carry is still required, so the re-scope in (ii) holds
       WITH an added note that naive substitution manufactures a second defect.
Prior expectation: (ii) or (iii), because the diagnostic is produced where a receiver type is turned into a
variant owner, not where bindings settle -- but the run decides, not this sentence.

## Harness post-mortem: three capture bugs in one lane, all mine, all silent-exit-0

The prereq-(b) A/B lost its verdict three times and every cause was in my own dispatch line, not in the
code under test. All three exited 0, so none of them announced itself:
1. `tail -c 4000` on a gate whose last act is a single 200 KB base64 return-tar -> the verdict was cut off
   and I nearly read the absence as a result. Fix: awk between explicit BEGIN/END markers.
2. A grep pattern listing `VOID|ABORT|FATAL` but not `panicked` -> a probe that crashed before printing
   anything looked like a run that simply had nothing to say. Fix: capture `panicked` too, and make the
   probe print a MARK before and after every step so an early exit names itself.
3. The run function wrote `$3` while the read loop grepped a name built from `$2`, so both sides executed
   (rc=0) and both logs were then searched under the wrong filenames. Fix: write and read must share ONE
   variable, and the read must echo the byte count so a missing file cannot masquerade as an empty verdict.
General lesson for this repo, where an exit code routinely says nothing about whether a verdict was
produced: a measurement is only as trustworthy as its capture, and the capture must be able to distinguish
"no diagnostics" from "no data". Two of these three would have been caught by asserting the log is non-empty
before grepping it.

## Alias-seam A/B via the repo's own witness runner — expectations written after dispatch, before reading

Dispatch j23134f2f (gate_alias_witness_14152.sh, ONE build of v1-compiler-tests, both trees). This is
NOT a clean pre-registration: the run was dispatched first. It is written before any result was read, and
the three outcomes are named now so I cannot retrofit to whichever appears.

Trees differ in exactly the params>0 term of transparent_alias_rhs (control ab0ec96c7b0: term live in both
src/v1/04_env.dag and the compiled mirror; mutant 88a531a2ed: term constant-false in BOTH). Four programs
come from dag/test/claim/algebra_receiver_alias_witness.dag (the authority CI runs), evaluated by its own
helpers total_blocking_count_for / blocking_subject_count_for, whose not-runnable arms return -1.

CONTROL expectation (must hold or the whole run is VOID -- it would mean the runner is not reproducing
what CI asserts):
  nominal_return_is_an_acceptance_fixture        total = 0
  alias_return_is_a_refusal_fixture              blocking subject Coproduct(FreeMonoid) class MethodNotFound = 1
  alias_field_is_a_refusal_fixture               same class + subject = 1
  explicit_initializer_is_an_acceptance_fixture  total = 0
  CONTROL-RING (the -1 probe)                    total = 0, class_probe = -1, subject_probe = -1
If the two refusal rows read 0 in CONTROL, my runner is measuring nothing and no conclusion is drawn.

MUTANT -- three named outcomes:
 (i)   both refusal rows go total = 0  => transparent_alias_rhs IS the boundary; prereq (b) stands as
       TypeBinding-gains-declared-parameters plus substitution at that guard.
 (ii)  both still report MethodNotFound / Coproduct(FreeMonoid) = 1  => the guard is NOT the boundary;
       prereq (b) re-scopes to the owner-identity carry at variant_reference_inferred_node
       (src/v1/04_infer.dag:741), which the search found already owned by
       gunbc.guarantee_stall.algebra_application_owner_resolution_stall.
 (iii) a DIFFERENT class appears, e.g. a complaint naming the alias's own parameter `element`, because
       substituting a parameterised RHS hands the caller FreeMonoid<element> with `element` unbound  =>
       the guard participates but is insufficient; (ii)'s re-scope holds, WITH the added finding that naive
       substitution manufactures a second defect. This is my leaning, because the RHS of List<element> is
       FreeMonoid<element> and nothing at the use site binds `element`.
NULL RESULT, also named: if CONTROL and MUTANT print byte-identical output for ALL rows, that is not
evidence the guard is irrelevant -- it may mean the mutated .rs/.dag pair is not what the built binary
used. The guard-state lines printed from the built tree are what distinguishes these two readings, so a
null result is only interpretable if those lines differ.

## PR-2 two-closure at the Acc-only head (session/quiet-hawk-507 = 97e47f8e562)

PRE-REGISTERED before the first dispatch:
  T1 both sides rc=0 and lines>0, else VOID (no counts readable).
  T2 the Acc-only head is NOT corpus-clean: Map-family structural refusals appear,
     self-host in the 70s, v2-native in the 60s, both closures emitted=0.
  T3 no main identity disappears (no silent fix), and the new identities include the
     "A bound as Map(Primitive(K),Primitive(V)) ... got Map(Primitive(String),Coproduct(..))" shape.

MEASURED (j05cf96e0, reproduced identically by an earlier independent dispatch):
  PINNED head=97e47f8e562 main=7956cb45df8 instrument=209c5ab7f54
  [head] self-host       rc=2 lines=88 type-mismatch=76 emitted=0   -> T2 CONFIRMED (76, self-host)
  [head] v2-native-cli   rc=2 lines=79 type-mismatch=67 emitted=0   -> T2 CONFIRMED (67, v2-native)
  [main] self-host       rc=0 lines=25 type-mismatch=0  emitted=2
  [main] v2-native-cli   rc=0 lines=32 type-mismatch=0  emitted=2
  comparator rc=0 both closures (json 1240 / 1363 bytes)
  T1 PASS (all four sides produced data; rc=2 is the MEASURED refusal, not an abort).
  T3 is POST-DISPATCH: the by-identity table is printed only by instrument 054fd88d1cf,
     run afterwards; read it before asserting "no main identity disappeared".

CORRECTION TO MY OWN EARLIER RECORD: the "0 hard diagnostics / 0 identities both
closures" entry was measured at 5cc80be8e59, which carries NEITHER UnifyGenericsAcc NOR
the span-keyed mint (verified by git grep on that ref). It validated PR-1 only and was
wrongly cited as PR-2 evidence. Any count from 5cc80be8e59/e5cc5d903e1/7c32c1359f3 says
nothing about PR-2's fold.

GATE DEFECT FOUND AND FIXED: gate_two_closures_14152.sh exits 0 for "measurement
completed", NOT for "green"; and it printed only the comparator JSON's byte count. Both
made a red head look like a completed pass. It now prints the per-module JSON in full.

## Family fixtures (probe/family-map-fixture, off the Acc head 97e47f8e562)

Data-bearing dispatch j2ee41ed9 at probe head b9450e06eac. The two earlier dispatches
(811e9f1493a, 4d3429c7f89) were COMPILE failures surfaced as VOID (E0603; FAM_LINES=0,
VOID_NO_TEST_RESULT) -- they produced no data and are not runs.

F1-F5 pre-registered in the message to stern before any dispatch:
  F1 unannotated empty_map() call as a Map argument -> REFUSES (the Map(Primitive(K),Primitive(V)) shape)
  F2 the identical program with the call bound through an annotation -> ACCEPTS
  F3 same_shape<T> over two distinct concrete Maps -> REFUSES (true disagreement)
  F4 same_shape<T> over two identical concrete Maps -> ACCEPTS
  F5 map_insert on a parameter-typed Map -> ACCEPTS (control; if it refuses everything is void)
  F1 refuses + F2 accepts => family (ii) PRODUCER DEFECT, home upstream of the Acc.

F6/F7 recorded here, written AFTER the fixtures were added but BEFORE reading the
data-bearing run's output (that run had not completed when these lines were written):
  F6 warm-wolf's verbatim shape (same_arm<T>(a:T,b:T) over P1{a:Int} vs Q1{a:Int}, two distinct
     coproducts) -> my prediction: ACCEPTS on the Acc head, i.e. the fold does not refuse it,
     because the tag route is the constructor-tag defect (one_type_variable_bound_to_two_distinct_coproducts),
     which PR-2's node_type_equals_core widening targets and which is a SEPARATE change (parent's (b) split).
     Consequence if confirmed: Charter 4's ExpectedRed rows do NOT flip from PR-2 as currently
     composed; they flip only if the equality widening lands. warm-wolf must be told this
     BEFORE their enrollment is relied on.
  F7 same_arm over two values of the SAME coproduct -> ACCEPTS (if it refuses, the Acc is
     over-strict on constructor comparison generally and subsumes F6's reading).

## PR-2 head's 04_types.dag is FOUR separable changes (read from origin/session/quiet-hawk-507 diff)
  1. EQUALITY WIDENING (parent's (b) split, PR-2a): type_head_carrier,
     type_head_is_declared_carrier_alias, type_heads_agree -- carrier-alias heads compare by
     carrier, else by declaration identity. Must land alone with its own RED.
  2. RENDERER (prereq (a)): node_type_shape now renders Product(name<fields>), Map(<real args>)
     instead of "Map(...)", Container(carrier,...), and a NEW UnresolvedDecl(...) arm. This is
     exactly prereq (a) "render what the comparator compares" -- already present in PR-2, so it
     must be split out too or PR-2a's RED cannot be attributed.
  3. ALIAS MACHINERY (prereq (b), CLOSED by parent): peel_alias_rhs_once + imports
     lookup_binding_by_name/alias_settled/agree. Parent (c): show a run where the Acc refuses
     without them, or they leave PR-2.
  4. MERGE ARTIFACT: TextNotAskedForCallableComponentResidue restored. Its own comment says #14287
     deleted the variant AND its construction; this branch's whole-function rewrite of
     callable_leaf_substitution auto-merged with no conflict and re-created the construction while
     the definition stayed dead -> a 30-min remote regen refused to emit ("undefined variable").
     No discriminating consumer (5 wildcard arms, 0 discriminating). This is the silent-semantic-merge
     class parent warned about; report it, do not fix it inside PR-2.

## Family-run failure modes (three dispatches, all reported as VOID, never as passes)
  j2ee41ed9 E0603 private re-export (compile); jb9701ba5 BUILD_RC=0 (4m20s warm, binary cached);
  j6c2e970a TEST_RC=124 at my internal 2700s; jf4e9bad2 TEST_RC=124 at 900s;
  jbad8b8df TEST_RC=124 at 2700s with "Running unittests" printed and FAM_LINES=0.
With --test-threads=1 libtest runs ALPHABETICALLY: the first test is fam_f1_unannotated_empty_map.
So the un-instrumented 45 minutes were consumed inside F1's census call, before its BEGIN could be
observed (BEGIN now prints before the call; the previous binary predated it). PENDING per-test
confirmation (jd52f0d3b, timeout 200 per test): F1 = unannotated empty_map() appears NON-TERMINATING
on the Acc head for a 6-line module. If confirmed, that is substantive evidence for parent's
family question: (ii) Acc/producer defect -- a true source disagreement would refuse, not diverge.
CI's 76-diagnostic run did not hang, so the divergence may need the single-module census path
(compiling std per call) rather than the corpus path; naming that difference is part of the verdict.

## CAPTURE BUG #5 (mine, jd52f0d3b): `--exact` with an unqualified name filtered everything out
All seven per-test invocations printed "test result: ok. 0 passed; 101 filtered out" rc=0 -- a
green report from a run that executed NOTHING. libtest --exact compares against the MODULE-QUALIFIED
path (map_family_disposition_test::fam_f5_...), so a bare fn name matches no test. The fix in the
dispatch (jc7841d74): substring filter, a `$BIN --list | grep -c fam_` presence assertion BEFORE
any run, and an explicit VOID_NO_MATCH when a run reports 0 passed. Rule extracted: a test command
must prove it selected the test it claims to run; "ok, 0 passed" is never a result.

UNEXPLAINED, and I will not paper over it: jbad8b8df ran `cargo test ... fam_ -- --nocapture
--test-threads=1` (substring, so it matched), printed "Running unittests", then consumed ~40 min and
TEST_RC=124 with FAM_LINES=0 -- yet MEASURED_AT=2790ba4bd6c which contains `println!("FAM ... BEGIN")`
BEFORE the census call. Either the print never reached the redirected stdout, or the first test never
started. If jc7841d74 (direct binary, per-test timeout) also shows no BEGIN, the fallback is
eprintln! (stderr is unbuffered in Rust) -- a measurement that cannot be observed is not a measurement.

## Family fixtures at the Acc head: INSTRUMENT RESULT (jc7841d74, MEASURED_AT 2790ba4bd6c)
Presence verified via $BIN --list (all seven module-qualified names). Every test printed BEGIN;
NO ARM line for ANY fixture; all seven killed at 200s -- including F5 (four-line control, no
generic call). RETRACTED: my PENDING hypothesis that F1 was the non-terminating shape; F5 diverges
too, so no shape is special. Comparison caveat: main-side census data (alias A/B, rows=24 in ~10
min) came from a different ref, so it is an indication, not identity-grade. Candidate boundary
NAMED (not concluded): the head's fold adds generic_alias_unfold / generic_type_unify_unfold /
peel_alias_rhs_once; an unfold/peel cycle making no progress fits the symptom. Consequences: the
census-based (i)/(ii) family verdict is UNDECIDABLE today at this head; Charter 4's F6 flip
prediction stays open; consistent with the gate's emitted=0 (the fold fails to return, not merely
refuses).

## STRIP measurement (parent msg_37357ae1): Acc head with node_type_equals_core reverted
Tree: scratch/acc-without-equality-widening (probe ref, never ships). .dag: node_type_equals_core
byte-identical to main (verified by function-level diff, IDENTICAL TO MAIN); the three helper fns
remain but are DEAD (zero call sites in the .dag). stage0 mirrors: REGENERATED BY THE EMITTER from
the .dag before any run (rule: mirrors are emitter output only; a .dag-only strip would measure
nothing, because the mirror is what executes -- the alias A/B lesson in reverse).
PRE-REGISTERED before dispatch:
  S1 both closures produce data on head AND main (rc + lines readable) else VOID.
  S2 if the head-vs-main identity table is UNCHANGED from the unstripped Acc head (17 identities/
     76 self-host; 18/67 v2-native, main empty), the widening has NO CONSUMER in PR-2 => PR-2a dropped.
  S3 if the Map-family identities (A bound as Map(Primitive(K),Primitive(V))...) persist with the
     widening stripped, the widening never was the Map-family cause -- supporting the view that the
     hardcoded-table objection survives whatever the consumer answer is.
  S4 (new, learned today) if either closure fails to terminate, record it as a finding and do not
     retry silently.

## PR-2 BLOCKER (stern ruling msg_f1427119): non-returning fold on trivial input
At the Acc head (97e47f8e562, probe 2790ba4bd6c) compile_dag_diagnostic_census did not return
within 200s for ANY of seven fixtures, including the four-line control F5 (fn f5(m: Map<String,Int>)
-> Map<String,Int> { map_insert(m: m, key: "k", value: 1) }). DESIGN §4 makes bounded, forward
execution a property of the substrate: a type checker that hangs on trivial input cannot land
however the families come out. LIMIT, stated as ruled: this is AT 200s and NOT YET SHOWN to be
divergence -- slowness is not excluded. Consistent with the gate's emitted=0 at the head.
NEXT PER RULING, only if the strip-head gate still diverges on F5: F5 ALONE at three heads --
main, the Acc head, and the Acc head minus generic_alias_unfold/generic_type_unify_unfold/
peel_alias_rhs_once. One input locates the boundary without a long run.

WITHDRAWN by stern-bat-401 (msg_216dbb24), as confounded, and the withdrawal is recorded here beside the
claim as instructed. The confound, measured: compile_dag_diagnostic_census_uncached (src/v1/stage0/src/
cli_run/emit_host.rs) re-indexes the witness-root corpus and closes the fixture's imports on EVERY call --
its content-keyed memo guard was deleted with run_required_floor -- and fixture F5's source says
`import std.algebra`, which is Charter 6's single largest native resolve root. So the 200s non-return was
corpus-scale work the four-line fixture never asked for, not the type checker failing on trivial input. The
three-head F5 discriminator is NOT run. Independent confirmation that the compile path is not hanging: the
two-closure gate's FULL-corpus compile completes at both the Acc head and the strip head, which is how
76/67 and 234/228 exist at all.
WHAT REMAINS, and it is not mine to fix: the uncached per-call re-index is a §6 cost-shape defect in the
instrument itself -- an instrument whose per-case cost is the cost of the whole corpus cannot be used to
measure one case. Routed to Charter 4 (warm-wolf-570), which owns that instrument.

## Strip measurement (stern-bat-401 msg_37357ae1 / msg_f1427119): does PR-2's Acc NEED the widening?
COMPOSITION, established by attribution (not assumption) with git log -S on the PR-2 head:
 - the EQUALITY WIDENING = commits ac0aa2dd711 + d943faa1fc1: it is BOTH the type_heads_agree call
   sites AND the applied/applied declaration-identity arm that calls it (head's .dag, struct arm,
   else-branch). Removing one without the other is not "the widening stripped".
 - the ALIAS MACHINERY = peel_alias_rhs_once / generic_alias_unfold / generic_type_unify_unfold
   (d943faa1fc1) — KEPT in the strip, because parent's own three-head discriminator treats it as
   separable from the widening.
 - the renderer and the #14287 merge artifact are untouched.
STRIP BRANCH: scratch/acc-without-equality-widening = PR-2 head with src/v1/04_types.dag's two
type_heads_agree call sites reverted to main's spelling. Helper DEFS stay (type_head_carrier is
live in 04_infer.dag's unify region; type_heads_agree/type_head_is_declared_carrier_alias become
dead but are pub fn, so they cannot cause a dead-code red).
STAGE0: mirrors MUST be regenerated by the emitter from the stripped .dag, because the gate builds
gunbc from the REF and the crate compiles the mirrors — a .dag-only strip would measure nothing.
regen_strip_14152.sh emits, installs, re-builds, re-emits until a round installs 0 (fixed point).

PREDICTIONS, written before any strip run:
S1  regen reaches a fixed point (a round installs 0). If not, the installed mirrors are not emitter
    output and the strip result is VOID, not red.
S2  after regen, the strip's compiled mirror contains ZERO type_heads_agree call sites while the
    head's mirror contains 2 (the pre-regen drift I measured: 3497, 3595). If the mirror still
    calls it, the regen did not take and nothing below means anything.
S3  primary: the strip head's by-identity mismatch sets equal the Acc head's (self-host 76,
    v2-native 67, emitted=0 both). If so the widening has NO consumer in PR-2 and PR-2a is dropped.
S4  falsified if the strip head's counts DROP toward main (both closures): then the widening IS a
    cause of mismatches; report which families move, and any future widening must come from the
    declared alias in std.types, not from the hardcoded carrier table.
S5  the F5 fold divergence: if the strip head still diverges on F5, the boundary is not the widening
    (already ruled out: F5 diverges on main and the Acc head, where the widening is absent); the
    next discriminator is the 3-head set parent named (main / Acc head / Acc head minus alias
    machinery), on F5 alone.

## CORRECTION to the "fold does not return on a four-line module" blocker (measured by reading the builtin)
stern-bat-401's ruling treats a fold that hangs on a four-line module as a PR-2 blocker in its own right.
The fixture F5 IS four lines of SOURCE, and I stated it as if it were four lines of WORK. It is not, and
that was checkable before the run. The census builtin's own body (src/v1/stage0/src/cli_run/emit_host.rs,
compile_dag_diagnostic_census_uncached) does, per call:
  build_module_path_index_from_witness_roots()      <- indexes the witness-root corpus
  resolve_virtual_source_with_imports(...)          <- closes the fixture's imports over that index
  compile_fixture_rendering_only_what_is_read(...)  <- the FULL v1 pipeline to the Rust render target
and it is UNCACHED: its doc comment says the prepared-inventory guard that armed the content-keyed memo
was deleted with run_required_floor. F5's source contains `import std.algebra { empty_map, map_insert }`,
and std.algebra is Charter 6's single largest native resolve root (~7948 modules downstream, per
stern-bat-401's own routing). So a 200s non-return on F5 does NOT establish that the type checker hangs
on trivial input; it is confounded by corpus-scale work the fixture never asked for.
CONSEQUENCE: the blocker must be re-measured on a FOLD-ONLY path before it can stand. The census .dag
itself demonstrates that path — it calls unify_generics directly on hand-built nodes with an empty acc
and TextNotAsked text (src/v1/tests/claim/generic_identity_census.dag, bound_from_basis /
bound_by_name_alone), touching no corpus. That is the input for the three-head discriminator parent named
(main / Acc head / Acc head minus the alias machinery), and it costs one test binary, not a corpus build.

### STRIP RESULT (executed). Pins: strip=406dc6d43f7 (Acc head + reverted widening, mirrors = emitter
### output, fixed point round 2 installed=0), compared side = Acc head 97e47f8e562. Instrument
### gate_two_closures_14152.sh, both closures, fresh clones, CI's verbatim invocation.
self-host      strip rc=2 type-mismatch=234 ids=60 emitted=0 | Acc head rc=2 tm=76 ids=17 emitted=0
v2-native-cli  strip rc=2 type-mismatch=228 ids=56 emitted=0 | Acc head rc=2 tm=67 ids=18 emitted=0
BUILDS rc=0/errors=0 both sides, so these are refusals, not compile failures.
S1 HELD (fixed point). S2 HELD (mirror type_heads_agree occurrences 3 -> 1, the surviving def only).
S3 FALSIFIED. S4 FIRED: the widening has a large consumer, so PR-2a is NOT droppable.
S5 not triggered as framed: the GATE path completes at the strip head (it produced numbers); the 200s
   non-return was on the census BUILTIN, whose path is corpus-scale (see correction above) — so the
   three-head F5 discriminator cannot locate a fold hang and was not run.

SUPPRESSION IS PURE: occurrences in strip and absent from the Acc head = 319 (96 distinct); occurrences
in the Acc head and absent from strip = 0. The widening NEVER creates a refusal; it only removes them.
So the Acc alone is 234/228 and the widening is what brings it to 76/67 — the widening is load-bearing
for PR-2's own number, and PR-2a is entangled with PR-2, not separable from it.

WHAT THE WIDENING ADMITS (classified from each row's OWN two printed sides, not by reading code):
  286 (89%)  both sides print IDENTICALLY, e.g.
             expected 'type variable T bound as Node(String<Product(Char<Primitive(Int)>)>) (first
             argument)', got 'Node(String<Product(Char<Primitive(Int)>)>) (later argument)'
             (src/v2/std/text.dag:310-320)
   33 (10%)  sides print DIFFERENTLY, dominated by Container(FreeMonoid,X) vs FreeMonoid<X> — 27 of the
             33 sit in dag/std/algebra.dag, 4 in dag/std/types.dag, e.g.
             expected 'type variable A bound as Container(FreeMonoid,Coproduct(LexRule))', got
             'Node(FreeMonoid<Coproduct(LexRule)>)' (dag/std/algebra.dag:4733-4743)
BY MODULE (suppressed rows): diagnostic 73, dag/std/algebra 47, qualified_name 43, fold_assembly 22,
dependency_demand 20, name_resolve 18, text 16, target_model 14, dependency_demand_census 12,
target_serialize 8.

ON THE §3 "HARDCODED TABLE" PREMISE — measured, and it needs one correction. The widening's carrier arm
does not read a hand-typed List -> FreeMonoid literal. carrier_container_algebra_rows() and
carrier_container_alias_rows() are BOTH derived: they fold algebra_carriers() (dag/std/algebra.dag) and
read each spelling's container_alias_row / container_algebra_row, so "List" arrives via
pascal_alias_spelling(text: "List") declared on the FreeMonoid carrier. The §3 objection still holds in
substance — std/algebra's carrier roster is a SECOND authority next to std/types' declared
`type List<element> = FreeMonoid<element>` — but the fix is not "replace a table with the declaration",
it is "make one authority cite the other", which is a different change and a smaller one.

WHAT THIS DOES NOT SETTLE (stated so the number is not over-read): the 89% identical-print class cannot
be assigned to (i) true disagreement or (ii) Acc-too-strict from shape alone, because identical printing
is exactly what the renderer defect produces — node_type_shape never prints a Disj's children, so two
nodes that differ only there print alike. The discriminator is structural, not textual: build the pair at
the fold and compare printed shape against structural equality. The head already ships that harness
(src/v1/tests/src/cross_coproduct_generic_locate_test.rs, fold_leaf/fold_tv_leaf/fold_app/fold_disj), and
that file's own header records a fold-level assertion that measured FAILED on the shipped seed — so a
fold-level check there can be RED, which is what makes it usable. It is an INSTRUMENT, not a PR-2 carrier:
parent's ruling (d) keeps PR-2's discriminators in dag/test/claim.

## Fold-arm disposition rows (stern-bat-401 msg_356ca7c1: dispose each family with a minimal two-arm
## fixture, not by reading the code). PREDICTIONS WRITTEN BEFORE DISPATCH.
Instrument run_fold_arms_14152.sh; rows fold_arm_rows_appendix.rs appended to the head's own
cross_coproduct_generic_locate_test.rs (so they use its verified harness); heads = strip 406dc6d43f7 and
Acc head 97e47f8e562; rows come from the instrument ref once and are hashed.
 - Both heads: BUILD_RC=0, INSTRUMENT_ROWS_LISTED=7 (a build failure is reported raw, never grepped).
 - D1 one declaration / two printings: FAIL at STRIP, PASS at ACC_HEAD. That pair is the over-refusal claim.
 - D2 one spelling / two declarations: PASS at BOTH (a widening that admits this is silently wrong).
 - D3 no declarations at all: PASS at BOTH (bare spellings must not compare equal).
 - D4 List vs FreeMonoid: PASS at BOTH, expected via arm 1 (carrier), independent of the widening. If it
   REFUSES, the carrier roster is unreachable from the fold and my arm-1 attribution is false.
 - D5 Map without declarations: REPORTED, not asserted; PREDICTED refused at BOTH heads (Map is not a
   carrier and arm 2 needs refs), which is what makes the dominant Map family an arm-2 story.
 - D6 Map with ONE declaration both sides: PASS at ACC_HEAD, FAIL at STRIP.
 - D7 Disj children invisible to the renderer: PASS at BOTH (pins the blind spot). RED would mean the
   renderer already prints what the comparator compares, i.e. prereq (a) landed, and the triage must be
   re-run before any 19/11/13-style count is trusted.
A row that behaves identically at both heads where a difference was predicted is reported as
NON-DISCRIMINATING, not quietly dropped.

## Item 4 (stern msg_216dbb24): node_type_shape must print what the comparator compares.
SCOPE MEASURED BEFORE BUILDING (blast radius, so the fix does not arrive as a surprise red):
 - node_type_shape is NOT purely diagnostic. 60 .dag consumers; one decision site compares its STRING
   OUTPUT to a literal: src/v1/04_infer.dag `got_is_string = node_type_shape(...) == "Primitive(String)"`.
   Its output also becomes diagnostic SUBJECTS that CI-run claims pin (MethodNotFound receiver_type,
   TypeMismatch subject). So an arm change can go red in dag/test/claim, not only in messages.
 - Claim-pinned strings per arm, at MAIN (measured with git grep over dag/test, src/v2, src/v1/tests):
     UnresolvedDecl( / Node(Name<..>)  = 0    <-- the APPLIED-HEAD arm, where declaration identity belongs
     Coproduct(Name)                   = 5    <-- the Disj arm; printing its children would perturb these
     Product(Name<..>)                 = 4
     Primitive(Name)                   = 3    (also the decision literal above)
     Map( / Container( / Named(        = 0
 - THEREFORE: printing declaration identity on the applied-head arm has MEASURED zero claim-pin blast
   radius. The Disj-children half of the fix is not free (5 pinned subjects) and is the half prereq (a)
   named; it goes in the same row with its own RED, and if a pinned subject moves, the claim moves with it
   and says why.
 - Row target: dag/gunbc/recurring_failure_mode. Candidate existing row to extend rather than a new one:
   search first for a renderer/shape row (the Box<Int> / Box<Int><Bool> printing class quiet-eagle-395
   recorded) before declaring a new one.
