# R3 gate #87 lens cementing decomposition

**Gate:** `lens_cementing_test_discipline_complete`

**Authority:** `docs/r3-structure.md` §Acceptance row for gate #87, `TESTING.md`
§"Cementing tests (Band C — lens subsumption)", and
`docs/v3-lens-capability-register.md` discipline rule 6.

## Scope

Gate #87 covers the `LensRegistryEntry` rows in
`src/v3/compiler/regen.dag`. It does not claim closure for every lens-like
`.dag` file in the tree. Non-`regen.dag` lenses remain on the normal Band-C /
register ratchet.

The invariant to preserve is:

1. every generated lens registry row has a merge-visible cementing receipt;
2. rows with a real v2 counterpart use frozen-oracle / `DifferentialEquals`
   style evidence;
3. v3-native or `N/A` rows use `LensOutputEquals` or an equivalent published
   output contract;
4. helper or intentionally partial rows may use explicit `Compiles` placeholders
   only with a dissolution trigger and paired Rust pin where needed;
5. `t_pb_b_1_dag_runner_test`, `cementing_dispatch.dag`, and
   `r3_gate_87_lens_cementing_regen_receipts_test` fail closed on drift.

## Dispatch Slices

### G87-A: registry inventory and runner lockstep

**Files:** `src/v3/compiler/regen.dag`,
`src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`,
`src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`.

**Work:** verify that the live `LensRegistryEntry.name` set exactly matches
`R3_GATE_87_CEMENTING_REGEN_SUITES` and that the suite table remains the single
authority for PB-B-1 harness execution.

**Acceptance:** `r3_gate_87_regen_lens_registry_names_match_fixture_inventory`
passes, and any new registry row requires a same-PR
`t_r3_gate_87_cementing_regen_<name>.dag` harness.

### G87-B: v2-counterpart parity receipts

**Files:** `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost.dag`,
`src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost_symbolic.dag`,
`src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`.

**Work:** keep real-v2-counterpart rows on behavioral evidence:
`DifferentialEquals`, frozen v2-oracle projections, or focused Rust pins when
the `.dag` runner cannot yet author the full published carrier.

**Acceptance:** cost and symbolic-cost receipts exercise the same minimal
fixture through the v3 lens and frozen oracle/projection path, not through a
mere compile check.

### G87-C: v3-native complete-row output contracts

**Files:** `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_provenance.dag`,
`src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_unused_parameters.dag`,
`src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_structural_resolution.dag`,
`src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost_target_realization.dag`,
`src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_variant_payload.dag`.

**Work:** ensure v3-native / `N/A` rows use `LensOutputEquals` or an equivalent
published carrier equality on minimal constructed programs or `Dag` shapes.
Where the available `.dag` predicate is narrower than the Rust API, keep a
paired Rust receipt in `r3_gate_87_lens_cementing_regen_receipts_test.rs`.

**Acceptance:** complete rows fail if the published output contract changes
silently; helper-only compile receipts are not counted as behavioral parity.

### G87-D: partial/helper placeholder discipline

**Files:** `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_infer_helpers.dag`,
`src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_lower_helpers.dag`,
`src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_effect_enumeration.dag`,
`src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`,
`docs/v3-lens-capability-register.md`.

**Work:** audit every `Compiles` placeholder to confirm it is limited to
helper / partial scope, names its dissolution trigger, and is paired with a
Rust compile or contract pin when the shipped API surface is broader than the
`.dag` claim.

**Acceptance:** `effect_enumeration` and helper rows are honest about
`PARTIAL` / `N/A` status; no placeholder is allowed to masquerade as a complete
behavioral receipt.

### G87-E: Band-C dispatch projection and PB-B-1 execution

**Files:** `src/v3/compiler/tests/dag/cementing_dispatch.dag`,
`src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs`,
`src/v3/std/verification.dag`,
`src/v3/compiler/src/cementing_dispatch.rs`.

**Work:** prove that the Band-C receipt list, the capability-register
projection, and the gate #87 runner table agree. This slice owns drift closure
between `.dag` receipt stems and PB-B-1 execution.

**Acceptance:** `cementing_dispatch_suite_passes_through_runner` and the
`CementingDispatchMatchesProjection` claim pass; adding or removing a receipt
without updating the shared projection fails closed.

## Dispatch Notes

- These slices are intentionally parallel except that G87-E should consume the
  final receipt names after G87-A through G87-D settle.
- `parallelism.dag` is a `regen.dag` row, but current register status is
  `PARTIAL`; it belongs to the register ratchet until promoted, at which point
  this same checklist applies in the promotion PR.
- Any child PR that promotes a row to `COMPLETE` must update the prose register,
  `src/v3/std/verification.dag`, the gate #87 harness, the runner table, and
  `cementing_dispatch.dag` in the same PR.
