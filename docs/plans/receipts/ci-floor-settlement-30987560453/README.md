# CI floor settlement — run 30987560453

Immutable one-time harvest inputs for the banked closeout in
`gunbc.ci_floor_population_settlement_receipt` (`banked_run_30987560453_*`).

## Files

| File | Role |
|------|------|
| `actions-job-steps.json` | Actions job step `started_at` / `completed_at` (second grain) |
| `artifact-subject.tsv` | Floor-evidence artifact subject coordinates + digests |
| `settlement-summary.json` | Compact derived standing (admission / native / warm-hit / exclusive job) |

## Deterministic exclusive elapsed

For each classified step family:

```
exclusive_ms = max(0, completed_at_unix - started_at_unix) * 1000
```

Actions timestamps are second-grain. Classified wall is the sum of exclusive
populations. Reconcile:

```
unattributed = ci_job_wall - classified
Reconciled iff unattributed * 10000 <= ci_job_wall * 500
```

(≤5% unattributed; banked result 4000/2840000 → 9985 bp ≈ 99.85%).

This pack is not a recurring harvester. Future per-run settlements are external
artifacts; do not re-commit moving PR-head identities into source.
