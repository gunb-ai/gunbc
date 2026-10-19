# Pre-registered predictions -- producer repair + collision RED (#14152)

Fixed and committed BEFORE any measured run reads them. The run is
`repair_collision_14152.sh <SHA-of-test-commit>` (fresh clone at a printed SHA on the
runner; the `.dag` at that SHA is pre-repair because the repair is applied by the patcher
inside the run, never committed in advance).

Base facts already measured (not predictions): pre-repair two-closure gate at the WIP head
`8d8d8aac0e1` is self-host 77 hard diagnostics / 17 identities, v2-native-cli 67 / 18, main
0 / 0 with the emitted marker. Map family = 46 rows: 39 from producer 1 (one node,
`dag/std/algebra.dag:73644-73647`) and 7 from producer 2 (`bare_map_node`, `<synthetic>:0-0`).

## S1 RED -- unpatched `.dag`, rows present
| row | predicted |
|---|---|
| `collision_two_zero_arg_generic_calls_both_type` | **FAIL** -- both `empty_map()` calls carry the callee's own `Map(K, V)` |
| `collision_pair_still_refuses_when_instantiated_twice` | PASS (fold-level twin, independent of the repair) |
| the 6 existing `fold_*` rows | PASS (the repair touches nothing they test) |

If the collision row PASSES at S1 the fixture is not discriminating and I must report that,
not call the repair verified.

## S2 WITHDRAWN mutation (seed dropped, code shape otherwise identical)
Every row's verdict must be **identical to S1**. A row that passes here but failed at S1 is
testing the surrounding code, not the seed.

## S3 NAME-KEYED mutation (`type_variable_node(id: g)`)
`collision_two_zero_arg_generic_calls_both_type` must **FAIL**: the two calls share variable
`K`, so one call's settled instance meets the other's binding. That is the parent's
discriminator (msg_78d4eb21) for call-site-keyed vs name-keyed.
**If it PASSES, the collision is not observable in this harness** -- I report that as a
limit of the fixture and do not claim the freshness requirement is met by test evidence.

## S4 REPAIR (span-keyed fresh variables)
All 8 rows PASS. No row may go MISSING.

## S5 regeneration
The mirrors regenerate to a fixed point and the seed MUST move (pre-repair infer mirror
`b471b493356ae5e8`); a moved seed with unchanged rows is what exposed my first, inert patch.
`fold=` presence must stay 41.

## S6 SHIPPED -- rows against the regenerated mirrors (what actually ships)
All 8 rows PASS.

## S7 suite
`32 passed / 0 failed` for this file's crate (30 today + the 2 new rows), nothing else moved.

## Two-closure gate after the repair (separate run, same method, fresh clones)
- self-host: **77 -> 37..44** hard diagnostics.
- v2-native-cli: **67 -> 44..48**. The floor of that band is producer 2's 7 rows surviving.
- **Producer 2 prediction, fixed now:** the 7 `bare_map_node` rows **remain**. They are minted
  map literals that never pass the call arm at `04_infer.dag:9272`, so a call-arm repair
  cannot reach them. They are a named residual with its own boundary, not a reason to revert
  producer 1's fix.
- main re-measured in the same run: **0 / 0 with the emitted marker**, or the run is void.
- **HARD TEST, per closure:** `set(post identities) - set(pre identities) == {}`. Any NEW
  identity is a FAIL to be root-caused, not a finding to be explained. Identities that
  disappear must all be Map-family rows; non-Map-headed identities must be unchanged.

## If the bands miss
Under-shooting (fewer than 37) means the repair is clearing rows I attributed to producer 2
or to the outcome-C family, which would contradict the probe split -- investigate before
believing it. Overshooting (any new identity) is a regression, full stop.

---

## AMENDMENT (same session, before any measured result was read)

Two corrections made after the first dispatch aborted on a compile error, and before any
stage result existed. Recorded rather than silently edited, because pre-registration is only
worth anything if the edits are visible.

1. **The fixture changed shape.** The first version used two *separate top-level functions*.
   That cannot test a cross-binding: each call is compared only against its own formal, so the
   two call types never meet. Both calls are now in **one body**, the first constrained to
   `Map(Int, Int)` before the second is introduced against `Map(String, Int)`. The name-keyed
   mutation now has a place to fail.
2. **A ninth row was added**, `fold_accepts_a_binding_whose_arguments_are_all_fresh_call_variables`:
   bound = `Map(fresh K@site, fresh V@site)`, actual = `Map(String, Int)`. Predicted **PASS** at
   every stage: it is the exact node the repair emits, and `binding_has_type_variable_leaf`
   recurses into children, so an all-variable head is unsettled by the arm that already ships.
   Its value is that it locks the *accept* duty to the repaired shape, per the parent's
   msg_ad1a90c2 (do not keep a lock that asserts obsolete behaviour).
3. **What is measured.** `src/v1/stage0/build.rs` does not regenerate from `.dag`, and the tests
   import `unify_generics` from the **checked-in mirror**. An earlier script patched only
   `04_infer.dag` and would have measured the same unpatched binary at every stage. Stage B1
   now regenerates the mirror from the patched `.dag` (regeneration is the only sanctioned way
   a mirror changes), prints the emitted text of the seeded region, and FAILS LOUDLY with
   `REGEN_NOOP FATAL` if the regenerated mirror is byte-identical to the pre-repair one.
4. S1 predictions are unchanged for the 8 original rows. Row `collision_two_zero_arg_generic_calls_both_type`
   is predicted **FAIL at RED** and **PASS at repair**; if it passes at RED the fixture is not
   discriminating and I say so instead of calling the repair verified.

## AMENDMENT 3 (10:45Z, before any result is read): three dispatches invalidated by my own harness, not by the tree

Predictions above are UNCHANGED and still unrevised. What follows is what the runs so far cost,
so that no number in them is mistaken for a measurement, and so a rerun is not blind.

1. ROWS, first attempt (label `repair`, 10:11-10:41Z): INVALID, no verdict. It ran the whole
   crate filter (`cargo test --lib cross_coproduct`) under one 1500s bound, but
   `RUST_TEST_THREADS=1` (.cargo/config.toml:17) serialises the suite and EVERY
   `compile_to_resolved` row recompiles the std corpus (~2 min/row, 35 rows). Exit 124 = my own
   bound; 8 of 9 rows reported MISSING_FATAL which under that exit code means ONLY "the suite
   never reached them" -- the first two rows did get verdicts (collision_pair PASS, and the
   log shows the third row `collision_two_zero_arg_generic_calls_both_type` started and was
   still compiling when the bound fired). PASS on collision_pair is real; nothing else is.
   Fixed by one `--exact` invocation per row on its own ROW_TIMEOUT plus positional row
   selection. NOT a fixture-hang diagnosis: the collision row was still in its first compile,
   which is consistent with it being merely slow. It gets its own 1200s dispatch.
2. GATE, two attempts: INVALID, the script never ran. First dispatch passed `LOGS=... HEAD_REF=...`
   as separate ctrl-build args, which are word-split, so `bash -lc` took `LOGS=/tmp/x` as its
   COMMAND and the rest as $0 -- exit 0-ish in 13 seconds with zero script output. Second
   attempt omitted them and the script's own `LOGS=${LOGS:?LOGS required}` killed it in 6 seconds.
   Both logs look like "the gate printed nothing"; the difference between an unmeasured gate and
   a gate that measured nothing is only visible in the log, which is why the wrapper now carries
   them inside the quoted -c string and says so in its header.
3. A THIRD failure mode was avoided only by accident: the gate scripts are tracked at the repo
   ROOT on the instruments line. A dispatch naming `instruments/gate_two_closures_14152.sh`
   would die on "No such file or directory".

Consequence for reading results: a fast exit with no `T=` stamps means the harness, not the tree.
Every row verdict below comes only from a run that printed `ROW <label> <row> PASS|FAIL rc=`.

## AMENDMENT 4 (11:30Z): the collision row is NOT a RED-before, and I must say so

Measured, both by `test result:` summary line, both on fresh clones at printed SHAs:

| head | mirror REPAIR_MARKS | collision_two_zero_arg_generic_calls_both_type |
|---|---|---|
| 714346740aca (pre-repair, Acc only) | 0 | `1 passed; 0 failed` in 63.24s |
| 81824abfa3c (shipped repair)         | 1 | `1 passed; 0 failed` in 63.00s |

`FAIL@RED` was pre-registered (AMENDMENT: "collision FAIL@RED PASS@repair"). It is FALSIFIED. The
row accepts the program at both heads, so it does not demonstrate the repair and must not be
described as doing so. Why it accepts pre-repair: both calls get the callee's own declaration form
`Map<K, V>` (K, V the parameters), the two call types are shape-equal, `m.get(1)` yields Int
against a formal Int, and the fold's unsettled arms replace rather than refuse -- so nothing fires.
The row is therefore an ACCEPT CONTROL ONLY: it proves the repair does not break a program that
must type. Its last chance to be discriminating is the name-keyed MUTANT, where the two calls mint
the SAME variable ids; predicted FAIL there (spurious diagnostic from cross-binding). If the mutant
also passes, the row is vacuous and I will say the fixture cannot expose sharing, keeping the
freshness case on what does support it -- the escape asymmetry (the seed at 04_infer.dag:11775 is
consumed in place; the substituted type at :9272 escapes into the body environment) -- not on
this row.

Two corrections to my own earlier reporting in this session, also recorded before results:
- I asserted "pre-repair PASSED in 147s" from a run that had actually died on `reference is not a
  valid name` (a full SHA typed from memory). There was no such measurement until 11:19Z, above.
- `MISSING_FATAL rc=0` in these logs meant "the --nocapture output split the result line", not
  "row missing"; fixed by deciding from the summary line (verified over all three summary shapes).

## AMENDMENT 5 (11:52Z, fixed BEFORE the mutant is run): prediction for the name-keyed mutant

Mutant built by `patch_producer_instantiate.py --mutate name-keyed` from the unpatched base
`8d8d8aac0e1`, regenerated to a fixed point by the stage0 generator (mirror seed
`3e62363f58c07132`, round 1 installed=1 / round 2 installed=0), grafted onto the candidate as
`786e30feb1ffb74708e1cb439a38c9f1b869d239` on `refs/heads/probe/mut-namekeyed` (probe ref: never
merged, never reviewed). Verified in the tree before running: `uninstantiated_call_generics`,
`fresh_call_variables` and `map_merge(call_subst, ...)` all present (3 sites in the .dag, 4 in the
mirror) and the shipped `id: concat(g, "@", span.file...` expression GONE -- so the mutant differs
from the shipped repair in exactly one expression, `map_insert(acc, g, type_variable_node(id: g))`
at 04_infer.dag:9275. (The other `id: g` at 11741 is the pre-existing
`unbound_owner_param_variables` fold, not my site.)

PREDICTION, per row, at the mutant (all rows at once, release profile):
1. `collision_two_zero_arg_generic_calls_both_type` -- **FAIL**. Both calls mint `K`/`V` by
   declared parameter name, so they share variable identity; the first call's constraint
   (Map(Int,Int) via `takes_int_map`) binds K and V, and the second call's settled actual
   Map(String,Int) then conflicts against that binding instead of being checked against a fresh
   variable. Either a hard diagnostic appears where the program is correct, or a wrong binding is
   accepted; the row asserts zero hard diagnostics, so either way it fails.
2. `collision_pair_still_refuses_when_instantiated_twice` -- **PASS** (a fold-level refusal that
   holds regardless; it is the twin, not the discriminator).
3. `fold_accepts_a_binding_whose_arguments_are_all_fresh_call_variables` -- **PASS**, unchanged: it
   builds node shapes directly and never enters the call arm.
4. The five remaining fold rows -- **PASS**, unchanged, for the same reason.

So exactly ONE row is expected to change under the mutation, and it is the collision row. If the
collision row PASSES at the mutant, then this harness cannot expose call-site sharing and I will
report the collision row as a fixture limit -- NOT as evidence for the freshness requirement, which
would then rest only on the escape asymmetry (seed consumed in place at :11775 vs substituted type
escaping at :9272). If MORE than one row fails, the mutant is not isolated and the comparison is
void; I will say so rather than keep the favourable row.

## AMENDMENT 6 (12:20Z): the name-keyed mutant passes every row -- the fixture cannot expose sharing

All nine rows at the mutant `786e30feb1ffb74708e1cb439a38c9f1b869d239` (`probe/mut-namekeyed`,
`REPAIR_MARKS 2`, `FOLD_REFS 41`, mirror `3e62363f58c07132`), one `--exact` invocation each, release
profile, verdicts from each run's own `test result:` summary line:

| row | verdict |
|---|---|
| collision_two_zero_arg_generic_calls_both_type | PASS 65.19s |
| the eight fold rows | PASS, 0.00s each (fold-level; no std compile) |

AMENDMENT 5 predicted the collision row would **FAIL** at the mutant. It PASSED. So the mutation is
undetected by every row I have, and the honest conclusion is the one AMENDMENT 5 pre-declared:
**this harness cannot expose call-site sharing.** The collision row is therefore an accept control
only, at all three trees (pre-repair, shipped repair, mutant) -- it proves the repair does not break
a program that must type, and nothing more.

Why, mechanically -- THIS PARAGRAPH IS A READING, NOT A MEASUREMENT. The row asserts zero HARD
diagnostics, and the unsettled arms replace a binding rather than fixing it, so a shared variable
would not necessarily surface as a diagnostic. Two candidate explanations fit the PASS and I have not
separated them: (a) the two calls' variable identities are shared and the arm replaces rather than
conflicts; or (b) the row never brings the two call types together at all -- each call is checked
against its own formal, and the second actual never meets the first binding -- in which case the
fixture could not expose sharing even with a compiler that conflicted. Distinguishing them needs the
two calls' variable ids printed, or a fold call built with deliberately shared ids. That is
instrumentation, so I am offering it to my manager as a decision rather than building it unasked. What
is measured is only: the mutant differs from the shipped repair in exactly one expression, and no row
changes verdict between them.

Consequences I will carry into the PR body and to stern-bat-401:
1. The freshness requirement from msg_78d4eb21 is SATISFIED BY CONSTRUCTION (per-call-site ids
   `g@file:start-end`), and its justification is the reading, not this row: the seed at
   04_infer.dag:11775 is consumed in place, whereas the substituted type at :9272 escapes into the
   body environment, which is what makes name-keying unsafe. I am not going to describe the collision
   row as evidence for it.
2. Making it discriminating would need the two calls' variable identities printed or a fold call with
   deliberately shared ids -- instrumentation, not a fixture. I will offer that to my manager as a
   decision rather than build it unasked.
3. No row regressed at the mutant, so nothing about the shipped repair is contradicted; but the
   mutant is also not refuted by the suite, and I will say that instead of implying it was.

## AMENDMENT 7 (12:40Z, parent msg_e210f391 ruling (b)): probe predictions, fixed before the probe runs

Parent ruled: freshness needs a RED, not a reading; use the probe once to separate the two readings
of AMENDMENT 6, then land a PERMANENT fold-level control that calls `unify_generics` directly with two
`Map(TypeVariable)` actuals -- shared id `K` bound to two different concrete Maps (must refuse, or must
not cross-bind: state which is correct) and distinct per-site ids (must accept) -- such that the
name-keyed mutant fails it. If the fold itself cannot distinguish them, say so and record it as the
limit. Also: drop the old 76 table, keep only the artifact-backed two-closure table.

PROBE (new rows, capture mode: they PRINT and do not assert, because asserting a value I have not
measured would be the guess I am here to avoid). Both compile COLLISION_SOURCE (two `empty_map()` calls
in one body) and walk the resolved tree for the type-variable ids the compiler minted.

P1 -- ids at the SHIPPED repair (`81824abfa3c`): each call's `K`/`V` carries a `@file:start-end` suffix,
so the two calls' ids DIFFER (four distinct ids, two per call, the two spans distinct).
P2 -- ids at the MUTANT (`786e30feb1...`, `probe/mut-namekeyed`): the ids are the bare declared names,
so the two calls SHARE them (`K` and `V` appear twice each, i.e. two distinct ids for four positions).
If P2 instead shows distinct ids, the mutant did not take effect in the compiled tree and the whole
AMENDMENT 6 conclusion ("the fixture cannot expose sharing") is VOID, not a limit -- because then no
compiler ever produced the sharing the rows were supposed to fail on. That distinction matters and I
will check the mirror seed on the mutant run before interpreting it.

FOLD CAPTURE (same dispatch, `unify_generics` called directly, hand-built nodes, 0.00s rows):
F1 shared-id shape -- `T` bound to `Map(K, V)`, then bound to `Map(Int, Int)`, then `Map(String, Int)`,
   where K and V are the BARE declared names.
F2 distinct-id shape -- identical except the leaves are `K@src/v2/std/text.dag:1-2` / `V@...`.
My reading predicts BOTH accept, i.e. the fold compares head and shape and does not inspect leaf
identity, so the fold CANNOT distinguish shared from distinct. If so, the permanent rows cannot test
"shared vs distinct" through `unify_generics` at all, and I will record that as the limit exactly as
parent instructed rather than shipping a row that asserts a distinction the fold does not make. If
instead one of F1/F2 refuses, the fold does have a leaf-identity path, and the permanent RED goes there.
