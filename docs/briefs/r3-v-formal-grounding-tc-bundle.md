# R3 Verification — T-FormalGrounding TC1/TC2/TC3 Bundle (Absorbed Responsibility)

**Status:** PROPOSAL — **absorbed cross-cutting responsibility** of R3 Verification Manager (NOT a third owned lane). The structural authority [`docs/r3-structure.md`](../r3-structure.md) L108 names exactly **2 lanes + 1 ledger gate** for Verification scope; this brief tracks TC1/TC2/TC3 author-now-fire-later cadence + TC3 substrate-introduction triggering as audit-cadence work, not a dispatch lane. Research-tier work (~3-5 days per TC strengthening per PM Tier C estimate) when TC3 substrate-introduction worker brief is authored under the existing 2-lane scope.

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md) — §"Absorbed cross-cutting responsibility — TC1/TC2/TC3 bundle".

## Scope (this bundle)

Manage the three formal-grounding `TestClaim` author-now-fire-later commitments through their strict-fire activation. Each TC has a different unblock condition; this bundle coordinates across them and authors the substrate path for TC3 when its prerequisites land.

| TC | Claim | Status (2026-04-30 audit) | Strict-fire gate |
|---|---|---|---|
| **TC1** | η-equivalence (substrate-lens research) | **Author-now-fire-later landed** — fixture `src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag` via #1179; uses `SubstrateResearchDeferredClaim` carrier (runner-valid only for this fixture per [`r2-closure-ledger.md`](../r2-closure-ledger.md) L220). | T-Substrate-Lens-Primitive landing + lens producer retirement (per [`r2-closure-ledger.md`](../r2-closure-ledger.md) §216 ratification). |
| **TC2** | Church-Rosser / evaluation-order independence | **Slice-0 hook landed** — fixture `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag` (per [`r2-evaluator-manager.md`](r2-evaluator-manager.md) L127); current predicate is deferred (`Compiles`-shaped). | **R3-deferred from R2-Evaluator** — needs ≥2 executable strategies. PR-B.1 lands a single eager strategy per [`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) L154; second strategy is R3 residual. Strengthens to strict strategy-output equality over `DimensionReport<C>`. |
| **TC3** | Strong normalization (meta-theorem over well-typed fragment) | **Text-form only** in [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) L145-215; no fixture. **Substrate-encoding gap** — every existing `TestPredicate` variant is per-program; TC3 is a meta-theorem universally quantified across all well-typed programs. No existing variant carries that quantifier shape (per L171 of that brief). | **Substrate-fact-introduction via [`INVARIANTS.md`](../../INVARIANTS.md) §P1** + B5 audit + T-Substrate-Lens-Primitive (provides the `Lens<C>` shape TC3's evaluation-step witness consumes). Ownership transitions PB→Verification per L181-185 of `r3-pb-t-fixedpoint-worker.md`. |

## Owned deliverables

1. **TC strict-fire activation cadence** — when a TC's strict-fire prerequisites land, replace the deferred-claim variant with the strict predicate; preserve fixture path + claim name (stability invariant).
2. **TC3 substrate-fact-introduction (load-bearing)** — when B5 + T-Substrate-Lens-Primitive land, author the substrate path for the meta-theorem quantifier shape per INVARIANTS §P1 procedure. Cross-coordinate with Substrate Manager continuation (the carrier-type shape is substrate-owned per #1078 dispatch lock pattern). PB does not re-author after transition per L181-185 of `r3-pb-t-fixedpoint-worker.md`.
3. **Drift surfacing** — periodic audit (rolled into bridge-ledger audit cadence) that TC fixture paths + carrier shapes remain consistent on main. Surface drift to Director.

## TC3 substrate-fact-introduction — open design questions (parked)

These resolve when TC3 dispatch fires. Listed here for handoff continuity:

- **Quantifier carrier shape** — does TC3 need a new `TestPredicate` variant (e.g., `MetaTheoremOverTypedFragment`), or does it dissolve into a structural fold once T-Substrate-Lens-Primitive provides the evaluation-step witness shape? Per [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) L171: "no existing variant carries that quantifier shape" — so substrate introduction is the live candidate, dissolution is the hope.
- **Witness construction** — what does an evaluation-step witness look like for "every well-typed program reduces to normal form"? Consumes T-FixedPoint termination semantics + `Lens<C>` shape.
- **Fixture path** — proposed `src/v3/compiler/tests/fixtures/tc3_strong_normalization_deferred.dag` (mirroring TC1/TC2 naming convention). Lock at substrate-introduction PR.

## Dependencies summary

| TC | Blocker | Owner |
|---|---|---|
| TC1 | T-Substrate-Lens-Primitive + lens producer retirement | Substrate Manager + PB Manager continuations |
| TC2 | Second executable evaluation strategy | R3 Evaluator residual (post-R2-Evaluator close) |
| TC3 | B5 audit + T-Substrate-Lens-Primitive + substrate-fact-introduction | PB Manager (B5) + Substrate Manager + this bundle (introduction authorship) |

## Cadence

- **No active worker dispatch** until at least one TC's strict-fire prerequisites land.
- **Audit cadence** — bundled with bridge-ledger audit; refresh status table at each manager-brief PR.
- **TC3 dispatch trigger** — when both B5 and T-Substrate-Lens-Primitive are green, this bundle authors a TC3 substrate-introduction worker brief (~3-5 days research-tier per PM Tier C estimate).

## Explicitly out of scope

- **TC1 substrate-research authoring** — Substrate Manager continuation owns T-Substrate-Lens-Primitive; this bundle only consumes the strengthening signal.
- **TC2 second-strategy implementation** — R3 Evaluator residual (per [`r2-closure-ledger.md`](../r2-closure-ledger.md) Evaluator row); this bundle only consumes the strengthening signal.
- **TC3 declaration content** — already authored in [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) L145-215 (PB-authored declarative shape). This lane authors the substrate path + fixture, NOT the declaration text per L181-185 ownership-transition contract.
- **L4 / L5 / L7 verification claims** — Lanes 1 + 2 of this manager.

## Acceptance — `.dag` gates (each TC fires independently)

- `tc1_substrate_lens_eta_equivalence_strict_fire` — TC1 strengthens from `SubstrateResearchDeferredClaim` to strict predicate.
- `tc2_evaluation_order_independence_strict_fire` — TC2 strengthens from `Compiles`-shaped slice-0 to strict strategy-output equality over `DimensionReport<C>` per [`r2-evaluator-manager.md`](r2-evaluator-manager.md) L127.
- `tc3_strong_normalization_substrate_introduced` — TC3 substrate path authored + fixture lands; subsequent strict-fire when T-FixedPoint completes.

## Cross-refs

- Parent manager: [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md)
- TC3 upstream declarative shape (PB-authored, transition contract): [`docs/briefs/r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) §"TC3 — Strong-normalization TestClaim" (L145-215; L181-185 ownership transition)
- TC1 fixture authority: [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) §216 (Director ratification of #1179 direction)
- TC2 slice-0 hook: [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) L127
- B5 dependency for TC3: [`docs/briefs/r2-release-b5-loop-construction-closure-audit-worker.md`](r2-release-b5-loop-construction-closure-audit-worker.md)
- INVARIANTS substrate-fact-introduction procedure: [`INVARIANTS.md`](../../INVARIANTS.md) §P1 (TC3 introduction path)
- P4 Decidability (TC3 formal home): [`INVARIANTS.md`](../../INVARIANTS.md) §P4
