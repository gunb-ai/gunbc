# R3 Wave-1 S5 — Lens behavioral parity demonstration (#73)

**Owner**: Wave-1 Substrate worker
**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Authoring date**: 2026-05-12

---

## §0. Status — DISPATCH-READY

Closes gate #73 `lens_behavioral_parity_demonstration` — all 4 R3 lenses vs frozen v2-oracle snapshot.

## §1. Scope

Demonstrate behavioral parity between each of the 4 R3 lenses and their frozen v2-oracle snapshot:

Per `docs/v3-lens-capability-register.md` (lens enumeration authority), the 4 R3 lenses with v2-oracle parity candidates are:

1. **`src/v3/lenses/complexity.dag`** — COMPLETE per register; v2 counterpart `src/v2/complexity.dag` (5488L). Cementing receipt at `src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs` (frozen oracle while `.dag` TestClaims can't express `ComplexitySummary` / nested `SymbolicCost` directly).
2. **`src/v3/lenses/cost.dag`** — COMPLETE for abstract `SymbolicCost` scope per register; v2 counterpart is `CostExpr` (embedded in v2 `complexity.dag`). Cementing receipt at `src/v3/compiler/tests/integration/cementing/cost_lens_symbolic_consumer_test.rs`.
3. **`src/v3/lenses/idempotency.dag`** — COMPLETE per register; lens consumes `std.effects::lane2_workflow_idempotency_report`. Verify cementing-receipt presence + frozen-oracle parity step before claiming gate #73 closure for this lens.
4. **`src/v3/lenses/parallelism.dag`** — currently STUB (returns `LensSurfacePending` until Stage 2e walk lands). Gate #73 closure for this lens **depends on S1 #81 walker port landing first** (parallelism walker port from `src/v3/compiler/src/workflow_parallelism.rs` → `.dag`). If S1 hasn't merged when this brief dispatches, surface to warm-wolf-698: gate #73 may need to split into 3-lens-immediate + 1-lens-post-S1, or fully gate on S1 landing.

Grep verification step: re-read `docs/v3-lens-capability-register.md` at HEAD before authoring — drift since 2026-05-12 may have promoted parallelism or other rows. Note: `cost_target_realization.dag` (TERMINAL / N/A behavioral) is **NOT** in scope for gate #73 since it has no v2 counterpart; same for `effect_enumeration.dag` (v3-native, PARTIAL) and `infer_helpers.dag` (N/A for parity).

### Acceptance pattern (`LensOutputEquals` receipt)

For each lens, the test claim shape (per `feedback_boundary_enforcement_coherence_test` + TESTING.md):
```
LensOutputEquals {
  lens: <lens-id>
  input: <v2-snapshot-input>
  expected: <v2-oracle-output>
  actual: <v3-lens(input)>
}
```
The 4 receipts together close gate #73.

## §2. STOP conditions

1. **v2-oracle snapshot missing** — if any of the 4 lenses lacks a frozen v2-oracle snapshot file, **STOP** — that's a prerequisite that needs to be surfaced separately, not invented.
2. **Lens enumeration ambiguity** — if `docs/v3-lens-capability-register.md` lists more or fewer than 4 lenses at R3-completion-state, **STOP** — gate's "4-lens" framing assumes a specific enumeration; surface the discrepancy.
3. **Parity failure** — if any lens output diverges from oracle, **STOP** and surface. Do NOT modify the oracle to match the lens; do NOT silently widen the comparison. Divergence is a real finding, not a test-cleanup task.

## §3. Verification

- `cargo test --workspace` runs all 4 parity tests green
- Each test produces a `LensOutputEquals` receipt with concrete byte equality
- PR body cites each oracle path + each lens path explicitly

## §4. PR body framing

- Cite gate #73 closure
- Cite `docs/v3-lens-capability-register.md` as the lens enumeration authority
- Inline the 4 LensOutputEquals receipts

## §5. Out of scope

- Modifications to lens implementations (this gate is parity-verification, not lens-evolution)
- v2 oracle regeneration (if oracle is stale, that's separate Bridge-Retirement scope)
- Per-lens optimization / refactoring

## §6. Reference

- `docs/v3-lens-capability-register.md` — lens enumeration authority
- `docs/r3-remaining-work-dependency-graph.md:131` — gate-row metadata
- `feedback_boundary_enforcement_coherence_test` — fail-closed parity-test pattern
- TESTING.md band-A — structural test-claim discipline
