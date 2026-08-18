# The floor's per-row budget charges shared entry cost to whichever row touches it first

Finding from the 2026-08-18 runaway census
([measurement](measurements/floor-slow-rows-2026-08-18.md)). It changes what the
six runaways are, and therefore what may be done about them.

## The claim

The budget's unit of attribution is the row. The unit at which a large part of
the cost is INCURRED is the entry: the first row to reach an expensive shared
computation pays all of it, and every later row reaching the same computation
pays nothing. The ceiling then refuses that first row for a cost that is not
its own.

## The receipt — two rows, same computation, 1ms apart

`dag/test/claim/g2_data_reference_under_selection_witness_test.dag` declares a
deliberate pair. Both rows call the same `specimen_projection()`; both are
enrolled; both EXECUTED in run 32172125816 and both are reported, because both
are expected-red and failures are reported regardless of duration:

```
18:59:48.653  specimen_classifies_as_runtime_read        FAILED in 12 seconds
18:59:48.688  specimen_does_not_classify_as_local_read   FAILED in 31ms
```

12197ms and 31ms, 393x apart, for the same call, 35ms apart on the clock. The
second row is not cheaper; it is free because the first one already paid.

This is the discriminating pair the finding needs, and it is not the only
instance. In `test.claim.extdeps_scope_placement_gate_loudness_witness` exactly
one of seven rows is over the warn line at 71244ms; the siblings that call the
same `scope_placement_membership_verdict` on the same fixture are all under
100ms. The same row is the run's worst single memory event, `grew rss by
1.00GB`, which is the shape of a corpus-scale structure built once rather than
of work repeated per row.

Stated precisely, because the two are not the same claim: the g2 pair shows
directly that a SECOND call to the same computation is free, which is what
makes the first row's charge a shared cost rather than its own. The
extdeps_scope entry is consistent with that and does not independently
establish it — the floor reports duration only for rows that fail or cross the
warn line, so the six cheap siblings are known to be under 100ms but their
execution order relative to the runaway is not recorded, and this document does
not assume it.

## Why this matters for the lane

The lane's framing is to decompose over-budget rows into small complete
segments. For rows in this class, decomposition cannot work, and the repository
already says so: `gunbc.witness_row_cost`
`witness_decomposition_does_not_reduce_entry_cost_note` (operator, 2026-08-05)
rules that splitting a witness does not divide the entry's cost, it amortizes
it across more rows, so reading the resulting per-row drop as a saving is an
artifact of the split. This finding is that note's prediction observed
directly: splitting one of these rows would move the charge to whichever
fragment runs first and change no real cost.

So for these rows the honest remedies are two, and neither is a split:

1. Reduce what the shared computation reaches for — a real cost reduction,
   which moves the entry total and not just the attribution.
2. Attribute shared cost to the entry rather than to its first row, so the
   ceiling stops refusing a row for cost it did not incur. This is a change to
   the floor's measurement, not to any witness.

## What is NOT available, and why it is worth stating

The third remedy the refusal text itself offers — "move it to a designated
`long/` home that CI does not run" — is not available today, and taking it would
be a coverage deletion rather than a fix. There is no cadence executing the long
home: the repository carries exactly two workflows, `witnesses.yml` and
`fleet-converge.yml`, and the falsifier cadence was deleted in the floor cut. The
2026-08-04 operator ruling (DESIGN, "witness cost derives from purpose") names
this exact move as its originating specimen — a witness relocated under `long/`,
removed from discovery, given no cadence row, executed by nothing, with
admission still green. Relocation would reproduce it.

This bears on two files whose own notes still claim a path-based exclusion that
no longer exists. `required_floor.dag` excludes the long home by AUTHORED MODULE
NAME (`long_home_prefixes`: `test.claim.long.`, `v2.test.long.`,
`v2.test.claim.long.`), deliberately moved off path because a directory deciding
admission was the defect. `test.claim.rust_test_fixtures_import_closure_witness`
declares no long prefix and no `ReadsLiveTree`, so it is fully enrolled and pays
50.4s per run while its note says it does not run;
`v2.test.manual.grounding_lens_whole_tree` is enrolled the same way, its
`v2.test.manual.` prefix not being an excluded one. The notes were corrected in
gunbc#8430; the enrollment is real and is counted here.

## Scope of this document

This states a measured finding and its consequences. It changes no mechanism.
The attribution repair (remedy 2) belongs to whoever owns the floor's
measurement, and is not taken here.

## Follow-up: the 71-second row is not expensive in isolation

Measured after the above, because the row is 13% of all slow time on its own and
the question "is this one a first-toucher, or genuinely expensive?" decides
whether paring it is even aimed at anything.

Same head, same two source roots CI uses (`--source-root dag --source-root
src/v2`), `claim_batch` on that entry alone, reading the per-row `[witness]`
CPU line — the same quantity the floor's budget compares:

```
row run alone       red_seed_runner_failure_detail_projects_located_receipt   cpu=70ms
cheap sibling alone red_membership_fixture_names_change_and_path              cpu=0ms
both, in order      red_membership 0ms, then red_seed_runner 85ms
```

Against CI's `cost is exactly 71060ms against 1552ms` for that same row. Both
numbers are per-row CPU for the same row at the same head, so they are
comparable: **70-85ms isolated, 71060ms on the floor — about a thousandfold.**

The difference between the two runs is the evaluation context. The sandbox
resolves this entry's own closure (67 modules, 1425 resolved items); the floor
evaluates every row against one whole-corpus prepared subject (2376 modules).
So the row's cost is CONTRIBUTED BY THE CONTEXT, not by what the witness
computes — it computes almost nothing, and the witness that reaches for the
same `scope_placement_membership_verdict` beside it costs 0ms.

The consequence for the lane is the useful part: remedy 1, "reduce what the
shared computation reaches for", has almost nothing to bite on here. Paring this
witness would be aiming at 70ms of work in order to move a 71-second bill that
is not made of it.

A candidate mechanism, named as a candidate and not asserted: this row is the
one that reaches its companion by NAME through the seed runner
(`run_claim_failure_receipt` → `run_in_context` → `ctx.lookup_fn`), and the
prepared-floor scope is a flat whole-corpus namespace in which 378 helper names
are known to collide and bind last-write-wins (gunbc#8437). A name-keyed lookup
is the one thing this row does that its cheap siblings do not. That is a lead
for whoever holds the prepared-floor scope, not a finding: it is untested here,
and the measurement above stands without it.

One caveat on the sandbox, recorded so the number is not reused wrongly: single-
entry `claim_batch` is fixed-cost dominated — its whole-tree graph-facts phase
alone is 18-21s and its resolve-split load term is ~25s — and the repository
already rules that such a harness characterizes itself rather than the floor.
That is exactly why the comparison above is drawn on the per-row `[witness]` CPU
line and not on process wall. An earlier attempt here did compare process wall,
made the cheap sibling look like an 82-second row, and established nothing.

## The eight-sibling test: identity does not predict cost, position does

`v2.test.claim.effect_reach_test` carries eight rows that land within a 346ms
band just past the ceiling on the floor (1954-2122ms). Eight siblings clustered
like that admits two obvious readings — one shared fill with seven free riders,
or eight genuinely expensive witnesses — and they make opposite predictions
under isolation. Measured, with the order-reversal control that distinguishes
them:

```
row                                             alone    fwd(pos)    rev(pos)
path_data_init_derived_host_reading               246     252(1)     511(8)
path_data_init_red_when_import_severed            217     441(2)     488(7)
path_touch_selects_on_normalize_path              225     497(3)     459(6)
unrelated_path_does_not_select                    244     517(4)     518(5)
hermetic_fixture_stays_local                      241     589(5)     446(4)
prose_string_path_does_not_classify               251     498(6)     421(3)
concat_built_path_frontier_not_classified         247     496(7)     385(2)
live_03_normalize_witness_derived_host_reading    234     514(8)     231(1)

by position, mean of both orders:
  pos 1: 242   pos 2: 413   pos 3: 459   pos 4: 482
  pos 5: 554   pos 6: 478   pos 7: 492   pos 8: 512
```

**Neither reading is right.** Run alone, all eight cost 217-251ms — a 34ms
spread around a 238ms mean, which is as uniform as this harness measures. So
they are not eight independently expensive witnesses. But the first row in a
batch is also the CHEAPEST, not the most expensive, which is the opposite of a
shared fill with free riders.

What the reversal establishes is that **cost tracks position, not identity**.
`path_data_init_derived_host_reading` costs 252ms at position 1 and 511ms at
position 8; `live_03_normalize_witness_derived_host_reading` costs 514ms at
position 8 and 231ms at position 1. Same rows, same closure, same head — the
figure follows the slot. Position 1 reproduces the alone cost; every later
position carries roughly double it.

So there is a per-row overhead that appears once more than one witness runs in
a batch, and it is not the first-touch shape the g2 pair shows. Both are
attribution defects and they are not the same defect, which is worth keeping
distinct: first-touch charges one row for a shared computation others then use
free; this charges every row after the first for something that does not scale
with what the row does.

**What this does NOT license.** These figures are not a delta against the floor's
1954-2122ms. Those eight are refused rows, so their floor numbers are
overshoot-past-the-poll — lower bounds, not measurements — and the isolated
figures here are measurements of a different context (71 modules, 2694 resolved
items, against the floor's whole-corpus subject). The honest statement of the
pair is: each row's own work measures ~238ms in isolation, and the floor
attributes it at least 1954ms. The gap is real; its composition is not
established by this measurement.

The one firm consequence for the lane: `effect_reach_test` is not a paring
target. There is no version of "reduce what these witnesses reach for" that
addresses a cost which changes when you reorder the batch.
