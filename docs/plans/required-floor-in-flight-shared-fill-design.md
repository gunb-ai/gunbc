# Required-floor in-flight shared-fill refusal design

Status: implementation design, following
`required-floor-in-flight-shared-fill-findings.md`.

## Decision

Keep the required floor's existing 500ms attempt-safety ceiling. When its CPU poll fires during
an admitted cross-claim fill, classify the interruption as `FillBudgetExceeded` only when the
claim's marginal CPU without that still-active fill remains within the ceiling. The refusal
carries the claim entry, prospective producer, fill CPU lower bound, marginal CPU lower bound,
and unchanged limit.

This is not pre-publication fill credit. The clock still charges all uncommitted work and stops at
the same point. The change repairs the subject of the refusal: first-touch order no longer appears
as intrinsic claim cost when the active prospective fill is what crossed the bound.

## Boundary and nesting

`CrossClaimFillGuard` remains the one lifetime bracket. Its stack frame additionally retains the
producer and CPU start. At a poll, the outermost active frame identifies the prospective artifact;
its CPU is inclusive elapsed CPU less already-stored descendants. Those descendants remain netted
by the existing committed-fill accumulator, exactly as before.

If marginal CPU alone already exceeds the ceiling, the ordinary evaluation-budget refusal wins.
If no admitted fill is active, behavior is unchanged. Store, abandon, fill receipts, completion
accounting, wall accounting, memo admission, and memo serving are unchanged.

## Safety and evidence

The refusal does not make an over-budget fill pass, raise a ceiling, retry a row, or create an
artifact receipt. Its stable cause token makes the population countable. A real evaluator-path
test must force the poll while an admitted fill is active and assert the producer is carried; a
matched unadmitted control must retain the ordinary evaluation-budget refusal. This instrument
can provide evidence for a separately modeled fill envelope later, but creates no such policy.
