# Pre-registered predictions — PR-1 (producer repair) at its own base

Written and committed BEFORE the two measurements below have been read. If a prediction fails, the
failure is the finding and is reported as such; the text is not edited after the fact.

## Context that makes these predictions non-obvious

- The tests import `unify_generics` and friends from the **checked-in mirror**, and `build.rs` does
  NOT regenerate from `.dag`. So a `.dag`-only tree is invisible to cargo: on such a tree the new row
  cannot mint anything at all and would fail for a reason that is not the one the row tests. Both
  trees below therefore carry emitter-regenerated mirrors from fresh clones.
- PR-1's base is `origin/main` (891198514b8 at the time of branching). The Acc is absent by
  construction: `UnifyGenericsAcc` changed `unify_generics` from five arguments to six, so the
  fold-level rows cannot compile here. `grep -c UnifyGenericsAcc` on both trees must be 0.
- The old mutant ref `probe/mut-namekeyed` belongs to the Acc era. Verdicts measured there say
  nothing about PR-1's base, which is why `probe/pr1-mut-namekeyed` was cut from main.

## AMENDMENT A1 -- written after M1/M2 were written, before either was read

My own M1/M2 wording is wrong and is corrected here rather than quietly satisfied later: I wrote
"the row's prints show 24 positions, 4 distinct ids, 24 site-tagged, 0 bare". The row had NO
println! at all -- measurements appeared only in its FAILURE messages. A passing run would therefore
have printed nothing, making a passing run indistinguishable from a passing-but-vacuous one, which is
the exact defect the row is written to close. So the row now prints a MINT-INVENTORY line before
deciding (commit on probe/pr1-producer-repair), and M1/M2 below are read as "the row passes AND
prints the inventory" / "the row fails AND prints the inventory", not as a claim that prints existed.
Nothing else about M1-M5 changes, and in particular M2's requirement that the mutant fail on the
SHARING rather than on absence is unchanged.


**M1 — the RED passes on the repair at PR-1's base.**
Tree: `refs/heads/probe/pr1-producer-repair` + regenerated mirror.
Row: `generic_call_variable_freshness_test::minted_variables_are_fresh_per_call_site`.
Predict: `test result: ok. 1 passed`, exit 0, and the row's own prints show 24 positions, 4 distinct
ids, 24 site-tagged, 0 bare ids.
Predicted cost: ~60 s release (the fold rows in the Acc era cost 60–64 s; this row's fixture is
`fn collide()` plus the std closure, the same shape).
Falsified if: it passes with 0 distinct ids (that would mean the walk found nothing and the row is
vacuous — the row is written to fail in exactly that case, so a pass with an empty inventory is a
bug in the row, not a win), or if it fails (then the repair does not reach the call arm on main's
base, which would mean my `.dag` patch depends on Acc-era text — REVERT and re-anchor, do not widen).

**M2 — the RED fails on the name-keyed mutant at the same base.**
Tree: `refs/heads/probe/pr1-mut-namekeyed` (main + `id: g` at the call site + the same row) +
regenerated mirror.
Predict: `test result: FAILED`, exit code 101, and the row's prints show 2 distinct ids (`K`, `V`)
and 0 site-tagged — i.e. it fails on the SHARING, which is the defect, not on absence.
Predicted cost: ~60 s.
Falsified if: it passes. That would mean the row cannot see the mutation through the regenerated
mirror, and the honest report is "the RED does not discriminate at PR-1's base", not a re-wording.
Predicted specifically: the assertion that fails is the distinct-id count (4 expected, 2 found) or
the site-tagged count — NOT a compile error and NOT "no TypeVariable found".

**M3 — the mutant's mirror really changed.**
Predict: `MIRROR_CHANGED yes`, and the mutant's emitted mirror contains a
`type_variable_node(id: g)` inside the call arm while the repair's contains
`type_variable_node(id: concat(g, "@", span.file, …`. If both mirrors print the same markers, the
mutation never reached the mirror and M2 is void, not red.

**M4 — nothing else in the crate moves.**
Predict: `cargo test -p v1-compiler-tests` gains exactly one test versus main, and no pre-existing
row changes verdict. If another row moves, the repair is not the only difference and the tree is
wrong, so the run is void rather than a partial pass.

**M5 — the repair is inert for non-generic calls (parent msg_1249d701's requirement).**
Already measured on the Acc-era tree via the two-closure comparator: `set(post) - set(pre) = {}`
per closure, by identity. Predicted to hold here too — for a call with no declared type parameters
the overlay map is empty, so `map_merge` is a no-op and no node is minted. Stated as a prediction
rather than a claim because PR-1's own comparator run has not been taken at this base.

## What is deliberately NOT predicted

No prediction about the 43/45 corpus conflicts, the Acc, or emission: none of that exists on this
branch. A gate run here is expected to reproduce main's counts exactly; if it does not, the tree is
contaminated and the run is void.
