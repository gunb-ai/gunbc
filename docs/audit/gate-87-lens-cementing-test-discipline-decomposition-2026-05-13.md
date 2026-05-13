# Gate 87 Lens Cementing Test Discipline Decomposition

Date: 2026-05-13

Parent work item: `node://adhoc-b75b3d90-3d0`

Gate: `lens_cementing_test_discipline_complete`

Canonical acceptance text: `docs/r3-structure.md` §"Acceptance" requires every
`LensRegistryEntry` in `src/v3/compiler/regen.dag` to have a cementing receipt
through `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag`,
`t_pb_b_1_dag_runner_test`, and
`r3_gate_87_lens_cementing_regen_receipts_test`.

## Dispatch Items

1. `node://adhoc-8e3bdfe8-821` — gate-87: add missing parallelism
   `LensRegistryEntry` cementing harness and runner receipt.
   - Owns the currently missing `parallelism` row from `regen.dag`.
   - Expected files include the new
     `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_parallelism.dag`
     harness plus the shared runner table update in
     `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`.
   - The receipt should use `LensOutputEquals` or `DifferentialEquals` if the
     published carrier is authorable today; otherwise it must be an explicit
     Band-C `Compiles` placeholder with a concrete dissolution trigger and a
     Rust pin receipt if needed.

2. `node://adhoc-df0de456-87e` — gate-87: strengthen v2-backed `cost` and
   `cost_symbolic` cementing receipts against frozen oracle witness discipline.
   - Owns the rows whose capability-register v2 counterpart column names a real
     v2 source.
   - Must preserve the current frozen v2-oracle / projection discipline rather
     than replacing it with a weaker compile-only harness.
   - Expected checks include the existing `.dag` harnesses and Rust receipts for
     `complexity_lens_behavioral_completion` and
     `cost_lens_symbolic_consumer_test`.

3. `node://adhoc-b8a8f9b9-4ab` — gate-87: cement v3-native and helper
   `LensRegistryEntry` rows with explicit Band-C placeholders and dissolution
   triggers.
   - Owns `cost_target_realization`, `effect_enumeration`, `provenance`,
     `structural_resolution`, `unused_parameters`, `infer_helpers`,
     `variant_payload`, and `lower_helpers`.
   - Must keep helper-only rows honest: compile placeholders are acceptable only
     when they name the future carrier or witness shape that dissolves the
     placeholder.
   - Must keep Rust pin receipts where the `.dag` surface cannot yet express
     the public behavior carrier.

4. `node://adhoc-1cdc09b8-f94` — gate-87: add inventory ratchets proving
   `regen.dag`, the runner table, `.dag` harness files, and cementing dispatch
   cannot drift.
   - Owns the cross-file drift checks across
     `src/v3/compiler/regen.dag`,
     `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`,
     `src/v3/compiler/tests/dag/cementing_dispatch.dag`,
     `src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs`, and
     `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`.
   - The single visible inventory remains
     `R3_GATE_87_CEMENTING_REGEN_SUITES`; new parallel hand lists are out of
     scope unless they are generated from that table.

## Shared Constraints

- Scope is limited to lenses enumerated by `src/v3/compiler/regen.dag`.
  Lenses outside that registry remain under the broader Band-C/register-ratchet
  discipline, not this gate's enumeration surface.
- Each new `TestClaim` or Rust `#[test]` should make one structural claim per
  `TESTING.md` Band-C discipline.
- `BEHAVIORALLY COMPLETE` rows with a non-`N/A` v2 counterpart require a real
  behavioral receipt in the same PR as the status claim.
- Temporary Rust receipts must carry their deletion or `.dag`-port trigger in
  the same visible surface that keeps them wired.

## Closeout Criteria

The parent gate is ready for closeout when:

- every `LensRegistryEntry.name` in `regen.dag` appears in the gate-87 runner
  table-derived inventory;
- every runner-table entry has a corresponding `.dag` harness under
  `src/v3/compiler/tests/dag/`;
- `t_pb_b_1_dag_runner_test` executes every gate-87 suite;
- `r3_gate_87_lens_cementing_regen_receipts_test` fail-closes on registry or
  runner drift;
- `cementing_dispatch.dag` still matches the capability-register v2-complete
  projection; and
- the implementation PR documents which receipts are behavioral witnesses and
  which are temporary placeholders with dissolution triggers.
