# R3 Gate 87 Placeholder-Dissolution Ledger - 2026-05-13

Scope: classify the remaining Gate 87 `Compiles` placeholders and Rust pins, with dissolution blockers and owning lanes. This is a routing ledger, not a second receipt inventory.

Authorities checked:

- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag`
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`
- `src/v3/compiler/tests/integration/sg0_census_test.rs`
- `docs/v3-lens-capability-register.md`
- `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`

## Read This First

`R3_GATE_87_CEMENTING_REGEN_SUITES` is the executable harness inventory for the Gate 87 `.dag` receipts. `EXPECTED_HAND_AUTHORED_TEST` remains the SG-0 hand-Rust census. This ledger only names why entries have not dissolved yet and which lane owns the unblock.

`Compiles` is acceptable only as an explicit placeholder paired with a named dissolution trigger and, where needed, a Rust pin. It must not be re-described as behavioral cementing evidence.

## Gate 87 `.dag` `Compiles` Placeholders

| Lens row | Placeholder file | Current pin | Why `Compiles` remains | Dissolution trigger | Owning lane |
|---|---|---|---|---|---|
| `infer_helpers` | `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_infer_helpers.dag` | `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_infer_helpers_lens_source_compiles` | Helper-only row; no single public behavior/output carrier is authorable for `LensOutputEquals` without mislabeling source compilation as behavior. | Public `infer_helpers` output carrier becomes authorable as `.dag` expected data; replace with `LensOutputEquals` over that carrier and delete the Rust source-compilation pin in the same PR. | PB / compiler-std helper carrier lane; this is not a Verification-only port. |
| `lower_helpers` | `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_lower_helpers.dag` | `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_lower_helpers_lens_source_compiles` | Helper-only row; current generated surface is `expr_span(expr) -> SourceSpan`, and broader lower-helper behavior is still parked on parse / parse-surface convergence. | Public `lower_helpers` behavior carrier becomes authorable as `.dag` expected data; replace with `LensOutputEquals` and delete the Rust source-compilation pin in the same PR. | PB / parse-surface and lower-helper convergence lane. |
| `variant_payload` | `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_variant_payload.dag` | `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_variant_payload_lens_source_compiles` plus unit receipts in `src/v3/compiler/src/lib.rs::variant_payload::tests` | `VariantPayloadShapeLookup` needs a stable variant-declaration fixture and authorable expected literal. Source compilation is intentionally only a placeholder. | Stable variant-declaration fixture and `VariantPayloadShapeLookup` expected literal become authorable as `.dag` data; replace with `LensOutputEquals(variant_payload_shape, ..., expected)` and delete the Rust source-compilation / temporary unit pins in the same PR. | T-PB-B tests-as-data carrier completeness for generated lens output literals. |

Non-`Compiles` Gate 87 harnesses already use behavioral predicates at HEAD:

- `cost`: `LensOutputEquals` plus `DifferentialEquals`.
- `cost_symbolic`: `SymbolicCostExprEquals` / `SymbolicCostExprEqualsForBindParam`.
- `cost_target_realization`: narrow `LensOutputEquals`.
- `effect_enumeration`: narrow `LensOutputEquals` for the published partial contract.
- `provenance`, `structural_resolution`, `unused_parameters`: `LensOutputEquals`.

`parallelism` is present in `src/v3/compiler/regen.dag` but `docs/v3-lens-capability-register.md` marks it `PARTIAL`; it is not a Gate 87 complete-row `Compiles` placeholder. Its completion blocker remains typed left/right pairwise non-commute evidence on `WorkflowParallelismReport` / `ParallelismUnsupportedDetail`, owned by the T-Lens-Behavioral-Parity parallelism lane.

## Rust Pins

| Rust path | Classification | Blocker / dissolution trigger | Owning lane |
|---|---|---|---|
| `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs` | Gate 87 aggregate Rust pin. It ratchets regen registry names against the runner inventory and covers host-side receipts for rows whose full carriers are not yet authorable in `.dag`; its inventory-correspondence test is a single-authority guard, not a per-lens receipt. | Dissolves row-by-row as the corresponding `.dag` `LensOutputEquals` / stronger predicate can express the output carrier. The aggregate file should shrink only when the same PR updates the `.dag` harness and SG-0 census. It fully dissolves only when all host-only carrier projections represented there are expressible and exercised through `.dag` claims. | Mixed: T-PB-B for carrier authoring, T-Substrate / M1(2.8) strict-module carrier authoring, plus the specific lens owner for each row. |
| `src/v3/compiler/tests/integration/cementing/cementing_provenance_origin_integration_test.rs` | Cementing Rust residual outside the simple Gate 87 `Compiles` placeholder set. | `.dag` expected-carrier authoring for the `Origin` sum variants (`NoProducer`, `MissingPort`, `MissingBehavior`, `Source`, `Computed`, `Selected`, `Accumulated`). | Tests-as-data carrier completeness for sum-typed lens outputs (`docs/design-tests-as-data-completeness.md` §C5). |
| `src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs` | Complete-lens frozen-oracle Rust receipt. | `Gate73_ReportPredicateCarriers`: `.dag` `TestClaim` predicates cannot yet consume `ComplexitySummary` / nested `SymbolicCost` report carriers. | T-LBP / gate #73 report-predicate carrier authoring. |
| `src/v3/compiler/tests/integration/cementing/cost_lens_symbolic_consumer_test.rs` | Not a Band-C lens-cementing residual after the Gate 87 symbolic-cost `.dag` receipt landed. It is a host-wrapper pin for the gate #78 path. | Retire `per_call_pattern_at` / `symbolic_cost_of` host wrapper alias-collapse pin. | Gate #78 host-wrapper retirement, not Gate 87 cementing. |
| `src/v3/compiler/tests/integration/cementing/memory_peak_cost_basis_demo.rs` | Lens-application demonstration Rust pin, not a Gate 87 registry placeholder. | Parser-level `apply_lens(cost, DeclarationScope, Enforce { budget: SymbolicCost { dimension: Memory, ... } })` consumer. | T-LAS Slice B / gate #91, with gate #94 as the consumer-side demo. |

## Worker Rule

When a blocker clears, the implementation PR must update these surfaces together:

1. Replace the `Compiles` placeholder with the stronger `.dag` predicate.
2. Remove the paired Rust pin or narrow it to the still-blocked rows only.
3. Update `R3_GATE_87_CEMENTING_REGEN_SUITES` if a harness claim name changes.
4. Update `src/v3/compiler/tests/dag/cementing_dispatch.dag` if the receipt classification changes.
5. Remove the corresponding `EXPECTED_HAND_AUTHORED_TEST` entry when a Rust file dissolves.
6. State the SG-0 hand-path delta in the PR body.

Do not add a parallel "pending port" list. The only live inventories remain the runner table and SG-0 census.
