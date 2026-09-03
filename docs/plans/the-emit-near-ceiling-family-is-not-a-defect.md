# The emit near-ceiling family is not a defect

*A negative result, filed so the next lane skips four plays that do not work. Produced by
jolly-ferret-412 while owning the charge subject behind the required floor's 500ms per-claim CPU
ceiling. Every figure below names the producer that re-derives it; where a number and a producer
disagree, the producer is right.*

## What this closes

Three `v2.test.emit` identities refused the required floor on run `33695697323`. The question
routed to this lane was whether the emit family carries a repairable cost defect. **It does not.**
Four rows, two clusters, 292–350ms off the floor, none individually remarkable; what put two of
them over 500ms is the run they landed in.

That answer is only useful with the plays that were tried attached, because each looks obviously
correct from the outside:

| play | outcome | where it is recorded |
| --- | --- | --- |
| serve the shared producer across claim frames | **measured and refused**, three candidate rows, every one made its consumers worse | `v2.workflow.floor_pure_producer_share` `floor_cross_claim_refused_candidates` |
| find the one dominant shared producer, as gunbc#10133 did | **refuted twice**, independently | `eval_steps` fingerprint in `docs/plans/floor-cost-distribution-near-the-ceiling.md`; and the present-vs-absent runs behind the refused rows above |
| repair the worst row per-producer | **8ms**, against a question 87 rows deep | the top-down table in the same memo |
| read the interrupted rows' printed cost as a margin | **not a cost at all** — an interrupted row reports the ceiling that interrupted it | `v2.workflow.required_floor`, and the in-band sentence the floor prints beside it |

## The four rows, measured off the floor

Two of the three refusing identities were `budget_interrupted`, so their cost was **unbounded
above** — `required_floor_claim_cost.tsv` records them at 505/503 and 508/504, which is the
interrupt point, not a measurement. A row with no upper bound cannot be ranked, cannot be compared,
and cannot be shown to have improved by any repair, so the first step was to bound them.

Instrument: `claim_batch --source-root dag --source-root src/v2 --entry <module test> --functions
<identity>`, one entry per module, no floor and no budget.

| identity | floor | off-floor |
| --- | --- | --- |
| `produced_decl_two_targets_render_own_order` | ≥503 interrupted | 346ms |
| `rust_produced_decl_emits_named_add` | ≥504 interrupted | 294ms |
| `rust_produced_decl_emits_named_add2` | 489 passed | 292ms |
| `rust_produced_decl_name_discriminates` | 527 over-budget | 350ms |

**No absolute figure crosses between those columns.** The off-floor arm is a different
architecture and a different execution context; every conclusion here uses within-column ordering
only, four rows measured under identical conditions in each column.

## Two rows, two mechanisms

`named_add` off the floor is 294ms against its sibling `named_add2` at 292ms — the same row within
1% — while on the floor the sibling completed at 489 and it was interrupted. **Contention**, and
three instruments that share no mechanism agree:

1. µs/step from the floor artifact: 3.238 against the sibling's 2.990, 8.3% slower per unit of
   work — derived from `cpu_ms` and `eval_steps`, which are sampled in consecutive statements after
   `run_claim` returns and netted by the same shared-fill rule, so the ratio survives reordering.
2. off-floor sibling parity, above.
3. required multiple: both siblings needed ~1.70x to cross, above the measured p90, and exactly one
   of them did.

`two_targets` is the opposite. Its 188,416 steps at interrupt exceed every sibling's *completed*
total, so it does more work — but off the floor it lands at 346ms beside `name_discriminates` at
350ms. **Biggest in work, ordinary in time**, and time is what the ceiling adjudicates. A work
bound is not a cost property; inferring one from the other is the error this row exists to
document.

## What the ceiling is actually adjudicating

Eleven floor runs joined on claim identity: `33661252708`, `33664119594`, `33667640710`,
`33671317370`, `33688140761`, `33695697323`, `33699325123`, `33699540412`, `33700216949`,
`33700681714`, `33701938720`.

**No identity crossed twice.** Four runs had crossings; not one identity recurs. Producer:
`gunbc.floor_cost_distribution` `crossing_recurrence` and `recurring_crossers`, whose empty answer
over single-run crossings is the finding rather than the absence of one — and whose positive
control in `test.claim.floor_cost_distribution_witness` is what keeps the empty answer from being
vacuous.

**Crossings cluster by family within a run.** Run `33688140761`: ten crossers, all ten in
`test.claim.self_host_compile_phase_live_gate_witness`. Run `33695697323`: three, all in the emit
produced-decl family. A run tips a family; which identity inside it tips is arbitrary.

**The run factor is large.** Over 296 identities present in all eleven runs with median cpu ≥100ms,
taking each row's cross-run median as its own baseline, the per-run median factor runs
0.858 … 1.154 — 1.35x best to worst on the *median* row, not a tail. The near-ceiling population
tracks it: rows ≥350ms per run come out 3 on the 0.959 run and 36 on the 1.154 run. Producer:
`identity_inflation_permille` composed with `permille_percentile`.

So the run factor decides **whether** anything crosses; a row's baseline cost decides **which
family** is exposed when it does. The line adjudicates a product of two things and attributes it to
one.

## The instrument that needs no control population

`eval_steps` is host-independent and byte-identical for the large majority of identities across a
run pair. For those rows the only thing that could have changed did not, so any cpu movement is
inflation **by construction** — no normalisation, no control population to choose and defend.
Producer: `identity_inflation_permille_at_equal_work`, which drops any row whose step count moved
rather than smoothing it.

Its reach must be reported beside it — `count` it against `identity_inflation_permille` — because a
clean instrument over three rows is not a measurement of a corpus.

## What is NOT claimed

- **The vintage confound is load-bearing.** The eleven runs span different tree vintages, so "no
  identity crossed twice" is confounded by repair: the ten `live_gate` crossers stopped recurring
  partly because that family was fixed. This is not a pure contention result.
- **Equal steps is equal EVALUATOR work, not equal host work.** A row whose time sits inside one
  opaque host call reports few steps in both runs; `v2.workflow.required_floor` carries a declared
  rung drop for that region.
- **No safe-cost figure is derived here, and one was withdrawn.** A `500 / p90` derivation was
  circulated by this lane and retracted: a p90 answers a per-row question while the ceiling refuses
  the whole *run*, and the stratum quoted stopped one bucket short of the population that can
  actually reach 500 — where inflation rises again rather than continuing to fall. Both errors ran
  in the unsafe direction. The stratified table with n per bucket belongs with the decision-maker,
  and the quantile choice is theirs.
- **Nothing here is a recommendation about where the line should sit.**
