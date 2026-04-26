# T-Substrate sub-lane — Cardinality-for-int-lit subset `(REDUNDANT — superseded by existing t-substrate-cardinality-int-lit-worker.md)`

> **DO NOT DISPATCH AGAINST THIS BRIEF.** The R2 spin-up authoring
> wave authored this as a producer scoping brief without first
> grepping for an existing authority. **Existing brief is at
> [`docs/briefs/t-substrate-cardinality-int-lit-worker.md`](t-substrate-cardinality-int-lit-worker.md)** —
> the authoritative scope decisions, post-`wise-pike-578` STOP-AND-ESCALATE
> re-scope (carrier-widening deferred to sibling sub-lane,
> String-decimal range facts, etc.). This file documents the audit
> receipt and routes consumers to the existing authority.

## Authority audit — the brief already exists

Per `feedback_audit_adjacent_authority_first` + `feedback_verify_thesis_claims`: grep before authoring. The existing authority covers the same scope:

- **[`docs/briefs/t-substrate-cardinality-int-lit-worker.md`](t-substrate-cardinality-int-lit-worker.md)** — already authored; carries the canonical scope decisions:
  - **Re-scoped 2026-04-25** post-`wise-pike-578` STOP-AND-ESCALATE: keep `LiteralBits::Int(i64)` substrate-side; carrier-widening (Int128/UInt128/Word128Carrier) deferred to sibling sub-lane.
  - Range facts use String-decimal representation (width-independent; covers u64 bounds that exceed i64).
  - Reconciliation narrowing at `infer.rs:704-725`; `MagnitudeOutOfRange` diagnostic per C-8.
  - `i64::MIN` smoke test deferred to sibling sub-lane.
  - 5 numbered consumer-side requirements, slice, acceptance.

**My R2 spin-up brief was duplicate authoring.** Same pattern as the parametric-algebra-for-Dimensions reframe: assumed the brief didn't exist; would have caused single-authority violation.

## Disposition

- **This file: closed as redundant.** The existing authoritative brief is `t-substrate-cardinality-int-lit-worker.md`. All scope decisions, slice, acceptance, and STOP-AND-ESCALATE live there.
- **No producer-side work to author.** The existing brief is dispatchable as-is.
- **Consumer-side:** [`r2-modeling-int-lit-magnitude-worker.md`](r2-modeling-int-lit-magnitude-worker.md) is updated to cite the existing brief as its producer prerequisite.

## Pattern signal

This is the second R2 spin-up substrate brief closed as redundant (after parametric-algebra-for-Dimensions). The audit-first discipline applied at the time would have caught both. **Always grep `docs/briefs/` for existing authority before authoring R2 spin-up substrate briefs.**

## Cross-program note

- **Producer authority:** `t-substrate-cardinality-int-lit-worker.md` (existing).
- **Consumer:** Modeling Manager → `r2-modeling-int-lit-magnitude-worker.md` (cites the existing brief).
- **Sibling sub-lane (deferred):** Int128/UInt128/Word128Carrier substrate work — closes the `i64::MIN`-as-single-token gap. Per the existing brief's re-scope. Tracked separately; not part of this lane.
