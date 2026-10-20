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
