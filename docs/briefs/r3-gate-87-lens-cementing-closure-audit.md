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

## Ratchets

- `cementing_lens_registry_dispatch_test.rs` derives real-v2 complete rows from the capability register plus `src/v3/compiler/regen.dag` and requires the v2 receipt slice to match exactly.
- `r3_gate_87_lens_cementing_regen_receipts_test.rs` requires the regen registry names to match the gate-87 `.dag` runner inventory.
- This audit closed the only v3-native complete gap found during the walk: `variant_payload` had only a compile placeholder; it now has a behavioral Rust receipt for the published carrier.

## Placeholder-Dissolution Ledger

Refresh date: 2026-05-13 (G87-D3). Scope is only the `src/v3/compiler/regen.dag` gate-87 corpus and its paired files under `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag` plus `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`. Rows outside this corpus remain Band-C / #84 handoff scope in `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md` §3.

These rows are not silent exceptions to Band-C. A `Compiles` placeholder proves only that the lens source remains loadable; the paired Rust pin or narrower `.dag` predicate is the temporary receipt until the named carrier/runner capability exists.

| Lens row | Current gate-87 `.dag` predicate | Temporary Rust pin | Missing carrier or runner capability | Owning dissolution lane |
|---|---|---|---|---|
| `cost` | `LensOutputEquals(cost_of, …)` plus `DifferentialEquals(v3_program_cost, v2_oracle_cost, …)` in `t_r3_gate_87_cementing_regen_cost.dag` | None in `r3_gate_87_lens_cementing_regen_receipts_test.rs` for the gate-87 runner; broader frozen `ComplexitySummary` Rust receipt remains in `tests/integration/cementing/complexity_lens_behavioral_completion.rs`. | Full `ComplexitySummary` / nested report-carrier expected literals are not yet authorable as `.dag` `TestClaim` expected data. | T-LBP / gate #73 report-predicate carrier authoring. This is not a gate-87 inventory gap. |
| `cost_symbolic` | `SymbolicCostExprEquals` and `SymbolicCostExprEqualsForBindParam` in `t_r3_gate_87_cementing_regen_cost_symbolic.dag` | None for gate #87; residual `tests/integration/cementing/cost_lens_symbolic_consumer_test.rs` is a host-wrapper pin, not the COMPLETE-row cementing authority. | Host-wrapper retirement for `symbolic_cost_of` / `per_call_pattern_at`, not a missing gate-87 predicate. | Gate #78 host-wrapper retirement lane. |
| `cost_target_realization` | Narrow `LensOutputEquals` Int projection in `t_r3_gate_87_cementing_regen_cost_target_realization.dag` | `r3_gate_87_cost_target_realization_rust_receipt_resolves_type_realization_row` | `.dag` predicate cannot yet assert the resolved `TypeRealization` declaration identity directly; the Int projection only proves the row is present. | PB / declaration-shaped expected-value authoring for registry metadata. |
| `effect_enumeration` | Narrow `LensOutputEquals` Int projection in `t_r3_gate_87_cementing_regen_effect_enumeration.dag` | `r3_gate_87_effect_enumeration_rust_receipt_on_minimal_program` | Stable `.dag` expected values for `EffectEnumerationReport` / `TransactionalPattern` are not yet authorable. | T-Tests-As-Data carrier completeness for sum/record lens outputs. |
| `infer_helpers` | `Compiles` in `t_r3_gate_87_cementing_regen_infer_helpers.dag` | `r3_gate_87_infer_helpers_lens_source_compiles` | No single public `infer_helpers` output carrier is authorable as `.dag` expected data. | PB / compiler-std helper carrier lane. |
| `lower_helpers` | `Compiles` in `t_r3_gate_87_cementing_regen_lower_helpers.dag` | `r3_gate_87_lower_helpers_lens_source_compiles` | No single public `lower_helpers` behavior carrier is authorable as `.dag` expected data. | PB / parse-surface and lower-helper convergence lane. |
| `provenance` | Narrow `LensOutputEquals` Int projection in `t_r3_gate_87_cementing_regen_provenance.dag` | `r3_gate_87_provenance_origin_rust_receipt_on_literal_bind` | The `.dag` receipt covers the literal-origin seam through an Int witness; direct `Origin` expected literals and full per-`Behavior` `Origin` mirror cases are still host-side until sum-typed expected values are complete. | T-Tests-As-Data carrier completeness for sum-typed lens outputs. |
| `structural_resolution` | Narrow `LensOutputEquals` Int projection in `t_r3_gate_87_cementing_regen_structural_resolution.dag` | `r3_gate_87_structural_resolution_rust_receipt_on_literal_program` | `List<UnresolvedArrowBody>` expected literals are not yet stable as authored `.dag` data. | M1(2.8) strict user-module diagnostics / list-carrier expected-value authoring. |
| `unused_parameters` | Narrow `LensOutputEquals` Int projection in `t_r3_gate_87_cementing_regen_unused_parameters.dag` | `r3_gate_87_unused_parameters_rust_receipt_on_literal_program` | `List<UnusedParameter>` expected literals are not yet stable as authored `.dag` data. | M1(2.8) strict user-module diagnostics / list-carrier expected-value authoring. |
| `variant_payload` | `Compiles` in `t_r3_gate_87_cementing_regen_variant_payload.dag` | `r3_gate_87_variant_payload_lens_source_compiles`; unit receipts in `src/v3/compiler/src/lib.rs::variant_payload::tests` | Stable variant-declaration fixture and `VariantPayloadShapeLookup` expected literal are not yet authorable as `.dag` data. | T-PB-B tests-as-data carrier completeness for generated lens output literals. |

Replacement rule: when any row above unblocks, the same PR must replace the placeholder or narrow projection with the stronger `.dag` predicate, delete the matching Rust pin when it no longer covers unique behavior, and keep `R3_GATE_87_CEMENTING_REGEN_SUITES` plus `cementing_dispatch.dag` aligned with `src/v3/compiler/regen.dag`.

## P5 Receipt

Exactly one P5 checkable receipt applies to the `variant_payload` scaffold: explicit deferral. Lane: `T-PB-B`. Concrete roadmap row: `ROADMAP.md` section `Nine lanes`, row `T-PB-B` (Rust-authored tests migrate to `.dag` `TestClaim` declarations). Deferral: the temporary `src/v3/compiler/src/lib.rs::variant_payload::tests` Rust receipts dissolve when `.dag` TestClaims can express `VariantPayloadShapeLookup` expected literals and the gate-87 `variant_payload` harness can replace `Compiles` with the corresponding behavioral `LensOutputEquals` claims.
