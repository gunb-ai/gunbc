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
shared fill with free riders. Cost tracks position, not identity:
`path_data_init_derived_host_reading` costs 252ms at position 1 and 511ms at
position 8, while `live_03_normalize_witness_derived_host_reading` costs 514ms
at position 8 and 231ms at position 1. Same rows, same closure, same head.

### The step is a claim_batch artifact, not a floor cost — corrected

An earlier revision of this section called that step "a second attribution
defect", which implied the production path. It does not belong to the floor, and
three independent checks say so.

**Source.** `claim_batch` `run_entry_group` builds ONE ctx per entry group and
runs every function in the group against it, calling
`eval_call_memo_frame_exit` after each — so position 1 meets a virgin context and
positions 2..N meet one that has already had an eviction pass. The floor's
claim-evaluation fold in `cli_run.rs` builds `evaluation_frame` FRESH INSIDE the
per-claim loop, and says why in source: claims sharing one immutable scope must
not share the mutable evaluation caches a context owns. The mechanism that
produces a once-off step in a shared ctx is one the floor deliberately does not
have.

**Cross-entry measurement.** Batching two different entries shows no step at all
— each row costs its alone figure in either order, because each entry gets its
own ctx:

```
                              effect_reach row    root_d row
alone                              239               1774
batch [effect_reach, root_d]       229               1802
batch [root_d, effect_reach]       219               1732
```

Within one entry the step reproduces (235ms then 396ms for two rows of the same
file). So the effect is bounded by the entry group — a property of the shared
ctx, exactly where the source says it would be.

**Floor data.** If the floor had the same step, later rows within an entry would
cost about twice the first. Across the 32 modules in this run carrying five or
more uncensored slow rows, the mean ratio of rest-to-first is **1.27**, spanning
0.75 to 3.48 — several modules have a first row MORE expensive than its
siblings, which is the first-touch shape and the opposite of the step. A shared
mechanism would cluster near 2.0. It does not reproduce.

So the position step stands as a fact about `claim_batch` and says nothing about
floor attribution. The additive-versus-multiplicative question that would follow
from it is therefore not a question about the production path.

### What this did establish: the first confirmed paring target

`root_d_checkpoint_scalar_declared_arity_witness_holds` costs **1774ms of its
own work** run alone, in a closure of 5 modules and 160 resolved items. That
closure is far too small for the cost to be context: it is the witness's own
evaluation. The floor attributes 2792ms, so unlike every other row examined in
this lane the gap is modest — about 1.6x, not a thousandfold.

That makes it the first row in this lane whose expense survives isolation, and
therefore the first genuine subject for the operator's decomposition brief. It
is also the most stable over-budget row across runs (2792 / 2849 / 2855, a 2.3%
spread, against 22-30% for the large ones), which is what a real cost looks like
and what an attribution artifact does not.

## The stability screen: a sound negative filter, not a paring worklist

The one row whose cost survived isolation, `root_d`, was also the most stable
across runs (2.3% spread where the large rows swung 22-30%). That suggests a
cheap screen over all 720 slow rows — rank by cross-run variance rather than by
magnitude — since a cost that is genuinely a witness's own should not care what
else ran, while context, fill and position costs vary with everything around
them. Ranking by magnitude is known to sort mostly by who touched a cache first,
so a principled screen would be the first real way to build the paring worklist
without running an isolation harness 720 times.

Built over four runs (32172125816, 32177951514, 32185058245, 32187164199),
joined by identity, 650 rows with three or more usable observations.

**Censoring rule, and it decides the result.** An observation is dropped when
the row was explicitly interrupted (`cost is at least`) or when its figure sits
in the poll-pinned band, because those values are the deadline rather than a
cost. Observations from rows that were budget-refused but COMPLETED are kept:
those carry a real measurement. An earlier cut of this screen dropped every
budget-refused row indiscriminately and thereby excluded `root_d`, the 71s row
and all eight `effect_reach` rows — that is, every row worth screening — which
is what over-broad censoring looks like when the expensive population is exactly
the refused one.

**Validation.** Three rows with known ground truth from isolation:

```
row                            CV     mean      percentile   known truth
root_d                        1.5%    2854ms        18       real own work (1774ms isolated)
71s extdeps_scope row         7.8%   76988ms        86       context (70-85ms isolated)
g2 specimen row               7.4%   13292ms        84       first-touch
```

Two of three land correctly: the context-dominated rows sit in the variable head,
`root_d` in the stable tail.

**But it fails the fourth case, and the failure is the finding.** The eight
`effect_reach` rows come out STABLE — CV 1.9% to 6.4% around a ~2050ms mean —
and isolation has already established they do about 238ms of their own work.
A row can therefore be highly stable and still be carrying an order of magnitude
of cost that is not its own, because whatever context it carries is itself
reproducible from run to run.

So the screen measures REPRODUCIBILITY, not ownership. Those are different
properties and only the second is what paring needs:

- **Sound as a negative filter.** A high-variance row is certainly not measuring
  its own work, so the variable head can be excluded from a paring worklist
  without isolating anything. That is a real saving over 720 rows.
- **Unsound as a positive test.** A low-variance row may be real work
  (`root_d`) or may be stable context (`effect_reach`). Nothing short of
  isolation separates those, and the screen must not be presented as producing
  the worklist directly.

The honest procedure is therefore two-stage: screen out the variable head for
free, then isolate the stable tail. The stable tail above 1000ms is small enough
for that to be affordable, which is the actual saving on offer.

## Cluster-tightness: also suggestive, also not decisive

If N distinct witnesses — different names, different assertions, different
modules — land within a few percent of each other, that near-identity looks like
evidence the figure belongs to something they share rather than to any of them,
since independent work has no reason to coincide. Over the four-run join, rows
above 800ms fall into 19 clusters at 3% width, and one is remarkable: **18
identities across 4 unrelated modules** (roadmap static site, running-release
identity, deploy readiness, site-surface readiness) inside a 2.3% band at
1213-1242ms.

It correctly flags the `effect_reach` six at 2016-2067ms, which isolation had
already shown collapse to ~238ms. So the signal has one confirmed true positive.

**But the 18-row cluster does not collapse.** One member isolated from each of
its four modules:

```
module                        floor    isolated   closure
roadmap_static_site_witness    1213      888ms    -
running_release_identity       1216      791ms    630 modules / 15657 items
live_deploy.readiness          1218      819ms    632 modules / 15656 items
roadmap_site_surface_readiness 1218      827ms    632 modules / 15640 items
```

Two-thirds of each floor figure survives isolation. These are not eighteen rows
riding one fill; whatever they share, each still pays most of its cost alone. So
cluster-tightness is a suggestive prior and not a decisive test — exactly the
same standing as the variance screen, and for the same underlying reason: both
observe the SHAPE of a cost distribution, and neither can see whose work it is.

**What the run did surface is a better discriminator than either.** Compare cost
against closure size:

```
root_d                 1774ms over     160 resolved items      dense
cluster members       ~820ms over   15,650 resolved items      sparse
effect_reach            238ms over    2,694 resolved items      sparse
```

`root_d` is doing an order of magnitude more work per resolved item than
anything else examined. That density, not its magnitude and not its stability,
is what marks it as a witness whose cost is its own evaluation rather than its
surroundings.

**Not settled by this run, and stated so it is not assumed:** whether the
cluster's residual ~820ms is assertion work or per-process warm-up of a
15,650-item closure that the single running row must pay alone. Distinguishing
those needs two rows of one module measured on a harness without `claim_batch`'s
shared-ctx step, which does not exist today. Until then the honest reading is
that ~820ms is *not shared across the eighteen* — not that it is assertion work.

## root_d: the cost is one claim's fixture import, not the witness

`root_d_checkpoint_scalar_declared_arity_witness_holds` is one `test fn`
conjoining three claims, two of which call `compile_dag_rust_emit_check` on a
small virtual module. Measured by splitting them temporarily and running each in
isolation (5 modules, 163 resolved items):

```
claim                                              verdict     cpu
positive fixture (declared-arity leaf)               PASS      36ms
arity-0 scalar still strips phantom arguments        PASS    1782ms
authority answers for both fixtures                  PASS       0ms
```

**All three pass**, and one claim carries essentially the entire 1774ms. Two
things follow, and the first corrects an assumption made before measuring.

**The expensive claim is not the one carrying the defect.** The row is enrolled
in `floor_expected_red`, which invites the reading that its positive fixture is
red and expensive. It is neither: the positive fixture costs 36ms and passes.
The row passes as a whole and is therefore a stale quarantine, consistent with
the floor's `stale_quarantine=5`.

**The cost is an import in a fixture string, not an assertion.** Both expensive-
looking claims call the same checker on a two-line module. The difference between
36ms and 1782ms is that the second fixture's source begins
`import std.integer { Int8 }`, so the emit check compiles that closure, while the
first declares its `Witness<C>` inline and imports nothing. The 1774ms is the
cost of compiling `std.integer` inside a virtual fixture — not the cost of
proving anything about checkpoint-scalar arity.

**Why no split is proposed here.** Splitting would move the 1782ms onto a new
identity that is not on the expected-red roster, turning a budget-refused
quarantined row into an ordinary over-budget failure — worse than the state it
replaces. The decomposition only becomes correct together with the fixture fix,
and that fix is a judgement about whether the arity-0 wall can be witnessed by a
scalar that does not drag `std.integer` in, which belongs to whoever owns Root D
rather than to a cost lane. Recorded here with the measurement so that decision
can be made on facts rather than on the row's size.

So the corpus's one row whose cost survived every screen is, on inspection, also
not a witness-decomposition subject. Its expense is a fixture-authoring choice
with a localised cause and a plausibly cheap fix.
