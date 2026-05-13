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

## Non-Complete Registered Rows

These registry rows are intentionally outside the complete-lens closure set: `cost_target_realization` (`N/A`), `effect_enumeration` (`PARTIAL`), `infer_helpers` (`N/A`), and `lower_helpers` (`N/A`).

## Placeholder-Dissolution Ledger

Gate #87 allows a temporary `.dag` `Compiles` receipt or a narrow `LensOutputEquals` projection only when the receipt names the missing carrier / runner capability and the lane that can dissolve it. These rows are not silent exceptions to Band-C; they are explicit placeholders paired with Rust pins or narrower runner projections until the stronger `.dag` expected value can be authored.

| Registry key | Current placeholder | Missing carrier / capability | Owning unblock lane | Dissolution receipt |
|---|---|---|---|---|
| `effect_enumeration` | Narrow Int `LensOutputEquals` projection in `t_r3_gate_87_cementing_regen_effect_enumeration.dag` plus `r3_gate_87_effect_enumeration_rust_receipt_on_minimal_program`. | `EffectEnumerationReport` expected literals as stable `.dag` data. | T-Tests-As-Data carrier completeness for report-typed lens outputs. | Replace the Int projection with full-carrier `LensOutputEquals` and remove the Rust projection pin in the same PR. |
| `structural_resolution` | Narrow Int `LensOutputEquals` projection in `t_r3_gate_87_cementing_regen_structural_resolution.dag` plus `r3_gate_87_structural_resolution_rust_receipt_on_literal_program`. | `List<UnresolvedArrowBody>` expected literals as stable `.dag` data. | T-Tests-As-Data carrier completeness for list/sum lens outputs. | Replace the no-violation projection with full-carrier `LensOutputEquals` and remove the Rust projection pin in the same PR. |
| `unused_parameters` | Narrow Int `LensOutputEquals` projection in `t_r3_gate_87_cementing_regen_unused_parameters.dag` plus `r3_gate_87_unused_parameters_rust_receipt_on_literal_program`. | `List<UnusedParameter>` expected literals as stable `.dag` data. | T-Tests-As-Data carrier completeness for list/sum lens outputs. | Replace the no-finding projection with full-carrier `LensOutputEquals` and remove the Rust projection pin in the same PR. |
| `infer_helpers` | `Compiles` in `t_r3_gate_87_cementing_regen_infer_helpers.dag` plus `r3_gate_87_infer_helpers_lens_source_compiles`. | Public `infer_helpers` output carrier authorable as `.dag` expected data. | PB compiler-std helper-carrier lane. | Replace `Compiles` with behavior/output `LensOutputEquals` and delete the source-compilation Rust pin in the same PR. |
| `lower_helpers` | `Compiles` in `t_r3_gate_87_cementing_regen_lower_helpers.dag` plus `r3_gate_87_lower_helpers_lens_source_compiles`. | Public `lower_helpers` behavior carrier authorable as `.dag` expected data. | PB parse-surface and lower-helper convergence lane. | Replace `Compiles` with behavior/output `LensOutputEquals` and delete the source-compilation Rust pin in the same PR. |
| `variant_payload` | `Compiles` in `t_r3_gate_87_cementing_regen_variant_payload.dag` plus `r3_gate_87_variant_payload_lens_source_compiles` and temporary unit receipts in `src/v3/compiler/src/lib.rs::variant_payload::tests`. | Stable variant-declaration fixture and `VariantPayloadShapeLookup` expected literal authorable as `.dag` data. | T-PB-B tests-as-data carrier completeness for generated lens output literals. | Replace `Compiles` with `LensOutputEquals(variant_payload_shape, ..., expected)` and delete both the source-compilation pin and temporary unit receipts in the same PR. |

Non-gate-87 residuals stay out of this table. For example, `cost_lens_symbolic_consumer_test.rs` is now a gate #78 host-wrapper pin because `cost_symbolic` already has its gate-87 `.dag` symbolic-cost receipt; it is classified in `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md` §3, not here.

## Ratchets

- `cementing_lens_registry_dispatch_test.rs` derives real-v2 complete rows from the capability register plus `src/v3/compiler/regen.dag` and requires the v2 receipt slice to match exactly.
- `r3_gate_87_lens_cementing_regen_receipts_test.rs` requires the regen registry names to match the gate-87 `.dag` runner inventory.
- This audit closed the only v3-native complete gap found during the walk: `variant_payload` had only a compile placeholder; it now has a behavioral Rust receipt for the published carrier.

## P5 Receipt

Exactly one P5 checkable receipt applies to this PR: explicit deferral. Lane: `T-PB-B`. Concrete roadmap row: `ROADMAP.md` section `Nine lanes`, row `T-PB-B` (Rust-authored tests migrate to `.dag` `TestClaim` declarations). Deferral: the temporary `src/v3/compiler/src/lib.rs::variant_payload::tests` Rust receipts dissolve when `.dag` TestClaims can express `VariantPayloadShapeLookup` expected literals and the gate-87 `variant_payload` harness can replace `Compiles` with the corresponding behavioral `LensOutputEquals` claims.
