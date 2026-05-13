# R3 Gate 87 Lens Cementing Closure Audit

Audit date: 2026-05-13 (G87-D3 ledger refresh); prior walk 2026-05-12.

Scope: cross-check `docs/v3-lens-capability-register.md`, `src/v3/compiler/regen.dag`, `dsl/gunbc/tools/regen.dag`, `TESTING.md` Band-C, and the gate-87 cementing receipts for every registered lens whose capability-register row is `BEHAVIORALLY COMPLETE`.

## Placeholder-dissolution ledger (G87-D3)

Canonical row-by-row inventory of **every** gate-#87 harness `Compiles` placeholder and **every** narrow `.dag` witness paired with a Rust pin in `r3_gate_87_lens_cementing_regen_receipts_test.rs` lives in [`r3-cementing-discipline-pattern-2026-05-12.md`](r3-cementing-discipline-pattern-2026-05-12.md) §2.1 (Tables 1–2), including named **carrier** and **owning lane** per entry. Spot-check:

```bash
rg 'Compiles|dissolve|placeholder' \
  src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag \
  src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs \
  docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md \
  docs/briefs/r3-gate-87-lens-cementing-closure-audit.md
```

**G87-D3 audit result:** `R3_GATE_87_CEMENTING_REGEN_SUITES` lists ten harness stems; Tables 1–2 in the discipline brief plus its post-Table-2 note account for all of them. Three `.dag` files use `predicate: Compiles` (`infer_helpers`, `lower_helpers`, `variant_payload`). Five harnesses use behavioral predicates with **Int projections** plus a matching Rust test in `r3_gate_87_lens_cementing_regen_receipts_test.rs` (`effect_enumeration`, `provenance`, `cost_target_realization`, `structural_resolution`, `unused_parameters`). The `cost` and `cost_symbolic` harnesses use full `.dag` differential / symbolic predicates without pins in that module; their extra behavioral supplements stay in `tests/integration/cementing/` per the capability register and the cementing-discipline brief §3.

## Band-C Rule Applied

`TESTING.md` Band-C splits complete-lens cementing by v2-counterpart class:

- Real v2 counterpart: require a behavioral cementing receipt against the same fixture via v2 oracle / frozen v2 projection, or a documented reviewed projection when the carrier differs.
- `None (v3-native)` / `N/A`: require a behavioral receipt for the published v3 contract on minimal `Dag` shapes or a focused compile-to-DAG fixture.

`dsl/gunbc/tools/regen.dag` is the stage0 regeneration workflow, not the v3 lens registry. The registry authority for generated lenses is `src/v3/compiler/regen.dag`.

## Registered Complete Lenses

| Registry key | Lens file | Band-C class | Receipt status |
|---|---|---|---|
| `cost` | `src/v3/lenses/complexity.dag` | Real v2 counterpart (`src/v2/complexity.dag`) | Green: `.dag` differential receipt `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost.dag` plus temporary Rust frozen `ComplexitySummary` receipt `src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs`. |
| `cost_symbolic` | `src/v3/lenses/cost.dag` | Real v2 counterpart (v2 `CostExpr` embedded in complexity) | Green: gate-#87 harness `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost_symbolic.dag` carries full `SymbolicCostExprEquals*` behavioral claims. Temporary Rust `src/v3/compiler/tests/integration/cementing/cost_lens_symbolic_consumer_test.rs` is **not** a substitute for that harness — it pins gate #78 `per_call_pattern_at` / `symbolic_cost_of` host-wrapper parity until gate #78 retires those wrappers (discipline brief §3 `cost_lens_symbolic_consumer_test.rs` row). |
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

## P5 Receipt

Exactly one P5 checkable receipt applies to this PR: explicit deferral. Lane: `T-PB-B`. Concrete roadmap row: `ROADMAP.md` section `Nine lanes`, row `T-PB-B` (Rust-authored tests migrate to `.dag` `TestClaim` declarations). Deferral: the temporary `src/v3/compiler/src/lib.rs::variant_payload::tests` Rust receipts dissolve when `.dag` TestClaims can express `VariantPayloadShapeLookup` expected literals and the gate-87 `variant_payload` harness can replace `Compiles` with the corresponding behavioral `LensOutputEquals` claims.
