# T-Substrate sub-lane — Parametric-algebra-for-Dimensions subset `(NO-OP / closed by audit; substrate already in place)`

> **Substrate Manager program brief.** Originally framed as a producer
> sub-lane to land phantom-parameter substrate for T-Modeling
> `Dimension<Carrier>`. **Authority audit reveals substrate is already
> in place** — this lane is effectively closed pre-spin-up; T-Modeling
> Dimensions consumer brief
> ([`r2-modeling-dimensions-phantom-worker.md`](r2-modeling-dimensions-phantom-worker.md))
> is dispatchable immediately against the existing carrier.

## Authority audit — substrate is landed

Per `feedback_audit_adjacent_authority_first`: grep before designing. The phantom-parameter substrate is **already first-class** in v3:

- **[`src/v3/compiler/src/dag.rs:186`](../../src/v3/compiler/src/dag.rs)** — `Declaration.phantom_params: Vec<PhantomParameter>`. The carrier exists on the canonical `Declaration` struct.
- **[`src/v3/compiler/src/dag.rs:217-220`](../../src/v3/compiler/src/dag.rs)** — `PhantomParameter { parameter: DeclarationId, algebra: DeclarationId }`. Two-edge carrier: parameter declaration + algebra declaration that governs closure.
- **[`src/v3/compiler/src/dag.rs:148-160`](../../src/v3/compiler/src/dag.rs)** — doc comment explicitly states: *"The initial R2 Dimensions consumer needs only abelian-group closure, carried as a typed edge to the substrate algebra declaration: matching phantom values compose, mismatched values fail closed as a unit mismatch."* The substrate was authored explicitly for the R2 Dimensions consumer.
- **[`src/v3/compiler/src/infer.rs:1057, :1132`](../../src/v3/compiler/src/infer.rs)** — `phantom_unit_mismatch` function exists; type-equivalence and unit-mismatch dispatch are already wired through inference.
- **[`src/v3/compiler/src/dag.rs:1941`](../../src/v3/compiler/src/dag.rs)** — `AbelianGroup` algebra Conj cited as canonical authority for phantom-unit closure.

**Conclusion:** all four open design questions originally posed in this brief (carrier shape / type-equivalence rule / algebra-method dispatch / lifting-coercion) are **already resolved structurally** by the existing substrate. There is no gap to fill on the producer side.

## Disposition

- **Substrate side: closed.** No new carrier. No new variant. No new substrate-shape decisions required. The existing `phantom_params` carrier with its `(parameter, algebra)` shape is the substrate authority.
- **Consumer side: dispatchable now.** [`r2-modeling-dimensions-phantom-worker.md`](r2-modeling-dimensions-phantom-worker.md) consumes the existing carrier directly — no readiness signal from this lane is needed because the substrate is already landed.
- **No-op PR for this brief.** This brief documents the audit receipt and closes the lane. Optional follow-up: if Substrate Manager (post-spin-up) wants the brief deleted entirely as redundant scope, that's a tracking-doc cleanup; the audit receipt persists either way.

## Why the original framing was wrong

`feedback_verify_thesis_claims` violation on Director-side brief authoring: I assumed "the substrate gap" without grepping `phantom_params` / `PhantomParameter` first. The audit-first discipline applies to brief authoring, not just to worker dispatch.

Pattern signal: this is the fourth such violation in the R2 spin-up wave (after nested-optional gating-on-substrate, unhandled-diagnostic path-default, unenumerated-effects 8-req-elision, and now this lane being framed as producer when it's already closed). All four follow the same shape — assumed substrate state without grep.

## What's actually open (cross-program note)

If anything is genuinely open on the Dimensions side, it's in **T-Modeling consumer territory**, not Substrate Manager:

- Authoring `Dimension<Unit, Carrier>` itself (consumes `phantom_params`).
- Authoring core SI base unit declarations.
- Algebra-method dispatch decision (consumes existing `inhabits` algebra-attachment patterns + `phantom_unit_mismatch`).
- Lifting / coercion discipline (per design discipline: strictly nominal preferred).

All of that is in [`r2-modeling-dimensions-phantom-worker.md`](r2-modeling-dimensions-phantom-worker.md); this brief defers entirely.

## Acceptance (for the no-op disposition)

- [ ] This document records the audit receipt + closes the lane.
- [ ] Cross-reference from `r2-modeling-dimensions-phantom-worker.md`'s "Read first" updated to consume `Declaration.phantom_params` directly (no producer signal needed).
- [ ] Substrate Manager (post-spin-up) confirms no producer work needed and surfaces this lane as closed in their working state.

## Reporting

This brief documents a closed lane. No PR needed; the audit receipt stays in this file. T-Modeling Dimensions consumer brief dispatches independently when Modeling Manager spins up.

## Cross-program note

- **Producer:** none required (substrate already landed).
- **Consumer:** Modeling Manager → `r2-modeling-dimensions-phantom-worker.md` (dispatchable now).
- **Adjacent:** Impossible-Bugs Manager — unit-mismatch is a thesis-level impossible-bug class; the carrier is the substrate dependency for that proof. Heads-up at landing.

## Lesson saved

Per pattern across R2 spin-up reframes: **always grep the substrate before authoring producer briefs**, not just worker briefs. The discipline doesn't end at brief boundaries.
