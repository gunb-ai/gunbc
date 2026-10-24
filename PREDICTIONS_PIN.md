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

## Item 2 (stern msg_216dbb24): single-authority prerequisite, BEFORE PR-2, its own small PR with a RED.
GAP LOCATED BY READING THE EXISTING CLAIM, not assumed. dag/test/claim/algebra_carrier_roster_witness.dag
already has every_derived_alias_row_key_resolves_through_a_carrier_with_a_profile, which checks each key of
carrier_container_alias_rows() against algebra_carriers()' own spellings. It NEVER compares against
std/types' DECLARED aliases: dag/std/types.dag:119 declares `type List<element> = FreeMonoid<element>` and
nothing checks the roster against it. So the two authorities agree today by coincidence of authoring, which
is exactly the §3 second-authority defect -- a check that agrees with the thing it duplicates proves nothing
when either drifts.
RED TO WRITE (before any fix): a claim that reads std/types' declared alias set and asserts, both ways,
that it equals carrier_container_alias_rows() -- one direction per alias. It must go RED when either side is
perturbed in isolation (rename the declared alias, or drop the carrier's pascal_alias_spelling), which is
what proves it is a real cross-authority check rather than a restatement. Then the fix is to make ONE side
cite the other, and the claim turns green and keeps being the guard.

## FOLD-ARM DISPOSITION RESULT (executed at two heads). STRIP 406dc6d43f7 vs ACC HEAD 97e47f8e562.
Both BUILD_RC=0, INSTRUMENT_ROWS_LISTED=7 at each head, rows byte-identical (ROWS_SHA256=2d61dde6ca04240c),
head identity self-reported by each row from the compiled mirror (strip=1 occurrence, Acc head=3).
Row outcomes are read from the RAW log lines, not from the runner's verdict labels: the label logic keyed
only on the ok-line and mislabelled genuine panics rc=101 as VOID_NO_MATCH (fixed here; a red turned into a
void is worse than no red).

  ROW                                        PREDICTED                     MEASURED
  D1 one declaration / equal arguments       FAIL strip, PASS Acc head     EXACTLY THAT (strip refused=true,
                                                                             Acc head refused=false) -> the
                                                                             over-refusal is REAL and arm 2 fixes it
  D2 one spelling / two declarations         PASS (refuse) at both         PASS at both -> the widening does NOT
                                                                             over-admit at the fold
  D3 no declarations at all                  PASS (refuse) at both         PASS at both
  D4 List vs FreeMonoid                      PASS at BOTH (via arm 1)      FAIL at strip, PASS at Acc head ->
                                                                             arm 1 ALSO lives inside
                                                                             type_heads_agree: the widening created
                                                                             BOTH arms, not just the declaration one
  D5 Map pair, no declarations               predicted refused at both     ACCEPTED at both -> FALSIFIED, and it
                                                                             falsifies MY OWN earlier attribution
  D6 Map pair, one declaration both sides    PASS Acc head / FAIL strip    PASS at both (same mechanism as D5)
  D7 Disj children invisible to renderer     PASS (blind) at both          PASS at both -> blind at BOTH heads

MECHANISM READ AFTER THE RUN (04_types.dag node_type_equals_core, arms in order): the both_maps arm descends
the two Map arguments pairwise and NEVER compares the head or the declarations; the container arm calls
type_heads_agree first; the applied/applied arm calls type_heads_agree then compares arity and children.
So D5/D6 pass at both heads for a reason the widening does not touch, and the dominant corpus Map family
(59 of the Acc head's 76) CANNOT be attributed to arm 2 -- my earlier claim that "Map is not a carrier so its
rows can only reach arm 2" is WITHDRAWN as falsified by measurement. Those rows differ in something the
synthetic pair does not reproduce (candidate: the fabricated-placeholder actuals and different head names in
the real rows, not equal-head pairs).
CONSEQUENCES. (1) D1 and D4 are a genuine executed RED/GREEN pair for the widening -- the discriminator
PR-2a was held for lacking (msg_37357ae1). Per ruling (d) PR-2's SHIPPED discriminators must be .dag claims,
so D1/D4 are the property to re-express there, not the carriers themselves. (2) D2/D3 green at both heads is
the over-admission guard holding at the fold: the widening never makes two distinct declarations, or two
bare spellings, equal. (3) D7 says the 89% identical-print class is genuine renderer blindness at BOTH heads,
so printed-shape triage is unusable until item 4 lands.

## Item (b) single-authority claim — predictions (PRE-REGISTERED before dispatch, 2026-10-20T19:2x)
Head: feat/single-authority-alias-claim @ 9b3a3ca2432 (branch pushed).
Route: gunbc run --claim-run --entry dag/test/claim/kernel_container_carrier_identity_witness.dag --source-root dag
P1 build rc=0 (cargo build --release -p v1-compiler --bin gunbc on fresh clone, private target dir).
P2 EVERY test fn in the module PASSes (existing fixture-compile claims + 4 new: green claim + 3 controls). The green claim must be TRUE for all three arity keys {List->FreeMonoid, Set->FinitePowerSet, Map->FinitelySupportedFunction} — measured: all six anchor tokens present once in std/types.
P3 If a control fails while the green claim passes, the failure is a syntax/eval error (void), NOT a verdict; report rc + output verbatim, do not retry silently.
P4 A syntax error in the appended block does NOT affect the pre-existing claims — if the run reports old claims PASS and mine error, that isolates the new block; fix and re-dispatch.

## 2026-10-20T21:5x review 83586 (PR #14597): which authority binds the alias NAME — pre-registered BEFORE dispatch

Probe battery through gunbc.compile_diagnostic_census (resolves imports against the LIVE checkout),
scratch witness dag/test/claim/alias_drift_sensitivity_probe_witness.dag (scratch worktree only):
- P1 positive (List<Int> accepted at FreeMonoid<Int>, same for Set/FinitePowerSet, Map/FinitelySupportedFunction):
  ZERO blocking rows. If nonzero, the alias is not transparent for some carrier -> names that carrier.
- P2 wrong-carrier (List<Int> param accepted at FinitePowerSet<Int> return): >=1 BLOCKING row, class TypeMismatch.
  This is the anti-decoration control: the harness must be able to report refusal for this shape.
- P3 snake spelling `list<Int>` with NO std.types import of it: >=1 BLOCKING UnresolvedType row => the std/types
  DECLARED alias is the name binder => the census route is sensitive to alias drift => adopt it, retire the text
  grep. ZERO blocking => the roster binds names independently => the census is BLIND to the drift and the text
  check is the only instrument that sees the declared line => keep it, refute finding 1 by execution.
- P4 local shadow `type List<element> = PointwisePower<element>` beside a FreeMonoid use: >=1 BLOCKING row
  (kc_own_freemonoid sibling says local aliases are not unfolded).

## 2026-10-20T22:2x dispatch G: the drift battery RAN (self-correction of a wrong VOID diagnosis)

WRONG CLAIM RETRACTED: I first recorded that the battery was VOID because CompileDiagnosticCensusRow
"is not an export" -> falsified: it IS exported (type CompileDiagnosticCensusRow, gunbc/compile_-
diagnostic_census.dag:75), the precedent witness imports it the same way, and the run printed three
real PASS lines (members executed). What was genuinely ambiguous: blocking_total returns -1 for
CensusNotRunnable, and the anti-decoration control asks >= 1, so its FAIL could have meant "census
could not run" OR "the wrong-carrier shape was ACCEPTED (0)". Entry runs showed no per-file diagnostic
because probe modules declare no main (NoSuchFunction) -- not evidence either way about the census.
Dispatch G (fixed witness 9e3411beb8f, same head) predictions:
- G1: exactly one of {wrong_carrier >=1 PASS, wrong_carrier ==0 PASS} holds, and !=-1 PASSes; if both
  >=1 and ==0 fail while !=-1 fails, the census could not run -> instrument bug, fix before concluding.
- G2: list/set/map accept == 0 blocking rows (P1 unchanged).
- G3: if wrong_carrier is ACCEPTED (== 0), the census route cannot refuse the drift shape it is
  supposed to catch -> the review's proposed semantic route fails its own control; the text check
  stays, and finding 1 is answered by execution.

## 2026-10-20T22:4x dispatch H: is the census's uniform ==0 acceptance call-body typechecking or its ABSENCE?
Pre-registered BEFORE dispatch. Witness dag/test/claim/census_call_typecheck_control_witness.dag
(scratch branch instruments-scratch), census over a synthetic source whose ONLY defect is a call
argument of the wrong ground type: fn takes_int(x: Int) -> Int { 0 }; fn bad() -> Int { takes_int(x: "s") }.
- H1: blocking_total(that) >= 1 -> the census DOES typecheck call arguments; then dispatch G's
  wrong-carrier ACCEPTED (==0) is a real compiler decision (alias/roster driven), and the census is
  structurally unable to serve as the alias-drift instrument for that shape. Reply to review 83586
  finding 1 says exactly this.
- H2: blocking_total(that) == 0 -> the census performs no call-body inhabitance check at all -> it
  could never catch ANY drift shape of this kind; the route is rejected on the stronger structural
  ground. Same reply, different evidence.
- H3: != -1 passes (the census ran) either way; if it fails, the harness is broken and NOTHING is concluded.

## 2026-10-20T22:5x dispatch I: is the compiler's List-at-FreeMonoid acceptance SENSITIVE to std/types.dag's alias body?
On a scratch clone (instruments-scratch), run the accepted probe (alias_drift_list_ok_probe: List<Int>
actual at FreeMonoid<Int> formal, census blocking_total == 0 measured) after mutating dag/std/types.dag:
- I1 rename `type List<element> = FreeMonoid<element>` -> `type QList<...>`: the probe's `List` import
  stops resolving -> blocking_total >= 1 (or census not runnable). The NAME List is bound by std/types.dag.
- I2 wrong body `type List<element> = PointwisePower<element>`:
  (a) acceptance CHANGES to refusal (blocking_total >= 1) => the alias BODY drives the judgment =>
      a live-tree compile route CAN catch drift => adopt semantic route, retire text grep.
  (b) acceptance STAYS == 0 => the seed/kernel binds the carrier identity independently of the alias
      body (kernel_container_carrier_verdict by DeclarationRef) => std/types' alias body is DECORATIVE
      for judgment => no executing compile route can observe the drift => the text check over the
      declared line is the only instrument that sees it. Report (b) verbatim to review 83586.
- I3: baseline (no mutation) re-run stays == 0; if not, harness broken, nothing concluded.

## 2026-10-20T22:5x dispatch H+I SCORED (branch instruments-scratch, BUILD_RC=0, H_RC=0)

H: PASS the_typecheck_control_census_could_run, PASS the_typecheck_control_is_refused -> **H1 held**:
the census DOES typecheck call arguments (an Int formal receiving a String actual is BLOCKING).
Therefore dispatch G's wrong-carrier `== 0` is a REAL ACCEPTANCE, not an absent check. G1/G2/G3 held
(>=1 FAIL, ==0 PASS, !=-1 PASS).
I: baseline PASS (I3). I1 (alias head renamed QList in dag/std/types.dag, verified at line 120): the
member line disappeared -- the run no longer reaches it (consistent with `List` no longer resolving;
raw diagnostics to be captured in dispatch K before that is asserted anywhere). I2 (body repointed to
PointwisePower, verified at line 120): `list_is_accepted_at_its_roster_carrier` PASS, identical to
baseline -> **I2(b) holds**: the container-compatibility judgment is INSENSITIVE to std/types' declared
alias BODY. Combined with the in-file fixture contract (multi_module_compile_fixture: "no module index,
no corpus roots, and no filesystem read") and the empty .dag parse-lens grep, no executing compile route
observes the fact this claim guards => the text check over the declared line is the only instrument that
sees it. NAME binding does come from that file (I1); BODY binding does not (I2).

## 2026-10-20T23:1x dispatch L: §6-lens measurement -- does decl_facts carry the ALIAS rhs? (review 83623 finding 1)
My earlier "no .dag parse lens exists" was WRONG (crude grep; the pool has decl_facts/decl_facts_at/
import_resolution_facts). The review says the resolver can answer "which declaration List resolves to"
directly. Source says: concept_decl_node (coproduct_reflection.rs:596-606) marshals Disj->disj,
Conj->variant-arm-target, EVERYTHING ELSE (NoConnective, Arrow) -> unit_type_node = a Conj node with
ZERO children. An alias `type X = C` is a NoConnective item. PRE-REGISTERED PREDICTIONS:
- L1 (positive controls, must PASS or the harness is broken): a fixture RECORD
  (type AliasProbeRecord { first: String }) marshals to a fact whose node.children is NONEMPTY, and a
  fixture COPRODUCT (AliasProbeChoice = A | B) likewise. If these fail, decl_facts marshaling is broken
  and NOTHING is concluded about the alias arm.
- L2 (the measurement): fixture alias `type AliasProbeList<element> = FreeMonoid<element>` ->
  fact.node.children is EMPTY (== unit). PREDICTED PASS = the lens is blind to the alias body.
- L3 (the guarded fact itself): decl_facts_at(pool_roots: ["dag"], qn: "std.types.List") resolves
  (facts nonempty) AND its node.children is EMPTY. PREDICTED PASS = no .dag-callable read of the Node
  tree can see FreeMonoid on the List declaration => the §6 route "costs substrate edits" (a new host
  callable) => text check kept, review answered by execution.
- L4: if L2 or L3 flips (children nonempty / carries FreeMonoid), the lens route IS available: I adopt
  it in the claim and the review's finding 1 stands. Report which child name appears.
Fixture: dag/test/fixture/alias_lens_probe/specimens.dag; witness:
dag/test/claim/alias_lens_probe_witness.dag (both on instruments-scratch).
Sentinels (added): child-count reads return -1 (fact missing), -2 (ambiguous), >=0 (measured).
"fact resolves" members and "children empty" members are SEPARATE, so missing != blind != carries.

## 2026-10-20T23:2x dispatch M: rerun of L (witness parse bug fixed) + ledger parse check at CURRENT 14083 head
- M1: alias_lens_probe_witness resolves (line-continuation after `=` is illegal; both data decls are
  now single-line). If it still fails to parse, fix syntax again; no conclusions.
- M2: all 10 members report PASS/FAIL (claim-run reaches members); L1-L4 then read as pre-registered.
- M3: ledger parse check at the CURRENT session/quiet-hawk-507-evidence tip: PARSE_RC=0, all five
  imports resolve. The earlier PARSE_RC=1 tested an EARLIER head (before the missing-comma fix at
  5b5e421 and before the escaped-quote fix); if two rows STILL fail, that is a real parse defect in
  my receipts on main -> get raw diagnostics per file and fix.

## 2026-10-20T23:3x dispatch N: locate WHY two ledger rows are unindexed, and rerun the lens probe with src roots
N1: per-file `gunbc run --entry <row> --source-root dag --source-root src` for the five rows.
    A file that PARSES reaches member/entry execution -> error is NoSuchFunction/main (or ExitSuccess);
    a file that FAILS to parse -> "parse-failed at <line>:<col>" or "no readable reference set".
    PREDICTED: silent_first_wins... and a_value_of_any... FAIL (they are the two the import census said
    "not found"; unparseable files are excluded from the module index -> import says not found);
    the other three parse. If a "not found" row instead PARSES here, the cause is indexing, not syntax.
N2: localize the regression per file: run the SAME entry on main's copy (git show origin/main:<path>)
    and on the branch copy. silent_first_wins... landed via PR-1 #14485; if main's copy ALSO fails,
    the defect is on main (report to parent); if only the branch copy fails, my append broke it -> fix now.
N3: lens probe with BOTH roots registered: all 10 members print PASS/FAIL (v2.std.decl_index resolves
    under src). L1-L4 then read as pre-registered.

## Dispatch O — alias lens (review 83623 finding 1) + #14597 bound-trigger re-verify
Pre-registered BEFORE the run. Roots = `dag` + `src/v2` (matches every CI `gunbc run` call; my
earlier lens run used `src` which pulled src/v1 and voided on a std.algebra root collision — that
was a harness fault, not a substrate answer, so it is re-run and NOT counted).

LENS (alias_lens_probe_witness, claim-run): predicted from concept_decl_node
(coproduct_reflection.rs:596-606) which marshals Disj→disj, Conj→variant-arm target, EVERYTHING
ELSE (NoConnective/Arrow = the `=`-form alias shape) → unit_type_node (ZERO children).
 - the_record_control_carries_children            PASS (Conj > 0)
 - the_coproduct_control_carries_children          PASS (Disj > 0)
 - the_real_std_types_record_control...            PASS
 - the_local_carrier_alias_carries_no_children     PASS (==0)
 - the_free_monoid_alias_carries_no_children       PASS (==0)
 - the_guarded_list_fact_carries_no_children       PASS (std.types.List ==0)
If ANY control reads 0 → harness broken, no conclusion. If an alias reads >0 → the §6 lens route
IS available and the review wins (adopt it). If all controls >0 and all aliases ==0 → a pure
Node-tree reader cannot see an alias body; text check is the only instrument → answers finding 1
by execution.

#14597 claim file (bound trigger): predicted ALL members PASS, EXCEPT the pre-existing
kc_local_alias_of_the_carrier_admits_its_constructor FAIL (documented in body, off required path).
Specifically the_dissolution_fires_only_when_the_named_predicate_retires PASSES (DissolutionFired on
[], Pending on the named ref, Pending on the lookalike). If it FAILS → retires_dissolution over a
same-file predicate name does not partition as the firmware idiom claims; re-read dissolution_status.

## Dispatch R (Map-family disposition on fierce-bee-518's C1.2c branch, requested msg_6df8b8da)
fierce-bee-518 branch session/fierce-bee-518 @ PR head 4cef33dbeb (their cited d3c1f18 is an
ancestor; PR #14594 head is authoritative). Instrument: gate_two_closures_14152.sh (head=their
branch vs BASE_REF=main), comparator from scratch/rebind-census, then rebind_census.sh over the
self-host logs. Pre-registered BEFORE dispatch:
 - CONTROL: main's own self-host rebind census must be reproducible from this run's main log
   (the gate's built-in control; if main is not the previously-recorded baseline, instrument wrong).
 - ARITY-N PREDICTION: the descent makes Map<K,V>/FSF<K,V> BIND K and V instead of binding nothing.
   On main the arity-1-only arm binds nothing for a 2-param generic, so those call sites produce NO
   rebind refusal (silent Optional<>). On their head the positional descent CAN now produce a rebind
   refusal where the two actuals disagree. So the MAP-class count should RISE on head vs main where
   the pair was previously silently accepted, and CLEAR (fall) only where both actuals already agreed.
   Therefore "clear" for fierce-bee means: the Map(Primitive(K),Primitive(V)) identities that were
   PRESENT on main fall to 0 on head. If they PERSIST unchanged, the descent does not touch them and
   they belong to a different owner (the fabricated-placeholder / declaration class).

## Dispatch S1 — #14597 at review-83674 fixes (PR tree, head bf49043d08fe)
Pre-registered BEFORE the run. Claim-run, --source-root dag only.
 - ALL members PASS except the pre-existing kc_local_alias_of_the_carrier_admits_its_constructor
   (present on origin/main, FAILS at clean tree P1, passes only when a hand-typed presentable
   alias line is readable — the F2 path-derivation change must NOT change which file is read:
   layout_module_storage_path("std.types") must equal the old "dag/std/types.dag"; if the
   text-reading members go RED that equality broke).
 - the_dissolution_fires_only_when_the_named_debt_retires PASSES: [] => Fired (std.types List
   absent), [std.types/List] => Pending (present), [std.types/Set] => Fired (exact ref still
   absent; presence keyed on full (module_path, decl_name)).
 - BUILD_RC=0 (the new import gunbc.source_layout lives under the dag root: dag/gunbc/source_layout.dag).

## Dispatch S2 — #14597 at review-83706 fixes (capability trigger + ledger row)
Pre-registered BEFORE the run. Claim-run --source-root dag.
 - New member the_dissolution_fires_only_when_the_capability_arrives PASSES: [] => Pending
   (container_alias_declaration_rows absent on the real corpus), [std.types/container_alias_declaration_rows]
   => Fired, [std.types/container_template_alias_rows] => Pending (same-module lookalike; presence is
   exact-ref keyed). Verified facts: container_alias_declaration_rows ABSENT in std/types today;
   container_template_alias_rows PRESENT (line 83) as the real lookalike.
 - All other members unchanged vs S1: 13 PASS, kc_local_alias_of_the_carrier_admits_its_constructor
   FAIL (pre-existing on origin/main).
 - New ledger row file parses: gunbc run --entry <row file> --source-root dag => NoSuchFunction{main}
   (parse+resolve OK; ledger rows have no main, that IS the pass signal — established in dispatch N).

## S1-S3 OUTCOMES (post-run record)
 - S1 (review-83674 fixes @ bf49043d08fe): as pre-registered — 13 PASS + dissolution member PASS,
   kc_local_alias_of_the_carrier_admits_its_constructor FAIL (pre-existing; reproduces on main's own
   file at dispatch P1; passes only on scratch's hand-typed presentable alias line).
 - S2 (review-83706 capability trigger @ c1b5e2a859a0): as pre-registered — 13 PASS incl.
   the_dissolution_fires_only_when_the_capability_arrives (Appears polarity arms executed); same
   single pre-existing FAIL.
 - S3 (row parse check): ROW_RC=1 with cause NoSuchFunction{main} after resolve_entry_graph_inclusive
   (closure_files=8) — PARSE+RESOLVE OK per dispatch N's established signal. Row file good.
 - Gate R (fierce-bee-518 request, 4cef33dbeb vs main): both arms build rc=0 self-host rc=0
   emitted=2; census Map-class 0 on BOTH — instrument scope negative (Acc lives only on PR-2 head);
   reported to fierce-bee-518 + tidy-raven-397. Correct arm = Acc-head ⊕ C1.2c vs Acc-head, queued
   into PR-2 rebase.

## Dispatch T1 — #14597 @7e8b463 (review-83725 fixes)
Changes are text-only (row receipts + comments; CONTROL 4 test MOVED verbatim, no member logic
edits). Predicted: identical to S2 — 13 PASS, kc_local_alias_of_the_carrier_admits_its_constructor
FAIL (pre-existing), ROW NoSuchFunction{main}. Any member flip here means the comment move broke a
parse (e.g. orphaned fn) — would need an immediate fix.

## Dispatch U1 — renderer row parse check (renderer-d @3b28ae1da3d9, feat/renderer-shape-row)
ORDERING SLIP: dispatched before recording this (rule is pre-register FIRST). Signal established
from dispatch N, so risk is nil, but the slip is recorded rather than papered over.
 PREDICT: U1BUILD_RC=0 (cached or fresh), U1ROW_RC=1 with cause NoSuchFunction{name:"main"} —
 the ledger rows have no main and that is the parse+resolve PASS (dispatch N). A different error
 (type mismatch, unresolved import) would mean the row text is malformed — must fix before PR.

## Dispatch U2 — #14612 row after Charter-4 route repoint (retirement route = census-row
## typed expected/got fields, NO host intrinsic). Text-only edit to the NEXT-RUNG CARRIER receipt.
 PREDICT (recorded BEFORE dispatch): U2BUILD_RC=0 (likely cached), row run U2ROW_RC=1 cause
 NoSuchFunction{name:"main"} = ledger parse+resolve pass. A type-mismatch/unresolved-import would
 mean the edited receipt text broke the string literal (e.g. an unescaped quote).

## Dispatch U3 — #14612 row after the §5→§4b meta-obligation citation fix (review 83742's remark,
## verified true: the four meta-obligations are under the ## 4b header; §5 is Fail-closed).
 Text-only receipt edit. PREDICT: U3BUILD_RC=0, U3ROW_RC=1 cause NoSuchFunction{name:"main"}.

## Pre-staged recipe — Acc⊕C1.2c family disposition (fires when C1.2c lands on main)
1. git worktree pr2-fbp: git reset --hard origin/session/quiet-hawk-507 (Acc head; PR #14152 @97e47f8e562b
   — re-check for a newer PR head first via gh pr view 14152 headRefOid).
2. Merge the LANDED C1.2c (main, post-#14594) — conflicts and resolutions:
   (a) src/v1/04_infer.dag unify_generics: keep BOTH — their arity-N child-pair loop AND PR-2's Acc
       fold + text/module_name diagnostic params (same fn, disjoint edits).
   (b) mirrored seed .rs (v1_compiler_infer.rs, v1_compiler_infer_types.rs, v1_tests_claim_generic_identity_census.rs):
       regenerate from merged .dag (stage0 = emitter output; fixed point installed=0, hash-verify).
   (c) one_type_variable_bound_to_two_distinct_coproducts.dag: keep both appended receipts (their C1.2c
       receipt + PR-2's Acc receipt).
   (d) constructor_tag_route_probe_red.dag: KEEP OURS (restore verbatim per warm-wolf-570 agreement).
3. Run gate_two_closures_14152.sh (HEAD=merge, BASE=main) + tools/rebind_census.sh on head + main logs
   (dispatch R pattern, /tmp/dispatchR.txt). Question: do the Map(Primitive(K),Primitive(V)) identities
   (59/76, 34 at map_insert) clear on the merge vs Acc-head-alone? Acc-head-alone baseline: 76/67
   distinct/occurrences, 17/18 ids (strip-vs-head diffs in /tmp/report_strip.md).

## PRED-V (2026-10-21, local merge pr2-fbp: Acc-head 97e47f8e562b x C1.2c head 2af7eb0e1d4f; then remote gate + census, HEAD=merge BASE=main)
Pre-registered BEFORE merge/dispatch. Baselines: Acc-head-alone census 76 distinct/67 occ, 17/18 ids (/tmp/report_strip.md); gate_strip_vs_acc showed 43 identities at HEAD main lacks. Gate R: Map class 0/0 on main-vs-C1.2c (instrument scope).
- P-V1: merge builds self-host rc=0 both arms — unify_generics edits are disjoint-in-same-fn (their arity-N child-pair loop; PR-2's Acc fold + text/module_name params). A resolve/parse error here means my 04_infer resolution merged the edits incorrectly — fix resolution, never drop an edit.
- P-V2: Map(Primitive(K),Primitive(V))-class identities at merge <= Acc-head-alone (59/76, 34 map_insert). C1.2c's arity-N descent refines applied-vs-applied mismatches into structural child pairs; it cannot CREATE head-level Map identities that Acc-head alone lacks. Any ADDED Map identity = mechanism interaction, investigate before reporting.
- P-V3: any identity present at merge but absent at Acc-head is either (a) in main's census already, or (b) a CHILD-level identity (formal/actual proper subterms of an Acc-head identity's) produced by descent. Anything else = unexplained, do not report until explained.
- P-V4: stage0 mirrors must be regenerated from merged .dag (emitter output only); self-host emit diff must be 0 (installed=0) or the merge resolution left stale .rs.

## PRED-B (2026-10-21, #14056 condition-(a) feasibility, TWO-ARM MEASUREMENT — nothing pushed to #14056)
Question: is the binder/leaf discriminator census-observable on an authorable program? Arms: FIXED=3a698d4 (equals-branch head, commits 57ebbd2/e05dc01/dc1b92c/09b26e2), MAIN=origin/main. Probe = dag/test/claim/binder_census_probe.dag (scratch-only branch, committed so the runner clones it; NOT #14056's branch).
- P-B1 (discriminator): program `fn same_arm<A>(a: A, b: A)` fed TWO unmarked leaves `T` from one signature (uses_two(x: T, y: T) with calls same_arm(a: x, b: y) — via a forwarder) : MAIN count 0 (spelling-equal leaves unify), FIXED count >= 1 IF e05dc01 gives each unmarked occurrence its own inference id (its message says 'Identify unmarked type-variable leaves by inference id'). If FIXED is also 0, the binder comparison is NOT reachable from any authorable census program -> condition (a) is UNSATISFIABLE as scoped; report to parent with the two counts before the rebase, do not invent a different claim shape unasked.
- P-B2 (no-regression, accept side): same_applied_box_int_is_accepted and list_int_equals_freemonoid_int => count 0 on BOTH arms (the fix must not turn accepts into refusals).
- P-B3 (no-regression, refuse side): applied_box_int_vs_bool_is_refused => count >= 1 on BOTH arms.
- P-B4: same-binder accepts stay 0 both arms: fn f<T>(x: T, y: T){same_arm(a: x, b: y)} and the generic-caller flexibility specimen.
Both arms build self-host; binary rc captured raw (never via tail). Void/abort != pass: an arm that fails to build gets NO count reported for it.

### PRED-B results so far (MAIN arm @f34616e, claim-run rc=1, members printed; FIXED arm raw log pending)
- MAIN p_b1 PASS (unmarked same-spelling T's from one signature: count>=1, REFUSED). So main false-refuses same-fn unmarked T's; the fix's 'two T's of one fn are the same parameter' should FLIP this to accepted (==0). p_b1's >=1 assertion is main-polarity; restatement belongs to the claim-home version.
- MAIN p_b4 PASS, p_b2 PASS (same-binder accept + Box<Int>==Box<Int> accept hold on main).
- MAIN p_b3 FAIL (Box<Int> vs Box<Bool> ACCEPTED on main, count 0) -> the applied-container refusal machinery is NOT on main; it arrives with the PR's applied half (superseded by PR-2 arm 2). Consistent with the applied half being PR-introduced.
- MAIN p_b5 FAIL (T-of-f vs T-of-g same spelling accepted on main, count 0) -> THE BUG the binder half fixes, reproducible on an AUTHORABLE census program. If FIXED shows p_b5 PASS (>=1) with p_b4 still ==0, condition (a) is SATISFIABLE and the claim-home set = {p_b5 green-with-fix red-on-main, p_b4+p_b2 holds, p_b1 restated to same-fn==0}.

### PRED-B infra note + FIXED-arm pre-registration (dispatch je3c85f09, @3a698d4)
Root cause of FIXED VOID x2: the gunbc binary re-execs itself via systemd-run into gunbc-user-work.slice (cli_run/user_work_scope.rs); on a bare runner with no user manager it prints 'cannot wrap ... UserSessionAbsent' rc=2 and dies. Designed exemptions: GITHUB_ACTIONS set (CiExempt) or runner-slot cgroup needle. Runners differ (MAIN arm's run wrapped fine; FIXED's instances didn't). Fix = export GITHUB_ACTIONS=1 (the designed CI exemption) — not a bypass.
FIXED-arm predictions, registered BEFORE this dispatch: p_b5 PASS (>=1: distinct binders refused), p_b4 PASS (==0), p_b2 PASS (==0), p_b1 PASS (>=1 — main also refused this; the fix makes same-fn T's the SAME inference id, so p_b1 may FLIP to ==0/FAIL — that flip is the fix working, the claim-home restatement covers it; either polarity recorded, neither treated as regression). p_b3 was FAIL on main (applied machinery absent pre-PR) — FIXED arm should PASS p_b3 if the PR's applied half refuses Box<Int>/Box<Bool>; recorded either way.

### Runner-infra lesson (2026-10-21, PRED-B dispatches 4-7)
Runner instances rotate; some have no systemd user manager AND no cgroup limit. Then: (a) the gunbc binary's self-wrap into gunbc-user-work.slice fails -> 'cannot wrap ... UserSessionAbsent' rc=2; (b) with wrap skipped but no cgroup, the memory governor refuses UNKNOWN (rc=101 panic). Both have DESIGNED escape hatches, now the standard preamble for every runner claim-run dispatch:
  export GITHUB_ACTIONS=1            # CiExempt: skip systemd wrapping (the CI path)
  export GUNBC_MEMORY_BUDGET_BYTES=10737418240  # declared-unverified 10GiB planning ceiling (corpus-precedent value, host_budget_source.dag:351 receipt)
Also: ctrl-build mirrors LOCAL worktree state (git apply patch) — a fresh clone inside the dispatch is still subject to runner rotation; the binary path must be re-derived from CARGO_TARGET_DIR IN THE SAME dispatch as the run, never relied on from a previous dispatch.

### PRED-B2 (dispatch 8, both arms with escape-hatch env). p_b5 was a FLAWED specimen: in
###   same_arm(a: y, b: f(x: y)) both sides are Box<T_g> — ONE binder — so acceptance is CORRECT
###   and p_b5 discriminates nothing (dropped). Real discriminator = p_b1 FLIP, measured with BOTH
###   polarities per arm (member PASS/FAIL pairs read as the measurement):
- m_b1_accepts (count==0): MAIN FAIL (measured 1), FIXED PASS (dc1b92c makes unmarked T of the fn its inference variable). If FIXED m_b1_accepts FAILs (still refused): the binder fix does NOT reach authorable census programs -> condition (a) as scoped UNSATISFIABLE -> report to parent with both counts, invent no alternative claim shape unasked.
- m_b1_refused (count>=1): MAIN PASS, FIXED FAIL (mirror of above).
- m_b3_refused (Box<Int> vs Box<Bool> >=1): MAIN FAIL (measured 0). FIXED: PASS if the PR head's still-present applied half (04_types.dag:1436+ retains Applied handling) refuses it; FAIL => applied refusal is PR-2-only. Recorded, drives which half #14056 keeps.
- m_b2/m_b4 ==0 on both arms (measured PASS on both so far; re-measured identically).
- Text-diagnostic route: real-file probes (probe_src/probe/binder1.dag, binder3.dag) through `gunbc run --entry` on EACH arm: main binder1 prints a refusal naming T (class recorded VERBATIM — this is what distinguishes unbound/unknown-T from unify-mismatch); fixed binder1 expected clean (no diagnostic) if the flip is real. Any panic/rc!=0 on the runs: record, do not interpret.

### PRED-B FINAL (dispatch 8, both arms executed, GITHUB_ACTIONS+budget env; 15m)
Table (claim-run members; both arms BUILD_RC=0, CLAIM_RC=1 from the intentional FAIL pair):
  member            FIXED@3a698d4   MAIN@b4d1564
  m_b1_refused      PASS            PASS        (unmarked T pair refused on BOTH arms)
  m_b1_accepts      FAIL            FAIL        (count>=1 both; NO FLIP)
  m_b2_accepts      PASS            PASS        (Box<Int>==Box<Int> both)
  m_b4_accepts      PASS            PASS        (same declared binder binds both)
  m_b3_refused      FAIL            FAIL        (Box<Int> vs Box<Bool> ACCEPTED on both -> applied-container refusal exists on NEITHER arm; it is PR-2 arm-2 territory)
  raw run binder1/reg1: RUN_RC=1 NoSuchFunction{main} both arms = parse-pass signature, no refusal text captured.
CONCLUSION: main and fixed are BEHAVIOURALLY IDENTICAL on every authorable census specimen swept. Why, from the call-site map: the census comparison path is unify_generics (04_infer:6513), which has its own leaf logic (already distinct-unmarked on both arms, e05dc01's resolve-side change); node_type_equals is reached only from 04_access (string/int shape checks, can't carry user type vars), 04_emit_info (variant field summary), 04_types callable-agreement/Applied-children arms. The PR's changed arms (binder-vs-binder, declared-binder-beats-unmarked) fire only on pairs authorable programs never hand them — the Rust tests construct those pairs synthetically (leaf(TypeParameter{declared_inference_variable})).
=> CONDITION (a) AS SCOPED IS UNSATISFIABLE on the pre-PR-2 head: no dag/test/claim RED exists that is red-without-the-fix for the leaf/binder half. One route remains open and untested: the fn-VALUE comparison route (callable_signatures_agree at 04_types:1164/1277/1349 — let-bound fn-typed values with user type vars at signature positions) — probeable at the rebase step, when PR-2's Applied-children arm (04_types:1465, child_type_node pairs) actually changes what node_type_equals consumes; the binder comparison becomes census-reachable THERE (Box<T_f> vs Box<T_g> under a fn-type equality) if that route fires.

## PRED-C (2026-10-21, parent ruling msg_d6381674: confirm arm-2 refusal of Box<Int> vs Box<Bool> at PR-2 head). One dispatch, PR-2 head 97e47f8e562b, self-host bin, escape-hatch env (GITHUB_ACTIONS=1 + GUNBC_MEMORY_BUDGET_BYTES=10737418240, the designed CI exemptions found in PRED-B).
Probe members (census route only — new claim files cannot import v1.compiler.* per the CI-scan rule; census is the executing in-lane surface):
- c_b3_refused: Box<Int> vs Box<Bool> under same_arm >= 1. THE CONFIRMATION. If >=1: arm-2 refuses at the executing level -> becomes a PR-2 RED at rebase. If ==0: silent wrong-type accept survives arm 2 -> file a RecurringFailureMode row per ruling.
- c_b2_accepts: Box<Int> vs Box<Int> == 0 (no over-refusal regression from the Acc fold).
- c_b1_refused: unmarked-T pair >= 1 (cross-arm control; was PASS on main+14056 head).
- c_b4_accepts: same declared binder == 0 (control).
- c_fnval_refused: fn-VALUE route EXPLORATORY (param f: (Box<T>) -> Bool called with g: (Box<U>) -> Bool from another signature): recorded either way; the DECISION run stays at the rebase head per ruling (i).
- control: PR-2's own applied_constructor_type_equals.dag claim-run must still print PASS PASS PASS (executed control).
Both claim files run on the SAME built binary; rc captured raw. Void != pass: any build/abort -> no dispositions reported.

### PRED-C RESULTS (dispatch j1eb20038 @PR-2 head 97e47f8e562b, self-host BUILD_RC=0, escape-hatch env)
- c_b3_refused PASS -> ARM 2 REFUSES Box<Int> vs Box<Bool> under a shared formal at the executing census level. The silent wrong-type accept seen on main AND 14056-head (both accepted, count 0) is CLOSED by PR-2 arm 2. -> per parent msg_d6381674 this becomes a PR-2 RED (green-guarding claim) in the claim home dag/test/claim/applied_constructor_type_equals.dag at the rebase step: member box_int_against_box_bool_is_refused, census route, same blocking_count helper. No failure-mode row needed (refusal exists).
- c_b2_accepts PASS (identical applications still accepted; no over-refusal from the Acc fold). c_b1_refused/c_b4_accepts PASS (cross-arm controls hold at PR-2 head).
- CONTROL: applied_constructor_type_equals.dag CONTROL_RC=0 -> ALL its existing members PASS at the PR-2 head (same_applied_box_int_is_accepted printed; rc=0 means every member green).
- c_fnval_refused FAIL (count 0): the EXPLORATORY fn-value route (same_arm(a: f, b: g) with f:(Box<T>)->Bool from fn f<T>, g:(Box<U>)->Bool from fn g<U>) is ACCEPTED at the current PR-2 head. Recorded as the pre-signal for ruling-(i)'s decision run: if it still accepts at the REBASE head (where 04_types:1465 Applied-children descent changes what callable_signatures_agree consumes), the binder comparison has no observable consumer on any today's route -> close #14056, fold leaf/binder half into PR-2 as side fix with Rust tests labelled vacant (or drop), per ruling.

## PRED-E (2026-10-21, pre-bake PR-2 RED in its claim home; self-contained dispatch on PR-2 head 97e47f8e562b)
Write the FINAL applied_constructor_type_equals.dag = its 2 existing members + new member box_int_against_box_bool_is_refused (>=1) with a §6-style comment naming the silent-wrong-type-accept it refuses (the class PRED-C confirmed arm 2 closes). Harness-execute the WHOLE file (claim-run) on the PR-2 binary.
- P-E1: all 3 members print PASS; claim-run rc=0. (2 existing green proven CONTROL_RC=0; c_b3 body proven PASS in PRED-C — this validates the combined FILE text: header/imports/helper unchanged, new member appended.)
- P-E2: no NoSuchFunction/ResolveError (member name unique; imports unchanged from the committed file).
If any member FAILS the file is NOT staged; investigate (likely the new source string) before the rebase. Void/build-fail != pass.

### PRED-E RESULTS (dispatch je9e85cff @PR-2 head 97e47f8e562b, BUILD_RC=0)
CLAIM_RC=0, members: PASS same_applied_box_int_is_accepted, PASS list_int_equals_freemonoid_int, PASS box_int_against_box_bool_is_refused. FINAL claim-home text validated EXACTLY as staged (this dispatch's heredoc = the committed content of prebake_applied_constructor_type_equals.dag in this branch). REBASE NOTE: at the rebase the file may need only import-compat tweaks if the census module moves; if any member changes, RE-RUN this harness before pushing — never push a member that hasn't printed PASS at the head being pushed.

### PR-2 REBASE SCRIPT (fully scripted while blocked; heal.yml lines 60-120 read at 07:35)
After #14594 lands, the mirror regen is NOT a guess: build regenerator `cargo build --release -p v1-compiler --bin gunbc --bin claim_executor`, then
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2   # converge stage0 to fixed point (round1 installs, round2 installs 0 + prints seed sha)
  gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet
  gunbc run ... --entry dag/gunbc/instruments/seed_build_artifacts_regen.dag --function main_wet
PR-2 body already documents the sanctioned rule: stage0 mirrors are 'regenerated, never hand-merged'; cli_run + module_path_index are HAND_MAINTAINED (must NOT appear in the changed list). So the fbp merge's 40 stage0 errors ARE the hand-merge the rule forbids — the fix is regen, matching PRED-V P-V4. Changed files named by `git status --porcelain -- src/v1/stage0/src` in the guest, not by hand.
Rebase order: (1) fetch landed main, (2) merge into PR-2 branch resolving src/v1/04_infer.dag per PRED-V (arity-N loop + Acc fold + text/module_name), 04_types import union, coproduct row receipts-union, constructor_tag_route_probe_red KEEP OURS, (3) run the regen block above until round-2 installs 0, (4) apply prebake_applied_constructor_type_equals.dag (PRED-E validated CLAIM_RC=0 @97e47f8), (5) re-run the WHOLE applied_constructor_type_equals.dag claim-run on the NEW head — never push a member that hasn't printed PASS at the head being pushed, (6) fn-value decision run for #14056 (ruling i), (7) body: strip receipt + D1/D2/D4 + Map frontier + seed-sha section updated.

## PRED-G (2026-10-21, regen-mechanics dry run on pr2-fbp merge 4a76af1a — TOOLING TEST, no branch updated)
Chicken-and-egg question: claim_executor/gunbc build FROM src/v1/stage0/src mirrors; the fbp merge build fails there (40 errors, P-V4). Can the regen route actually run there?
Steps+predictions:
- G1: at 4a76af1a, cargo build -p v1-compiler --bin gunbc --bin claim_executor FAILS (rc!=0) — expected, reproduces PRED-V.
- G2: restore main's mirror set (git checkout origin/main -- src/v1/stage0/src) => build SUCCEEDS rc=0. Prediction: succeeds — .rs mirrors are pure Rust (the crate does not read .dag at compile time), main's mirror set is a known-good self-consistent generation. If it FAILS, mirrors ARE build-time-coupled to .dag via include/seed artifacts -> the rebase needs a different order (take main's .rs + land C1.2c's own mirrors, i.e. rebase on landed main whose .rs are already consistent WITH C1.2c, making this whole question moot — report which).
- G3: on the merged .dag, run regen trio: claim_executor --regen-round-cost --source-root dag --source-root src/v2 (expect round prints + installs N), gunbc run heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet (rc=0), seed_build_artifacts_regen.dag --function main_wet (rc=0). Then git status --porcelain -- src/v1/stage0/src lists candidate files.
- G4: PREDICTION: changed set is subset of {v1_compiler_infer.rs, v1_compiler_infer_types.rs, v1_tests_claim_generic_identity_census.rs} + possibly other emitted mirrors; cli_run.rs and module_path_index.rs MUST NOT appear (HAND_MAINTAINED). Their absence from main's checkout is re-added by regen if the merged .dag emits them... NO: HAND_MAINTAINED means regen does not write them -> they must stay main's.
- G5: final rebuild after regen rc=0. This is the proof the rebase script works end-to-end.
Escape-hatch env on all gunbc/claim_executor runs. All outputs raw; aborts recorded not interpreted.

### PRED-G RESULTS (dispatch j0b7ccd44 @4a76af1a, 24m)
G1 rc=101 (40 errors: 34 E0425 + 2 E0063 + 2 E0432 + 1 E0560) as predicted. G2 rc=0: restoring main's src/v1/stage0/src over the merge makes the tree BUILDABLE => mirrors are build-time-INDEPENDENT of .dag; the regen route HAS a valid bootstrap (checkout main's mirrors -> build regenerator -> regen -> rebuild). G3A rc=1: regen-round-cost REFUSED — v2 self-compile 1 hard diagnostic: "method 'map' cannot be resolved: receiver type 'Product(SpanMapping)' ... (src/v1/trace.dag:132:5)". G3B/G3C rc=0. G4 EMPTY (round-0 refusal installs nothing) => G5 rc=0 was VACUOUS (main's mirrors unchanged). HAND_MAINTAINED guard held trivially.
DIAGNOSIS: trace.dag is NOT on main and NOT in C1.2c — it is PR-2-side (97e47f8 has it, identical in fbp merge and 2af7eb0e). The method-surface the trace.dag:132 'map' needs presumably arrives with recent main (16 commits landed since the fbp merge's main@06:35 base, incl Lane-N P1.3 which touched 02_parse + emitted mirrors). The fbp merge never tested PR-2 x CURRENT-main. => the true rebase shape (landed main ⊕ PR-2) must be rehearsed separately. #14594 CI (v2-bootstrap+heal) is GREEN at ce3f934 (its third head of the day; evidence rework commits only — trace.dag unchanged there).

## PRED-H (2026-10-21, REHEARSAL of the TRUE rebase: origin/main@6256b52 x PR-2@97e47f8. Measurement only, nothing pushed.)
- H1: merge origin/main into PR-2 head — predicted conflicts = the PRED-V set (04_infer unify region [main's Lane-N P1.3 touched 02_parse only, so 04_infer conflict comes from PR-2-vs-main Acc drift], 04_types import union, coproduct row, constructor_tag_route_probe_red DU, 3 stage0 mirrors). Record actual set; resolve per script (mirrors -> take MAIN's side as bootstrap).
- H2: cargo build -p v1-compiler --bin gunbc --bin claim_executor after taking main's mirrors => rc=0 (G2-proven bootstrap).
- H3: regen-round-cost => PREDICTION rc=0 (the trace.dag:132 'map' resolves against current main's method-surface authorities — if it still refuses, record the diagnostic VERBATIM; that is a pre-rebase blocker to report, not to work around).
- H4: regen trio rest rc=0; G4-style changed set must EXCLUDE cli_run.rs/module_path_index.rs.
- H5: rebuild rc=0. H6: whole-tree self-host gate rc=0 on the rehearsed head (final proof the rebase lands green).
Escape-hatch env on. If H3 refuses, STOP (no H4-H6), report.

### PRED-H RESULTS (dispatch jc0fb0525, main@6256b52 x PR-2@97e47f8; H2 build failed -> chain aborted at H2)
- H1 TRUE conflict set (much smaller than fbp merge predicted, because C1.2c is NOT yet on main — verified: main has NO unify_generic_child_pairs, unify_generics still 5-arg/Map; alias_names_settled_agree IS on main via another PR, imported at 04_types:8 + used at 1239):
  UU one_type_variable..coproducts.dag | DU constructor_tag_route_probe_red.dag (main keeps, last touched dd83b0065 #14436; PR-2 deletes per 5d750054b347) | UU 04_infer ONE hunk | UU 04_types import union | UU stage0 {v1_compiler_infer.rs, v1_compiler_infer_types.rs}. NO 04_infer_env conflict.
- H1b trace.dag/probe_emit_interp/v1_compiler_trace.rs AUTO-DELETE 3->0. G3A's Product(SpanMapping) refusal is fully EXPLAINED: an artifact of the fbp TWO-parent merge (both PR-2 and C1.2c carry trace.dag from pre-#14497 bases; no main parent to delete it). In the true rebase the module dies with main's deletion (#14497, lane F, 00:57).
- H1 hunk resolutions (inspected locally @/tmp/rebase-test):
  1. 04_infer ONE hunk @7232-7242: HEAD side = the v1.compiler.trace FRONTIER ROW (module_name v1.compiler.trace/method map/Product(SpanMapping...) receiver_shape) + closing '},' + the '{' opening the NEXT record; main side EMPTY (main retired the frontier row with the module, review 82934). RESOLUTION: delete the entry (7233-7239 = fields + '},'), KEEP line 7240 '{' (opens the auto-merged next record), delete the 3 markers. Net: take main's retirement.
  2. 04_types import: UNION (keep both lookup_binding_by_name [PR-2] and alias_names_settled_agree [main]; verify used-or-not by compile — main's line already has alias_names so conflict is purely PR-2's addition).
  3. Coproduct row: receipts-union, delete 3 marker lines (our block ends '," so concatenation valid).
  4. constructor_tag_route_probe_red: KEEP DELETION (git rm; PR-2's 5d750054b347 discharged it; warm-wolf agreed in fbp).
  5. stage0: bootstrap = take MAIN'S WHOLE src/v1/stage0/src (NOT per-file: H2 PROVED partial-take mixes generations — kept-PR-2 mirrors call 7-arg unify_generics while main's define 5-arg -> E0061 5-vs-7 + no-field-subst. NEVER mix). Then regen trio regenerates from merged .dag.
- H2 lesson (script bug, recorded): force-taking only CONFLICTED mirrors keeps auto-merged other mirrors at PR-2 generation -> guaranteed mismatch. Fix: whole-dir main-side take.
- H2/H5 build rc=101 as above; H3-H6 never ran (rc=127 no binary). CHAIN UNVERIFIED past H2 — do NOT claim regen works on true rebase yet. PRED-H2 = corrected rehearsal with these exact resolutions, full chain to claim-run PASS.

## PRED-H2 (2026-10-21, corrected true-rebase rehearsal; resolutions per PRED-H analysis; measurement-only)
- H2-0: merge origin/main into 97e47f8 -> same conflict set as H1 (recorded; deviation = surprise, stop).
- H2-1: apply the 5 resolutions verbatim (frontier-row deletion taking main's retirement; import union; receipts-union; git rm probe; whole-dir main-side stage0 take).
- H2-2 build rc=0 (main's whole mirror set + PR-2 .dag = buildable bootstrap; H2 proved the per-file take breaks it).
- H2-3 regen-round-cost rc=0 installs>=0, NO 'refused' (trace.dag is gone -> Product(SpanMapping) cannot recur; if it DOES recur, report verbatim).
- H2-4 heal_wet + seed_regen rc=0; changed set hand-maintained-free.
- H2-5 rebuild rc=0. H2-6 prebake claim file (cp from scratch prebake) claim-run: 3/3 PASS rc=0.

### PRED-H2 RESULTS (dispatch jc34c678f, chain ABORTED at H23 as designed)
H20 conflict set matched H1 exactly (no surprise). R2/R3/R4/R5 clean. R1 FAILED: my keep-'{' heuristic produced '[ { {' — the record-opening '{' BEFORE the hunk (04_infer.dag:7231) is common context; the HEAD-side '{' (7240) opened the sccache record which in main's shape is opened by that same common brace. H23 regen-round-cost: "module index refused: 1 unparseable .dag source(s)" = the designed abort (chain stopped, nothing claimed, no push).
CORRECTED R1: DELETE the ENTIRE HEAD side (trace fields + '},' + '{'); keep only the common pre-hunk '{'. Verified by structure read: PR-2 = '[ { trace-record },{ sccache-record }]' (2 records, PR-2 edited trace's key text), main = '[ { sccache-record }]' (1 record, #14497 deleted trace row with its module). Taking main's retirement = drop 9 HEAD lines. PR-2's edit to that frontier row's key dies with the dead module — correct, nothing to preserve.
Note for rebase pass: PR-2 modified the deleted trace frontier row -> the row's deletion must be TAKEN (main), the edit abandoned; the row guarded v1.compiler.trace which #14497 proved ZERO-importer dead.

### PRED-H2b RESULTS (dispatch j635448e2 @ main 432424a, PR-2 97e47f8)
Conflict count 6 (all cleared by R1-R5; H21_remaining empty). Corrected R1 (drop ENTIRE HEAD side) WORKED: 04_infer clean, merged tree BUILDS (H22_BUILD_RC=0).
H23 --regen-round-cost: rc=1 AFTER completing round.seed_build x2 + round.install + round.rebuild_from_installed (4-5 min each) → a "refused" line exists beyond my head -8 grep window (round.* marks crowded it out); my own guard aborted the chain before H24. LOG NOT CAPTURED (runner /tmp ephemeral). Refusal text unknown.
MAIN moved to a4662d0 (C1.2c still NOT landed, unify_generic_child_pairs count 0; #14594 OPEN).
### PRED-I (preregistered, dispatch TBA)
Diagnosis+continuation: same proven merge/resolutions/build; run regen-round-cost up to 3x; capture FULL refusal lines + tails; then continue chain PAST the guard regardless (heal+seed regen+rebuild+appended-member FULL claim-run).
PREDICTIONS: (I-a) refusal text names round-2 non-zero installs / digest-mismatch / hard-diagnostic; if it is an idempotence-style refusal, invocation #2 exits 0 (heal.yml loops it). (I-b) if regen idempotent: H25_BUILD_RC=0 and appended-member FULL claim-run = 3 PASS 0 FAIL. (I-c) cli_run.rs/module_path_index.rs NEVER in stage0 status diff. If refusal names a hard diagnostic or digest mismatch on specific mirrors, chain still records but I do NOT declare rebase-ready; report to parent when messaging.

### PRED-I RESULTS (dispatch j708c34aa @ main 0978181, PR-2 97e47f8)
Merge+R1-R5 again clean (6 conflicts, all cleared), merged tree BUILDS (I_BUILD_RC=0). Regen refusal root cause = MY HARNESS: "stage seed build refused; checkpoint restored: StageOutputExecutableUnbound: read <repo>/target/release/claim_executor: No such file". required_regen_host.rs:4792 reads workspace-joined target/release/claim_executor; heal.yml builds+runs regen from default workspace target/ (lines 75,94); I had CARGO_TARGET_DIR=/root/gunbc_target_ph2. DISPI=255 = ctrl-build transport died after round 2 (runner rotate). NOT a tree defect.
### PRED-J (preregistered)
Same merge/resolutions; ONLY env change: CARGO_TARGET_DIR=$PWD/target (workspace, matches heal.yml). PREDICTIONS: (J-a) regen rc=0 at round 1 or 2 with NO 'refused' line (idempotent fixed point; heal.yml loops it so 2 invocations are normal). (J-b) heal_stage0_overlap_refusal_wet rc=0 + seed_build_artifacts_regen main_wet rc=0. (J-c) stage0 status diff never touches cli_run.rs/module_path_index.rs. (J-d) post-regen rebuild rc=0; appended-member claim-run = 3 PASS 0 FAIL. (J-e) if J-d holds the rebase is FULLY DE-RISKED against pre-C1.2c main; remaining unknown is only the C1.2c 04_infer region, already pre-staged via fbp resolutions.

### PRED-J RESULTS (dispatch j7d7e43f9 @ main 2538f83, PR-2 97e47f8, 44m) — REHEARSAL GREEN
ROOT-CAUSE FIX CONFIRMED: the ONLY blocker was my private CARGO_TARGET_DIR (regen host reads workspace-relative target/release/claim_executor, required_regen_host.rs:4792; heal.yml:75/94 build+run from workspace target/). With unset CARGO_TARGET_DIR (workspace target/):
  merge RC=1 (6 conflicts, all cleared by R1-R5, J_remaining empty)
  build RC=0; REGEN round-1 RC=0 NO refusal line (idempotent fixed point J_REGEN_IDEMPOTENT_AT=1)
  heal_stage0_overlap_refusal_wet RC=0 (the seed-build-writer OWNERSHIP gate — repo authority)
  seed_build_artifacts_regen main_wet RC=0
  post-regen rebuild RC=0
  appended-member claim-run (3 members) RC=0: PASS same_applied_box_int_is_accepted / PASS list_int_equals_freemonoid_int / PASS box_int_against_box_bool_is_refused
J_VIOLATION_HANDMAINT is a FALSE flag of my crude `git status|grep cli_run`: regen legitimately rewrites cli_run.rs (main itself diverges from fresh regen — hence heal.yml regenerates+commits via the heal bot). The repo's OWN ownership check (heal_stage0_overlap_refusal_wet) PASSED RC=0, which is the authority. Tree builds + claim 3/3 PASS.
CONCLUSION: the PR-2 rebase recipe is PROVEN end-to-end against pre-C1.2c main, INCLUDING regen. Remaining unknown is only the C1.2c 04_infer hunk (arity-N region), pre-staged via the fbp resolutions (@4a76af1a09b8). At the REAL rebase: after regen, do NOT blindly keep regen's cli_run.rs if PR-2 hand-edited it (+120/-31 vs main's -106/+1) — take the MERGE result for the two hand-maint files (cli_run.rs/module_path_index.rs) and let regen own everything else, then rely on heal_stage0_overlap_refusal_wet RC=0 as the ownership truth.

### PRED-K (preregistered 2026-10-21 ~12:20Z) — rebase rehearsal INCLUDING C1.2c
Simulate landed state: throwaway clone of PR-2@97e47f8; merge origin/main (resolve R1-R5 EXACTLY as PRED-J, proven); then merge origin/session/fierce-bee-518@ce3f934 (C1.2c, the #14594 head parent is side-chatting). New conflicts expected: 04_infer unify region (C1.2c arity-N descent vs Acc rewrite) + C1.2c's own stage0 mirrors. Resolutions: 04_infer = take origin/session/gentle-carp-16-fbp's 04_infer.dag (hand-resolved Acc+descent @4a76af1a09b8, never build-validated end-to-end — that is what K tests); mirrors = take origin/main whole-dir then REGEN (proven route); any other conflict = print + keep merging what's safe (receipt-union for rows, both-sides for ledger rows).
ENV as PRED-J: unset CARGO_TARGET_DIR (workspace target/), GITHUB_ACTIONS=1, GUNBC_MEMORY_BUDGET_BYTES.
PREDICTIONS: (K-a) first C1.2c merge conflicts ONLY in 04_infer + stage0 mirrors (else: list them). (K-b) with fbp's 04_infer: build RC=0. (K-c) regen idempotent by round 2 with no refusal; heal_stage0_overlap_refusal_wet RC=0; seedgen RC=0. (K-d) rebuild RC=0; 3-member applied_constructor claim-run 3 PASS 0 FAIL. (K-e) if any fails, capture exact error lines; do NOT hand-fix blindly on the runner beyond the recorded resolutions.
CAVEAT vs truth-at-landing: fbp's 04_infer snapshot predates Lane F deletions (#14621). At landing, diff the adopted file vs landed main's 04_infer to confirm no deleted dead fn is resurrected; if K-d passes but resurrects one, the real rebase adopts main's version of those hunks.
NOTE: this rehearses ONLY; nothing committed/pushed. Parent's defer on committing regenerated mirrors stands.

### PRED-K RESULTS (dispatch j22788089 @ main f9b9595, PR-2 97e47f8, C1.2c @ce3f934452a1, 46m, K-e path hit)
K-a: M1 (main) 6 conflicts as always; M2 (C1.2c) conflicts resolved via adopted fbp 04_infer + mirrors-from-main; K_M2_remaining EMPTY. Both merges mechanically clean.
K-b: PASS — build RC=0 with adopted fbp 04_infer AGAINST MAIN'S COMMITTED (stale) mirrors.
K-c: FAIL — regen round1+2 refused: "cargo build (round.rebuild_from_installed) failed exit 101" (rustc lines not captured). Chain died there; dispatch then hit ctrl-build DeadlineExceeded (transport ~105min). Claim run NOT reached.
fn-diff (adopted fbp file vs landed main 04_infer): adopted carries 16 fns main deleted (binding_has_type_variable_leaf, generic_alias_unfold, generic_types_unify, unify_generic_child_pairs, ... — fbp snapshot @4a76af1a09b8 PREDATES Lane F dead-code deletions, resurrects them); main has missing_parent_env_diagnostics which adopted LACKS.
=> fbp 04_infer is NOT a valid landing resolution: compiles only against stale mirrors; regen (fresh mirrors from current dag) refuses. Real rebase must HAND-RESOLVE 04_infer from landed main (adopt missing_parent_env_diagnostics + whatever C1.2c landed as, drop main-deleted fns, thread text/module_name into Acc fold).
### PRED-L (preregistered) — capture regen's rustc errors for the fbp adoption
Same merge+resolutions as K; SKIP heavy claim/seedgen extras; after regen round1 capture FULL r1 log (cargo lines) + run plain cargo build (deterministic same failure, faster) capture error[XXXX] lines. PREDICTION: (L-a) errors localize to fbp-adopted regions: missing parent_env threading (missing_parent_env_diagnostics absent) and/or resurrected-fn references; (L-b) rc=101 again with 1-3 distinct error codes; (L-c) if errors are OUTSIDE 04_infer-derived code, fbp adoption is fine and regen/mirrors has its own issue. Cheap partial: reuse K's proven merge path; do NOT re-run seedgen/claims this dispatch (dispatch-budget: 3 builds max).

### PRED-L SUBSUMED by PRED-M (K failure fully explained; L would only re-capture it)
K root cause: fbp 04_infer@4a76af1a09b8 predates C1.2c LATEST head (2d43e01..ce3f934 added hunks at unify_generics @@-14619, topo_resolve_types, realize_module, compiler_kernel_type_env, rewire_*_parent_links + BuildTypeEnvResult type). Whole-file adoption = landed-C1.2c other-file expectations vs stale 04_infer -> regen's fresh-mirror cargo build rc=101. K's build RC=0 was only against STALE committed mirrors (which regen then refused to regenerate).

### PRED-M (preregistered) — LANDFALL SIMULATION: hand-resolve 04_infer at the real conflict
Phase-1 LOCAL (branch session/gentle-carp-16-rehearse, throwaway): worktree PR-2@97e47f8; merge origin/main w/ J resolutions; commit; merge origin/session/fierce-bee-518@ce3f934; 04_infer conflict -> resolve = START FROM THEIRS (C1.2c head, has arity-N descent + parent_env fns + main's deletions) then re-apply OURS's Acc change-set (from base->ours diff): unify_generic_child_pairs delegating to UnifyGenericsAcc fold w/ text+module_name threading; reference = fbp file's unify region for integration shape; keep EVERY fn main deleted stay deleted; keep C1.2c's missing_parent_env_diagnostics. Then push branch; Phase-2 RUNNER: build -> regen(idempotent) -> heal RC=0 -> seedgen RC=0 -> rebuild -> appended 3-member claim 3 PASS.
PREDICTIONS: (M-a) 04_infer theirs-base hunk-set matches the 9 C1.2c hunks (no other file conflicts beyond stage0 mirrors+rows). (M-b) hand-resolved file BUILDS against committed main mirrors first try or after one correction. (M-c) regen idempotent by round 2 (this is the acid test: fresh mirrors generated from hand-resolved dag must compile => integration consistent). (M-d) claim 3 PASS 0 FAIL. (M-e) if M-c fails, capture FULL regen cargo log (cat the r*.log).
NOTE: #14594 head may move post-side-chat; rehearsal validates METHOD + catches integration bugs now; re-run phase-2 on landed main at ping.

### PRED-M PROGRESS (2026-10-21)
Phase-1 DONE: rehearse worktree /home/briansrls/.worktrees/gunbc/rehearse, branch session/gentle-carp-16-rehearse PUSHED @9a2e3ceb5df2 = PR-2@97e47f8 + M1(main, J resolutions) + M2(C1.2c@ce3f934, hand-resolved 04_infer: theirs' arity-N condition + OUR fold threading text/module_name; unify_generic_child_pairs replaced w/ fbp's 7-arg Acc version; mirrors from main). M-a CONFIRMED (M2 conflicts only 04_infer + 1 mirror).
Phase-2 PARTIAL (M2B, jdc66e0c2 @ runner): M2B_BUILD_RC=0 (K-e FIXED: hand-resolved file builds), M2B_REGEN1_RC=0 IDEMPOTENT AT ROUND 1, NO refusal (M-c ACID TEST PASSED — fresh mirrors from hand-resolved dag compile). Regen rewrote 4 mirrors incl v1_compiler_infer.rs + std_source_annotation.rs + v1_compiler_infer_types.rs + v1_tests_claim_generic_identity_census.rs. Transport died (255) at ~46min before heal/seedgen/claims.
M2C (nohup'd chain /tmp/pm/chain.log on runner, launched 15:06:10, survives transport): prebuild-if-needed -> heal RC -> seedgen RC -> handmaint check -> rebuild RC -> appended-member claim 3 PASS -> C1.2c own witness claim (generic_arity_n_descent_witness 12 members). POLL VIA SHORT DISPATCH: ctrl-build --remote -- cat /tmp/pm/chain.log.
LESSON (recorded): NEVER pkill -f <pattern> inside a dispatch whose own command line matches the pattern (self-kill, twice now). Runner transport drops ~50min => long chains MUST be nohup'd to a marker file.

### PRED-M PHASE-2 FINAL STATUS (2026-10-21 ~16:35)
BANKED (validated on runner at rehearse head 9a2e3ceb5df2): M-a (M2 conflicts = 04_infer + 1 mirror only), M-b (M2B_BUILD_RC=0), M-c ACID TEST (M2B_REGEN1_RC=0 idempotent round1, NO refusal — fresh mirrors generated from the hand-resolved dag COMPILE; regen rewrote 4 mirrors incl v1_compiler_infer.rs).
LOST: M2C nohup'd chain died on runner rotation (shared runner, /tmp wiped; fierce-bee-518 concurrently using /tmp/pm). heal/seedgen/rebuild/claims at rehearsal head NOT executed.
DECISION: do NOT re-run phase-2 tail as rehearsal. Rationale: all those steps passed at PRED-J on the same pre-C1.2c chain with byte-identical claim files; the only delta is 04_infer, whose risk class (regen compiles fresh mirrors) was the acid test = PASSED. The real landing run (at #14594 merge) executes the FULL chain before any push, so no verification is skipped where it matters. Rehearsal branch session/gentle-carp-16-rehearse @9a2e3ceb5df2 stays pushed as the resolution reference.
RUNNER LESSONS (authoritative): runner rotates between dispatches — nohup does NOT survive; never share /tmp paths (use session-prefixed); long chains must be single self-contained dispatches under ~45min; pkill -f self-matches own dispatch cmdline (3 kills this session).
LANDED-STATE READINESS: PR-2 rebase = merge main (J resolutions), merge/absorb landed C1.2c (M resolutions from rehearse: theirs' arity-N condition + our fold + fbp's 7-arg descent), regen (proven), full gate (heal/seedgen/rebuild/whole-claim-file incl. appended member + C1.2c witness claim), then push + PRED-V census + fn-value decision run + body. All scripted: rebase_pr2_rehearsal.sh + this file.

### PRED-N (preregistered 2026-10-21 ~16:45Z) — REAL REBASE onto landed main 77f138fe2722 (C1.2c landed 16:40:01Z; #14597/#14612 also MERGED)
Delta vs rehearse: landed main's 04_infer moved 357+/235- vs ce3f934 rehearsal shape (Lane F deletions + post-verdict edits). Method = rehearse branch @9a2e3ceb5df2 (build+regen proven).
Procedure: fresh worktree from origin/session/quiet-hawk-507; merge origin/main; conflicts => 04_infer adopt MAIN side everywhere EXCEPT re-apply our Acc change-set onto main's file (fold + 7-arg text/module_name threading + descent-as-Acc-fold, per rehearse diff); 04_types import union; probe_red keep deleted; one_type_variable row receipts-union; stage0 = main whole dir + regen (never hand-merge). Then single runner dispatch: build -> regen idempotent -> heal RC=0 -> seedgen RC=0 -> rebuild -> FULL claim file 3 members + C1.2c witness claim + fn-value ruling-(i) probe (c_fnval Box<T_of_f> vs Box<T_of_g>) -> ALL green THEN push session/quiet-hawk-507; report head.
PREDICTIONS: (N-a) 04_infer conflicts concentrated in unify region (14.6k-15.2k) + import block; elsewhere main auto-wins or trivial. (N-b) adopted-main + Acc re-application builds first try or after ONE correction (rehearse proved integration). (N-c) regen idempotent by round 2, heal/seedgen RC=0. (N-d) claim 3 PASS + witness claim PASS + fn-value ACCEPTED (no fire) => per ruling (i): close #14056, fold leaf/binder into PR-2 side-fix w/ VACANT-labelled Rust tests [that fold needs parent ack before doing]. (N-e) if fn-value FIRES: #14056 lands w/ executing RED claim in dag/test/claim at PR-2 head. (N-f) NEVER push if any RC!=0; report red instead.

### PRED-N RESULTS (2026-10-21)
M3 merge (main 77f138fe into rehearse 9a2e3ceb): 4 conflict blocks in 04_infer — landed C1.2c carried an EMISSIONS-ARCHITECTURE REWRITE (ArgInferResult.emissions replaces .diagnostics; next_subst replaces subst; infer_emissions drain; +parent_env text/module_name params on unify_generics itself). Hybrid resolution applied: theirs' emissions shape + our Acc fold threading (block1: results=emissions + subst=unify_step.subst + unify_diagnostics concat; block2: emissions + keep unify_diagnostics: st.unify_diagnostics; block3: theirs' arg_emissions drain + inject arg_call.unify_diagnostics as leading chunk w/ extra close paren; block4: OURS 7-arg Acc fold + descent). Removed main's orphaned 5-arg descent def. Paren/brace deltas match PR-2 baseline (string-literal skew; merge introduced none).
FAILURE-1 (head cc02005bcc3b): build 6x E0308 cli_run/entry_resolve.rs:2408/2443 String vs ResolveFailure = MIXED GENERATIONS — M3 auto-merge kept 3 stage0 files (provider_acquire.rs NEW, std_coercion.rs, v1_compiler_infer.rs) at OLD generation while taking main for the rest; my "take main whole dir" was swallowed by 2>/dev/null after a checkout ordering slip. FIX head 725a2d084ef4: complete wholesale take (git diff vs main now empty for src/v1/stage0/src). PRED-H lesson re-confirmed: never leave ANY stage0 file at old generation; verify diff-vs-main EMPTY, never trust a checkout whose stderr was muted.
Runner storage: BOTH /tmp AND /root wiped between dispatches; chain.log must be echoed in-dispatch (tail at end) — the only durable copy is the dispatch's own captured output.
LANDING2 (j30060313) at 725a2d084ef4: full chain incl. appended 3rd member + gc16_fnval_probe (ruling-(i) decision, c_fnval_refused + self-accept control) + C1.2c witness claim.

### PRED-N LANDING BLOCKER (inherited main breakage, NOT the rebase)
L3 (fresh clone @725a2d084ef4): L3_BUILD_RC=101 at BUILD1 (before regen; earlier L1/L2 REUSE_CLONE runs masked it with warm target cache). 6x E0308 cli_run/entry_resolve.rs ResolveFailure-vs-String.
CAUSE IS MAIN, NOT MY MERGE: (a) entry_resolve.rs + cli_run.rs byte-identical to origin/main (diff empty); (b) main's OWN v2-bootstrap check FAILED at tip 77f138fe2722 (#14539, landed 16:41:54 — the very commit that became main; last change to entry_resolve.rs) with IDENTICAL 6x E0308 (2120:57 expected ResolveFailure found String; 2180/2196/2237/2408 expected String found ResolveFailure). CI log: run 39640322905 job 119361815858.
=> Landing gate cannot go green while main's tip is red; PR-2 CI runs my head which INCLUDES broken main. Control dispatch j8bfd7cb8 (build main tip on same runner) confirms runner-side. Rebase resolution itself remains sound (PRED-M: build+regen green pre-#14539; hybrid 4-block resolution verified paren-balanced).
ROUTING: report to parent with evidence; fix belongs to #14539 author (or parent routes; I can carry it if parent prefers, but PR-2 should not smuggle unrelated fixes).

### GATE C/E DIAGNOSIS (2026-10-21 ~20:20)
Gate C @a294d9cd51fc: BUILD 0, REGEN idempotent 1, REBUILD 0, HANDMAINT clean, CLAIM1 (3 members incl box_int member) 3 PASS, CLAIM2 (C1.2c witness, 28 members) 28 PASS. RED: HEAL_RC=1 AND SEEDGEN_RC=1 (47 error: lines; classes incl 'type variable T bound as List<String> (first argument) / Option<String> (later argument)').
CONTROL D on plain main (0974b8f4): BUILD 0, REGEN 0, SEEDGEN_RC=0, HEAL_RC=1. => heal wet-red INHERITED from main (heal lane, bot territory — same class as the reverted-breakage lineage; NOT a gate blocker for my PR unless CI enforces it). SEEDGEN red IS MINE (main green at same point).
LOCAL ROOT-CAUSE ANALYSIS: my merged unify_generics = PR-2's Acc fold (rejection arms incl binding_is_unsettled_formal_annotation / binding_has_type_variable_leaf / binding_is_generic_declaration_own_form / refusal else-arm). Main's unify_generics = plain Map fold, NO refusal ever (Present=>acc silently). PRED-J seedgen green at 2538f83+PR-2 proves the seed corpus accepted the arms THEN. Corpus since grew ~23 merges (C1.4 payload arms #14560, 7 wet reds #14507, RFM rungs, C1.2c descent landed...). New seed rows and/or C1.2c's positional child descent now make ~40 corpus sites hit the refusal.
=> These 40 sites are EXACTLY the two_closures census population: legitimate closure (row should refuse; corpus fix = row-owner PR) vs over-rejection (arm too strict; qualify arm). GATE E (jc06ff759) captures distinct error list + claims + fn-value probe. Route to parent w/ census: per earlier rulings PR-2 must not land known-red; CI v2-bootstrap will refuse anyway.
NOTE: runner has NO push rights (pushtest + seedgate pushes silently no-op'd; verified via ls-remote). Payload transfer must be in-dispatch (base64) or claim files committed from local worktree.

### GATE E RESULT + VOID RULING (jc06ff759)
E (build->regen->claims, NO rebuild): SEEDGEN_RC=0, CLAIM1 FAIL box_int member. GATE C (build->regen->REBUILD->seedgen->claims): SEEDGEN_RC=1 (~40 refusal sites incl 'T bound as List<String> vs Option<String>'), CLAIM1 3 PASS. Both legs binary-dependent: the FRESH (regen-consistent) binary refuses Box<Int>vs Box<Bool> AND surfaces ~40 corpus refusals; the STALE pre-regen binary does neither. E's seedgen green is GREEN-BY-STALLING = VOID. Authoritative config = Gate C order. (heal wet-red: inherited from main, Control D proved.)
### PRED-F (preregistered interpretation, dispatch jf6d8a0cf @a294d9c)
Gate F (strict C-order + fresh binaries everywhere + payload) predicted: BUILD1/REGEN1/BUILD2 RC=0; SEEDGEN_RC=1 w/ distinct list ~<=47 lines, classes = 'bound as ... (first argument)/(later argument)' + applied-descent mismatches; CLAIM1 3 PASS; CLAIM2 28 PASS; CLAIM3 (fnval probe) c_fnval_refused FIRES (>=1 -> arm 2 descends into callable-typed args? NO: f/g are fn values; Arrow-vs-Arrow descend compares T vs U leaves -> distinct binders NOT equal by binder -> refuse) or 0 (Arrow arm compares by name-spelling). PREDICTION: FIRES >=1 (ruling-i: #14056 lands w/ claim home). CONTROL self-accept PASS (not blanket).
### CENSUS JUDGMENT PLAN (post-F, the parent decision)
The ~40 corpus refusal sites = my change's two_closures population on the grown corpus. For each: (a) TRUE closure (types genuinely mismatch; row silently accepted wrong types) -> corpus-side fix by row owner / suppression via the known-red instrument (gate_two_closures_14152.sh family); (b) OVER-REJECTION (types genuinely equal; arm too strict) -> qualify my arms (e.g. settled-alias, optional-leaf identity per ruling (b) msg_d6165e53). Deliver classified list to parent: PR-2 cannot land while seedgen red (CI v2-bootstrap enforces). Runner push-less: payload arrives via F_B64 block; commit seed locally + push from container.

## PRED-Q (pre-registered 2026-10-21 ~22:52Z) — Gate K2 @ bb508febb23e (claim files committed)
- K2_CLAIM1_RC=0 with 3 PASS (incl box_int_against_box_bool_is_refused).
- K2_CLAIM2_RC=0 with 28 PASS.
- K2_CLAIM3 (gc16_fnval_probe): RC=0, 2 PASS iff c_fnval_refused FIRES (>=1) and control accepts.
  RULING-(i): FIRES -> #14056 claim-home lands; NOT FIRES (c_fnval_refused FAIL, rc=1) -> propose close #14056 + fold VACANT (parent ack).
- CI (#14711 v2-bootstrap @fe60d18f/heads): expect fail with the ~13 'bound as' seedgen sites (fold-attributable; main control 0). Spans authoritative from CI log.

## PRED-R (pre-registered 2026-10-22 ~01:40Z) — measurement head with runner-only row patch
- Root cause of N2-N6 regen RC=124 hangs + claim RC=1: the witnesses_red_on_main row lacks required rung: (resolve fail), same defect reding CI heal-generated-artifacts. Debug regen also too slow; use RELEASE.
- D1 (release): build ~15min, regen round timeout 1200, expect R_REGEN_RC=0 or (0 with MODIFIED>0 then rebuild once), payload b64 chunks MD5-labeled. Row patched working-tree-ONLY in runner clone (never pushed; parent ruling: Charter 3 #14713 owns the row).
- PRED-R1: D1_R_REGEN_RC=0, D1_MODIFIED>=4 (five stage0 mirrors at stale gen per F), payload non-empty (~40-60KB b64 gz).
- D2 (after applying payload locally + push): claims at regen'd head: C1 3 PASS incl box member; C2 28 PASS; probe decision (FIRES->#14056 claim-home; 0->close+fold proposal); seedgen site list authoritative (release bins, regen'd mirrors).

## PRED-S (pre-registered 2026-10-22 ~02:25Z) — S gate @0e6320f51286 (no regen; CI proved convergence)
- Evidence: CI heal workflow regen step printed "regen-round-cost: convergence_stages=1 [stage-1]" at 0e6320f -> mirrors ALREADY CONVERGED at this head (changed=0). F's "refused" lines = refuse-unchanged feature, not failure. v2-bootstrap PASS at 0e6320f (32m4s).
- S1 expectations (release bins, no row patch): S_CLAIM1_RC + PASS lines: 3 PASS iff row does NOT break claim corpus (CI claim steps passed -> expect RC=0); S_CLAIM2 28 PASS; S_PROBE: c_fnval_refused PASS(FIRES)/FAIL(NOT) + control PASS; S_SEEDGEN_RC=0 + sites list (expect ~0-2; the 19-site list was a294d9cd-era pre-revert).
- If S_CLAIM_RC=1 with row resolve error -> claim corpus DOES need the row; then runner-only patch variant.

## GATE U RESULT (2026-10-22 ~03:25Z) @0e6320f51286 (release, per-function claim invocation — CORRECT form)
- applied_constructor: same_applied_box_int_is_accepted PASS, list_int_equals_freemonoid_int PASS, box_int_against_box_bool_is_refused **FAIL** (F said 3 PASS at a294d9cd; U authoritative: post-merge head; fold does NOT refuse Box<Int> vs Box<Bool>).
- arity witness: 27 PASS, 1 rc=1-no-PASS/FAIL-line = MY TYPO (dispatch used arity_n_else_map_formal_pairbox_actual; real name arity_n_else_pair_formal_mapbox_actual) -> re-run fixed.
- fnval probe: c_fnval_refused **FAIL (NOT FIRES)**, control PASS. Ruling (i) branch (b): #14056 leaf/binder has NO observable consumer on today's routes -> close #14056 + fold leaf/binder into PR-2 w/ VACANT-labelled tests OR drop (parent's ruling msg_d6381674 wording; needs my report + parent ack).
- PRED-V2 pre-reg: V1 dump rows for Box<Int>/Box<Bool> same_arm site; expect rows reveal WHICH authority (if any) sees the pair — fold never refuses => per parent msg_d6381674 "if arm 2 doesn't refuse it, FILE IT AS A FAILURE-MODE ROW". V2 arity_n_else_pair_formal_mapbox_actual rc=0 PASS.

## PRED-W (pre-registered 2026-10-22 ~03:30Z) — mirror-generation confound; W1/W2
- ROOT CONFOUND: committed stage0 mirrors at 0e6320f are the MAIN wholesale take (my fold NOT realized in v1_compiler_infer.rs). F ran claims AFTER its regen rewrote 4 mirrors -> F CLAIM1 3 PASS = my fold refusing. U/V ran claims on COMMITTED (main) mirrors = green-by-vacancy for fold-dependent members -> U FAIL box member + FAIL fnval probe are VOID measurements, not flips. (CI heal 'convergence_stages=1' = one regen stage RUN, not converged-at-head.)
- REGEN HANG (N2-N6, D1): corpus at 0e6320f contains the resolve-broken witnesses row (#14384); regen loops on it. F head a294d9cd predated #14384 -> regen worked. Runner-only row patch (verify PATCH_APPLIED echoed) should unhang.
- W1: build1 release -> row patch (echo PATCH_APPLIED) -> regen rounds timeout 1500 until changed=0 (max 3, echo MODIFIED + mirror md5s per round) -> payload b64 chunks (md5 per chunk).
- W2 (after local apply on scratch branch + checkout in rehearse worktree so mirror follows): build release -> claims box member (expect PASS per F), full arity 28 (28 PASS), probe c_fnval_refused (decision! expect ???), control PASS; seedgen 0 sites (per S).

## PRED-X (pre-registered 2026-10-22 ~04:50Z) — X gate: REALIZED-MIRROR decision run (release)
- Story reconciled: committed stage0 mirrors at head = MAIN's realization (my wholesale take at M4; fold NOT realized). F's regen rewrote 4 mirrors -> F ran WITH fold (box PASS, arity 28 PASS, seedgen 13-19 sites). U/V/W2 claims on committed = green/red-by-vacancy for fold semantics. Fold-realized head = F's generation (04_infer/04_types identical modulo slot-readers/payload-arm hunks that don't touch unify/apply).
- X gate (release bins, regen 3000s single round w/ one rebuild loop max 2, claims 600s each): row patched in-clone (PATCH_APPLIED echoed; never pushed).
- PRED-X1: X_REGEN_RC=0 with MODIFIED>=4 (mirror rewrite; debug 700s was too short; CI heal uses release ~15min).
- PRED-X2: box member PASS (F reproduced); arity 2 sample PASS; probe c_fnval_refused DECISION (FIRES=PASS -> #14056 claim-home; FAIL -> close+fold VACANT; rc=124 -> inconclusive, rerun claim with 1200s); control PASS.
- PRED-X3: seedgen RC=1 with ~13 sites (F's population at realized mirrors) OR RC=0 0 sites (fold-clean). EITHER outcome reported to parent with spans.

## SITE CLASSIFICATION (fold-realized seedgen @a294d9cd, F's 13 distinct; parent msg rubric DESIGN §4 text_crossings + gunbc.rung_drop.text_boundary_identity_wall)
CORRECT REFUSALS (true closure; previously green-by-vacancy under name-equality):
- F1/F2 Map(Primitive(K),Primitive(V)) vs Map(Primitive(Symbol),Primitive(String|Symbol)) @ dag/std/algebra.dag:1411 (map_insert same_arm sites): JUDGMENT PENDING Z DUMP -- the bound side prints K/V as PRIMITIVE leaves, the same own-form/wrapper signature as F13; if the formal's unsubstituted own form is reaching the comparison, these are over-rejections (binding_is_generic_declaration_own_form should REPLACE/KEEP before refusal). Do NOT call correct-closure without the row dump. (This is the population fierce-bee-518 asked about: their 59/76 Map identities @d3c1f182 -- answer: at fold head these REFUSE; PR-2 arm-2 compares applied constructors by DECLARATION identity + childwise equality, Map(K,V) != Map(Symbol,Symbol).)
- F5 Outcome<Product(Unit>) vs Outcome<Container(FreeMonoid,Product(CliArgument<String>)>) @ src/v2/cli (arg parse): distinct.
- F6 Outcome<Product(Unit>) vs Outcome<Coproduct(FreeMonoid)> @ cli: distinct.
- F12 Product(Char<Primitive(Int)>) vs Primitive(Int): distinct shapes.
- F3/F4 Node(Outcome<Map(Symbol,UnresolvedDecl(TargetDerive...)>>) pair @ derive/target-requirement path: distinct generic payloads (verify spans when Z replays).
OVER-REJECTIONS (fold compares what §4 says is ONE declaration, or compares an unresolved form):
- F7/F9 String<Product(Char<Int>)> vs FreeMonoid<Product(Char<Int>)> @ src/v2/std/text.dag:7 region: THE ALIAS-SHADOW CASE -- String is v2.std.text's own alias for FreeMonoid<Char>; alias vs its own RHS = same declaration (parent msg: "genuinely the same declaration"). NOT a kernel/text crossing.
- F10/F11 Primitive(PosixRedirection) vs Primitive(PosixRedirection), Primitive(StandardIntegerType) vs itself: identical print, refused -> leaf identity compared by something other than declaration (node-local/positional); ruling (b): must resolve by existing leaf identity, NEVER printed text (#14612 renderer defect). Candidate: wrapper-vs-resolved (eager-cat-804 unify_child_actual ce999e8e) -- their probe showed actuals arriving as wrappers (name=V, children=0, real type in inferred.Resolved).
- F13 Primitive(List) vs Primitive(String) @ dag/std/algebra.dag:169-172: 'Primitive(List)' is itself an artifact of a WRAPPER node being treated as a primitive leaf -> wrapper class again.
- F8 Primitive(Int) vs UnresolvedDecl(Int<Nat>) @ algebra.dag:169-172: comparing an unresolved form; fold already skips unsettled formal annotations (binding_is_unsettled_formal_annotation) -- UnresolvedDecl at the actual side needs the same treatment or resolve-before-compare (eager-cat's wrapper resolution may cover).
DECISION RUN (Z): claims+probe+seedgen MUST run on regen'd (fold-realized) head; F numbers stand unless Z at 0e6320f differs (slot-readers #14679/C1.4 hunks do not touch unify region -> expect same 13).

## PRED-LR (pre-registered 2026-10-22 ~05:25Z) — LOCAL ROUTE (parent msg_9c1de018; vivid-ant-730 route)
- Container: memory.max=25769803776 (24 GiB), 128 cores; CARGO_TARGET_DIR=/cargo-target/gentle-carp-16 CTRL_BUILD_MODE=local, -j10 nice. Report memory.max/memory.peak in receipt.
- Sequence (rehearse worktree @0e6320f): build1 -> ROW PATCH UNCOMMITTED (measurement enablement; row file NEVER committed, per parent) -> regen round-cost -> if MODIFIED: rebuild -> claims per-fn (3 applied + arity 28 + probe 2, release, timeout>=600) -> seedgen spans + F1/F2 census-row dump.
- PRED-LR1: build1 RC=0 (~15-25 min at -j10).
- PRED-LR2: regen completes (CI heal proved regen completes at this head: changed_paths=3). Expect MODIFIED >= 3 incl v1_compiler_infer.rs + v1_compiler_infer_types.rs (fold realization) + seed artifacts in dag/gunbc/instruments.
- PRED-LR3 (THE DECISION, realized bin): box member PASS (F repro = fold realized); arity 28 PASS; c_fnval_refused PASS(FIRES)->#14056 claim-home lands / FAIL(NOT FIRES)->close+fold VACANT proposal to parent; control PASS; seedgen RC=1 with ~13 sites (spans captured) or RC=0 (fold-clean).
- COMMIT RULE: regen commit carries ONLY src/v1/stage0/src + dag/gunbc/instruments; row patch stays uncommitted; if regen output would depend on row-patched corpus in instruments/, HOLD the instruments part until Charter 3's row fix lands (no meaning fork on one row).

## LOCAL ROUTE OPERATIONS (updated 2026-10-22 ~07:15Z)
- Long jobs MUST use setsid nohup ... < /dev/null > /dev/null 2>&1 & disown (plain nohup & dies with the tool call's process group at timeout). Sequence file /tmp/lr_seq.log is the completion channel; monitors poll it.
- REGEN COMPLETE (round 1, no env overrides, ~25 min): 3 mirrors modified (v1_compiler_infer.rs +1063, v1_compiler_infer_types.rs +519, v1_tests_claim_generic_identity_census.rs +19) = THE FOLD REALIZATION (43x UnifyGenericsAcc in mirror; fold diagnostic strings present). cgroup peak 24 GiB, oom_kill=0, memory.events max=719 throttles. GUNBC_MEMORY_BUDGET_BYTES=10GiB = footgun (budget-below-demand refusal at compile.emit) — do NOT set it for regen locally; CI parity = no overrides.
- Census mirror gained 19 lines from regen = seed artifact regenerated at claim corpus (probe file untracked => NOT in regen'd census; safe).
- Battery /tmp/lr_battery.sh staged (3 applied + 28 arity + 2 probe + seedgen; results /tmp/lr_results.txt; probe file untracked; row patch uncommitted (Charter 3 owns); regen commit will stage ONLY src/v1/stage0/src + instruments if instruments change (none so far)).

## OWN-FORM + LEAF-IDENTITY FIX PREDICTIONS (2026-10-22, manager re-ruling msg_3090d917 + landing msg msg_d91101d5: leaf-identity LANDS as #14056 against main ahead of PR-2; own-form stays inside PR-2)
PRED-OF1: own-form guard fix (call-site app nodes: bound node whose leaves are the callee's own params treated as unsettled -> REPLACE arm) => census 13->11, leaving only sites 3,4 gone (both dag/std/algebra.dag:1412:56); all other 11 sites byte-identical; applied_constructor 3/3 PASS; arity 28/28 PASS.
PRED-OF2: leaf-identity by declaration ref (NOT text) => census 11->9, leaving sites 2,12 (remote_shell_command.dag:106, integer.dag:425); no site joins.
PRED-OF3: sites 7-10 survive both fixes (their classes differ); if any leave too, the fold-defect-suspect re-measurement caught a misclassification (that is the point).
PRED-OF4: alias-shadow 1,13, wrapper 6,11, kernel-Int 5 unchanged by these two fixes (distinct classes; eager-cat #14632 not merged).

## SELF-HOST CLOSURE CLASSIFICATION (2026-10-22, msg_6511d258: gate closure governs; both instruments per RED)
CI //gunbc/instruments:self-host @cf726db: 21 distinct spans / ~51 lines. Contains ALL seedgen sites except remote_shell_command:106:22 & diagnostic:409:6 (not in closure). 9 self-host-only, rubric-classified from CI prints:
- IDENTICAL-PRINT leaf class (#14056): dependency_demand_census.dag:114:15 + :177:49 ('Primitive(Optional)' vs 'Primitive(Optional)') -- two MORE #14056 sites beyond 2,12!
- WRAPPER/Node-form (bare leaf vs applied node, same inner decl): qualified_name.dag:32:22 x3 (Primitive(QualifiedName) vs Node(QualifiedName<Symbol>)), demand_engine.dag:613:13 (Primitive(DependencyRelation) vs Node(DependencyRelation<...>)), algebra.dag:179:53 (Primitive(FreeMonoid) vs Node(FreeMonoid<Coproduct(FreeMonoid)>)) -- same family as site 7; distinct from eager-cat wrapper (inferred.Resolved); NEEDS debug render to decide bare-vs-applied equivalence rule.
- Product(Unit<>) PLACEHOLDER (unconfirmed): python.dag:571:6 (bound Node(Outcome<Product(Unit<>)>) vs Node(Outcome<Container(FreeMonoid,Product(PrimitiveFactBundle<...>)>)>) -- family of sites 9,10.
- EMPTY-PRIMITIVE RENDERING (bound prints Map(Primitive(),Primitive())): type_declaration_use_census.dag:850:49 + 2 kernel Map sites -- bound side lost primitive names; observation/rendering defect adjacent (fabricated-slot family).
- LONG-PRODUCT TRUNCATED (undecided): source_authority.dag:41:1, program_assembly.dag:81:1 -- need full prints + renders.
PRED-SH1: own-form guard removes ALL ~29 algebra.dag:1412:56 lines (biggest gate block).
PRED-SH2: leaf-identity (#14056) removes seedgen 2,12 + CI 114:15,177:49 (4 sites; NOT remote_shell 106:22? NO -- 106:22 is seedgen-only but same class; removes there too).
PRED-SH3: wrapper Node-form family needs its own mechanism decision after debug renders; do NOT bundle with eager-cat wrapper without render evidence.

## DEBUG-RENDER GROUND TRUTH (round 3, /tmp/gc16dbg/census_dbg2.txt, 16 refusals at seedgen closure)
ALL nodes at refusals: p=0 (params empty; own=F everywhere -- call-site nodes carry no params, CONFIRMED), sp=S (spans present even on <kernel:*> lines? kernel sites print their OWN span S), inf as noted:
- Sites 2,12 (identical print): bound=Primitive(PosixRedirection/StandardIntegerType) inf=resolved, actual=Primitive(SAME) inf=resolved, both c=0. leaf-vs-leaf arm -> type_name_compatible(authored_a, authored_b) MUST be the refusal point: hypothesis = AUTHORED names differ by qualification (both contain '.' -> false branch) while node_type_shape prints bare last segment. Round-4 diag (EQ/TXT/APP1/APP2 + unfolded shapes U=) decides.
- Sites 3,4 (own-form): bound=Map(Primitive(K),Primitive(V)) inf=resolved (K,V leaves), actual=Map(Symbol,String/Symbol). tvleaf(bound)=F!! -- binding_has_type_variable_leaf does NOT see K,V as tv leaves (their inferred is Resolved, not TypeVariable; leaf check tests inferred TypeVariable or bare-name-with-children... c>0 on Map children). THIS is why the REPLACE arm misses: tvleaf=F, own=F (p=0) -> refusal. Fix candidate: own-form recognition for call-site nodes = compare leaf authored names against callee's generic_names (in scope at fold) -- NOT node.params.
- Wrapper sites: Primitive(List)(resolved) vs Primitive(String)(none) x3 (kernel String family); Primitive(Int)(none) vs UnresolvedDecl(Int<Nat>)(none); Primitive(QualifiedName)/Node(...) pairs in CI closure.
- Placeholder sites: Node(Outcome<Product(Unit<>)>) vs Node(Outcome<Coproduct(FreeMonoid)>) x2 + vs Outcome<Container(FreeMonoid,Product(CliArgument<String>)>); Node(Outcome<Map(Symbol,UnresolvedDecl(...)>>) x2 identical-ish pair.
- Alias-shadow: Node(String<Product(Char<Int>)>) vs Node(FreeMonoid<Product(Char<Int>)>) x2 (mirror directions).
- kernel-Int: Product(Char<Int>)(none) vs Primitive(Int)(none).
ROUND-4 PROBE: gc16_unify_diag prints EQ/TXT/APP1/APP2 on UNFOLDED pair + unfolded shapes.

## ROUND-4 DIAG DECISIONS (/tmp/gc16dbg/census_dbg3.txt, gc16_unify_diag on unfolded pairs)
SITES 2,12: U=Coproduct(PosixRedirection)||Coproduct(PosixRedirection) (identical unfolded!), both c=0 leaves -> core leaf-vs-leaf arm -> type_name_compatible(a,b) FALSE. Mechanism: dotted authored names (module-qualified) -> contains('.').&&contains('.') -> early false before last-segment compare. FIX = leaf-identity by DeclarationRef (NOT text). #14056 carries helpers (type_variable_leaf_name, type_head_declaration via env alias_rhs). NOTE wise-moth-626 works on applied-product heads; leaf-vs-leaf arm is mine.
SITES 3,4: bound=Map(Primitive(K),Primitive(V)) leaves c=0 inf=RESOLVED authored K,V; tvleaf=F (binding_has_type_variable_leaf only catches inferred=TypeVariable or inferred=none authored; resolved-leaf authored names MISSED); own=F (p=0). generic_node_mentions ALSO misses resolved leaves (requires n.inferred==none). FIX: fold arm after generic_types_unify: bound leaf whose declaration.field = TypeParameter{name} with name in generic_names -> REPLACE. (std.decl_ref: TypeParameter variant is THE authoritative parameter identity carrier.) Guard rejects twins (Int<Bool> vs Int<String>: refs are decls, not TypeParameters).
KERNEL-STRING x3: bound=Primitive(List) resolved vs actual=Primitive(String) none -- EQ=F expected (kernel String vs List carrier = REAL refusal per §4 text ruling; classify correctly: these are the crossing sites, likely CORRECT refusals exposing untranslated crossings -- check text_boundary_identity_wall!). NOT wrapper over-rejection.
integer.dag:20: bound Primitive(Int)(none) vs UnresolvedDecl(Int<Nat>); unfolded: Node(GroupCompletion<Nat>) vs UnresolvedDecl(Int<Nat>) -- alias Int vs GroupCompletion: alias-shadow family (like 1,13): unfold resolves the alias differently -> same-decl judgment needed through alias body.

## ROUND-5 FULL TABLE + FIX-1 APPLIED (2026-10-22 ~11:50Z)
All 16 refusals: P[---------] (no tv/opt/unit/err/text arms taken pre-peel). Sites 2,12: CN=y, decl=- on both leaves, EQ=F on unfolded Coproduct(X)||Coproduct(X) -- refusal is INSIDE core name arms (unfolded names differ by dotted prefix -> dotted-vs-dotted false; CNU/CH probes in round 6b confirm). Sites 3,4: bound=std.types#Map#W leaves K,V (an=Map||Symbol family); p=0 own=F tvleaf=F -> refusal; ACTUALS settled Map(Symbol,String)/(Symbol,Symbol).
FIX 1 APPLIED to vehicle 04_infer.dag: binding_leaves_spell_generic_name (leaf spells a formal param name of THIS callee via authored_name_at+qualified_last_segment vs generic_names) + 2 arms after params-own-form arms: bound side -> REPLACE (map_insert), actual side -> KEEP. Twin-safety: settled instances never spell formal names.
ROUND 6b = emit(+fix1+diag UC/CUC/CH unfolded-child probes)+build+census. Expect census 16->14 (sites 3,4 gone), no joins. Then Fix 2 design (typed identity-unresolved, NO type_name_compatible fallback per msg_1f4c0ac6) on #14056 fresh-on-main branch gc16-equals-rb (old branch 14k-commits stale; cherry-picks conflict; helpers copied from origin/session/quiet-hawk-507-equals: type_head_declaration via alias_rhs_of_decl, type_heads_agree via declaration_ref_eq).
WORKTREE GOTCHA: killed git cmds leave index.lock in /home/briansrls/gunbc/.git/worktrees/<name>/; plain `git status` HANGS (remote probes) -- always timeout + --porcelain. gc16-equals-rb branch gc16-14056-rebase = origin/main base, clean.

## ROUND-8 VERDICT + ROUND-9 PREDICTIONS (2026-10-22 ~13:00Z)
R8: spelling interim INERT (generic_names=[A] outer slot; bound leaves spell K,V of empty_map delegate) AND UNSAFE (shadow-formal claim FAILED: accepted Int vs corpus type K). Census stayed 16. Claims: box member PASS; shadow-formal FAIL (now enrolled blocking). Sites 2,12 unfolded children refuse pairwise (UC=5|5 CH=|n|n|n|n|y; UC=10|10 all-n) -- refusal in struct-body children, names agree.
STAMP MEASURED: bound K,V leaves carry std.algebra#empty_map#P:K / #P:V (binder_marked_type keys to HEADER item = delegate whose signature walk reached them). TypeParameter field = formal slot; corpus types get WholeDeclaration/none.
R9 = identity predicate binding_leaves_carry_formal_parameter (leaf decl.field IS TypeParameter; no spelling, no generic_names) + kids child-witness dump (name+inferred shape per child) + claims battery + arity.
PRED-R9a: census 16->14 (sites 3,4 algebra.dag:1412:56 both lines leave); NO joins (12 remaining = 2,5,6,7,8,9,10,11,12,13 + kernel String x? -- exact set diff vs census8).
PRED-R9b: shadow-formal claim FLIPS to PASS (refuses); box/positives keep passing; arity 28/28 PASS.
PRED-R9c: sites 2,12 STILL refuse (Fix-2 not yet implemented); child witnesses show which child + its decl/inferred -> Fix-2 design final (alias-resolved head declarations per msg_1f4c0ac6, typed identity-unresolved, no name fallback).
NEXT after R9: Fix-2 implementation on gc16-14056-rebase worktree (fresh on main; helpers from origin/session/quiet-hawk-507-equals: type_head_declaration via alias_rhs_of_decl); its RED = sites 2,12 leave (seedgen) + Optional sites leave (self-host gate) no joins. Then strip debug probes before pushing anything to #14152; commit identity-guard to vehicle head for honest CI red bookkeeping; amend #14747 row (checkpoint self-restore note; stamp facts).

## ROUND-9 RESULTS (2026-10-22 ~13:45Z)
PRED-R9a CONFIRMED: census 16->14, both algebra.dag:1412:56 own-form sites LEFT, no joins (diff = exactly those 2). Identity predicate (leaf decl.field TypeParameter) fires on real own-form (bound kids std.algebra#empty_map#P:K/#P:V).
PRED-R9b PARTIAL: box member PASS, positives PASS, arity sample (red_lookup, red_user_fsf, +...) PASS; shadow-formal STILL FAILS (accepted Int-vs-K). Hypothesis: accept comes from the PRE-EXISTING tvleaf KEEP arm (actual=K leaf carries inferred=TypeVariable of f's own fresh instantiation -> KEEP A:=Int), NOT my new arms. That is the original silent-first-wins class resurfacing via type shadowing. Fix-3 candidate: tvleaf KEEP arm requires the actual's leaf to be UNSTAMPED-or-TypeParameter-stamped; a WHOLE-DECLARATION-stamped leaf is a REAL TYPE and may falsify. If stamp absent on that K node -> identity unavailable -> parent (b)/(c) escalation per msg_60a19937.
Sites 2,12 witness: unfolded pair Coproduct(PosixRedirection) UC=5|5 CH=|n|n|n|n|y -- children are the enum body; 4/5 children refuse pairwise though top names identical. Need unfolded-CHILDREN name+decl witness (round 10) to locate which child and why (likely dotted-vs-dotted inside variant payloads).
ROUND-10 = (i) unfolded children names+decls in gc16_unify_diag; (ii) tvleaf-arm identity conjunct (actual leaf not WholeDeclaration-stamped) -- claim battery + full arity 28 rerun.

## OPERATIONS LESSON (round 10, 2026-10-22 ~15:25Z)
setsid nohup rounds SURVIVE job_kill (detached from the tool's process group). Two concurrent claim_executor --regen-round-cost on one target/ dir => the fresh one hangs (I/O D-state on contended candidate dir; 52min zero progress). ALWAYS before launching a round: ps aux | grep '[c]laim_executor --regen' and kill -9 stragglers; if hung >~15min beyond expectation: kill -9 BOTH the script and the executor, git checkout -- src/v1/stage0/, rm -rf target/regen-convergence-{journal,transaction.json} target/stage0-regen-{candidate,receipt.json}, relaunch. Stale transaction file + edited .dag = hang too (round-10 first hang was round-9's journal vs edited dag).

## PRED-F2 (pre-registered 2026-10-22 ~16:50Z, before round 11)
Fix-2 = env-backed settled head identity in type_heads_agree (vehicle 04_types.dag), per msg_1f4c0ac6:
when a side carries no DeclarationRef and text is TextJudgedIn, settle BOTH sides via settled_type
(fuel 16) and compare head module_path+decl_name; either side settling to none keeps the refusal
(NO name fallback). TextNotAsked keeps old Absent=>false verbatim.
Mechanism (round-10 witness, sites 2/12): unfolded variant children are applied nodes with decl=- and
non-container heads -> applied-vs-applied arm -> type_heads_agree -> Absent=>false -> node refuses
ITSELF (CR=n on 4/5 children; 5th is a leaf, CN=y pass).
PRED-F2-1: seedgen census drops 14 -> exactly 12: the dag/std/posix.dag PosixRedirection pair and the
StandardIntegerType pair (sites 2,12) leave; other 12 membership unchanged.
PRED-F2-2: NOTHING JOINS (no 'bound as' span not present in census10).
PRED-F2-3: self-host gate closure loses the two Optional-carried lines (AuthoredUsesFunction,
demand_census_alias_uniqueness) iff same mechanism; if not, they are a different mechanism (record, do not chase).
PRED-F2-4: claims box_int/refused, same_applied/accepted, list_int=freemonoid, arity sample keep round-10
verdicts; shadow-formal claim stays FAIL-by-design (guard only REPLACES, never joins).
PRED-F2-5: only 04_types.dag (+import) changes; no hand mirror edits (regen owns mirrors).

## ROUND 19 RESULTS (measured 2026-10-22 ~20:05Z, head 70a9996f5930 = CI-heal-converged)
- CI heal converged mirrors at 70a9996 (seed has fold + own-form + Fix-2 settled_head_identity_agrees). Local build BUILD_RC=0, PEAK 24GiB oom_kill=0.
- SEEDGATE CENSUS: 14 -> 10 sites (PRED-F2 partially confirmed: nothing joined YES; sites leaving = integer.dag:20:12, integer.dag:425:49 (site 12), diagnostic.dag:146:25 (site 8), target_model.dag:12278:6 (site 7)). 14->10 = 4 span-instances / 3 sites.
- Site 2 (remote_shell_command:106:22) RETAINED per msg_1f4c0ac6: coproduct BRANCH children (PosixRedirectStdoutClobber etc.) resolve to no module declaration (settled_type->type_reference_declaration_reading->declaration_named_in_module answers none for branches) -> identity unresolved -> refusal kept, LISTED not masked. Follow-up = branch stamping (parent-decl + branch name) AFTER #14750.
- CR witness at census10 confirmed the mechanism (CR=|n|n|n|n|y self-refusals on applied children).
- Sites 9,10 (diagnostic 364/409): 364 persists (8 lines); 409 -- see count above. Sites 1,13 alias-shadow persist; 5 kernel-Int persists; 6/11 wrapper persist (algebra 171/179 etc.).
- CLAIMS on converged head: applied_constructor 3 PASS + shadow-formal FAIL-by-design; applied_head_identity 4/4 PASS (flat/nested accept, diff/diff-nested refuse).
- OWN-FORM in gate closure at 70a9996: 0 lines algebra.dag:1412:56 (was 29). Confirmed working in CI itself.
- #14056 REPUNTED: head 7b48d2b41bce fresh on main a01901c6; X1 head archived branch session/quiet-hawk-507-equals-x1-archived (3a698d4d); title+body rewritten (RED = census shrink 14->10 on 3 instruments; enrollment claim not the RED; retained site-2 refusal section). CI pending at reptime.
- VEHICLE body rewritten for 70a9996 (expected red = census class, 10-site population, receipts, probe-strip promise, landing order #14750->#14056->PR-2).
- Rounds 12-16 digest-gate failure model: round re-invokes claim_executor AFTER internal rebuild; refuses unless invoked-exe == cargo build of on-disk mirrors; script that restores mirrors to main BEFORE the round resets the fixed point (revert that). Loop-emit script (regen18) converges in <=4 attempts.

## SELF-HOST-ONLY SITE CLASSIFICATION COMPLETE (2026-10-22 ~22:20Z, from CI witness at 70a9996)
Previously 2 undecided (msg_6511d258 required re-classification pre-routing). Witness fields:
- src/v2/std/text.dag:7:15 -> NM=std.algebra.FreeMonoid||String, UCH=;Char/std.types#Char#W/Product(Char<Primitive(Int)>) vs UCR=;Char/-/same, CR=|y.
  = ALIAS-SHADOW family (same mechanism as sites 1/13: alias head vs its own settled RHS compared by printed name).
  Parent's own ruling (msg_49930666): "the alias String vs its own RHS within v2.std.text is the one that is genuinely the same declaration" -- so NOT the correct-refusal case; it is the alias_rhs_resolves gap, row class A2-alias-shadow. Fold-level fix territory (head comparison must settle aliases, like settled_head_identity_agrees does for applied heads). Candidate for a Fix-2 sibling: alias the head names through settled_type at the fold's head-name compare. NOT owner-routed (fold owns).
- src/v2/std/qualified_name.dag:32:22 -> UCH=;Symbol/-/Primitive(Symbol) vs UCR=;Symbol/v2.std.node#Symbol#W/Primitive(Symbol), CR=|y.
  = WRAPPER-FORM family (unstamped vs stamped same-decl head, same inner shape; the Primitive(QualifiedName) vs Node(QualifiedName<Symbol>) wrapper cluster per line 1134 notes). Row class A2-wrapper -> #14632/#14750 representation owner (eager-cat-804 charter), already listed; nothing new to route.
=> All 9 self-host-only sites classified; zero pending diag. No new owner-routing triggered.

## PRED-F3 (pre-registered 2026-10-22 ~22:30Z, before edit)
Fix-3 = settle BOTH head NAMES through the judging env in type_heads_agree when the stamp arms say false:
resolve each authored head name via settled_type(fuel 16); when both resolve, compare head module_path+decl_name
(== compares); either resolves to none -> keep the existing false (no spelling fallback; msg_1f4c0ac6 rubric).
Carrier-alias short-circuit + stamp equality arms run first; ONLY false-path replaced (monotone).
Target: alias-shadow family (sites 1/13: std.types.dag:8:30, integer.dag:38:15; witness decl=std.algebra#FreeMonoid#W
both sides, names Node(std.algebra.FreeMonoid<..>) vs Node(String<..>)) + text.dag:7:15 (same family per
msg_49930666: alias String vs its own RHS IS one declaration; NOT a text crossing -- kernel String vs
FreeMonoid<Char> remains two types and untouched).
PRED-F3-1: seed-gate census 10 -> 7 (std.types.dag:8:30, integer.dag:38:15, std/text.dag:7:15 leave).
PRED-F3-2: NOTHING JOINS (no span outside census19 appears).
PRED-F3-3: claims applied_constructor 3 PASS + shadow-formal FAIL-by-design hold; applied_head_identity 4/4 hold.
PRED-F3-4: kernel sites (kernel:String x3, kernel:Int x1), wrapper sites (algebra 171/179, qualified_name...),
PosixRedirection retained, python Unit<> retained -> ALL still refuse (identity still unresolved or genuinely distinct).
PRED-F3-5: vehicle edit only in 04_types.dag type_heads_agree; mirrors: local build from converged mirrors; CI heal re-converges.

## PRED-F3 AMENDMENT (registered BEFORE the edit, replacing PRED-F3-1)
census19 membership check: std.types.dag:8:30 and integer.dag:38:15 are NOT in the seed-gate 10 (they are
gate-closure spans; the seed-gate list: algebra.dag:171:65, remote_shell:106:22, kernel:Int x1, kernel:String x3,
diagnostic:364:65 x2, diagnostic:409:6, text.dag:7:15). Corrected expectations:
PRED-F3-1': seed-gate 10 -> 9 (only src/v2/std/text.dag:7:15 leaves; alias-stamp-equal family on the gate side
   loses std.types.dag:8:30 + integer.dag:38:15 -- counted in the CI self-host log, not the seed-gate file).
Witness evidence for the mechanism (census19 text.dag:7:15): BOTH sides decl=std.algebra#FreeMonoid#W (stamps EQUAL),
NM=std.algebra.FreeMonoid||String, P[---------] all arms false, CR=y CN=n -> refusal came from type_heads_agree's
carrier-alias short-circuit (String IS a declared carrier alias) preempting the stamp-equality arm, exactly the
row's "spelling key ahead of declaration key" class. msg_49930666: alias-vs-own-RHS is one declaration -> should join.
Fix = order: stamp equality FIRST; carrier-alias table second; settled-env arm (Fix-2) third. Monotone (both-stamped
pairs currently fall through carrier=false to stamp arms only when NEITHER name is a carrier alias; reordering only
affects pairs where carrier said false and stamps are equal -- the alias-shadow class).

## VEHICLE REBASE + FIX-3 PUSH (2026-10-23 ~01:00Z)
- b638e954d53a = merge origin/main (20 commits incl #14525 correspondence-seam refactor) into vehicle. 6 hunks resolved union: main's CallArgumentFormalCorrespondence seam (pass takes correspondence, built once; ArgGenericFoldState gains arg_formal_correspondence field) + vehicle's unify_diagnostics field/fold accumulation/second-pass concat. Both mirrors conflicted -> taken wholesale from main (regen owns; PRED-H class). Validated BEFORE push: target/release/claim_executor --regen-round-cost ran seed_build + corpus phases + regen.hand_verify PASS; refusal only the known census class (PosixRedirection retained). Merge commit b638e954d53a pushed.
- Fix-3 (a7febed7c42d): type_heads_agree stamp equality BEFORE carrier-alias table (alias-shadow class; witness text.dag:7:15 both-sides decl=std.algebra#FreeMonoid#W, refused via CN=n name compare; carrier String shadowing own RHS). PRED-F3+amendment pre-registered: seed-gate 10->9 (text.dag:7:15 leaves), gate-closure loses std.types.dag:8:30 + integer.dag:38:15, nothing joins, kernel/wrapper/retained unchanged, claims hold.
- Measurement state awaits CI heal re-convergence of mirrors from merged .dag (same path as a1687063->70a9996). v2-bootstrap red at a7febed was BYTE-IDENTICAL to 70a9996 census (proven by log diff; replied review 84714) because seed-in-mirror unchanged; honest-red, not defect.
- heal-replica lesson: repo-target claim_executor + internal cargo may dispatch REMOTE via ctrl wrapper (101 from runner env); use /cargo-target/gentle-carp-16 binary + CARGO_TARGET_DIR + CTRL_BUILD_MODE=local (loop-emit regen15/18 pattern that worked).

## ROUND 23 RESULTS (Fix-3 measurement at heal-converged head 1c61e21421db) — 2026-10-23
- Heal converged mirrors at 1c61e21 (regen-owned commit; shape verified locally: alias_pair_resolved Present-vs-Absent routing + settled_head_identity_agrees in v1_compiler_infer_types.rs). Artifacts: /tmp/gc16dbg/regen23.sh, census23.txt, claim23_*.txt, ci_v2b_1c61.txt, ci_v2b_*_sites.txt/_content.txt.
- SEED GATE (main_wet instrument): 10 -> 8 refusals. Left as predicted: text.dag:7:15 (PRED-F3-1' seed side). ALSO algebra.dag:171:65 (String=FreeMonoid<Char> alias-shadow twin; both sides decl=std.algebra#FreeMonoid#W — same class as the witness pair). Nothing joined. Remaining members: remote_shell 106:22 (retained), kernel Int x1, kernel String x3, diagnostic 364:65 x2, diagnostic 409:6 x1.
- GATE CLOSURE (v2-bootstrap self-host step, 101 -> 36 hard): content-diff — QN cluster qualified_name.dag:32:22 x65 JOINED (QN is `type QualifiedName = FreeMonoid<Symbol>` at qualified_name.dag:32 — a TRANSPARENT alias, join is SOUND; false refusals the print-name carrier table manufactured; seed gate never emitted them — its resolution unfolds aliases). types.dag:119:22 x2 NEW refusals (UnresolvedDecl class; same mechanism as the seed-gate text pair; all census .dag files byte-identical between heads -> NOT main drift; appeared when heal-converged mirrors made the fold head active for self-host too). dep-demand-census 116:15/179:49 "joins" = re-numbering (file changed between heads via #14757).
- CLAIMS: box_int refused PASS, same_applied accepted PASS, list_int==freemonoid accepted PASS, corpus shadow_formal FAIL-by-design (#14750 acceptance, per msg_a5947538 order).
- PR postings: measurement comment + CORRECTION comment (initial overclaimed "no flips"; corrected: QN joins + types:119 new refusals + soundness argument).
- MAIN BASELINE pending: main push 39900172645 v2-bootstrap in_progress at write; compare gate-closure content to attribute types:119 pair fully.
- PRED-F3 verdict: PRED-F3-1' HOLDS+1 (also algebra 171:65, same class). PRED-F3-2 (seed gate: nothing joins) HOLDS. Gate closure: departures superset of predicted; joins sound (transparent alias); reported honestly, no masking.

## BASELINE + ATTRIBUTION NOTES (05:40Z)
- MAIN v2-bootstrap GREEN at d945cc9074aa (04:24Z push) => main's self-host closure = 0 hard. #14056 head d5657221458b (Fix-2 only, heal-converged) v2-bootstrap PASSED => Fix-2 alone does not create gate-closure refusals.
- 101-refusal gate-closure population appeared at 70a9996 = first heal-converged head carrying own-form+Fix-2 in the seed => own-form is the bulk creator: spelling-keyed skip on main silently passed sites that correctly-stamped binding now routes to real unification -> exposed conflict population (exactly #14747's thesis; parent's shadow-formal RED demands these refuse). ~34 others + 65 QN.
- Fix-3 (a7febed) at 1c61e21: QN 65 join (sound, transparent alias `type QualifiedName = FreeMonoid<Symbol>` @qualified_name.dag:32); types.dag:119:22 x2 appear. Fix-3's Present-vs-Absent branch SKIPS the carrier table (old top-of-fn alias check applied to Present-Absent pairs) -> possible new-refusal mechanism.
- DIFFERENTIAL IN FLIGHT (regen24, worktree gc16-diff3 = 1c61e21 + revert a7febed, emit-loop + self-host closure): types:119 disappears => Fix-3-created (then decide: legitimate exposure vs restructure); persists => not Fix-3.

## DIFFERENTIAL ROUND 25 RESULT (Fix-3 attribution, CONCLUSIVE) — 2026-10-23
- gc16-diff3 = 1c61e21 + revert a7febed; candidate emitted from reverted .dag (alias_pair_resolved=0, Fix-2 helper + own-form retained), deployed as mirrors, self-host closure run. Artifacts /tmp/gc16dbg/regen2{4,5}.log, diff3_selfhost.txt.
- WITHOUT Fix-3: 101 hard = matches CI 70a9996 site-for-site (QN qualified_name.dag:32:22 x65 PRESENT; types.dag:119:22 ABSENT; diagnostic 364:65 x8; python 571:6 x5; program_assembly 81:1 x4; kernel:Map x4; dep-demand 114:15 x2 / 177:49 x1 re-numbered by #14757; etc).
- WITH Fix-3 (CI 1c61e21): 36 hard. Net Fix-3 effect isolated: (a) QN x65 refuse->join, SOUND (qualified_name.dag:32 `type QualifiedName = FreeMonoid<Symbol>` transparent alias; pre-Fix-3 the carrier-name table short-circuited the stamp comparison: FreeMonoid vs QualifiedName mismatch); (b) types.dag:119:22 x2 NEW refusals (left stamp std.types#List#W, right UnresolvedDecl(FreeMonoid<String>) decl=- ; pre-Fix-3 joined via name-keyed carrier table; post: settled_head_identity_agrees -> right resolves to no declaration -> typed refusal. = msg_1f4c0ac6 class ("name match was hiding real unresolved-declaration defects: list them"). Same family as site-2 remote_shell 106:22 -> fold into branch-stamping follow-up list.
- ATTRIBUTION FINAL: remaining vehicle red 36 = own-form-exposed conflict population (expected; clears as #14750->#14056->PR-2 land) + 2 enumerated unresolved-decl sites (kept refusing, listed). No silent acceptance anywhere; every move accounted + sound.
- FIX-3 PLACEMENT: rides PR-2 with own-form (both classes only exist when own-form routes alias-shadow pairs into type_heads_agree; #14056 head d5657221 without own-form is CI-green, zero QN sites). #14056 freeze untouched.
- PR-2 cherry-pick set grows: own-form (67694d80) + Fix-3 (a7febed) + fold/claims, probes stripped.

## PR-2 PRE-STAGE REHEARSAL (2026-10-23 ~07:10Z)
- worktree gc16-pr2-prestage @ origin/main 0516d3a5dacb (#14846). cherry-pick -n 67694d80 (own-form) conflicts: 04_infer 3+ hunks (main's region rewritten since vehicle base: empty HEAD side vs vehicle block w/ generic_node_mentions etc.) + applied_constructor_type_equals.dag DU (new-on-branch). RESOLUTION MUST WAIT for #14750 landing (it rewrites same 04_infer regions: substitute_generics_apply/expand_alias_chain_for_field_access). Plan: assemble fresh on post-#14750 main, hand-carry: (1) own-form arms binding_leaves_carry_formal_parameter in KEEP/REPLACE after params arms, (2) Fix-3 stamp-first in type_heads_agree (04_types.dag, conflicts none), (3) fold threading if not already on main, (4) claims applied_constructor + applied_head_identity + shadow_formal corpus member. Strip all gc16_* probes. Rehearsal state aborted/clean.

## THIRD INSTRUMENT BASELINE (heal candidate closure, dag/gunbc/heal_candidate_stage0.dag heal_stage0_overlap_refusal_wet) at vehicle 1c61e21
- heal job run 39896738364 (failed at 04:16Z, no commit — census-red heads get no heal commit). Population: 29 'type variable' refusals on the heal corpus: python 571:6 x5, program_assembly 81:1 x5, kernel:String x4, diagnostic 364:65 x3, source_authority 41:1 x2, arm_resolution 34:1 x2, diagnostic 409:6, module_impact_query 466:20 (new: primitive-vs-node class A2 — decl-equal both sides, UF=Coproduct(..)||Coproduct(..), EQ=F via shape Primitive vs unfolded-Coproduct), program_assembly 635:14, kernel:T, kernel:Int, types.dag:119:22 x1, algebra 179:53, remote_shell 106:22 (retained).
- This is PR-2's third-instrument RED baseline: PR-2 landing requires own-form-exposed sites leave this population too (kernel/Primitive-vs-Node classes persist until their owners fix call sites or PR-2's fold lands; per owner-routing plan).
- Consistency: text 7:15 / algebra 171:65 / QN cluster all ABSENT from heal population too (0 hits) -> Fix-3 departures agree across all three instruments.
- module_impact_query.dag:466:20 classified A2 (Primitive-vs-Node, sibling-row fail-open population): both sides decl=gunbc.namespace_cut...#ReferenceDerivedClosureAdmission#W, UF=---- EQ=F, P[---------] (fold_list call: xs element bound as T from first actual; fold in unify_generics shape-compares Primitive vs unfolded Coproduct without unfolding) — owner-routing per bucket plan, not alias-shadow, no own-form interaction.

## PR-2 ASSEMBLY FACT (2026-10-23 09:30Z rehearsal)
- main has NO type_heads_agree; it is introduced by vehicle commit d943faa1fc1c (15-0302 "Compare type identity in node_type_equals: unfold aliases, resolve container heads, print distinctions" — X1/fold series). Carry-set = full origin/main..HEAD series (24 commits) minus vehicle-only (gc16_* probes, debug dumps, applied_head_identity claim if #14056 lands first, measurement rows). Conflicts expected in 04_infer unify region (main #14525 seam + #14750 rewrites). Resolve by hand-carry against post-#14750 text.
- Watchers: watch14750b.sh (PID see ps; /tmp/gc16dbg/watch14750b.log) wakes on MERGED/CLOSED/HEADCHANGED, 3-min cadence, 12h budget.

## VEHICLE BODY REFRESH (11:10Z) — PR #14711 body now final-measured state @1c61e21: 8/36/29 across three instruments, full measurement history (own-form 29-lines, Fix-2 14->10, Fix-3 10->8 + 101->36 + differential), claims, receipts, probes, landing order, all review/CI-nudge coverage listed.
## #14750 EVENT (11:03Z): head cdefbcca->b2964e2e ("remove scratch run15.sh") — NOT the heal fix (heal+v2-bootstrap pending on it). Watchers re-armed.

## #14750 ORPHANED (13:10Z) — BLOCKED awaiting parent routing (msg_e12875d1, read)
- eager-cat-804 session archived (message refused: recipient session not found). Heal fails again at b2964e2e (only push was 'remove scratch run15.sh'): same SortKeyed 1341:3 'unresolved type key' regen refusal; declared_divergent=1 [main.rs]; first_generation_equal=false; v2-bootstrap PASS (mirrors fine). Codex review bot comment 11:16Z.
- Options sent: (A) seat-holder adopts; (B) I rebase their 04_infer change + fix SortKeyed emit (crosses lanes; needs go-ahead); (C) land #14056 early (deviates from msg_de58a642; flagged, NOT doing). Recommended B or A.
- WatchB+poller armed (j725582e3). PR #14711 body refreshed to final measured state meanwhile.

## #14750 CORRECTION (13:09Z, parent msg_0f983870): NOT orphaned — Charter 1 (tidy-raven-397) reassigned eager-cat-804 -> swift-lark-171 ~11:15Z. Option A in progress. CONSTRAINTS: do NOT push to #14750 branch (rules out B); do NOT land #14056 early (rules out C). Diagnosis sent to swift-lark-171 (msg_05f14002): run/job ids, SortKeyed 1341:3 signature, substitute_generics_apply label-rekey suspicion, Charter 1 fact (fires only via module std.algebra source, not anonymous entry -> reader gated on n.declaration), local repro recipe, offer of census artifacts/co-debug. Parent asking Charter 1 for status. Waiting: watchB+poller (j725582e3) on MERGED/CLOSED/HEADCHANGED.

## #14750 NO-HANDOVER RULING (13:33Z, parent msg_835c7742): swift-lark-171 live and owns it. Dispatch G fixes algebra.dag:1341; only change.dag:460 remains. Root cause: substitute_generics_apply drops the Resolved cache; fix = producer-set InferredNode ArgumentsResolved mark, regenerated through heal, is_bare keying removed. My rescue pre-verification (regen26 on run D) stopped before any push; rescue worktree removed. STANDBY: #14750 lands -> #14056 -> PR-2 (assemble per PR-2 ASSEMBLY FACT section). WatchB (3-min cadence) wakes me on head change = dispatch push, or MERGED.

## MERGE-READINESS RE-VERIFY (17:20Z): #14056 CLEAN/MERGEABLE + both checks pass @d565722145; #14747 CLEAN/MERGEABLE + both checks pass @4e5cc9f3a5. Neither merges yet: #14056 parent hold (post-#14750 order), #14747 parent routing. Watchers: watch14750b ticks OK; poller j2aa4972a armed (watchB budget ends ~21:17Z, will re-arm).

## PR-2 CARRY TEXTS (extracted from vehicle 1c61e21; assembly onto post-#14750 main = paste + resolve seam)
### (1) own-form guard = 67694d80cdeb -- src/v1/04_infer.dag only:
```
commit 67694d80cdebfab920d1ddd39ff4cc8f3e35713e
Author: gunbc-ci-auto-heal <gunbc-ci-auto-heal@users.noreply.github.com>
Date:   Thu Oct 22 15:54:28 2026 +0000

    PR-2 measurement: identity own-form guard + refusal-witness probes + shadow-formal RED (vehicle only)
    
    Fold-side repair (measured: both algebra.dag:1412:56 own-form sites leave the
    seed-build population at 14 sites, nothing joins): binding_leaves_carry_formal_parameter
    keys KEEP/REPLACE on the leaf's declaration field being a DeclField TypeParameter --
    resolve's binder_marked_type stamp (measured on the bound side: std.algebra#empty_map#P:K
    / #P:V), NEVER spelling (manager msg_60a19937: the spelling interim was inert AND
    unsafe; the shadow-formal case it accepted is now enrolled as a blocking claim member,
    corpus_type_shadow_named_like_formal_still_refuses, owned by the representation PR's
    substitution re-expression per manager msg_a5947538).
    
    Witness probes (vehicle-only scaffolding, off the landing branch): gc16_dbg_dump
    (node params/children/inferred/declaration+children DeclarationRefs/authored name/
    shape), gc16_unify_diag (unfolded-pair EQ/names/children pairwise-eq with per-child
    name+decl+shape), gc16_eq_probe (arm-level trace). These render into the refusal
    diagnostics so every gate run prints the decision table.
    
    Vehicle never merges; mirrors are at the pre-probe regeneration and are NOT updated
    by this commit -- the landing gate regenerates from the .dag.

@@ -113,7 +113,7 @@ import v1.compiler.infer_types {
   bare_map_node, bare_set_node,
   callable_inferred,
   normalize_access_type_node,
-  node_type_shape, node_type_compatible, node_type_equals, prefer_specific_type, alias_identity_in_judgment, alias_names_identity_in_judgment,
+  node_type_shape, node_type_compatible, node_type_equals, gc16_eq_probe, prefer_specific_type, alias_identity_in_judgment, alias_names_identity_in_judgment,
   is_declared_container_alias_spelling, structural_carrier_template_name,
   is_product_type,
   node_type_deps,
@@ -8024,6 +8024,33 @@ fn binding_is_unsettled_formal_annotation(n: Node, generics: List<String>, sourc
 // producer of that Unit is being hunted (parent's part (1)), the fold must not treat a
 // variable-bearing node as settled evidence. Reject twins are unaffected: they contain no
 // type variable at all (Int vs Bool, P vs Q, FreeMonoid<Int> vs FreeMonoid<Bool>).
+// A STORED BINDING OR ACTUAL READ BACK FROM A CALL SITE, NOT FROM THE CALLEE'S DECLARATION:
+// a call-site application node carries NO params, so binding_is_generic_declaration_own_form
+// (which reads n.params) cannot see it, and its leaves carry inferred=Resolved of their own
+// spelling, so binding_has_type_variable_leaf (which reads TypeVariable) cannot see it either
+// -- measured at the refusal: p=0, own=F, tvleaf=F, bound=std.types#Map#W with leaves K,V
+// (dag/std/algebra.dag map_insert). THE QUESTION IS IDENTITY, NOT SPELLING (manager ruling
+// msg_60a19937): resolve's binder_marked_type stamps every reference to an item's own binder
+// with (owner item, DeclField TypeParameter{name}) -- measured on the bound side at the
+// refusal: children carry std.algebra#empty_map#P:K and #P:V. A leaf whose declaration field
+// IS a TypeParameter denotes a formal parameter slot, not a type; a real corpus type named
+// like a formal carries WholeDeclaration (or nothing) and is NOT mistaken -- enrolled as
+// corpus_type_shadow_named_like_formal_still_refuses in dag/test/claim/
+// applied_constructor_type_equals.dag, which the earlier spelling-keyed interim ACCEPTED.
+fn binding_leaves_carry_formal_parameter(n: Node) -> Bool {
+  if n.children |> count == 0 {
+    match n.declaration {
+      Absent => false
+      Present { value: d } => match d.field {
+        TypeParameter { name: _ } => true
+        _ => false
+      }
+    }
+  } else {
+    n.children |> any(c => binding_leaves_carry_formal_parameter(n: c))
+  }
+}
+
 fn binding_has_type_variable_leaf(n: Node) -> Bool {
   match n.inferred {
     Present { value: TypeVariable { id: _ } } => true
@@ -15197,6 +15224,87 @@ fn generic_head_matches_application(head: Node, applied: Node, source_indices: M
   } else { false }
 }
 
+fn field_str(d: DeclarationRef) -> String {
+  let fs = match d.field {
+    WholeDeclaration => "W"
+    NamedField { field_name: fname } => concat("F:", fname)
+    TypeParameter { name: pname } => concat("P:", pname)
+  }
+  concat(d.module_path, "#", d.decl_name, "#", fs)
+}
+
+fn gc16_dbg_dump(n: Node, source_indices: Map<String, NewlineIndex>) -> String {
+  let kind = match n.inferred {
+    Absent => "none"
+    Present { value: Resolved { node: _ } } => "resolved"
+    Present { value: TypeVariable { id: _ } } => "tv"
+    _ => "other"
+  }
+  let has_span = if n.ident_span != none { "S" } else { "-" }
+  let decl_str = match n.declaration {
+    Absent => "-"
+    Present { value: d } => field_str(d: d)
+  }
+  let kids_str = fold(n.children |> enumerate, init: "", f: (acc, pair) => concat(acc, ";", to_string(value: pair.first), ":", match pair.second.declaration {
+    Absent => "-"
+    Present { value: d } => field_str(d: d)
+  }, "/", authored_name_at(source_indices: source_indices, node: pair.second), "/", match pair.second.inferred {
+    Absent => "i-"
+    Present { value: Resolved { node: rt } } => concat("iR:", node_type_shape(n: rt, source_indices: source_indices))
+    Present { value: _ } => "iT"
+  }))
+  concat(
+    "p=", to_string(value: n.params |> count),
+    " c=", to_string(value: n.children |> count),
+    " inf=", kind,
+    " sp=", has_span,
+    " own=", if binding_is_generic_declaration_own_form(n: n) { "T" } else { "F" },
+    " tvleaf=", if binding_has_type_variable_leaf(n: n) { "T" } else { "F" },
+    " unin=", if unify_binding_is_uninformative(n: n) { "T" } else { "F" },
+    " decl=", decl_str,
+    " kids=", kids_str,
+    " an=", authored_name_at(source_indices: source_indices, node: n),
+    " shape=", node_type_shape(n: n, source_indices: source_indices)
+  )
+}
+
+fn gc16_flag_str(n: Node) -> String {
+  let err = if n.inferred != none { is_compiler_error(inferred: n.inferred.value) } else { false }
+  let tv = if n.inferred != none { is_type_variable(inferred: n.inferred.value) } else { false }
+  let opt = n.return_cardinality == CardOptional
+  let unit = is_unit_like(n: n)
+  concat(if err {"E"} else {"-"}, if tv {"V"} else {"-"}, if opt {"O"} else {"-"}, if unit {"u"} else {"-"})
+}
+
+fn gc16_unify_diag(left: Node, right: Node, source_indices: Map<String, NewlineIndex>, text: TextJudgment) -> String {
+  let left_u = generic_type_unify_unfold(n: left, text: text, depth: 0, source_indices: source_indices)
+  let right_u = generic_type_unify_unfold(n: right, text: text, depth: 0, source_indices: source_indices)
+  concat(
+    " UF=", gc16_flag_str(n: left_u), "||", gc16_flag_str(n: right_u),
+    " EQ=", if node_type_equals(left: left_u, right: right_u, source_indices: source_indices, text: text) { "T" } else { "F" },
+    " TXT=", if generic_types_are_the_text_pair(left: left_u, right: right_u, source_indices: source_indices) { "T" } else { "F" },
+    " APP1=", if generic_head_matches_application(head: left_u, applied: right_u, source_indices: source_indices) { "T" } else { "F" },
+    " APP2=", if generic_head_matches_application(head: right_u, applied: left_u, source_indices: source_indices) { "T" } else { "F" },
+    " U=", node_type_shape(n: left_u, source_indices: source_indices), "||", node_type_shape(n: right_u, source_indices: source_indices),
+    " CNU=", if type_name_compatible(a: authored_name_at(source_indices: source_indices, node: left_u), b: authored_name_at(source_indices: source_indices, node: right_u)) {"y"} else {"n"},
+    " UN=", authored_name_at(source_indices: source_indices, node: left_u), "||", authored_name_at(source_indices: source_indices, node: right_u),
+    " UC=", to_string(value: left_u.children |> count), "|", to_string(value: right_u.children |> count),
+    " CH=", fold(left_u.children |> enumerate, init: "", f: (acc, pair) => concat(acc, "|", match right_u.children |> skip(pair.first) |> first {
+      Present { value: rc } => if node_type_equals(left: pair.second, right: rc, source_indices: source_indices, text: text) { "y" } else { "n" }
+      Absent => "?"
+    })),
+    " CR=", fold(left_u.children |> enumerate, init: "", f: (acc, pair) => concat(acc, "|", if node_type_equals(left: pair.second, right: pair.second, source_indices: source_indices, text: text) { "y" } else { "n" })),
+    " UCH=", fold(left_u.children |> enumerate, init: "", f: (acc, pair) => concat(acc, ";", authored_name_at(source_indices: source_indices, node: pair.second), "/", match pair.second.declaration {
+      Absent => "-"
+      Present { value: d } => field_str(d: d)
+    }, "/", node_type_shape(n: pair.second, source_indices: source_indices))),
+    " UCR=", fold(right_u.children |> enumerate, init: "", f: (acc, pair) => concat(acc, ";", authored_name_at(source_indices: source_indices, node: pair.second), "/", match pair.second.declaration {
+      Absent => "-"
+      Present { value: d } => field_str(d: d)
+    }, "/", node_type_shape(n: pair.second, source_indices: source_indices)))
+  )
+}
+
 fn unify_generics(formal: Node, actual: Node, generic_names: List<String>, source_indices: Map<String, NewlineIndex>, acc: Map<String, Node>, text: TextJudgment, module_name: String) -> UnifyGenericsAcc {
   let bind_name = type_node_label(n: formal, source_indices: source_indices)
   let f_bare = formal.children |> count == 0 && formal.connective == NoConnective
@@ -15247,16 +15355,27 @@ fn unify_generics(formal: Node, actual: Node, generic_names: List<String>, sourc
           // placed earlier would also re-route bindings that unify today, whose behaviour is not
           // in that measurement.
           UnifyGenericsAcc { subst: map_insert(acc, bind_name, actual), diagnostics: [] }
+        } else if binding_leaves_carry_formal_parameter(n: bound) {
+          // SAME MEANING AS THE ARM ABOVE, one representation wider: the stored binding is a
+          // CALL-SITE spelling of the callee's own formal parameter names (p=0, own=F, tvleaf=F
+          // measured). It was never settled for this call -- REPLACE with the first settled
+          // actual, exactly as the params-based own-form arm does.
+          UnifyGenericsAcc { subst: map_insert(acc, bind_name, actual), diagnostics: [] }
         } else if binding_is_generic_declaration_own_form(n: actual) {
           // SYMMETRIC: an actual that is still a declaration's own form carries unsubstituted
           // slots, so it cannot falsify the stored binding -- the slot could denote exactly that
           // binding. KEEP the first binding, as binding_has_type_variable_leaf(n: actual) does.
           UnifyGenericsAcc { subst: acc, diagnostics: [] }
+        } else if binding_leaves_carry_formal_parameter(n: actual) {
+          // SYMMETRIC to the call-site bound arm above: a call-site actual still spelling the
+          // callee's own formal parameter names carries no settled type and cannot falsify the
+          // stored binding. KEEP.
+          UnifyGenericsAcc { subst: acc, diagnostics: [] }
         } else {
           UnifyGenericsAcc {
             subst: acc,
             diagnostics: [type_mismatch_error(
-              expected: concat("type variable ", bind_name, " bound as ", node_type_shape(n: bound, source_indices: source_indices), " (first argument)"),
+              expected: concat("type variable ", bind_name, " bound as ", node_type_shape(n: bound, source_indices: source_indices), " (first argument)", " GC16DBG[", gc16_dbg_dump(n: bound, source_indices: source_indices), "][", gc16_dbg_dump(n: actual, source_indices: source_indices), "]", gc16_unify_diag(left: bound, right: actual, source_indices: source_indices, text: text), gc16_eq_probe(left: bound, right: actual, source_indices: source_indices, text: text)),
               got: concat(node_type_shape(n: actual, source_indices: source_indices), " (later argument)"),
               span: actual.span,
               module_name: module_name
```
### (2) Fix-3 = a7febed7c42d -- src/v1/04_types.dag only (type_heads_agree + alias_pair_resolved + settled_head_identity_agrees):
```
commit a7febed7c42d5fbf9912a6b8a1730cbe3470955e
Author: gunbc-ci-auto-heal <gunbc-ci-auto-heal@users.noreply.github.com>
Date:   Thu Oct 22 23:16:13 2026 +0000

    PR-2 measurement: declaration identity decides head agreement before the carrier-alias table (vehicle only)
    
    type_heads_agree keyed on PRINTED head names first (type_head_is_declared_carrier_alias), so a
    transparent alias shadowing its own RHS refused a same-declaration join: at 70a9996f5930 the
    src/v2/std/text.dag:7:15 site carries decl=std.algebra#FreeMonoid#W on BOTH sides - one
    declaration - and still refused (witness: stamps equal, CN=n between names
    std.algebra.FreeMonoid vs String, all core arms false). The carrier table now runs only when the
    stamps disagree or are missing; stamp equality decides first. Unstamped sides keep the
    settled_head_identity_agrees env resolution (120331d63220) unchanged; TextNotAsked keeps the
    refusal verbatim. Monotone: only pairs the carrier table refused with EQUAL stamps change verdict.
    
    Targets the alias-shadow class (sites 1/13: src/v2/std/types.dag:8:30, src/v2/std/integer.dag:38:15;
    plus src/v2/std/text.dag:7:15) of gunbc.recurring_failure_mode
    stale_seed_hides_the_first_wins_conflict_population_the_fold_exposes. The kernel-String vs
    FreeMonoid<Char> text crossing (text_crossing_by_identity, gunbc.rung_drop
    text_boundary_identity_wall) runs ahead of this arm and is untouched: two types, never nicknames.
    
    Pre-registered PRED-F3 + amendment in scratch/PREDICTIONS_PIN.md (seed-gate 10 -> 9; nothing joins;
    kernel/wrapper/retained sites unchanged; claim verdicts hold). Mirrors regen-owned: CI heal converges.

@@ -889,17 +889,29 @@ fn type_head_is_declared_carrier_alias(name: String) -> Bool {
 // keeps the refusal - there is no name-spelling fallback path (gunbc.recurring_failure_mode
 // stale_seed_hides_the_first_wins_conflict_population_the_fold_exposes; PR-2 measurement, lands as #14056).
 fn type_heads_agree(left: Node, right: Node, left_name: String, right_name: String, source_indices: Map<String, NewlineIndex>, text: TextJudgment) -> Bool {
-  if type_head_is_declared_carrier_alias(name: left_name) || type_head_is_declared_carrier_alias(name: right_name) {
-    type_head_carrier(name: left_name) == type_head_carrier(name: right_name)
-  } else {
-    match left.declaration {
-      Present { value: ld } =>
-        match right.declaration {
-          Present { value: rd } => ld.module_path == rd.module_path && ld.decl_name == rd.decl_name
-          Absent => settled_head_identity_agrees(left: left, right: right, source_indices: source_indices, text: text)
-        }
-      Absent => settled_head_identity_agrees(left: left, right: right, source_indices: source_indices, text: text)
-    }
+  // DECLARATION IDENTITY DECIDES FIRST (measurement at 70a9996f5930: src/v2/std/text.dag:7:15 carried
+  // decl=std.algebra#FreeMonoid#W on BOTH sides - one declaration - and still refused, because the
+  // carrier-alias table below keys on PRINTED head names and a transparent alias (String = FreeMonoid<Char>)
+  // is not the table's own key. Same stamp, same declaration, must join (gunbc.rung_drop
+  // text_boundary_identity_wall; the alias-shadow class of gunbc.recurring_failure_mode
+  // stale_seed_hides_the_first_wins_conflict_population_the_fold_exposes). When the stamps disagree or are
+  // missing, the carrier table and the settled-env resolution below decide exactly as before.
+  match left.declaration {
+    Present { value: ld } =>
+      match right.declaration {
+        Present { value: rd } =>
+          if ld.module_path == rd.module_path && ld.decl_name == rd.decl_name { true }
+          else if type_head_is_declared_carrier_alias(name: left_name) || type_head_is_declared_carrier_alias(name: right_name) {
+            type_head_carrier(name: left_name) == type_head_carrier(name: right_name)
+          }
+          else { false }
+        Absent => settled_head_identity_agrees(left: left, right: right, source_indices: source_indices, text: text)
+      }
+    Absent =>
+      if type_head_is_declared_carrier_alias(name: left_name) || type_head_is_declared_carrier_alias(name: right_name) {
+        type_head_carrier(name: left_name) == type_head_carrier(name: right_name)
+      }
+      else { settled_head_identity_agrees(left: left, right: right, source_indices: source_indices, text: text) }
   }
 }
 
```
### (3) type_heads_agree helper introduced by vehicle d943fa1fc1c (grep it in rehearse HEAD src/v1/04_types.dag -- main has none; carry its definition too).
### (4) claims: dag/test/claim/cross_coproduct_generic_locate.dag (box_int + same_applied + list_int==freemonoid + shadow_formal FAIL-by-design members) from vehicle HEAD; gc16_* probes STRIPPED.

## ROW #14747 R2 (2026-10-24)
- Side-chat REQUEST_CHANGES at 6e736ad5 accepted (A2/A3/A4 keys prose-derived). Fixed at head 7894bc170a81: population receipt = TWO LEVELS (mechanical CLASS KEYS for A1/(C)/(D); A2/A3/A4 headers say "TEST, NOT PRODUCER-DERIVED" naming the needed extra knowledge: A2 both-sides decl identities (no producer emits), A3 callee formal header, A4 corpus alias table). (A) intro + RED sentence + (B) A2-assignment carry same distinction. Body matches + upgrade path (post-#14750 stamps -> mechanical).
- Push lesson: killed push leaves stale index.lock in common dir (/home/briansrls/gunbc/.git/worktrees/<wt>/index.lock); pushes are SLOW (~2-3min) -> allow 300s timeouts.
- PENDING at compaction: parent reply msg (text in /tmp/gc16dbg — resend when dashboard back: summary of R2 fix, option 2 rationale); row-gate watcher for machine-tagged APPROVE; #14750 watcher (both re-armed 03:55Z, logs /tmp/gc16dbg/watch_row_gate.log, watch14750c.log).

## ROW #14747 MERGE-READY (2026-10-24 ~05:00Z)
- Gate READY: approvals ['claude'] (review 85244 APPROVE at 7894bc170a81), RC 0, checks passing, CLEAN/MERGEABLE. Parent told (msg_b252dbc5); merge ask routed (suggested jointly w/ #14056 post-#14750, or earlier per parent -- dag-only independent).
- Non-blocking style remark (receipt length) deliberately not acted on; prior review 85203 judged receipts placement correct.
- STANDBY: #14750 watcher (watch14750c.sh; budget ends ~10:55Z; re-arm: rm /tmp/gc16dbg/.w14750b_head + rotate log).

## #14750 MONITOR 2026-10-24
- Pushes: b2964e2e -> 26753141 -> 2df86caa4d0 (current). At 2df86caa: v2-bootstrap PASS, heal FAIL run 40077044120 converge-step: E0425 'element' + E0282 at freshly-emitted extdeps_version_semver.rs:637/641, in semver_identifier_list_parse (fold over segments building Vec<Rc<SemVerIdentifier>>; emitted 'element' = v2 List formal leaked where binding should substitute -- A2 fold_list-family signature). Reported to parent msg_6bf9eaa6. swift-lark-171 owns; Charter-1 monitoring only.
- Row #14747 merge-ready, awaiting parent/operator routing (do not merge myself).
- Watcher armed (budget ~15:55Z; HEAD baseline 2df86caa4d0).
