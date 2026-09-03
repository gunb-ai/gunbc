# The floor cost distribution near the 500ms ceiling

*Measurement, not a repair. Commissioned by tidy-lynx-804 after gunbc#10133 to decide whether
further per-producer work is a treadmill. The question is whether the population near the ceiling
is dense, whether the rows now crossing share a root, and what an honest margin policy would be.*

## The instrument

**Every figure below is produced by `tools.floor_cost_distribution_instrument`
`floor_cost_distribution_report`, and the derivations it composes are
`gunbc.floor_cost_distribution`.** Re-derive rather than trust this page:

```
# one download per run id in `floor_cost_sampled_runs`, into `floor_cost_artifact_root`:
gh run download <run-id> -n required-floor-claim-cost -D /tmp/floor-cost/<run-id>

gunbc run --source-root dag --source-root src/v2 \
  --entry dag/gunbc/instruments/floor_cost_distribution_instrument.dag \
  --function floor_cost_distribution_report
```

The first version of this memo named `gunbc.witness_floor_workflow` as its instrument. That
workflow produces the raw rows and performs none of the analysis — the ten-run intersection, the
band histogram, the counterfactual replay and the inflation percentiles existed only as prose plus
numbers, so a reader could check the arithmetic of nothing. Review 58983 refused it on exactly that
ground. The figures that remain below are illustrative of what the instrument returned on the runs
it names; the instrument is the authority, and where the two disagree the instrument is right.

**The input needs no floor run.** `required-floor-claim-cost` is uploaded by every required-floor
run and carries the COMPLETE executed population — the `[over-cost]` lines in the job log are a
25-row preview of the same data and say so. The runs are named as data in
`floor_cost_sampled_runs`, because a measurement over "whatever happened to be downloaded" is not
re-derivable: a second reader would get different numbers with no way to tell a real change from a
different sample.

**An incomplete sample refuses.** `floor_cost_distribution_check` exits non-zero naming every run
that failed to load. One artifact in an eleventh run (`33667468330`) downloaded empty during the
original analysis and silently emptied a ten-way identity intersection before it was caught — a run
that reads as zero rows is indistinguishable from a run in which nothing crossed, which is the
absorbing fallback DESIGN §5 forbids. Both refusal arms are witnessed in
`test.claim.floor_cost_distribution_witness`, along with a discriminating red for each derivation
below.

## 1. The band is dense, and the tail is a plateau

Post-repair run `33691893294`, CPU basis, 3,498 rows: median 2ms, mean 34ms, p90 86ms, p99 397ms.

| CPU band | rows | cumulative from top |
| --- | --- | --- |
| ≥500 | 5 | 5 |
| 400–500 | 27 | 32 |
| 300–400 | 55 | 87 |
| 200–300 | 89 | 176 |
| 100–200 | 148 | 324 |
| <100 | 3,174 | 3,498 |

**There is no gap below the ceiling.** The decision-relevant form of that is what happens if modules
are repaired top-down. Taking the worst row remaining across all ten runs, removing modules in
max-cost order after the three gunbc#10133 already fixed:

| modules repaired | worst row remaining | runs refused /10 |
| --- | --- | --- |
| (the 3 from #10133) | 533 | 1 |
| +`rust_produced_decl_emit` | 525 | 1 |
| +`emit_host_field_access_equals_eval` | 511 | 1 |
| +`emit_host_fold_closure_equals_eval` | 490 | 0 |
| +`semantic_decl_serialize_parity` | 487 | 0 |
| +`emit_host_meet_join_equals_eval` | 486 | 0 |
| +5 more modules | 429 | 0 |

The first three modules buy 43ms. The next seven buy 61ms — roughly **10ms per module**. That is the
treadmill, quantified: below the top of the band each repair returns about one fiftieth of the
ceiling.

## 2. The new crossers do NOT share a root the way the #10133 three did

`eval_steps` is a cheap structural fingerprint of shared work, and it separates the two cases
cleanly.

The three modules gunbc#10133 repaired reported **612,662–658,336** eval steps — a 1.07x spread
across seventeen rows in three modules with unrelated subjects. That uniformity *is* the signature of
one shared dominant producer, and it was: `compile_phase_frontier_standing`.

The families now near the ceiling report **63,196–197,227** eval steps — a 3.1x spread. They are
three distinct shapes rather than one:

| family | rows ≥200ms | eval_steps | wall/cpu |
| --- | --- | --- | --- |
| `v2.test.execution.emit_host_*_equals_eval` | 33 | 63k–174k | **2.05** |
| `v2.test.emit.semantic_decl_*` | 27 | 78k–141k | 1.01 |
| `v2.test.emit.rust_*` / `produced_decl_*` | 48 | 77k–197k | 1.00 |

The `emit_host_*` family spends half its wall clock off-CPU, which is the region
`v2.workflow.required_floor` already carries a §4b rung drop for (`OpaqueHostCallUnbounded`).

**There is one shared-producer lead, and it is not a plan.** `required-floor-cross-claim-demand`
ranks `bind_outcome` (`v2.std.diagnostic`, unkeyed, 679 claims / 108,085 evals / 183 modules) at the
top by a factor of 13, and its module samples reach both the emit and the execution families. But
that artifact's own header states `cost_columns=inclusive_of_callees_do_not_sum`, and its column
totals 1,736,654ms against the run's actual `claim_cpu_total_ms=120,177` — a factor of 14. It ranks
candidates; it cannot quantify a saving, and it must not be cited as one.

## 3. The margin, measured

Run-to-run variation on the **same identity** across the sample, restricted to rows with enough
signal that integer milliseconds are not noise:

- median **1.16x**, p90 **1.35x**, p95 ~1.43x, max ~**2.0x**.

Two mechanism hypotheses were tested and both are refuted, which is what makes a single global margin
defensible:

- **Expensive rows do not inflate more.** By baseline-cost bucket the median inflation is 1.18, 1.13,
  1.19, 1.17, 1.07, 0.92 for the 20–50 … 400–500ms buckets.
- **Host-call-heavy rows do not inflate more.** By wall/cpu class the median inflation is 1.16 (pure
  CPU), 1.13, 1.16 (host-heavy ≥1.8).

Run-level contention alone spans only 0.92x–1.15x across the ten runs, so most of the spread is
per-row rather than per-run. And run `33688140761` **refused with a below-median run factor of
0.96**, which is why a refusal cannot be read as evidence that the fleet was busy.

**PR runs are harsher than `main` runs.** All four modules reported crossing on PR runs
(#10109/#10122/#10123) have `main`-run medians of 314–391ms and `main`-run maxima of 417–487ms, so
the PR path reaches roughly 1.6x on a median row.

At 1.6x a row must sit under **312ms** on a clean run to be safe; at the observed 2.0x, under
**250ms**. **87 rows exceed 300ms today.**

## 4. What this corrects

The framing that prompted this measurement was that gunbc#10133 promoted the next rows down the
ranking into the margin. **The promotion mechanism is refuted, and the repair was disproportionate
rather than one step of a treadmill.**

*Refuted:* rank does not cause a crossing, absolute cost does, and removing rows above a row does not
make it slower. Measured directly, the five rows that crossed on the post-repair run went **above
their own pre-repair maxima** — pre-repair median 343ms and max 472ms over 45 observations, against
504–533ms post-repair. They did not inherit a margin; they got slower, on the most contended run in
the sample (run factor 1.15, the highest of the ten).

*Disproportionate:* replaying all ten runs with the three repaired modules excluded —

| | runs refused |
| --- | --- |
| as observed | **5 / 10** |
| with the #10133 modules excluded | **1 / 10** |

Across the nine pre-repair runs, **no row outside those three modules ever crossed 500ms.** Those
three modules were 17 of the 22 distinct identities that crossed anywhere in the sample. They were an
outlier — one shared 612k-step producer — not a sample of the plateau.

## 5. What follows

The plateau is real and per-producer repair is the wrong unit for 87 rows returning ~10ms each. Two
remedies are sanctioned by `v2.workflow.required_floor`: reduce what the witness reaches for, or
enroll it in a lane declaring its own dated ceiling **and** naming the row as an executing consumer.

**Raising the ceiling is not a third option and should not be re-proposed.** That module records the
value being raised 500→1552→5000 and both raises being repudiated: 1552 was measured-max plus a
scheduler quantum on the same current tree, which is the oracle DESIGN §5 forbids by name, and 5000
rode in as a passenger on a ruling about something else. The population that accumulated under that
drift is frozen in `v2.workflow.floor_cost_debt` precisely so it is not granted a higher ceiling.

Which of the two remedies fits the emit and execution families — and whether a `bind_outcome` repair
buys real margin or only a rank change — is a decision this measurement informs and does not make.

## 6. The premise is measured false: there is no fixed above-ceiling identity set, and the budget is not mis-set

*Added after the §5 question was worked. It is recorded here rather than in the thread that produced
it because the belief it refutes is the one a reader re-forms from the same symptom — a red floor
lane naming a handful of emit rows — and re-deriving it costs a night.*

**The claim being retired: "three (later seven) self-host witness modules sit above the 500ms hard
CPU ceiling." Measured, no fixed above-ceiling identity set exists.** Completed green runs place this
family's band below 500ms; run variance can carry its top across, and four completed-over-cost rows
were observed at 506–515ms. What is retired is that the family is *intrinsically* above the ceiling —
a row does not stably "sit" on either side of an attempt-safety boundary — not the crossings
themselves, which are real and recur. Both remedies §5 names are
refused for this family on evidence, and the ceiling is correctly denominated. What remains is a
different subject, stated at the end.

### How to re-derive this, because there is no instrument module for it

The §5 instrument reads one artifact per run. The join below needs a second axis — the same
identities measured off-CI — and **no authority owns it today**, which is a gap and not a style
choice. The recipe, at identity grain:

```
# CI side: per-claim identity/outcome/cpu_ms for a completed run
gh run download <run-id> -R gunb-ai/gunbc -n required-floor-claim-cost -D /tmp/ci/<run-id>

# local side: ONE function per process, which matches the floor's fresh-frame-per-claim
claim_batch --source-root dag --source-root src/v2 \
  --eval-budget-ms 300000 --entry <entry> --functions <one identity>
```

Join on identity. Use runs whose rows **completed**; prefer green runs for the baseline factor.

### Three readings that are wrong, and the control that exposes each

**An `interrupted_before_verdict` figure is the budget, not the row.** The deadline preempted the
measurement, so the cost is unbounded above 500. Every interrupted row reports approximately whatever
ceiling stopped it, so a set of them looks like a tight cluster "just over" no matter what the true
costs are. Only `completed_over_cost_requirement` rows carry costs. Raising `--eval-budget-ms` on a
probe is how you measure one; that is instrumenting, and is not the ceiling-raise §5 repudiates.

**The crossing set is not the expensive set.** Measured with the budget raised, the row in
`v2.test.emit.rust_body_add_emit` that *passed* on CI is the most expensive of its four, and the
three that were interrupted are all cheaper — a single-digit-ms spread separates survivor from
casualty. Repairing "the rows that crossed" therefore fits noise. **Always measure the non-crossing
siblings**; they are what reveals the band.

**A one-anchor local→CI factor does not transfer.** A factor taken from one identity predicted
another row comfortably under the ceiling that CI had in fact interrupted. Take the factor from
many identities across several runs, or not at all.

### What the join shows

Joined over the emit and execution rows of this family across three main runs, the local→CI cpu
factor is **stable across green runs and materially higher on a failed one** — the same tree and the
same rows, with every row inflated together. At the green factor the family's whole band lands well
inside the ceiling; at the failed run's factor its top crosses. That is CI run-to-run variance, and
it independently corroborates the inflation distribution §3 measures from the other direction — two
instruments built for different purposes agreeing on one phenomenon.

`v2.test.execution.emit_host_fold_closure_equals_eval` is the worked example: interrupted at
`cpu_at_least=522ms` on one job, it completes under budget on every green run in the join.

**The host-independent form of this, which is stronger and needs no factor at all.** `eval_steps` is
carried in the same artifact and does not depend on the machine. Across the three runs, **every row
of this family that completed has a byte-identical step count** — identical work — while its
`cpu_ms` swings by up to the full green-to-failed ratio. Milliseconds move; the work does not. That
settles the question without any local→CI conversion, and it is the axis to use.

The single row whose step count differs is the one that was **interrupted** on the failed run: its
count is truncated where the deadline stopped it. So the one apparent exception is a second
demonstration of the first misreading above — an interrupted row is not measured, in steps or in
milliseconds. *(The `eval_steps`-as-control technique is `gunbc#10158`'s; see the caveat below.)*

### Why reducing the cost is the only lever, and what the cost is

Both §5 remedies are refused for this family:

- **Reduce what the witness reaches for** — refused by measurement. Decomposing `rust_target_model`
  component-by-component (each in its own process, against a baseline replicated across four
  independently written probe modules) shows every component except the bundle costs nothing, and
  the bundle is what the emission reads. There is no oversized subject to trim, the test fixture is
  free, and both cost-shape defects were looked for and are absent — the module fold is linear, and
  the two `list_snoc_item`-in-a-fold sites operate over ~8 and 3 elements.
- **Enroll it in a lane declaring its own dated ceiling** — no such lane exists. Every candidate
  either withholds rows from execution (`v2.workflow.floor_cost_debt`, whose own header states
  membership deletes coverage, and which is shrink-only and closed to new crossers) or names a
  cadence with no workflow (`FalsifierSubstrateLongLane`; no workflow in the tree carries a
  `schedule:` trigger). Taking that arm would mean *building* a lane.

The per-claim cost is a **fixed floor plus a small per-declaration term**, not per-declaration work:
the second declaration emitted in a frame costs a fraction of the first, and the module fold adds
nothing over separate calls. The fixed part splits into `rust_target_model` construction, the
arrow-body projection, and two smaller serialize steps. The projection term is **measured at the
`v2.std.compilers.target_model` `target_project_arrow_body_to_value_expression` frame and attributed
by elimination** to the primitive-apply step beneath it — the neighbouring candidates
(`dag_value_expression_projection`, `dag_surface_operator_canonicalization_member`, the
translation-rules lookup, the inner-arrow construction) each measure zero, the last two tested with
inputs that would have exposed a table build even on a miss.

**One live lead, left undetermined on purpose.** The first projection in a frame costs; subsequent
ones do not, *regardless of which declaration body is projected*. That is equally consistent with a
memo keyed above the body and with a one-time warm of a shared structure that any first caller pays.
Those want different providers, so the mechanism must be separated before any provider is proposed —
that ambiguity, not the cost, is the open question. Two prior sharing attempts on this surface are
already refused: a `data`-row promotion cannot reach the floor at all (`required_floor_runner` builds
a fresh evaluation frame per claim, so `data_cache` never spans claims), and `rust_target_model` was
enrolled in `v2.workflow.floor_pure_producer_share` and withdrawn after measuring this exact cluster
*worse* — read that roster's header before re-proposing either.

**One caveat on that withdrawal, added because `gunbc#10158` landed while this section was being
written.** The roster's rust-row measurements were normalised against *rows in no consumer module of
any enrolled key*, and #10158 shows that control is **biased by composition** — it demonstrated the
point by splitting a subject's own rows on `eval_steps` into those the serve reached and those doing
byte-identical work, and finding the supposedly untouched group moved almost as much. Rows elsewhere
are an assumption about the corpus; rows doing byte-identical work are an observation about the row.
That does not overturn the withdrawal, and the roster's own next trigger still governs — but the
strength of "measured worse" now rests on a control known to be biased, so a re-proposal should
re-measure against the step-split control rather than treat the question as closed.

A value check was run before treating any of this as a caching opportunity: the projections of
`x + y` and `y + x` are compared and **differ**, carrying both a positive control that the equality
works and a by-construction-false row proving the harness reports failures at all. A cheaper-looking
result that lost operand order would have been a correctness defect, not a saving.

### The subject, restated

The floor is not flipping because a few rows are too expensive. **The emit surface sits at roughly
two-thirds to four-fifths of the per-claim budget on a normal run, and CI run-to-run variance is
large enough to carry its top across the line** — so which rows cross is a property of the run, not
of the rows. Reducing the shared fixed cost raises the margin against that variance. Nothing else
currently on the table does, and per-row repair aimed at whichever identities crossed last is
repairing the sample rather than the population.
