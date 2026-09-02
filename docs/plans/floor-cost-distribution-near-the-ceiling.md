# The floor cost distribution near the 500ms ceiling

*Measurement, not a repair. Commissioned by tidy-lynx-804 after gunbc#10133 to decide whether
further per-producer work is a treadmill. The question is whether the population near the ceiling
is dense, whether the rows now crossing share a root, and what an honest margin policy would be.*

## The instrument

Every required-floor run uploads `required-floor-claim-cost`, a per-claim TSV carrying
`identity, module, outcome, verdict_reached, wall_ms, cpu_ms, eval_steps, cost_line_ms` for the
complete executed population — 3,498 rows. The `[over-cost]` lines in the job log are a 25-row
preview of the same data and say so ("the complete population is in the per-claim cost TSV uploaded
by this run"). Two further artifacts are read below: `required-floor-cross-claim-demand` and the
run's own `required-floor:` summary line.

Nothing here needed a floor run. Every figure is a reading of artifacts already published by ten
`main` runs of `gunbc.witness_floor_workflow`: `33668368846`, `33670164259`, `33673806092`,
`33680659583`, `33681059548`, `33682522296`, `33683651328`, `33688140761`, `33690512169`,
`33691893294`. The last is the merge commit of gunbc#10133 and is the only post-repair run in the
sample; the other nine precede it.

**One artifact in an eleventh run (`33667468330`) downloaded empty.** It is excluded. It is recorded
because it silently emptied a ten-way identity intersection before it was caught — an empty result
is a claim about the instrument first.

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
