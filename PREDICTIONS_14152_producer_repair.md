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
