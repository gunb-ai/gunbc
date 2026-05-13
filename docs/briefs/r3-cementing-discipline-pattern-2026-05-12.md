# R3 Cementing-Discipline Pattern Brief — 2026-05-12

**Owner:** Verification Mgr (`clever-tern-670`).

**Role:** post-#87 synthesis surface for Cluster M Phase 3 (#84) cementing-class bulk-port workers. Gate #87 is already `CONSUMER_LANDED + PASSING` in [`docs/r3-program-plan.md`](../r3-program-plan.md) row #87; this brief does not reopen #87 design. It records the landed pattern that #84 workers reuse.

**Substrate-of-truth (cite-and-execute):**
- [`TESTING.md`](../../TESTING.md) section "Cementing tests (Band C — lens subsumption)".
- [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §5 and §C5.
- [`docs/v3-lens-capability-register.md`](../v3-lens-capability-register.md).
- `src/v3/std/verification.dag` `TestPredicate` variants and lens-capability data.
- `src/v3/compiler/regen.dag` `LensRegistryEntry` rows for the gate-#87 corpus.
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs` `R3_GATE_87_CEMENTING_REGEN_SUITES`.
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`.
- Prior #87 worker brief: [`r3-v-cluster-m-87-cementing-worker.md`](r3-v-cluster-m-87-cementing-worker.md), now historical for pattern bring-up.

---

## §0. Pattern Summary

Every `regen.dag` `LensRegistryEntry` whose row is behaviorally complete has a paired `.dag` `TestClaim` cementing receipt. The receipt either compares the v3 lens against a real v2 counterpart on a shared fixture or pins the v3-native behavioral contract against expected data. The runner table `R3_GATE_87_CEMENTING_REGEN_SUITES` is the single inventory used by the runner and dispatch checks.

Band-C cementing is the broader discipline: any claim that v3 subsumes, matches, or replaces a v2 lens must carry a behavioral regression that would fail on silent semantic drift. Gate #87 proves that discipline for the `regen.dag` registry corpus. Phase 3 #84 applies the same discipline to remaining SG-0 hand-Rust test rows where the replacement is a `.dag` TestClaim or generated target-language test.

## §1. Single-Authority Surfaces

These artifacts must move together when a `regen.dag` cementing row changes:

1. `src/v3/compiler/regen.dag` — `LensRegistryEntry` row.
2. `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs` — runner inventory.
3. `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag` — per-lens receipt.
4. `src/v3/compiler/tests/dag/cementing_dispatch.dag` — Band-C dispatch list.

`src/v3/std/verification.dag` and `docs/v3-lens-capability-register.md` are the data/prose mirrors. If a worker needs to edit one surface without the matching surfaces, STOP and ping the Coordinator.

Registry-corpus drift smoke:

```bash
cargo test -p v3-compiler r3_gate_87
```

## §2. Predicate Taxonomy

Use the existing `TestPredicate` surface. Inventing new predicate variants is out of scope for #84 cementing-class bulk port.

| Class | Predicate shape | Use when |
|---|---|---|
| C-DiffEq | `DifferentialEquals` | The row has a real v2 counterpart. |
| C-LensOutEq | `LensOutputEquals` | The row is v3-native and the expected carrier is authorable; scalar Int projections are handled here as `LensOutputEquals` against an Int expected value. |
| C-SymCostEq | `SymbolicCostExprEquals` | The contract is a symbolic cost expression. |
| C-CompilesHelper | `Compiles` plus paired receipt | The row is helper-only or intentionally narrower than a full behavioral predicate. |
| C-HandRustBlocker | hand-Rust receipt with named blocker | The row cannot yet cement as data because a structural/compiler prerequisite is missing. |

Predicate-axis boundaries:
- `BinaryDimensionReportEquals` belongs to Pattern-A DimensionReport comparisons, not cementing-class bulk-port.
- `ProgramGenerator`, `Quantifier`, and `QuantifiedTestClaim` belong to property-based family claims, not per-lens cementing receipts.

### §2.1 Gate-#87 placeholder-dissolution ledger (G87-D3, refresh 2026-05-13)

**Inventory rule:** `predicate: Compiles` appears only under `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_{infer_helpers,lower_helpers,variant_payload}.dag` — three helper registry rows. Every other gate-#87 harness uses a behavioral predicate; several still use a **narrow Int projection** in `.dag` because the full public carrier literal is not stable as authored test data. Those rows pair with a Rust pin in `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs` (same PR must retire both sides on dissolution).

**Verification (spot-check harness + ledger surfaces):**

```bash
rg 'Compiles|dissolve|placeholder' \
  src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag \
  src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs \
  docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md \
  docs/briefs/r3-gate-87-lens-cementing-closure-audit.md
```

#### Table 1 — `.dag` `Compiles` placeholders (complete set)

| Lens (`regen.dag` `name`) | Harness | Interim carrier / evidence | Owning lane | Paired Rust pin | Dissolution trigger |
|---|---|---|---|---|---|
| `infer_helpers` | `tests/dag/t_r3_gate_87_cementing_regen_infer_helpers.dag` | Lens source compiles; no `LensOutputEquals` carrier yet | PB / compiler-std helper output carrier lane | `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_infer_helpers_lens_source_compiles` | Public `infer_helpers` output (e.g. `Lookup<DeclarationId>` / template helper surfaces) authorable as `.dag` expected values → replace claim with `LensOutputEquals` and delete Rust pin. |
| `lower_helpers` | `tests/dag/t_r3_gate_87_cementing_regen_lower_helpers.dag` | Lens source compiles; no behavioral harness carrier yet | PB / parse-surface + lower-helper convergence lane | `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_lower_helpers_lens_source_compiles` | Public `lower_helpers` behavior carrier authorable in `.dag` → `LensOutputEquals` + delete Rust pin. |
| `variant_payload` | `tests/dag/t_r3_gate_87_cementing_regen_variant_payload.dag` | Lens source compiles; `VariantPayloadShapeLookup` literals not in harness | T-PB-B generated lens output literal lane | `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_variant_payload_lens_source_compiles` **and** `src/v3/compiler/src/lib.rs::variant_payload::tests` (per-shape behavioral pins) | Stable fixture + `VariantPayloadShapeLookup` expected literal authorable as `.dag` data → harness `LensOutputEquals(..., expected)`; fold unit tests into `.dag` or delete when redundant; delete `Compiles` + compile-only Rust pin. |

#### Table 2 — Narrow `.dag` harness witness + paired Rust pin (not `Compiles`)

| Lens (`regen.dag` `name`) | Harness | `.dag` witness | Full carrier deferred (name) | Owning lane | Paired Rust pin | Dissolution trigger |
|---|---|---|---|---|---|---|
| `effect_enumeration` | `tests/dag/t_r3_gate_87_cementing_regen_effect_enumeration.dag` | `LensOutputEquals` → `Int` (no-transaction projection) | `EffectEnumerationReport` (facts, coverage, transaction scaffold) | T-PB-B sum/report literal lane; lens row stays **PARTIAL** until resource-threading work (register) | `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_effect_enumeration_rust_receipt_on_minimal_program` | Replace Int shim with full report `LensOutputEquals` when literals + register **PARTIAL**→**COMPLETE** criteria align; retire Rust pin in same PR. |
| `provenance` | `tests/dag/t_r3_gate_87_cementing_regen_provenance.dag` | `LensOutputEquals` → `Int` (literal-origin projection) | `Origin` sum (`Source` / `Computed` / …) | Tests-as-data carrier completeness for sum-typed lens outputs (`design-tests-as-data-completeness.md` §C5) | `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_provenance_origin_rust_receipt_on_literal_bind` | `Origin` literals comparable in `.dag` → direct `LensOutputEquals` on `Origin`; delete Rust origin-class pin. |
| `cost_target_realization` | `tests/dag/t_r3_gate_87_cementing_regen_cost_target_realization.dag` | `LensOutputEquals` → `Int` (meta-present projection) | Full `type_realization_meta` / realization row carrier vs present-bit | T-CostLens-Composition ε / strict-module carrier lane (registry **N/A** row) | `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_cost_target_realization_rust_receipt_resolves_type_realization_row` | Author full realization-meta witness in `.dag` (or stronger predicate) → drop Int shim + Rust name-keyed pin together. |
| `structural_resolution` | `tests/dag/t_r3_gate_87_cementing_regen_structural_resolution.dag` | `LensOutputEquals` → `Int` (no-violation projection) | `List<UnresolvedArrowBody>` | M1(2.8) strict-module / list-carrier authoring lane | `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_structural_resolution_rust_receipt_on_literal_program` | List carrier stable in `.dag` without opaque-body diagnostics → full `LensOutputEquals` on list; delete Rust `check` pin. |
| `unused_parameters` | `tests/dag/t_r3_gate_87_cementing_regen_unused_parameters.dag` | `LensOutputEquals` → `Int` (no-findings projection) | `List<UnusedParameter>` | M1(2.8) strict-module / list-carrier authoring lane | `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_unused_parameters_rust_receipt_on_literal_program` | List carrier stable in `.dag` → full `LensOutputEquals`; delete Rust `UnusedParametersLens::query` pin. |

**Remaining `R3_GATE_87_CEMENTING_REGEN_SUITES` harnesses (not Tables 1–2):** `cost` and `cost_symbolic` ship full behavioral `.dag` predicates (`LensOutputEquals` / `DifferentialEquals`, `SymbolicCostExprEquals*`) with no `Compiles` row and no paired pin in `r3_gate_87_lens_cementing_regen_receipts_test.rs`; Band-C supplements live under `tests/integration/cementing/` per §3 and the closure audit.

**Non-placeholder Rust in this module:** `r3_gate_87_regen_lens_registry_names_match_fixture_inventory` is the regen name ↔ runner-table ratchet (not a lens carrier pin); it dissolves only if the single-authority model in §1 changes, not via carrier authoring.

## §3. Known Hand-Rust Cementing Dispositions

These are not a frozen dispatch inventory; the authoritative list is `EXPECTED_HAND_AUTHORED_TEST` in `src/v3/compiler/tests/integration/sg0_census_test.rs`. The table below records the disposition the #84 cementing-class worker must check before selecting pilots — predicate class (§2), blocker if any with owning lane, and the SG-0 hand-path census delta on a successful port. Refresh date: 2026-05-13 (G87-D5 handoff alignment). On disposition change, update this table together with the live census comment above the same row.

Census coverage check (G87-D5, refresh 2026-05-13): the four rows below are the complete set of paths under `src/v3/compiler/tests/integration/cementing/` that appear in `EXPECTED_HAND_AUTHORED_TEST` at HEAD. No new cementing-test residual rows have landed since the prior refresh; no row in this table has been ported away from Rust since the prior refresh, so the §3 dispositions stand. This brief makes no census-delta claim — it is docs-only.

| Rust module | Predicate class (§2) | Blocker / owning lane | SG-0 census delta on port |
|---|---|---|---|
| `tests/integration/cementing/cementing_provenance_origin_integration_test.rs` | C-LensOutEq once unblocked; C-HandRustBlocker today. | Expected-carrier authoring for the `Origin` sum (`NoProducer` / `MissingPort` / `MissingBehavior` / `Source(NodeId)` / `Computed(NodeId)` / `Selected(NodeId)` / `Accumulated(NodeId)`) in `.dag` `TestPredicate` expected values. Owning lane: tests-as-data carrier completeness for sum-typed lens outputs (`docs/design-tests-as-data-completeness.md` §C5). Target `.dag`: `tests/dag/t_r3_gate_87_cementing_regen_provenance.dag` already cements the seam projection; this Rust module pins the per-`Behavior` mirror that the `.dag` receipt cannot yet express. | -1 when the per-`Behavior` mirror moves into the existing `.dag`. |
| `tests/integration/cementing/complexity_lens_behavioral_completion.rs` | C-HandRustBlocker today; C-LensOutEq on unblock. | `Gate73_ReportPredicateCarriers` — `.dag` `TestClaim` predicates cannot yet consume the published `ComplexitySummary` report carrier. Owning lane: T-LBP / `docs/r3-program-plan.md` gate #73 report-predicate carrier authoring. No `.dag` target yet (waiting on the carrier). | -1 on Gate-73 unblock + same-PR move into a new `tests/dag/t_r3_*_complexity_*.dag` receipt. |
| `tests/integration/cementing/cost_lens_symbolic_consumer_test.rs` | NOT a Band-C lens-cementing row — do not select for #84 cementing-class bulk-port. | `cost_symbolic` COMPLETE-row cementing already landed as data at `tests/dag/t_r3_gate_87_cementing_regen_cost_symbolic.dag` (PR #2769 dissolved `M1_2_8_STRUCTURAL_SYMBOLIC_COST_DATA`). The residual is gate #78 `per_call_pattern_at` / `symbolic_cost_of` host-wrapper pin — owning lane: gate #78 host-wrapper retirement, NOT cementing. | 0 from this class. Census decrement happens with the gate #78 host-wrapper retirement, not by reclassifying this row as cementing. |
| `tests/integration/cementing/memory_peak_cost_basis_demo.rs` | C-HandRustBlocker today; C-LensOutEq or C-CompilesHelper on unblock depending on demonstration shape. | Parser-level `apply_lens(cost, DeclarationScope, Enforce { budget: SymbolicCost { dimension: Memory, … } })` consumer. Owning lane: T-LAS Slice B lens-fold consumer, `docs/r3-program-plan.md` gate #91; gate #94 (`memory_peak_cost_basis_demonstrated`) is the consumer-side gate this receipt evidences. No `.dag` target yet (waiting on the parser-level consumer). | -1 on gate #91 unblock + same-PR move into a new `tests/dag/t_r3_*_memory_peak_*.dag` receipt that preserves max-dominance composition and `LensEnforcement` orientation semantics. |

Rows that remain blocked after this check route to D2 with the named blocker. Rows unblocked by later substrate/compiler work hand back to V2 for same-PR replacement artifact plus census decrement under §4. This table is the only hand-maintained disposition surface for these rows; #84 cementing-class workers must not create a parallel inventory.

### §3.1 Gate-#87 census rows that are NOT Band-C lens-cementing residuals

These rows in `EXPECTED_HAND_AUTHORED_TEST` are named after, or split from, gate-#87 / `cementing_*` infrastructure but are **not** per-lens cementing receipts. They are listed here so #84 cementing-class workers do not select them as bulk-port targets. Their dissolution belongs to the named owning lane, not to cementing-class port:

| Census row | Why not Band-C lens-cementing | Owning dissolution lane |
|---|---|---|
| `tests/integration/common/wiring_scanner_test.rs` | Hand-Rust unit tests for the `tests/integration.rs` wiring scanner — split out of the retired `cementing_lens_registry_dispatch_test.rs` during gate-#87 pattern landing. The scanner is consumed by `cementing_dispatch.rs` (§1) but the assertions cement scanner behavior, not lens output. | T-Tests-As-Data scanner-as-data port (no current gate row); dissolves when the scanner moves into a `.dag` reflection consumer. |
| `tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs` | Single Rust pin paired with the full `tests/dag/t_r3_gate_87_cementing_regen_*.dag` runner inventory; ratchets regen-registry-name ↔ harness-inventory correspondence and host-side carrier projections that `.dag` predicates cannot yet express. Not a per-lens receipt that can be ported in isolation. | T-Substrate / M1(2.8) strict-module carrier authoring; dissolves row-by-row as `LensOutputEquals` carriers (e.g. `ComplexitySummary`, `VariantPayloadShapeLookup`) become authorable in `.dag` test data and the corresponding `Compiles` placeholders flip behavioral. |

These rows must not be re-classified into §3 by a #84 cementing-class worker, and porting them does not satisfy a Band-C cementing-class hand-path decrement. Their census-delta accrues to their respective owning lanes.

## §4. Phase 3 Worker Rule

Each #84 cementing-class worker owns a live inventory slice from `EXPECTED_HAND_AUTHORED_TEST` at dispatch time. Do not use this brief as a frozen path list.

For each migrated row:
1. Port the hand-Rust assertion to the matching predicate class from §2.
2. Add the `.dag` claim under `src/v3/compiler/tests/dag/`.
3. Delete the replaced hand-Rust test or otherwise remove its SG-0 test census entry.
4. Land the replacement artifact and `EXPECTED_HAND_AUTHORED_TEST` decrement in the same PR.
5. Record the PR-body SG-0 hand-path delta.
6. Get lane-owner signoff on behavioral fidelity.

If the row is a gate-#87 `regen.dag` registry row, update all single-authority surfaces from §1 in the same PR.

## §5. Resistant Rows

Rows that cannot cement because they need structural/compiler fixes route to D2 with the blocker named. They are not silent V2 scope and they are not closure-allowed exceptions. If D2 later unblocks the row, it hands the row back to V2 for the same replacement-artifact plus same-PR census-decrement discipline.

## §6. Sequencing

The #87 pattern/dispatch receipt is already landed. Phase 3 cementing-class dispatch waits on:
- Coordinator live inventory.
- Per-entry predicate classification.
- Pilot and bulk-wave split.
- Lane-Mgr signoff routing.

#85 SuiteClaim wrapper work is wrapper-level coupling only; it does not change the per-claim cementing predicate taxonomy above.

## §7. Verification

Workers should run the smallest relevant cementing runner slice locally when practical, and CI/BuildBuddy must run the authoritative checks. For registry-corpus edits, the named local smoke is:

```bash
cargo test -p v3-compiler r3_gate_87
```

Docs-only coordinator changes do not need this command, but any concrete cementing migration does.

---

**End of brief.**
