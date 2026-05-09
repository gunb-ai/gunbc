# PR-D — Cross-target equivalence harness primitives (Evaluator Manager)

**Status:** PROPOSAL — **slice 0** opens the named `TestClaim` + runner-visible suite hook (`evaluator_cross_target_equivalence_harness_primitives_landed`); **slice 1** adds `evaluator_cross_target_equivalence_harness_primitives_differential_scaffold` (`TestPredicate::DifferentialEquals` + fixture-local Lane-E stub pair), same suite. Deeper harness surfaces deferred with explicit dependencies below. Use **landed** in manager-brief consumption rows only **after** the relevant slice merges (keeps open-PR status language honest).

**Parent:** [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) — sub-lane **Cross-target equivalence harness primitives** + design-lock row **PR-D**.

**Design lock:** [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md) — semantic equality, corpus curation, oracle validity, float policy, side-effect normalization, and R3 consumption gates. This brief owns harness primitives only.

**R3 consumer:** [`docs/r3-structure.md`](../r3-structure.md) — **T-Verification-L5-Corpus** acceptance `l5_cross_target_consistency` (Rust / Python / Go equivalent runtime behavior on the certification corpus). R2 lands **primitives only**; corpus authoring stays R3.

## Scope (this PR-D program)

1. **Harness contract (R2):** a stable, named structural gate `evaluator_cross_target_equivalence_harness_primitives_landed` so R3 Verification Manager can depend on a single fixture path without inventing parallel claim names.
2. **Primitives, not corpus:** stub programs and `std.verification` `TestPredicate` variants **already declared** on substrate (`Compiles` for slice 0; **scaffold** variants `ForAllTargets` / `LensOutputEquals` / `DifferentialEquals` at `src/v3/std/verification.dag` ~L147–L181 — each marked 🟡 in-file with a dissolution comment). **Strict L5-shaped harness rows** (multi-target emit/eval parity, certification corpus) remain **ungated until §Dependencies** — this brief does not treat those receipts as grounded. No curated L5 corpus rows at R2.
3. **Algebraic equivalence framing:** L5 compares **computational results** across targets (per `r3-structure.md`), not byte identity. The comparison policy is locked in [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md). PR-D does not introduce new `TestPredicate` variants — substrate introduction is [`INVARIANTS.md`](../../INVARIANTS.md) §P1 / Grounding Manager only if a **future** fact cannot be expressed even after the existing sum + runner wiring is exercised.

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
| `evaluator_cross_target_equivalence_harness_primitives_differential_scaffold` | same | same |

**Slice 1 strengthening:** the differential claim exercises existing `DifferentialEquals` runner wiring (host vs emit cost parity on a bundled witness program). It does **not** assert multi-target emit or L5 corpus behavior.

**Slice 1 dissolution trigger:** `ProgramOutputBind` still bridges output-bind identity through fixture `data` (`output_ref` names the bind the runner looks up in compiled `TestClaim.source`). Dissolve when authored claims carry output-bind identity that resolves entirely inside the compiled program `Dag` instead of through fixture stubs (see `test_runner.rs` `program_input_role` comment on the cross-`Dag` bridge).

**Deferred fixture path (if this file moves):** keep the **declaration name** `evaluator_cross_target_equivalence_harness_primitives_landed` stable; update only the module `std.r2_evaluator_cross_target_equivalence_harness_primitives` path in one PR with Evaluator Manager brief + integration test `include_str!` path co-updated.

## Next implementation slices (ordered)

1. ~~**Slice 1 — oracle stub pair:**~~ **Landed:** second `TestClaim` in the same module using the **existing** `TestPredicate::DifferentialEquals` constructor (substrate sum at `verification.dag`; runner: Lane-E / `test_runner.rs`) with subject/oracle on fixture-local `v3_program_cost` / `v2_oracle_cost` stubs (`miss_int_lookup()` bodies) — proves this fixture file is the home for differential receipts **without** multi-target emit. No new `TestPredicate` variants.
2. **Slice 2 — emit-scoped receipt:** once §Dependencies (LanguageSpec + Shape A targets) are ready, add a claim using the **existing** `TestPredicate::ForAllTargets` constructor **only** if Director-approved for this fixture (same release-deferral discipline as other scaffold uses); otherwise keep deferral documented here and in manager brief. **Does not** assert `ForAllTargets` is absent from substrate — only that **using** it for real cross-target evidence waits on emit infra + approval.
3. **Slice 3 — R3 handoff doc:** one paragraph in Verification Manager spawn brief naming `r2_evaluator_cross_target_equivalence_harness_primitives.dag` as the structural import surface for `l5_cross_target_consistency` corpus wiring.

## Dissolution

When `l5_cross_target_consistency` is fully expressible as strict `TestClaim` rows on the certification corpus with no `Compiles`-only stub, delete or downgrade the stub source inside `evaluator_cross_target_equivalence_harness_primitives_landed` to match the strict predicate — do not delete the **claim name** without a Director-amended acceptance table in `r2-evaluator-manager.md`.
