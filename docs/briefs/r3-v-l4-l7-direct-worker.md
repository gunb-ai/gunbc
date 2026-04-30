# R3 Lane 1 — T-Verification-L4-L7-Direct Worker Brief

**Status:** STANDBY — gates on R2-Evaluator PR-A.3 implementation carriers + PR-B body evaluator landing. Brief authored at R3 Verification Manager spawn (2026-04-30); converts to dispatch-ready when prerequisites fire.

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md) — Lane 1 of 2 owned lanes (per `r3-structure.md` L108 authority).

**R3 lane authority:** [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" row **T-Verification-L4-L7-Direct** (L92).

## Scope (this lane)

Per-target equivalence harness for the **L4** (emit/eval match) + **L7** (algebraic-law witness construction) verification claims. Direct-mode runtime equivalence: compares computational results between emit-target output and `.dag` evaluator result on the certification corpus.

**NOT a `Lens<C>` instance** per codex BLOCKING `f5f63c7d9` — the lens framework's `read: (Dag, Behavior) → Witness<C>` cannot read emitted target artifacts; L4/L7 are runtime equivalence checks, not structural folds. The lane *consumes* `Lens<C>` instances as inputs where useful (e.g., `Lens<SymbolicCost>` for cost-related claims, `Lens<EmissionPathPresent>` for structural pre-checks) but the lane itself is corpus-driven runtime, not structural fold.

## Dependencies (gates)

| Dependency | Source | Why |
|---|---|---|
| **R2-Evaluator PR-A.3 carriers** | [`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) + [`r2-pr-a3-implementation-blocker-audit.md`](r2-pr-a3-implementation-blocker-audit.md) | Need closed strategy carrier + structural memo-key identity to evaluate `.dag` programs at the eval-side of the equivalence check. |
| **R2-Evaluator PR-B body evaluator (eager baseline)** | [`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) | Body evaluator executes `.dag` bodies — without it, no eval-side computational result to compare. |
| **PR-D `DifferentialEquals` scaffold** | [`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md) §slice 1 | Worker B's #1226 scaffold provides the `TestPredicate::DifferentialEquals` runner-wiring primitive this lane consumes. **Already landed** at slice 1; lane standby is on PR-A.3 + PR-B, not PR-D. |
| **Shape A targets emit-ready** | R2-Grounding-Rust (production); R2-Grounding-Python / Go pending | L4 per-target evaluation needs at least 1 emit target on the same `Dag`; full L4 across Shape A requires 3. |

## Implementation slices (when dispatch fires)

1. **Slice 1 — single-target L4 receipt:** one `TestClaim` using `TestPredicate::DifferentialEquals` comparing Rust emit-target output vs `.dag` eval result on a minimal corpus program. Fixture: `src/v3/compiler/tests/fixtures/r3_verification_l4_direct_rust.dag` (proposed). Suite: `r3_verification_l4_direct_suite`.
2. **Slice 2 — multi-target L4:** add per-target receipts as Python / Go grounding lands. Each target adds one `DifferentialEquals` row using existing predicate; no new `TestPredicate` variants.
3. **Slice 3 — L7 algebraic-law witness (early slice; does NOT close Lane 1):** use `TestPredicate::AlgebraicLaw` (already on substrate at `src/v3/std/verification.dag` L189; `AlgebraicLawKind` enum L103) for at least one named law (associativity / commutativity / identity) per the lens-framework I4 + I9 TestClaims. **This is a coverage-seed slice only.** L7 closure per [`docs/r3-structure.md`](../r3-structure.md) L54 (`l7_algebraic_laws_witnessed`) requires every algebra declared in `dsl/std/algebra.dag` to have a runtime-constructed witness for **each of its applicable laws** (associativity, commutativity, identity, distributivity). Subsequent slices grow to full coverage; Lane 1 closure gate is the full r3-structure.md authority, not the seed.

## Structural acceptance — `.dag` hooks (per [`r3-structure.md`](../r3-structure.md) L53-54 authority)

| Gate name | Authority | Fixture (proposed) | Suite |
|---|---|---|---|
| `l4_emit_eval_match` | `r3-structure.md` L53 — every `.dag` program in the certification corpus has emitted target output equal to `.dag` evaluation output (algebraic equality, not byte-equal) | `src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag` | `r3_verification_l4_l7_direct_suite` |
| `l7_algebraic_laws_witnessed` | `r3-structure.md` L54 — every algebra in `dsl/std/algebra.dag` has a runtime-constructed witness for each applicable law (associativity, commutativity, identity, distributivity); `AlgebraicLaw` `TestPredicate` evaluates via Evaluator-constructed witnesses, not host-mediated harness | `src/v3/compiler/tests/fixtures/r3_verification_l7_algebraic_laws.dag` | `r3_verification_l4_l7_direct_suite` |

Both gates close under Lane 1; partial-coverage early slices (slice 1 single-target / slice 3 single-law seed) do NOT satisfy these gates and do NOT close the lane.

**Stability invariant** (per [`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md) §"Deferred fixture path"): the **claim name** stays stable; only update fixture-module path with manager-brief + integration-test co-update in one PR.

**No L5-absorbs-L4 dissolution.** L4 (per THESIS L179: emitted code executes and matches `.dag` evaluation) and L5 (per THESIS L180: same `.dag` produces same behavior across Rust/Python/Go) are categorically different equivalences — L5 passing does not entail any target matching `.dag` eval, so L5 cannot subsume L4. This lane is corpus-driven runtime by design (per codex BLOCKING `f5f63c7d9` — NOT a `Lens<C>` instance) and has no current structural dissolution trigger; it stays as runtime-verification work absorbing per-target evidence as Shape A targets ground.

## Explicitly out of scope

- **L5 cross-target equivalence corpus** — Lane 2 ([`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md)).
- **L6 form coverage** — moved to R2-T-Ground-CrossTarget-Meta per [`r3-structure.md`](../r3-structure.md) L92-93 (NOT in Verification Manager scope).
- **`Lens<SymbolicCost>` authoring** — T-CostLens-Composition under Substrate continuation per Director-locked 2026-04-28; this lane consumes the lens, does not author it.
- **New `TestPredicate` variants** — substrate introduction via [`INVARIANTS.md`](../../INVARIANTS.md) §P1 only if existing scaffold (`DifferentialEquals` / `AlgebraicLaw` / `ForAllTargets` / `LensOutputEquals`) cannot express a future fact.

## Cross-refs

- Parent manager: [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md)
- R3 lane row: [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" T-Verification-L4-L7-Direct (L92)
- Upstream PR-D scaffold: [`docs/briefs/r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md)
- Lens framework consumer: [`docs/design-lens-framework.md`](../design-lens-framework.md)
- THESIS surface: [`THESIS.md`](../../THESIS.md) §"Tier 3 — Verification from structure" (L4 + L7 claims)
