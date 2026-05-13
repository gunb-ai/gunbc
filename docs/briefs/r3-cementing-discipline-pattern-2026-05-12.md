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
- Follow-on decomposition for dashboard dispatch: [`r3-gate-87-lens-cementing-test-discipline-decomposition-2026-05-13.md`](r3-gate-87-lens-cementing-test-discipline-decomposition-2026-05-13.md).

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

## §3. Known Hand-Rust Cementing Dispositions

These are not a frozen dispatch inventory; the authoritative list is `EXPECTED_HAND_AUTHORED_TEST` in `src/v3/compiler/tests/integration/sg0_census_test.rs`. The table below records the disposition the #84 cementing-class worker must check before selecting pilots — predicate class (§2), blocker if any with owning lane, and the SG-0 hand-path census delta on a successful port. Refresh date: 2026-05-13 (G87-D handoff). On disposition change, update this table together with the live census comment above the same row.

| Rust module | Predicate class (§2) | Blocker / owning lane | SG-0 census delta on port |
|---|---|---|---|
| `tests/integration/cementing/cementing_provenance_origin_integration_test.rs` | C-LensOutEq once unblocked; C-HandRustBlocker today. | Expected-carrier authoring for the `Origin` sum (`NoProducer` / `MissingPort` / `MissingBehavior` / `Source(NodeId)` / `Computed(NodeId)` / `Selected(NodeId)` / `Accumulated(NodeId)`) in `.dag` `TestPredicate` expected values. Owning lane: tests-as-data carrier completeness for sum-typed lens outputs (`docs/design-tests-as-data-completeness.md` §C5). Target `.dag`: `tests/dag/t_r3_gate_87_cementing_regen_provenance.dag` already cements the seam projection; this Rust module pins the per-`Behavior` mirror that the `.dag` receipt cannot yet express. | -1 when the per-`Behavior` mirror moves into the existing `.dag`. |
| `tests/integration/cementing/complexity_lens_behavioral_completion.rs` | C-HandRustBlocker today; C-LensOutEq on unblock. | `Gate73_ReportPredicateCarriers` — `.dag` `TestClaim` predicates cannot yet consume the published `ComplexitySummary` report carrier. Owning lane: T-LBP / `docs/r3-program-plan.md` gate #73 report-predicate carrier authoring. No `.dag` target yet (waiting on the carrier). | -1 on Gate-73 unblock + same-PR move into a new `tests/dag/t_r3_*_complexity_*.dag` receipt. |
| `tests/integration/cementing/cost_lens_symbolic_consumer_test.rs` | NOT a Band-C lens-cementing row — do not select for #84 cementing-class bulk-port. | `cost_symbolic` COMPLETE-row cementing already landed as data at `tests/dag/t_r3_gate_87_cementing_regen_cost_symbolic.dag` (PR #2769 dissolved `M1_2_8_STRUCTURAL_SYMBOLIC_COST_DATA`). The residual is gate #78 `per_call_pattern_at` / `symbolic_cost_of` host-wrapper pin — owning lane: gate #78 host-wrapper retirement, NOT cementing. | 0 from this class. Census decrement happens with the gate #78 host-wrapper retirement, not by reclassifying this row as cementing. |
| `tests/integration/cementing/memory_peak_cost_basis_demo.rs` | C-HandRustBlocker today; C-LensOutEq or C-CompilesHelper on unblock depending on demonstration shape. | Parser-level `apply_lens(cost, DeclarationScope, Enforce { budget: SymbolicCost { dimension: Memory, … } })` consumer. Owning lane: T-LAS Slice B lens-fold consumer, `docs/r3-program-plan.md` gate #91; gate #94 (`memory_peak_cost_basis_demonstrated`) is the consumer-side gate this receipt evidences. No `.dag` target yet (waiting on the parser-level consumer). | -1 on gate #91 unblock + same-PR move into a new `tests/dag/t_r3_*_memory_peak_*.dag` receipt that preserves max-dominance composition and `LensEnforcement` orientation semantics. |

Rows that remain blocked after this check route to D2 with the named blocker. Rows unblocked by later substrate/compiler work hand back to V2 for same-PR replacement artifact plus census decrement under §4. This table is the only hand-maintained disposition surface for these rows; #84 cementing-class workers must not create a parallel inventory.

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
