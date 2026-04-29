# PR-D — Cross-target equivalence harness primitives (Evaluator Manager)

**Status:** PROPOSAL — slice 0 structural hook landed (named `TestClaim` + runner-visible suite); deeper harness surfaces deferred with explicit dependencies below.

**Parent:** [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) — sub-lane **Cross-target equivalence harness primitives** + design-lock row **PR-D**.

**R3 consumer:** [`docs/r3-structure.md`](../r3-structure.md) — **T-Verification-L5-Corpus** acceptance `l5_cross_target_consistency` (Rust / Python / Go equivalent runtime behavior on the certification corpus). R2 lands **primitives only**; corpus authoring stays R3.

## Scope (this PR-D program)

1. **Harness contract (R2):** a stable, named structural gate `evaluator_cross_target_equivalence_harness_primitives_landed` so R3 Verification Manager can depend on a single fixture path without inventing parallel claim names.
2. **Primitives, not corpus:** stub programs and `std.verification` predicates already in substrate (`Compiles`, later `DifferentialEquals` / `ForAllTargets` per existing scaffolds in `src/v3/std/verification.dag`). No curated L5 corpus rows at R2.
3. **Algebraic equivalence framing:** L5 compares **computational results** across targets (per `r3-structure.md`), not byte identity. PR-D does not introduce new `TestPredicate` variants — substrate introduction is [`INVARIANTS.md`](../../INVARIANTS.md) §P1 / Grounding Manager if the existing envelope cannot express a needed fact.

## Explicitly out of scope (Worker A / PR-A / PB-Runtime)

- **Runtime `Value` / `EvalFrame` / closed-over environments** — Evaluator PR-A + PB-Runtime §3.2–§3.3 in [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md). PR-D harness receipts must not smuggle runtime-value design; cross-target comparison stays at “compile + emit + observe” boundaries the **existing** predicates already model (`ForAllTargets` scaffold, etc.).
- **Full `fold_lens<C>` / `DimensionReport<C>`** over emitted artifacts — lens consumer shape in [`docs/design-lens-framework.md`](../design-lens-framework.md); L4/L7 harness semantics in `r3-structure.md` §lane table. PR-D does not replace T-Verification-L4-L7-Direct.

## Dependencies (blockers for “strict” L5-shaped slices)

| Dependency | Why |
|---|---|
| **R2-T-Ground-LanguageSpec** (parallel lane) | Per-target primitive realization + typed capability edges for comparable observations across Rust / Python / Go. |
| **All three Shape A targets grounded** | `l5_cross_target_consistency` requires emit paths for each target on the same `Dag` (see `r3-structure.md` T-Verification-L5-Corpus row). |
| **T-Verification-L4-L7-Direct corpus exists** | R3 critical path: Direct seeds L5 coverage suite. |

Until those land, **slice 0** is intentionally thin: the named `TestClaim` wires through `TestRunner` with `Compiles` on a minimal witness program so CI exercises the same compilation path future `DifferentialEquals` / per-target receipts will attach to.

## Structural acceptance — `.dag` hook (authoritative path)

| Gate name | Fixture | Suite `name` |
|---|---|---|
| `evaluator_cross_target_equivalence_harness_primitives_landed` | [`src/v3/compiler/tests/fixtures/r2_evaluator_cross_target_equivalence_harness_primitives.dag`](../../src/v3/compiler/tests/fixtures/r2_evaluator_cross_target_equivalence_harness_primitives.dag) | `r2_evaluator_cross_target_equivalence_harness_primitives_suite` |

**Deferred fixture path (if this file moves):** keep the **declaration name** `evaluator_cross_target_equivalence_harness_primitives_landed` stable; update only the module `std.r2_evaluator_cross_target_equivalence_harness_primitives` path in one PR with Evaluator Manager brief + integration test `include_str!` path co-updated.

## Next implementation slices (ordered)

1. **Slice 1 — oracle stub pair:** second `TestClaim` in the same module using `DifferentialEquals` with subject/oracle both on trivial `.dag`-expressible functions (same as Lane-E pattern in `test_runner.rs`) — proves harness file is the home for differential receipts **without** multi-target emit.
2. **Slice 2 — emit-scoped receipt:** once LanguageSpec + second Shape A target are ready, add a claim using `ForAllTargets` **only** if Director-approved for this fixture (same release-deferral discipline as other scaffold uses); otherwise keep deferral documented here and in manager brief.
3. **Slice 3 — R3 handoff doc:** one paragraph in Verification Manager spawn brief naming `r2_evaluator_cross_target_equivalence_harness_primitives.dag` as the structural import surface for `l5_cross_target_consistency` corpus wiring.

## Dissolution

When `l5_cross_target_consistency` is fully expressible as strict `TestClaim` rows on the certification corpus with no `Compiles`-only stub, delete or downgrade the stub source inside `evaluator_cross_target_equivalence_harness_primitives_landed` to match the strict predicate — do not delete the **claim name** without a Director-amended acceptance table in `r2-evaluator-manager.md`.
