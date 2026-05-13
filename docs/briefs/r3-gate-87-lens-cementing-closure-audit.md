# R3 Gate 87 Lens Cementing Closure Audit

Audit date: 2026-05-12

Scope: cross-check `docs/v3-lens-capability-register.md`, `src/v3/compiler/regen.dag`, `dsl/gunbc/tools/regen.dag`, `TESTING.md` Band-C, and the gate-87 cementing receipts for every registered lens whose capability-register row is `BEHAVIORALLY COMPLETE`.

## Band-C Rule Applied

`TESTING.md` Band-C splits complete-lens cementing by v2-counterpart class:

- Real v2 counterpart: require a behavioral cementing receipt against the same fixture via v2 oracle / frozen v2 projection, or a documented reviewed projection when the carrier differs.
- `None (v3-native)` / `N/A`: require a behavioral receipt for the published v3 contract on minimal `Dag` shapes or a focused compile-to-DAG fixture.

`dsl/gunbc/tools/regen.dag` is the stage0 regeneration workflow, not the v3 lens registry. The registry authority for generated lenses is `src/v3/compiler/regen.dag`.

## Registered Complete Lenses

| Registry key | Lens file | Band-C class | Receipt status |
|---|---|---|---|
| `cost` | `src/v3/lenses/complexity.dag` | Real v2 counterpart (`src/v2/complexity.dag`) | Green: `.dag` differential receipt `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost.dag` plus temporary Rust frozen `ComplexitySummary` receipt `src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs`. |
| `cost_symbolic` | `src/v3/lenses/cost.dag` | Real v2 counterpart (v2 `CostExpr` embedded in complexity) | Green: temporary Rust frozen symbolic-cost projection receipt `src/v3/compiler/tests/integration/cementing/cost_lens_symbolic_consumer_test.rs`; `.dag` carrier is blocked on nested `SymbolicCost` / `SizeVariable` expected literals. |
| `provenance` | `src/v3/lenses/provenance.dag` | v3-native | Green: `LensOutputEquals` harness `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_provenance.dag` plus Rust compile-to-DAG origin receipt. |
| `structural_resolution` | `src/v3/lenses/structural_resolution.dag` | v3-native | Green: `LensOutputEquals` harness `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_structural_resolution.dag` plus Rust clean-program receipt. |
| `unused_parameters` | `src/v3/lenses/unused_parameters.dag` | v3-native | Green: `LensOutputEquals` harness `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_unused_parameters.dag` plus Rust clean-program receipt. |
| `variant_payload` | `src/v3/lenses/variant_payload.dag` | v3-native | Green after this audit: unit cementing receipts in `src/v3/compiler/src/lib.rs::variant_payload::tests` pin empty, positional-single, single named-field, multi named-field, missing-declaration, and non-product outcomes as one claim per test. The gate-87 `.dag` harness remains `Compiles` until `VariantPayloadShapeLookup` expected literals are authorable as `.dag` data. |
| `idempotency` | `src/v3/lenses/idempotency.dag` | v3-native, non-`regen.dag` row | Green: `src/v3/compiler/tests/integration/m2_lens_idempotency_migration_test.rs` compiles `idempotency.dag`, emits Rust, links it into a harness, and asserts emitted `analyze_workflow` matches `v3_compiler::analyze_workflow` for both registered and missing workflow-effect cases; `m2_lens_idempotency_emit_test.rs` pins Rust/Go/Python emission. This row is outside the gate-87 `regen.dag` runner inventory but inside the COMPLETE v3-native register audit. |

## Non-Complete Registered Rows

These registry rows are intentionally outside the complete-lens closure set: `cost_target_realization` (`N/A`), `effect_enumeration` (`PARTIAL`), `infer_helpers` (`N/A`), and `lower_helpers` (`N/A`).

## Ratchets

- `cementing_lens_registry_dispatch_test.rs` derives real-v2 complete rows from the capability register plus `src/v3/compiler/regen.dag` and requires the v2 receipt slice to match exactly.
- `r3_gate_87_lens_cementing_regen_receipts_test.rs` requires the regen registry names to match the gate-87 `.dag` runner inventory.
- `lens_register_correspondence_test::r3_gate_87_closure_audit_covers_complete_v3_native_register_rows` derives COMPLETE + v3-native rows from the capability register and requires this audit table to list each one, including non-`regen.dag` rows such as `idempotency`.
- This audit closed the `variant_payload` behavioral gap and re-checked the non-`regen.dag` COMPLETE v3-native row: `idempotency` already has a compile/emit/round-trip behavioral receipt, so it is listed here rather than added to the gate-87 runner inventory.

## P5 Receipt

Exactly one P5 checkable receipt applies to this PR: explicit deferral. Lane: `T-PB-B`. Concrete roadmap row: `ROADMAP.md` section `Nine lanes`, row `T-PB-B` (Rust-authored tests migrate to `.dag` `TestClaim` declarations). Deferral: the temporary `src/v3/compiler/src/lib.rs::variant_payload::tests` Rust receipts dissolve when `.dag` TestClaims can express `VariantPayloadShapeLookup` expected literals and the gate-87 `variant_payload` harness can replace `Compiles` with the corresponding behavioral `LensOutputEquals` claims.
