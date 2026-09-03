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

## 6. The margin of §3 is a two-run point estimate; the between-run envelope is wider

*Commissioned after `v2.test.emit.rust_produced_decl_emit.rust_produced_decl_name_discriminates`
passed at 406ms against the 500ms ceiling on green `main` and refused at `cpu_at_least=516ms` on an
unchanged tree — a required-floor refusal with `claims_failed=0`.*

**§3's figures index over rows inside one pair of runs, not over runs — and its prose says
otherwise.** §3 reads "run-to-run variation on the same identity across the sample", but the
producer behind it, `identity_inflation_permille`, takes a `base` and an `other` and is driven with
exactly two named runs (`floor_cost_baseline_run`, `floor_cost_contended_run`). Its percentiles are
therefore computed across ROWS inside one pair, and describe how unevenly one host-pair ratio lands
over different rows. Two runs are one sample of the between-run quantity and one sample has no
spread, so the pairwise median is a point estimate wearing a distribution's clothes. The instrument
was right and the sentence over it was not, which is the failure the "name the producer" rule exists
to make detectable. If the admitted execution envelope set is wider than the
pair sampled, it is a **lower bound**, and a budget set from it moves the cliff rather than closing
it. §3 is not withdrawn — it measures what it measures — but it may not be used as a budget input.

**The producer for this section is `gunbc.floor_cost_distribution` `identity_envelopes` /
`complete_envelopes` / `work_invariant_envelopes` / `worst_envelope_permille` /
`run_extreme_census`, driven by `tools.floor_cost_distribution_instrument`
`floor_cost_envelope_report`.** The sample is twelve green `main` runs of `witnesses.yml` on twelve
distinct runner registrations across three hosts, named with their runners in
`floor_cost_envelope_sampled_runs` — the runner is not in the artifact and is acquired from the
run's `required-witnesses-floor` job. Re-derive rather than trust this page; where the two disagree
the instrument is right.

`floor_cost_envelope_check` is the exit-code actuator and refuses an incomplete sample by name;
`floor_cost_envelope_report` returns the lines, and `gunbc run` renders them through its
entry-point refusal because the host requires a `ProcessExit` from an entry function — the sibling
report in §1 has the same shape.

One row's envelope is its max over the twelve runs against its min over the same twelve, computed
only over identities present in every run with a verdict and a baseline at or above 50ms, and
restricted to rows whose `eval_steps` were identical in all twelve so the remaining movement is
inflation rather than a tree change.

**Between-run envelope, work-invariant population (n=398 of 400 complete rows, 3,628 identities
observed across the twelve runs):** median **1.367x**, p90 **1.653x**, p95 **1.819x**, p99
**2.046x**, worst observed **2.280x** — the instrument's own order statistics, which select an
observed member and never interpolate. §3's pairwise median of 1.16x understates the median by a
fifth and the worst case by more than half.

That the step counts are identical for 398 of the 400 rows — across twelve *different* trees — is
itself the corpus-grain observation the rung-drop row's missing item (b) asks for in a narrower
form: the cpu column moves by up to 2.28x while the evaluator-step column does not move at all.

**No cost trend.** Restricting the population by baseline leaves the envelope essentially flat —
p50 1.37 / p90 1.65 at ≥50ms, 1.35 / 1.58 at ≥100ms, 1.36 / 1.64 at ≥200ms, 1.47 / 1.51 at ≥300ms
— which confirms §3's refutation of "expensive rows inflate more" over runs rather than over one
pair, and is what makes a single global margin defensible.

## 7. The verdict IS a property of which runner picks up the job

A median run factor cannot answer this: it is robust exactly where the envelope is driven. The
discriminator is `run_extreme_census`, which counts how often each run holds a row's maximum and its
minimum. Under per-row jitter each of twelve runs is the extreme for about a twelfth of the
population.

It is not close to that. Over the 398 work-invariant rows, run `33766436293` (**srv4-09**) holds the
minimum for **367**; run `33775106554` (**srv1-19**) holds the maximum for **160**. At host grain
srv1 holds the maximum for **279 of 398 rows from 3 of 12 runs**, against 83 for srv3 (5 runs) and 36
for srv4 (4 runs). The extremes are concentrated on particular machines, so a row near the line is
adjudicated by which host dequeued the job.

**An earlier reading of this measurement said the opposite and is corrected here rather than
deleted.** Per-run *median* factors span only 0.878–1.118 and per-host medians are nearly identical
(srv1 1.069, srv3 0.985, srv4 0.992), which reads as "the host does not matter". It is the wrong
statistic for the question: a ceiling is crossed by the extreme, not by the median, and the extreme
census shows the concentration the median averages away.

## 8. What this bounds, and what it does not

**The subject row.** `rust_produced_decl_name_discriminates` measured **313–444ms across the twelve
runs** (envelope 1.42x, `eval_steps` 169,297 in every one — the work never changed). Its minimum,
313ms, is above the clean-run budget the measured p90 implies (500 / 1.653 = **302ms**) and well above
the worst-case budget (500 / 2.28 = **219ms**). The row is inside the variance cliff by measurement,
not by anecdote, and the 516ms refusal is an ordinary member of this distribution.

**The bound is a floor, not a bound.** Twelve runs on three hosts are a subset of the admitted
execution envelopes. A wider sample can only find a larger worst case, so 2.28x may only rise and the
implied budget may only fall.

**Raising the ceiling remains not an option** for the reasons §5 gives. What this measurement
supplies is a re-derivation the authority already asked for: `docs/design-rung-drops.md`, *Per-claim
cost qualification is unavailable at the subject grain the gate consumes*, sizes its attention
constant as the ceiling over the largest inflation floor observed to date and states that the
constant **must be re-derived the moment a larger floor is measured**. The floor it was sized against
was 1.777 from a single 501→282 pair. This sample measures **2.28**, so the constant falls from 280ms
to **219ms**, and the row is updated accordingly. Restricting the derivation to rows at or above
200ms — where whole-millisecond quantisation cannot dominate — gives 1.874 and a constant of 266ms,
so the direction of the re-derivation does not depend on the small-baseline tail.

**Which of the sanctioned remedies the subject family takes is not decided here.** Reducing what the
witness reaches for, or rehoming it in a lane declaring its own dated ceiling and naming it as an
executing consumer, are both open; this section supplies the margin either decision needs.
