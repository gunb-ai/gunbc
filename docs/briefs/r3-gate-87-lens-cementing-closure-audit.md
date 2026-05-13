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

## V3-Native Output-Contract Audit

The G87-C audit pins the output-contract shape for the v3-native / no-v2-counterpart registry
rows that are easy to blur into source-compilation smoke tests:

- `provenance`, `unused_parameters`, and `structural_resolution` use `.dag`
  `LensOutputEquals` claims over narrow Int projections, with paired Rust receipts in
  `r3_gate_87_lens_cementing_regen_receipts_test.rs` covering the public Rust output surface.
- `cost_target_realization` is not a COMPLETE behavioral row, but it is a v3-native registry
  output contract for the ε realization-meta lookup. Its `.dag` receipt is
  `LensOutputEquals(gate87_cost_target_realization_meta_present, ...)`, paired with the Rust
  `type_realization_meta` declaration-resolution receipt.
- `variant_payload` is the only row in this set that cannot yet author the public
  `VariantPayloadShapeLookup` carrier as `.dag` expected data. It therefore remains an explicit
  `Compiles` placeholder plus Rust pins, and its harness names the replacement
  `LensOutputEquals(variant_payload_shape, ..., expected)` contract and dissolution trigger.

Executable ratchet: `r3_gate_87_v3_native_output_contract_harness_predicates_are_pinned` fails if
these four projection rows stop using `LensOutputEquals`, or if the `variant_payload` placeholder
loses the missing-carrier / Rust-pin / replacement-contract breadcrumbs.

## Non-Complete Registered Rows

These registry rows are intentionally outside the complete-lens closure set:
`cost_target_realization` (`N/A`, but covered by the output-contract receipt above),
`effect_enumeration` (`PARTIAL`), `infer_helpers` (`N/A`), `lower_helpers` (`N/A`), and
`parallelism` (`PARTIAL`, covered by the explicit source-compilation placeholder
`t_r3_gate_87_cementing_regen_parallelism.dag` until the public report carrier exposes typed
pairwise-noncommute evidence).

## Ratchets

- `cementing_lens_registry_dispatch_test.rs` derives real-v2 complete rows from the capability register plus `src/v3/compiler/regen.dag` and requires the v2 receipt slice to match exactly.
- `r3_gate_87_lens_cementing_regen_receipts_test.rs` requires the regen registry names to match the gate-87 `.dag` runner inventory.
- This audit closed the only v3-native complete gap found during the walk: `variant_payload` had only a compile placeholder; it now has a behavioral Rust receipt for the published carrier.

## P5 Receipt

Exactly one P5 checkable receipt applies to this PR: explicit deferral. Lane: `T-PB-B`. Concrete roadmap row: `ROADMAP.md` section `Nine lanes`, row `T-PB-B` (Rust-authored tests migrate to `.dag` `TestClaim` declarations). Deferral: the temporary `src/v3/compiler/src/lib.rs::variant_payload::tests` Rust receipts dissolve when `.dag` TestClaims can express `VariantPayloadShapeLookup` expected literals and the gate-87 `variant_payload` harness can replace `Compiles` with the corresponding behavioral `LensOutputEquals` claims.
