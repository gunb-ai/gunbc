# R3 Verification — T-FormalGrounding TC1/TC2/TC3 Bundle (Absorbed Responsibility)

**Status:** PROPOSAL — **absorbed cross-cutting responsibility** of R3 Verification Manager (NOT a fourth owned lane). The structural authority [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure" Item 2 now names exactly **3 lanes + 1 ledger gate** for Verification scope after the 2026-04-30 expansion; this brief tracks TC1/TC2/TC3 author-now-fire-later cadence + unified-predicate strict-fire coverage requirements as audit-cadence work, not a dispatch lane. Director ratified unified substrate-introduction at [#828 c#4356050427](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356050427) and [#828 c#4356138359](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356138359): one `BinaryDimensionReportEquals` `TestPredicate` variant absorbs TC1/TC2/TC3 strict-fire surfaces.

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md) — §"Absorbed cross-cutting responsibility — TC1/TC2/TC3 bundle".

## Scope (this bundle)

Manage the three formal-grounding `TestClaim` author-now-fire-later commitments through their strict-fire activation. Each TC has a different coverage requirement, but all three now target the same Substrate-owned strict-fire carrier: `BinaryDimensionReportEquals`, a generalized binary structural equality predicate over `DimensionReport<C>` with reflection-aware modifiers. PR #1309 named this Option 2 first for TC1; PR #1316 independently validated the same shape for TC2; Director ratified the unified disposition on #828.

| TC | Claim | Status (2026-04-30 audit) | Strict-fire gate |
|---|---|---|---|
| **TC1** | η-equivalence (substrate-lens research) | **Author-now-fire-later landed** — fixture `src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag` via #1179; uses `SubstrateResearchDeferredClaim` carrier (runner-valid only for this fixture per [`r2-closure-ledger.md`](../r2-closure-ledger.md) L220). | Strict-fire activates through `BinaryDimensionReportEquals` with an eta-equivalence reflection modifier once the unified predicate lands and TC1's lens-producer / lens-primitive prerequisites are satisfied. |
| **TC2** | Church-Rosser / evaluation-order independence | **Unified consumer landed** — fixture `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag` (per [`r2-evaluator-manager.md`](r2-evaluator-manager.md) L127); `BinaryDimensionReportEquals` with paired `DimensionReport<Dag>` roles for strategy-order (LeftFirst vs RightFirst). Runner report equality NYI until `DimensionReport<C>` production lands. | Strict-fire **evaluation** activates after ≥2 executable strategies exist **and** the runner compares produced reports. PR-B.1 lands a single eager strategy per [`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) L154; the second strategy remains R3 residual. |
| **TC3** | Strong normalization (meta-theorem over well-typed fragment) | **Stage-(a) fixture landed** — `src/v3/compiler/tests/fixtures/tc3_strong_normalization_deferred.dag` (`tc3_strong_normalization_substrate_introduced`). Declarative theorem text remains PB-authored in [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) §"TC3 — Strong-normalization TestClaim (author-now-fire-later, PB → R3 Verification transition)". **Substrate-encoding unified** — TC3 consumes `BinaryDimensionReportEquals` with evaluation-step role pairing (runner NYI until report production lands). | **Two-stage gate (single authority — both stages required for strict-fire):** (a) stage-(a) fixture + coverage requirements land against unified predicate (this row). (b) **Strict-fire:** T-FixedPoint completion (termination semantics). Ownership transitions PB→Verification per §"Transition to R3 Verification" in [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md). **No fire-before-(b) path exists** — full theorem witnessing requires (b). |

## Owned deliverables

1. **TC strict-fire activation cadence** — when a TC's strict-fire prerequisites land, replace the deferred-claim variant with the strict predicate; preserve fixture path + claim name (stability invariant).
2. **Unified predicate coverage requirements (load-bearing)** — collect TC1 eta, TC2 strategy-order, and TC3 evaluation-step requirements for the Substrate-owned `BinaryDimensionReportEquals` predicate proposal. Verification authors the consuming requirements and fixtures; Substrate authors the predicate variant and carrier shape per INVARIANTS §P1. PB does not re-author TC3 after transition per §"Transition to R3 Verification" in [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md).
3. **Drift surfacing** — periodic audit (rolled into bridge-ledger audit cadence) that TC fixture paths + carrier shapes remain consistent on main. Surface drift to Director.

## Unified predicate substrate-introduction — open design questions (parked)

Director ratification resolved the former TC3-specific quantifier-carrier question. Remaining questions feed the unified proposal and resolve when the three TC coverage-input analyses mature.

- **Quantifier carrier shape — RESOLVED** — TC3 does **not** get a TC3-specific `MetaTheoremOverTypedFragment` predicate. Per Director ratification [#828 c#4356050427](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356050427) + [#828 c#4356138359](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356138359), the substrate target is unified `BinaryDimensionReportEquals`: binary structural equality over `DimensionReport<C>` with reflection-aware modifiers. TC3 contributes the evaluation-step modifier / coverage requirements to that unified predicate.
- **Witness construction** — what does an evaluation-step witness look like for "every well-typed program reduces to normal form"? Consumes T-FixedPoint termination semantics + `Lens<C>` shape.
- **Fixture path** — proposed `src/v3/compiler/tests/fixtures/tc3_strong_normalization_deferred.dag` (mirroring TC1/TC2 naming convention). Lock at substrate-introduction PR.

## Dependencies summary

| TC | Blocker | Owner |
|---|---|---|
| TC1 | Unified `BinaryDimensionReportEquals` predicate + eta-equivalence modifier + T-Substrate-Lens-Primitive / lens producer prerequisites | Substrate Manager + PB Manager continuations; Verification supplies TC1 coverage requirements |
| TC2 | Unified `BinaryDimensionReportEquals` predicate + strategy-order modifier + second executable evaluation strategy | Substrate Manager + R3 Evaluator residual; Verification supplies TC2 coverage requirements |
| TC3 | Unified `BinaryDimensionReportEquals` predicate + evaluation-step modifier + B5 audit + T-FixedPoint strict-fire horizon | Substrate Manager + PB Manager (B5 / T-FixedPoint); Verification supplies TC3 coverage requirements |

## Cadence

- **No active worker dispatch** until at least one TC's strict-fire prerequisites land.
- **Audit cadence** — bundled with bridge-ledger audit; refresh status table at each manager-brief PR.
- **TC3 dispatch trigger** — when both B5 and T-Substrate-Lens-Primitive are green, this bundle authors TC3 evaluation-step coverage requirements feeding the unified `BinaryDimensionReportEquals` proposal (~3-5 days research-tier per PM Tier C estimate). It does not author a TC3-specific predicate brief.

## Explicitly out of scope

- **TC1 substrate-research authoring** — Substrate Manager continuation owns T-Substrate-Lens-Primitive; this bundle only consumes the strengthening signal.
- **TC2 second-strategy implementation** — R3 Evaluator residual (per [`r2-closure-ledger.md`](../r2-closure-ledger.md) Evaluator row); this bundle only consumes the strengthening signal.
- **TC3 declaration content** — already authored in [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) §"TC3 — Strong-normalization TestClaim (author-now-fire-later, PB → R3 Verification transition)" (PB-authored declarative shape). This bundle authors TC3 coverage requirements + fixture shape for the unified predicate, NOT the declaration text per §"Transition to R3 Verification" ownership-transition contract.
- **L4 / L5 / L7 verification claims** — Lanes 1 + 2 of this manager.

## Acceptance — `.dag` gates (each TC fires independently)

- `tc1_substrate_lens_eta_equivalence_strict_fire` — TC1 strengthens from `SubstrateResearchDeferredClaim` to `BinaryDimensionReportEquals` with the eta-equivalence modifier.
- `tc2_evaluation_order_independence_strict_fire` — TC2 **runner strict-fire** (report equality evaluates green): fixture already consumes `BinaryDimensionReportEquals` with strategy-order roles; strengthens further when both strategies + `DimensionReport<C>` production land per [`r2-evaluator-manager.md`](r2-evaluator-manager.md) L127.
- `tc3_strong_normalization_substrate_introduced` — TC3 evaluation-step coverage requirements + fixture shape land against the unified `BinaryDimensionReportEquals` predicate; subsequent strict-fire still waits on T-FixedPoint completion.

## Cross-refs

- Parent manager: [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md)
- Unified substrate-introduction ratification: [#828 c#4356050427](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356050427), [#828 c#4356138359](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356138359)
- TC1 deeper analysis / Option 2 source: [PR #1309](https://github.com/gunb-ai/gunbc/pull/1309)
- TC2 independent coverage analysis: [PR #1316](https://github.com/gunb-ai/gunbc/pull/1316)
- TC3 upstream declarative shape (PB-authored, transition contract): [`docs/briefs/r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) §"TC3 — Strong-normalization TestClaim (author-now-fire-later, PB → R3 Verification transition)"; ownership transitions per §"Transition to R3 Verification"
- TC1 fixture authority: [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) §216 (Director ratification of #1179 direction)
- TC2 slice-0 hook: [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) L127
- B5 dependency for TC3: [`docs/briefs/r2-release-b5-loop-construction-closure-audit-worker.md`](r2-release-b5-loop-construction-closure-audit-worker.md)
- INVARIANTS substrate-fact-introduction procedure: [`INVARIANTS.md`](../../INVARIANTS.md) §P1 (TC3 introduction path)
- P4 Decidability (TC3 formal home): [`INVARIANTS.md`](../../INVARIANTS.md) §P4
