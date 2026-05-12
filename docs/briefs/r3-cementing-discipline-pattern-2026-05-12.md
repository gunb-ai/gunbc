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

## §3. Known Hand-Rust Cementing Dispositions

These are not a frozen dispatch inventory; `EXPECTED_HAND_AUTHORED_TEST` remains live authority. They are the named blocker dispositions the #84 cementing-class worker must check before selecting pilots:

| Rust module | Current disposition |
|---|---|
| `tests/integration/cementing/cementing_provenance_origin_integration_test.rs` | v3-native provenance integration shape; port when the expected carrier is cleanly authorable as `LensOutputEquals`. |
| `tests/integration/cementing/complexity_lens_behavioral_completion.rs` | T-LBP complexity cementing; blocker is full `ComplexitySummary` carrier authoring for a data receipt. |
| `tests/integration/cementing/cost_lens_symbolic_consumer_test.rs` | Cost-symbolic cementing; named dissolution path is PR #2769 / `M1_2_8_STRUCTURAL_SYMBOLIC_COST_DATA`. On branches containing PR #2769, re-check live census and existing `.dag` cost receipts before assigning this row. |
| `tests/integration/cementing/memory_peak_cost_basis_demo.rs` | T-LAS gate #94 demonstration; port only when the memory-peak expected carrier and demonstration semantics can be expressed without weakening the test. |

Rows that remain blocked after this check route to D2 with the blocker named. Rows unblocked by #2769 or later substrate/compiler work hand back to V2 for same-PR replacement artifact plus census decrement.

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
