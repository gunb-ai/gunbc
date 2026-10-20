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
