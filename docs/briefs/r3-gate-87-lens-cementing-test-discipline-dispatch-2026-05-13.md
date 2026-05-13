# R3 Phase-3 Cementing Test-Discipline Dispatch - 2026-05-13

**Owner:** Verification Mgr (`wise-raven-208`).

**Scope:** decompose the Phase-3 tests-as-data cementing follow-on work that reuses the already-landed gate #87 `lens_cementing_test_discipline_complete` pattern. This brief is a dispatch artifact for child work items; it is not a new pass condition for gate #87.

**Authority:**
- [`docs/r3-structure.md`](../r3-structure.md) acceptance bullet `lens_cementing_test_discipline_complete`.
- [`docs/r3-program-plan.md`](../r3-program-plan.md) row #87, which records gate #87 as `CONSUMER_LANDED + PASSING` and keeps non-`regen.dag` Band-C work outside this gate id.
- [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) sections 5 and 8.3.
- [`TESTING.md`](../../TESTING.md) section "Cementing tests (Band C - lens subsumption)".
- [`docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`](r3-cementing-discipline-pattern-2026-05-12.md).
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`.
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`.
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`.

Gate #87 is closed for its canonical `regen.dag` enumeration surface. This brief only routes follow-on work that either keeps that landed pattern from drifting or hands broader Band-C Rust-test rows to the Phase-3 #84 tests-as-data bulk-port queue. The invariant preserved by all sub-items is unchanged: a lens may not be treated as behaviorally complete without a cementing receipt that would fail on semantic drift.

## Sub-Items

### P3-Cement-A - Landed #87 Pattern Drift Ratchet

**Goal:** keep the closed gate-#87 `regen.dag` inventory mechanically aligned when later PRs edit registry rows.

**Worker deliverable:** audit and, if needed, patch the lockstep among:
- `src/v3/compiler/regen.dag` `LensRegistryEntry` rows.
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs` `R3_GATE_87_CEMENTING_REGEN_SUITES`.
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag`.
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`.
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`.

**Closure checks:**
- New registry rows cannot merge without a runner table entry and per-lens `.dag` harness.
- `cementing_dispatch.dag` continues to evaluate the exact receipt triples required by the register projection.
- `cargo test -p v3-compiler r3_gate_87` passes.

### P3-Cement-B - Real-v2 Receipt Strengthening

**Goal:** strengthen receipt quality for rows already covered by the gate-#87 pattern without broadening the gate-#87 pass condition.

**Worker deliverable:** inspect the current `LensCapabilityV2RealV2` projection and strengthen or port the relevant receipts to data claims where substrate support exists. Today this includes the real-v2/cost-family rows represented by `cost`, `cost_symbolic`, and the temporary Rust `complexity_lens_behavioral_completion` receipt.

**Closure checks:**
- The `.dag` receipt uses `DifferentialEquals` or the existing narrower sanctioned predicate for the row.
- Any temporary Rust receipt names the exact missing data-carrier blocker and remains wired through `cementing_dispatch.dag` as `TemporaryRustModule`.
- The PR body records whether the SG-0 hand-authored census changed.

### P3-Cement-C - V3-native and Helper Receipt Strengthening

**Goal:** strengthen v3-native or helper receipts that are part of the landed `regen.dag` pattern, while preserving explicit blockers where the expected carrier is not authorable.

**Worker deliverable:** audit the `provenance`, `unused_parameters`, `structural_resolution`, `effect_enumeration`, `cost_target_realization`, `infer_helpers`, `lower_helpers`, and `variant_payload` gate-#87 harnesses. Convert `Compiles` placeholders to `LensOutputEquals` or a narrower behavioral predicate where the expected carrier is authorable; otherwise preserve explicit dissolution comments and Rust pin coverage.

**Closure checks:**
- Every placeholder has a named blocker tied to a missing public carrier or compiler capability.
- Every v3-native behavioral claim has either a `.dag` expected-output receipt or a Rust pin receipt cited from `r3_gate_87_lens_cementing_regen_receipts_test.rs`.
- No claim asserts more than one structural fact.

### P3-Cement-D - #84 Band-C Census Handoff

**Goal:** hand broader Band-C hand-Rust cementing rows to the Phase-3 #84 tests-as-data bulk-port queue without recasting them as gate-#87 scope.

**Worker deliverable:** refresh the hand-Rust cementing rows called out by `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md` section 3 and hand actionable rows to the #84 bulk-port queue with predicate class, blocker, and expected SG-0 census delta.

**Closure checks:**
- Rows that can port now have a concrete `.dag` target file and predicate class.
- Rows that cannot port now name one blocker and the owning lane for that blocker.
- No duplicate hand-maintained list of cementing rows is introduced.

## Dispatch Records

The dashboard work items created from this decomposition kept short `G87-*` titles because they consume the gate-#87 pattern machinery. Their canonical scope is the `P3-Cement-*` scope above:
- `G87-A registry/runner/dispatch inventory ratchet for lens cementing completeness`.
- `G87-B real-v2 lens cementing receipts and temporary Rust blocker audit`.
- `G87-C v3-native/helper lens cementing receipt strengthening`.
- `G87-D broad Band-C cementing census handoff to tests-as-data bulk-port`.

Workers should cite this brief in their PRs and update the relevant closure-check bullet in their PR body.

## Verification

This is a docs/dispatch PR. No compiler test is required for this brief alone. Any child PR that changes `src/v3/compiler/regen.dag`, `R3_GATE_87_CEMENTING_REGEN_SUITES`, `cementing_dispatch.dag`, or a gate-#87 harness must run:

```bash
cargo test -p v3-compiler r3_gate_87
```
