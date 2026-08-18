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

## Reconciled against the floor-path fill ledger (gunbc#8464)

The shared-fill ledger instruments corpus caches on the floor path and reports,
per fill, what it cost, who paid it, and who read it. Its run 32192150969
(9425 claims, 19 fills) settles two of this document's rows at identity grain,
and one of them corrects a claim made here.

**First-touch, confirmed on the production path.** `unbound_dissolution_empty_literal_refuses`
— one of the six runaways — paid a 51508ms `module_path_index` fill that **139
claims across 29 modules** then read. That is 99.8% of the row's measured cost
belonging to a computation twenty-nine other modules consume, observed on the
floor rather than inferred from a sandbox. It is the g2 pair's shape at corpus
scale, and it is the run's only shared fill.

**But the 71-second row is the opposite, and that refines what this document
said about it.** It paid a 29017ms `module_graph_facts` fill and a 24965ms
`reference_edges` fill, and **both are exclusive** — no other module reads
either. So its cost is genuinely its own, and the earlier framing here, "a cheap
witness billed for the context it runs in", is not quite right.

The two measurements reconcile once the variable is named. Isolated in a
67-module closure the row costs 70-85ms; on the floor's 2376-module subject it
costs 71s. Nothing about the witness changed — it asks for whole-corpus facts,
so its bill is a function of how much corpus there is. It is neither a cheap row
wearing someone else's cost nor an expensive assertion: it is a witness whose
subject IS the corpus, measured small in isolation and large on the floor. The
isolation run did not catch this because shrinking the corpus is precisely what
makes such a row look cheap.

That distinction is what decides the remedy, and it inverts between the two rows:

- the 51.5s `module_path_index` payer must NOT be quarantined — moving it
  recovers ~112ms and hands 51.5s to whichever of the other 29 modules runs
  first. The fix is to hoist that fill into preparation, where its consumers
  already are in spirit and where it stops being any witness's bill.
- the 71s row's fills are wanted by nothing else, so moving or reducing it
  recovers the whole amount.

Per-row wall time does not distinguish these two cases. Nothing in this
document's census does either — which is the strongest argument for the ledger
being the instrument the population needed, and the honest limit of a census
built from timings alone.

**And a floor of ~108s no paring can reach:** three fills — a 55392ms module
graph, a 35060ms path index, a 17646ms reference-edge scan — are paid before any
claim runs. Quarantining every witness on the floor would not move them.

## Retraction: the variance screen measured four different trees

External review on the subtree rollup caught a defect in the screen above that
neither its author nor its reviewer checked, and it is fatal to the statistic as
stated.

**The four runs have four different heads, on four different branches:**

```
32172125816   6aa80c79   work/last-two
32177951514   59653f56   main
32185058245   178db9ad   session/crisp-wren-479
32187164199   ccf21913   floor/cost-is-not-a-defect
```

Not one pair among them is a repeat measurement of the same subject. Repeatability
requires the same subject digest, claim code, manifest and order, budget and poll
policy, and source universe; these share none of that. So every coefficient of
variation in the section above is **agreement across four different worlds, not
repeatability under identical input** — and this lane has already established
that changing the source population moves both instrument sensitivity and
first-touch ownership, which is exactly the variable left uncontrolled.

`root_d`'s 1.5% figure therefore did not establish what it was used for.
Isolation vindicated the row independently, but the statistic did not earn that
agreement and should not be cited as though it had.

**The negative direction fails too.** This document demoted the screen to a
"sound negative filter" after `effect_reach` came out stable. Review rejects
that half as well: an intrinsically expensive claim can swing 20-30% from host
scheduling and frequency variation, page-cache state, allocator behaviour,
subprocess startup, lock contention, input-dependent branch and allocation shape,
or a nested shared fill reached during otherwise genuine work. High variance
therefore does not establish that a row is context-only, and the variable head
cannot be dropped for free.

And stable context cost is not the accident this document treated it as. Claim
order is deterministic, so the same identity always first-touches a given cache;
corpus and cache key are deterministic; every process starts cold; the same host
class recurs; and a cooperative deadline pins observations near one polling
boundary. Under those conditions a shared cost is *expected* to look stable.
`effect_reach` is the normal case, not a freak counterexample.

**What survives.** Variance is legitimate PRIORITISATION — high variance means
investigate order and context sensitivity early, low variance means
repeatable-cost candidate — and is never evidence of ownership in either
direction. The minimum evidence stack for ownership is three things together:
an uncensored completion rather than an interrupt-point value, repeated runs on
the SAME subject, and an order perturbation showing whether the charge moves to a
different first toucher. The reversal control used earlier in this document is
that third item; the second is the one this lane has never actually performed.

## The error bar on a single floor observation, measured

The lane had never measured the same subject twice. A GitHub re-run replays the
pinned merge ref, so re-running one completed run yields two observations of one
world — every row measured twice. Run 32182126916, attempts 1 and 2.

**Same world, verified rather than assumed:** identical subject digest
`b0eb3f2be1a200e0` and identical counters on both attempts (planned 9420,
executed 9420, passed 8951, failed 0, budget_refused 102). *(Capture the log
BEFORE triggering the re-run: GitHub serves only the latest attempt and the
earlier archive is unrecoverable. This pair exists only because attempt 1 had
been downloaded hours earlier for an unrelated check.)*

**Paired per-identity delta**, on the 673 rows uncensored in both attempts —
marginal distributions are not used, because opposing moves cancel and two
histograms can look identical while every row moved:

```
median |delta|   3.8%
p95    |delta|  12.7%
p99    |delta|  16.4%
largest moves   -5% on the 93.8s row, +12% on the 16.6s row
```

So a single floor observation carries a real error bar: about 4% typically and
over 12% at the tail, on an identical subject.

**Crossings of the 1552ms ceiling: zero.** Spread and crossings are different
facts and only the second is the enforcement question — a 200ms move on a 400ms
row is irrelevant, the same move at 1500ms flips a build. Across 673 genuine
rows measured twice, none changed sides.

**But that is not because the instrument is precise — it is because the danger
band is nearly empty.** Only **4** genuine rows sit within the p95 flip-radius
(191ms) of the ceiling, and the closest is:

```
margin 29ms (1.9%)   1523 -> 1517   where_refinement_cast_literal_oci_other_digest_algorithm_accepts
```

That row's margin is **below the median noise** of 3.8%, so it is the corpus's
one materially flip-prone identity. It is also, independently, the exact row
sharp-raven-273 observed at 1505ms and then 1274ms on a re-run of one frozen ref
in the 1500ms-ceiling era — a 15.4% move, near this distribution's p99. Their
flip is therefore the predicted behaviour of the closest-to-threshold row, not an
anomaly, and two lanes located the same identity by different routes.

**What this settles.** The ceiling is enforceable as a per-run threshold on
today's population: enforcement risk is concentrated in a handful of rows rather
than diffuse across the corpus. It is not safe in general — any row that comes to
rest within ~200ms of the ceiling inherits a pass/fail that is partly a property
of the runner, and a single-run quarantine decision inherits it too.

**Caveat that changes the reading, and it nearly went unstated.** The rows
*closest* to the ceiling overall sit at 1553ms and did not move at all between
attempts — they are poll-pinned censored values, not measurements, so they cannot
cross by construction. Including them makes the instrument look far steadier than
it is. Every figure above excludes them.

### Rule, not caveat: exclude censored values before computing any distribution

Three shapes of one class turned up in a single day on this log — `cost is at
least` vs `cost is exactly`, the `BUDGET-REFUSED` third channel, and the
poll-pinned 1553ms band inside a paired distribution. Each one flatters: it
makes the instrument look steadier, or the population smaller, than it is. So
this is a standing step rather than a thing to remember when suspicious:
**before computing any statistic over floor timings, drop every right-censored
value, say how many were dropped, and name the channel each value came from.**
The channel is part of the datum because the three are not equally trustworthy
and no field distinguishes them once the number is extracted:

```
BUDGET-REFUSED      an interrupt      -> right-censored, no upper bound
cost is at least    an interrupt      -> right-censored, lower bound only
cost is exactly     a completion      -> a measurement
[floor-witness-slow] elapsed          -> a measurement (fires at the 100ms warn)
a poll-pinned 1553ms with 1ms margin  -> neither; it cannot move by construction
```

(the channel refinement is sharp-raven-273's, and it is the right generalization:
"how many were dropped" hides that the dropped rows failed in different ways) An interrupt is not a measurement, and a
pinned value cannot move by construction, so neither can answer a question about
movement in either direction — they are unmeasured, which is a third state from
pass and fail.

## Is the noise relative or absolute? It is relative — so a flip radius scales

This decides whether the 1552ms result transfers to a lower ceiling. Mean paired
movement, bucketed by row magnitude over the 672 genuine rows:

```
magnitude      n     mean |rel|   mean |abs|
100-250ms    391        4.9%         7.9ms
250-500ms    142        5.4%        19.3ms
500-1000ms   109        4.3%        29.7ms
1000-2000ms   30        2.6%        30.4ms
```

Absolute movement grows nearly 4x across the range while relative movement stays
flat. The noise is therefore **proportional**, and a flip radius must be computed
as a fraction of the ceiling rather than carried across as a fixed millisecond
figure. Carrying the 1552ms p95 radius (210ms) down to a 500ms ceiling would
overstate that band by more than 3x.

## What each candidate ceiling costs, measured

p95 relative movement is 13.5%, so the flip radius at ceiling C is 0.135*C.
Over the 672 genuine paired rows:

| ceiling | p95 radius | rows in band | CROSSINGS observed | rows already over |
|---|---|---|---|---|
| 500ms  | 67.7ms  | 53 | **11** | 139 |
| 750ms  | 101.5ms | 40 | **9**  | 62 |
| 1000ms | 135.3ms | 24 | **3**  | 30 |
| 1552ms | 210.0ms |  4 | **0**  | 0 |

The crossings are observed, not modelled: each is a row that landed on opposite
sides of the ceiling in two runs of one identical subject.

**And they are not scattered coin flips — they cluster by module.** Six of the
eleven 500ms crossings are one module (`generic_item_clone_bound_witness`, every
row 476-491ms then 502-528ms), two more are `where_refinement_enforcement_witness`,
two are the python round-trip pair. Rows in a module share a closure, so they
move together: a ceiling in a crowded band refuses a whole module at a time on a
runner's luck, not one unlucky row.

**So the answer to a 500ms ceiling is that it is not a marginal adjustment.** It
refuses 139 rows on the first run for being over, and puts 53 more in a band
where 11 were observed changing sides between two runs of the same tree. A
build's outcome would then be partly a property of which runner picked it up.
1000ms is the first candidate where the band thins out (24 rows, 3 crossings) and
1552ms is empty of crossings entirely.

This does not say the corpus should stay slow — it says the ceiling cannot lead
the cost work. Rows have to leave the band before the ceiling descends onto it,
which is the same ordering #8470 demonstrated: the 45941ms row became 844ms by
moving a shared fill into preparation, and nothing about the ceiling changed.

## Audit of the two quarantine-decision runs

Both logs retrieved complete (terminal counter line present in each, so this is
not the CLI-truncation class):

```
run 32189985063  planned=8995 executed=8995 passed=8636 failed=1 budget_refused=0
  FAIL  emitted_lib_rs_module_declaration_witness_test.small_crate_lib_rs_omits_unemitted_dispatch_module
        cost is exactly 42545ms against 1552ms          (27.4x the ceiling)

run 32193032348  planned=8997 executed=8997 passed=8638 failed=1 budget_refused=0
  FAIL  emitter_nested_refinement_cast_witness_test.single_refinement_carrier_emits_no_unsupported_cast
        cost is exactly 45826ms against 1552ms          (29.5x the ceiling)
```

Neither decision is marginal: flipping a 42545ms observation needs a 96.4% drop
against a measured p99 of 17.7%. Both quarantines are clean on the evidence, and
both rows are the first-touch fill carriers this note is about — the second is the
row #8470 later brought to 844ms without touching the witness.

One discrepancy is recorded rather than resolved: each run's own terminal counters
report `failed=1, budget_refused=0`, so these two logs contain two measured
verdicts, not the eight-plus-one this audit was scoped to. The remaining
identities are not in these runs' counters and their provenance needs naming
before they can be audited or cleared.
