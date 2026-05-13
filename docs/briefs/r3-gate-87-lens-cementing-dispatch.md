# R3 Gate #87 Lens Cementing Dispatch

Gate: `lens_cementing_test_discipline_complete`

Authority: `docs/r3-structure.md` §Acceptance row for T-Tests-As-Data-Completeness.

## Scope

Gate #87 is the test-discipline cementing slice for compiler-generated lenses enumerated by `src/v3/compiler/regen.dag`. The invariant is:

- every `LensRegistryEntry` row has one merge-visible `tests/dag/t_r3_gate_87_cementing_regen_<name>.dag` harness;
- every harness is run through `t_pb_b_1_dag_runner_test`;
- every registry row has either a behavioral witness (`DifferentialEquals` / `LensOutputEquals` against its frozen v2 counterpart where that exists) or an explicit Band-C temporary receipt (`Compiles` plus dissolution trigger and Rust pin when no public behavior carrier is authorable yet);
- the runner inventory is derived from the single table in `v3_compiler::r3_gate_87_cementing_regen_runner_suites`, not from a second hand-maintained registry.

Rows outside `src/v3/compiler/regen.dag` remain Band-C / register-ratchet work and are not part of this gate's enumeration surface.

## Dispatched Sub-Items

### 87-A: Registry Inventory And Runner Ratchet

Owner surface:

- `src/v3/compiler/regen.dag`
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`
- `src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs`
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`

Acceptance:

- `regen.dag` `LensRegistryEntry.name` set equals `r3_gate_87_cementing_regen_lens_names_for_runner_table()`.
- Each `R3_GATE_87_CEMENTING_REGEN_SUITES` row points at a real `tests/dag/t_r3_gate_87_cementing_regen_<name>.dag` file and names exactly the suite/claims it expects.
- `t_pb_b_1_dag_runner_test::r3_gate_87_cementing_regen_lens_suites_pass_through_runner` fails closed over every table row by `ClaimResult::Pass` shape, not by stringified messages.
- Adding a new `LensRegistryEntry` without a matching harness/table row fails an integration test.

### 87-B: Complete Behavioral Lens Receipts

Owner surface:

- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost_symbolic.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost_target_realization.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_effect_enumeration.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_provenance.dag`
- matching Rust receipts under `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs` where the public `.dag` behavior carrier is still narrower than the Rust API.

Acceptance:

- Rows with concrete v2 counterparts in `docs/v3-lens-capability-register.md` carry `DifferentialEquals` or `LensOutputEquals` witnesses in the `.dag` harness where authorable today.
- Where a behavioral witness cannot yet be expressed in `.dag`, the Rust receipt pins the same row's behavior against a minimal representative program and the `.dag` harness documents the dissolution trigger.
- No fixture uses textual-message matching as the success condition.

### 87-C: Helper And V3-Native Placeholder Discipline

Owner surface:

- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_infer_helpers.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_lower_helpers.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_variant_payload.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_structural_resolution.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_unused_parameters.dag`
- paired Rust receipts in `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`

Acceptance:

- V3-native, N/A, and helper rows use explicit `Compiles` placeholders only when TESTING.md Band-C permits it.
- Each placeholder names the concrete trigger that retires it into a behavioral `.dag` witness.
- Helper rows that cannot sensibly compare to a v2 oracle still have Rust pins for compileability or public API behavior.
- `structural_resolution` and `unused_parameters` keep narrow executable receipts until strict user-module fixtures can author the list carriers directly.

### 87-D: Cementing Dispatch Projection Coherence

Owner surface:

- `src/v3/compiler/src/cementing_dispatch.rs`
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`
- `src/v3/std/verification.dag`
- `src/v3/compiler/tests/integration/lens_register_correspondence_test.rs`
- `src/v3/compiler/tests/integration/sg6_hand_authored_census_test.rs`

Acceptance:

- Cementing dispatch derives gate-87 PB-B-1 stems from `R3_GATE_87_CEMENTING_REGEN_SUITES`.
- `cementing_dispatch.dag` receipt rows match the structural projection and fail closed on drift.
- `docs/v3-lens-capability-register.md` rows with real v2 counterpart columns stay aligned with the structural Band-C v2-cementing projection.
- SG-6 census keeps `cementing_dispatch.dag` runner wiring visible while avoiding a second cementing inventory.

### 87-E: Final Gate Verification And PR Receipt

Owner surface:

- this brief
- PR description for the implementation PR
- the command receipts from the gate-specific integration tests

Acceptance:

- Run the focused gate-87 integration tests that cover registry inventory, PB-B-1 runner wiring, and cementing dispatch projection.
- PR body names gate #87, lists the dashboard child work items, and records validation.
- The parent manager can close `lens_cementing_test_discipline_complete` using only merge-visible artifacts: this brief, child work-item outcomes, and the PR test receipts.

