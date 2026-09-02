# Required-floor in-flight shared-fill findings

Status: characterization only. This record establishes the mechanism behind the
`lens_registry_required_ids_resolve_holds` interruption. It does not propose or authorize a
change to the required floor's cost model.

## Finding

The required floor applies its 500ms CPU ceiling uniformly to a claim's **marginal** CPU. The
apparent enforcement asymmetry comes from when a cross-claim computation becomes a shared
artifact: only a fresh, successful memo store commits the computation as shared fill. Until that
commit, the cooperative deadline sees the computation as marginal. An interruption abandons the
fill before it can commit, so the work remains marginal and no shared-fill record is emitted.

This is one policy with a temporal classification boundary, not two ceilings:

| Terminal arm | Fill classification | Budget consequence | Receipt consequence |
| --- | --- | --- | --- |
| Evaluation reaches a fresh store before a poll interrupts it | `CrossClaimFillGuard::mark_stored` makes `Drop` record the fill | `budgeted_cpu_nanos` and completion accounting subtract the committed fill | A `[floor-shared-fill]` line reports marginal, fill, and measured cost |
| A poll interrupts evaluation before store | The guard drops with `stored=false` and calls `on_fill_abandon` | No fill has been recorded, so all CPU spent so far remains marginal and can cross 500ms | The interruption reports only the right-censored lower bound; no shared-fill line exists |

Poll placement and arrival order can therefore decide which arm an otherwise identical cold
computation reaches. The classification changes only when publication succeeds.

## Code path

The boundary spans these named symbols:

- `v1_compiler.cli_run.required_floor_runner::run_claim_measured` samples raw thread CPU and the
  shared-fill accumulator, then enforces and records `measured_cpu_nanos - fill_cpu_nanos`.
- `v1_compiler.v1_interpreter::budgeted_cpu_nanos` performs the same subtraction for the live
  deadline clock. `eval_expr` polls that clock every 4096 evaluation nodes.
- `v1_compiler.v1_interpreter::CrossClaimFillGuard` begins with `stored=false`.
  `store_cross_claim_pure_memo` calls `mark_stored` only for `CrossClaimStoreOutcome::Stored`.
- `CrossClaimFillGuard::drop` records shared-artifact fill only on the stored arm. The other arm
  calls `on_fill_abandon`; it deliberately leaves the frame's own cost with the caller while
  retaining the accounting for any stored descendants.

`ChangedWitnessCostPolicy::ChangedCostDebtVerdictOnly` is a real, separate policy: it does not arm
the CPU deadline. It does not explain this occurrence. The failing run reported
`withhold_overridden_for_changed_verdict=0`; cost-debt identities withheld from execution also
cannot be the population of rows that produced measurements.

## Run evidence

The failing main run was GitHub Actions run `33553058260` at revision `2b56084270`. The lens
registry witness appeared only in the interruption diagnostic, with the CPU ceiling reached before
verdict and no upper bound on completed cost. It had no `[floor-shared-fill]` line. That is the
pre-store abandon arm above: interruption suppresses the record because no artifact committed.

The passing PR run was GitHub Actions run `33573600142`. Its uploaded artifact
`required-floor-claim-cost` (artifact id `9826348456`) contains
`required_floor_claim_cost.tsv`. The authoritative row is:

```text
v2.test.lens_registry.sg_claims.lens_registry_required_ids_resolve_holds	v2.test.lens_registry.sg_claims	pass	true	215	214	100
```

The final columns are wall milliseconds, marginal CPU milliseconds, and the completed-cost line.
The witness reached `pass` at 214ms marginal CPU against the 500ms safety ceiling: 286ms, or 57.2%,
below the line on that head. The row's absence from the console log was not absence of a receipt;
the console emits selected diagnostics while the TSV artifact carries passing per-claim costs.

The same failing log contained 634 `[floor-shared-fill]` lines, 122 with
`measured_cpu_ms > 500`. Those rows did **not** demonstrate unenforced ceiling crossings.
`measured_cpu_ms` includes completed shared fill, while the ceiling applies to marginal CPU. The
largest cited example measured 3675ms in total but split into 5ms marginal and 3670ms fill, so it
was 495ms below the enforced line. The 122-row comparison mixed two different quantities.

## Ownership at the commit boundary

Under the model implemented here, an in-progress first touch is not yet a shared artifact and is
attributable to the triggering row until its terminal disposition is known. This is not merely an
accounting convenience: publication can refuse because arguments are not hashable or portable,
the value is not portable, a capacity or byte bound is reached, or the entry is already present.
An abandoned computation never becomes available to another claim.

Successful store retroactively classifies the admitted computation as shared fill; abandonment
leaves the frame's own work marginal. Therefore the evidence does **not** support saying the work
has no owner until completion. Such a model would require a modeled in-flight shared identity or
reservation, which this path does not have.

The remaining defect subject is narrower: work that would become shared if allowed to finish can
be preempted as marginal before it reaches the store that proves and records its shareability.
Whether the floor should represent in-flight ownership or enforce that work differently is a
separate design decision. This findings record intentionally does not make it.

## What this establishes—and what it does not

PR #9978 removed a duplicate lens-registry traversal and preserved the duplicate and unbound
refusals. Its standalone per-witness receipts measured a 27.6–35.7% CPU reduction. That bounded
optimization is independent evidence and is not, by itself, proof that the intermittent floor
interruption is closed.

The passing floor artifact does establish comfortable margin for this witness on that head. A
single green observation does not establish stability across scheduling and cold-fill order, and
the earlier 4-green/2-red history forbids treating absence of interruption alone as the verdict.
The durable mechanism finding is the store/abandon classification boundary above.
