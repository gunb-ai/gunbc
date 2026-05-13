# R3 Gate 87 Lens Cementing Test Discipline Dispatch - 2026-05-13

**Scope:** `lens_cementing_test_discipline_complete`.

**Status baseline:** gate #87 is already recorded as `CONSUMER_LANDED + PASSING` in `docs/r3-program-plan.md` row #87. This dispatch does not reopen the pre-land Phase-2 checklist in `r3-v-cluster-m-87-cementing-worker.md`; it decomposes the remaining manager-owned assurance work for the landed invariant.

**Invariant to preserve:** every `LensRegistryEntry` in `src/v3/compiler/regen.dag` has a gate-87 cementing receipt path:

- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag`
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`
- `src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs`
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`

Rows with a real v2 counterpart must use frozen v2-oracle / `DifferentialEquals` / equivalent reviewed projections. V3-native, `N/A`, and helper-only scopes follow `TESTING.md` Band-C: `LensOutputEquals` where the carrier is authorable, otherwise an explicit `Compiles` placeholder plus a Rust pin receipt and a named dissolution trigger.

## Dispatch Items

### G87-A: Real-v2 Counterpart Receipt Audit

**Worker task:** audit the `cost` and `cost_symbolic` registry rows from `src/v3/compiler/regen.dag` through `src/v3/std/verification.dag`, `docs/v3-lens-capability-register.md`, `cementing_dispatch.dag`, the per-lens `.dag` harnesses, and the Rust frozen-oracle pin receipts.

**Acceptance:**

- Confirm the register rows that are `LensCapabilityBehavioralComplete` + `LensCapabilityV2RealV2` expand exactly to the Band-C v2 receipt list in `cementing_dispatch.dag`.
- Confirm each corresponding `.dag` receipt is executed by `R3_GATE_87_CEMENTING_REGEN_SUITES`.
- Confirm any residual Rust receipt names a live `.dag` carrier blocker or is no longer necessary and should be routed for deletion.
- Run the smallest relevant gate-87 test slice, or document why the change is audit-only.

### G87-B: V3-native Complete Lens Receipt Audit

**Worker task:** audit complete v3-native capability-register rows. For the current gate-87
regen corpus this includes `provenance`, `structural_resolution`, `unused_parameters`, and
`variant_payload`; the broader closure-audit mirror must also cover non-regen complete rows such
as `idempotency.dag`.

**Acceptance:**

- Confirm each complete v3-native row has a `t_r3_gate_87_cementing_regen_<lens>.dag` receipt.
- For complete v3-native rows outside `regen.dag`, confirm the closure audit names the row and
  records the non-regen receipt basis instead of silently omitting it.
- Confirm each receipt uses `LensOutputEquals` where the expected carrier is authorable, or has a paired Rust pin receipt with a named blocker where it is not.
- Verify there is no silent `Compiles` placeholder on a behavior-bearing complete row without a dissolution trigger.
- Run the smallest relevant gate-87 test slice, or document why the change is audit-only.

### G87-C: Non-complete / Helper Placeholder Discipline Audit

**Worker task:** audit non-complete or helper-only registry rows: `cost_target_realization`, `effect_enumeration`, `infer_helpers`, `lower_helpers`, and any other current `regen.dag` row whose capability-register status is not behaviorally complete with a real v2 counterpart.

**Acceptance:**

- Confirm each row has an explicit gate-87 harness despite being outside the complete-v2 receipt set.
- Confirm any `Compiles` placeholder is limited to helper, partial, or `N/A` scope and names the condition that would allow promotion to a stronger predicate.
- Confirm partial rows are not accidentally represented as complete in `docs/v3-lens-capability-register.md` or `src/v3/std/verification.dag`.
- Run the smallest relevant gate-87 test slice, or document why the change is audit-only.

### G87-D: Ratchet and Inventory Fail-closed Audit

**Worker task:** validate the structural ratchets that make future registry drift fail closed.

**Acceptance:**

- Confirm `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_regen_lens_registry_names_match_fixture_inventory` derives live `regen.dag` names and compares them with `R3_GATE_87_CEMENTING_REGEN_SUITES`.
- Confirm `cementing_dispatch_suite_passes_through_runner` executes `src/v3/compiler/tests/dag/cementing_dispatch.dag`.
- Confirm `.dag` receipt stems in `cementing_dispatch.dag` must be present in the PB-B-1 runner table for `DagHarness` receipts.
- Run `cargo test -p v3-compiler r3_gate_87`, or document any BuildBuddy/CI-only substitute used.

## Manager Closeout

Gate #87 remains closed only if the four slices above agree on the same corpus:

1. `regen.dag` registry names.
2. `verification.dag` capability rows.
3. `docs/v3-lens-capability-register.md` prose mirror.
4. `cementing_dispatch.dag` Band-C receipt list.
5. `R3_GATE_87_CEMENTING_REGEN_SUITES` runner inventory.

Any worker finding a real mismatch should fix the mismatched surfaces in one PR where possible. If the mismatch depends on a missing carrier or predicate substrate, route the blocker upward with the exact row, blocker, and required replacement predicate.
