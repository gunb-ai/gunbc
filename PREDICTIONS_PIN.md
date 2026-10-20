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
