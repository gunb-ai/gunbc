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
720 rows over the 100ms warn      543.0s
220 rows over 500ms               438.8s   <- the operator's target ceiling
103 rows over 1552ms                       <- today's armed ceiling
  6 rows over 5s                  194.1s   <- 36% of all slow time
```

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
enough to be interrupted at the poll never gets to be expensive. Run
32172125816 carries five `exactly` lines and zero `at least` lines. The census
above is built from `[floor-witness-slow]` instead, which fires at the warn line
and carries every row's real elapsed regardless of outcome.

## Seven rows are refused by 1-4ms

`gunbc.test.claim.dag_arrow_lambda_witness_test` (5 rows) and
`gunbc.test.claim.namespace_graft_body_dissolution_witness_test` (2 rows) land
at 1553-1556ms against the 1552ms ceiling. They are over by one to four
milliseconds on a shared CI host. Recorded, not adjudicated: whether that is a
real cost or the ceiling sitting inside its own measurement noise is a question
for whoever owns the ceiling, and moving it to make them pass would be
tuning a threshold to its current population rather than to a requirement.
