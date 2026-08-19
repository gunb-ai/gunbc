# Floor slow-row measurement, 2026-08-18

Per-row timings from ONE `witnesses` workflow run, parsed from its
`[floor-witness-slow]` lines. `floor-slow-rows-2026-08-18.tsv` columns:
`module`, `function`, `elapsed_ms`, `warn_ms`, `ceiling_ms`.

## Provenance — one run, not a merge

| field | value |
|---|---|
| run | 32172125816 |
| branch | `work/last-two` |
| head | `6aa80c79a1` |
| expected-red roster | 469 |

One run, deliberately. The 2026-08-17 table merged four and had to retract a
whole "second cause" family that was an artifact of the merge; this file does
not repeat that. `work/last-two` rather than `main` because on main at the same
hour the expected-red roster evaluated EMPTY, so no enrolled row could be
classified budget-refused at all and every timeout landed in the ordinary FAIL
population — main's numbers are deflated by a roster defect, not by being
cheaper.

## The population

```
720 rows over the 100ms warn     >=543.0s
220 rows over 500ms              >=438.8s   <- the operator's target ceiling
103 rows over 1552ms                        <- today's armed ceiling
  6 rows over 5s                 >=194.1s   <- 36% of all slow time
```

The totals are LOWER BOUNDS, not sums of measurements: 85 of the 720 rows are
censored at the cooperative interrupt point, and one of the six runaways is.
The counts are sound. See the censoring section below before using any total.

500ms is the operator's hard ceiling (2026-08-17, superseding the 5s rule);
1552ms is the accommodation currently armed in `v2.workflow.required_floor`
`required_floor_claim_budget_ms`, recorded there as measured-max-plus-one-
quantum. Both are stated because they select different populations.

## The six runaways

```
71244ms  test.claim.extdeps_scope_placement_gate_loudness_witness.red_seed_runner_failure_detail_projects_located_receipt
39678ms  test.claim.dissolution_census_witness_test.unbound_dissolution_empty_literal_refuses
29026ms  test.claim.rust_test_fixtures_import_closure_witness.ambient_pool_provider_preserves_listed_import
21364ms  test.claim.rust_test_fixtures_import_closure_witness.declared_import_closure_lists_rust_import
20558ms  v2.test.manual.grounding_lens_whole_tree.grounding_lens_whole_tree_witnesses
12197ms  test.claim.g2_data_reference_under_selection_witness_test.specimen_classifies_as_runtime_read
```

All six PASS. On run 32175452947 they are `PassedOverBudget` — stale_quarantine=5,
failed=0 — so they are cost defects with no content failure.

## Read the phrasing before parsing a budget line

A budget refusal is emitted two ways, and they are different data:

- `cost is at least Xms` — `BudgetCompletion::Interrupted`. A right-censored
  LOWER BOUND: the row was stopped at the poll and its true cost is unknown.
- `cost is exactly Xms` — `BudgetCompletion::CompletedOverBudget`. A real
  measurement: the row ran past the ceiling and finished.

See `BudgetCompletion::elapsed_reading` in `cli_run.rs`. This matters because a
parser matching only `at least` systematically drops the runaways: a row cheap
enough to be interrupted at the poll never gets to be expensive.

**And those two are not the whole vocabulary.** A third line,
`required-floor: BUDGET-REFUSED <identity> is enrolled as expected-red but was
BUDGET-REFUSED`, reports interruption for enrolled expected-red rows; run
32172125816 carries 98 of those and zero `at least` lines. Concluding "nothing
was interrupted" from the absence of `at least` is therefore unsound, and this
document did exactly that before being corrected. Enumerate all three channels
before making any claim about censoring.

The census above is built from `[floor-witness-slow]` instead, which fires at
the warn line and carries elapsed for every row regardless of outcome — but see
the censoring section below: carrying an elapsed figure is not the same as that
figure being a measurement.

## Seven rows are refused by 1-4ms

`gunbc.test.claim.dag_arrow_lambda_witness_test` (5 rows) and
`gunbc.test.claim.namespace_graft_body_dissolution_witness_test` (2 rows) land
at 1553-1556ms against the 1552ms ceiling. They are over by one to four
milliseconds on a shared CI host. Recorded, not adjudicated: whether that is a
real cost or the ceiling sitting inside its own measurement noise is a question
for whoever owns the ceiling, and moving it to make them pass would be
tuning a threshold to its current population rather than to a requirement.

## Why this run is usable as a baseline, and what it must not be used for

**Partly censored — corrected 2026-08-18, and the correction matters.** An
earlier revision of this section called the run uncensored, on the ground that
it carries five `cost is exactly` lines and zero `cost is at least`. That was
wrong, and wrong in this document's own failure mode: interruption is reported
in this run through a THIRD channel, `required-floor: BUDGET-REFUSED`, which
carries 98 identities and which that check never looked at. Absence of the
`at least` phrasing is therefore NOT evidence that nothing was interrupted.

What the data shows once that channel is read. 85 of the 720 slow rows sit in a
pinned band at the cooperative interrupt point — 28 at 1553ms, 28 at 1554,
18 at 1555, 8 at 1556 — which is a deadline, not a distribution of costs. Those
85 figures are LOWER BOUNDS. The remaining 635 are real measurements, including
every value above the band.

Consequences for the headline numbers, stated exactly:

- `220 rows over 500ms` — the COUNT is sound; each of those rows genuinely
  exceeded 500ms. The `438.8s` total is a LOWER BOUND, because 85 of its rows
  are censored at the interrupt point.
- `6 runaways, 194.1s` — five of the six are real completed measurements. One,
  `grounding_lens_whole_tree` at 20558ms, is budget-refused and is a lower
  bound, so 194.1s is a lower bound too.
- The pinned band must never be ranked internally. Its rows differ by
  milliseconds because the poll fired, not because they cost different amounts.

The deadline is cooperative — polled every 4096 evals — so a row can run well
past the ceiling before the poll fires. That is why 13 refused rows on main
carry real overshoot rather than sitting in the band, and why refusal lines do
carry measured elapsed rather than a ceiling readout. Cross-check available in
this run: `grounding_lens_whole_tree` appears as a slow line at 20558ms and as a
refusal at at-least 20475ms — two independent renderings of one row agreeing to
within 0.4%.

**Not a ranking of expensive witnesses.** This table records what the floor
spent and where. It does not record which witnesses are expensive, and the two
differ by exactly the first-touch effect documented in
[the attribution note](../witness-cost-first-touch-attribution.md): a row that
is first to reach a shared computation carries its entry's bill, and its
siblings show near zero for the same work. The 71-second row is the extreme
case — 70-85ms in isolation. Treating any row here as a per-witness cost, or
sorting the population to decide what to pare, reproduces the error the note
exists to prevent.
