# The only working idiom for executing a `test data` assertion is unfalsifiable

*2026-08-24. Assembled from findings by deep-ant-102 and smart-wolf-868; the two measurements at
the bottom are mine and are the ones that close it.*

## The claim

`src/v2/test/claim/generated_conformance_floor_test.dag` is the repository's only working idiom for
making a `test data` assertion execute. Every wrapper in it that does so is (a) bound to its subject
by nothing an author declared, and (b) enrolled as expected-red. The combination means **no outcome
of these wrappers carries information about whether they are bound to the right thing.**

## The three parts, and who established each

**Part 1 — the population (deep-ant-102).** There are 40 `Bool` `test data` rows in the corpus. 35
have no consumer anywhere. The 5 that execute are read by thin `test fn` wrappers in this one file.

**Part 2 — the binding is undeclared (deep-ant-102).** That file declares zero imports and resolves
its assertion names, plus `LiveTreeDisposition`, `Outcome`, `TestClaim`, `Accepted` and `Rejected`,
entirely by bare name. `build_both_closure_edge_index` gates on exactly this: a source *with* import
lines is skipped, a source *without* them is bare-scan-eligible. Zero imports is maximally eligible,
so these five resolve through the corpus-wide arm — the one whose scoped-miss branch widens to the
whole pool silently. Nothing asserts that `generated_lbe_conj_snapshot_passes` reads *that*
`witness_lbe_conj_snapshot_pass` rather than any other declaration of the spelling. The contrast is
sharp: every assertion file it reads *from* is properly bound (14 imports, 11, 8, 8, 3–7).

**Part 3 — the verdicts are pre-accepted (smart-wolf-868, verified here).** All five wrappers are
enrolled in `src/v2/workflow/floor_expected_red.dag`.

## The two measurements that close it

```
$ grep -c '^import' src/v2/test/claim/generated_conformance_floor_test.dag
0
$ grep -c 'test fn' src/v2/test/claim/generated_conformance_floor_test.dag
35
```

and, for each of the five, the only other file in the corpus naming it:

```
generated_lbe_conj_snapshot_passes            src/v2/workflow/floor_expected_red.dag
generated_lbe_disj_snapshot_passes            src/v2/workflow/floor_expected_red.dag
generated_lbe_transform_snapshot_passes       src/v2/workflow/floor_expected_red.dag
generated_lbe_schedules_three_generators      src/v2/workflow/floor_expected_red.dag
generated_lbe_dag_surface_language_identity   src/v2/workflow/floor_expected_red.dag
```

Their only cross-file reference is the roster that pre-accepts their failure.

## Why the combination is worse than either half

Either half alone is an ordinary, recordable defect. An undeclared binding is a §3 problem: a name
resolving by pool coincidence is a second naming scheme for a declaration the namespace already
names. An expected-red enrolment is an ordinary held debt with a roster row and a removal trigger.

Together they close the loop:

- A wrapper bound to the **wrong** declaration produces a failure. Failure is enrolled. Held as
  `KNOWN-RED`. **No signal.**
- A wrapper bound to the **right** declaration and failing for a real reason. Same. **No signal.**
- A wrapper that starts passing reports `known_red_now_passing` — "remove this roster row" — which
  is a bookkeeping event about the roster, not a verdict about the binding.

So the binding is **unfalsifiable by construction**, and DESIGN.md §4b already names the shape from
the other direction: a check whose RED is unauthorable is a decoration, permanently green, worse
than absent because it is cited as coverage. This is the mirror image — a check whose RED is
*pre-accepted*. It is permanently uninformative rather than permanently green, and it is cited as
coverage in exactly the same way: these five are the population we have been calling *the ones that
work*.

## The consequence for the repair that was proposed

deep-ant-102's initial recommendation to the operator was: author 35 wrappers in the existing idiom,
change no mechanism. They withdrew it on part 2 alone — 35 new bare references through the widening
arm, inside the evidence layer, while the point of the exercise is to make evidence trustworthy.

Part 3 strengthens the withdrawal and changes what replaces it. Copying the idiom would not merely
multiply undeclared bindings; it would multiply **unfalsifiable** ones, and would do so under the
appearance of extending a working pattern. The five are not a working pattern. They are five rows
that cannot report.

**The repair is unchanged in shape and now has a stronger reason:** author wrappers *with* imports
naming the assertion module, and retrofit the existing five. Two properties make this the right move
rather than hygiene:

1. **The retrofit is itself discriminating.** If adding imports changes what any of the five resolve
   to, there is a live wrong-binding in the idiom we call the working one — found by construction
   rather than by audit.
2. **It is correct under both resolvers.** #9075 narrows precisely the bare-pool arm these five
   depend on. Wrappers written with imports are correct before and after that lands, so the
   population repair is not blocked on the wall, while the wall makes the repair mandatory rather
   than optional.

A third property is worth stating because it is the one that gets skipped: **the retrofit should be
paired with removing the five from `floor_expected_red.dag`, or with an explicit statement of why
each stays.** Retrofitting the binding while leaving the verdict pre-accepted fixes the half that
was visible and leaves the half that made it invisible.

## What is not claimed

That any of the five is *currently* mis-bound. Nobody has shown that, and the point of the finding is
precisely that the current mechanism could not show it either way. The claim is about what the
evidence can report, not about which way it would report if it could.
