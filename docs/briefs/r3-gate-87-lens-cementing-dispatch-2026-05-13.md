# R3 Gate #87 Lens Cementing Dispatch — 2026-05-13

**Scope:** decompose `lens_cementing_test_discipline_complete` into concrete worker items for the lens-completeness invariant and dispatch them through dashboard work items.

**Authority:** `docs/r3-structure.md` acceptance for gate #87, `TESTING.md` Band-C cementing discipline, `docs/v3-lens-capability-register.md` discipline rule 6, `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`, and `src/v3/compiler/regen.dag`.

## Current State

Gate #87 has a landed pattern: every `src/v3/compiler/regen.dag` `LensRegistryEntry` is expected to have a `tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag` harness and a `R3_GATE_87_CEMENTING_REGEN_SUITES` row. `cementing_dispatch.dag` and the PB-B-1 runner consume that same inventory so registry, runner, and Band-C receipt drift fail closed.

One live drift exists after the `parallelism` registry row landed: the registry names include `parallelism`, but the gate-#87 harness inventory does not. The first dispatched item closes that direct invariant break. The remaining items do not reopen gate #87 design; they turn the landed discipline into concrete Phase 3 cementing-family work for the hand-Rust residuals listed by the SG-0 census.

## Dispatched Items

### G87-D1 — Parallelism Regen Harness Drift Fix

**Title dispatched:** `G87-D1 parallelism regen cementing harness drift fix`

**Owned surfaces:**
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_parallelism.dag`
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`

**Acceptance:**
- Add the missing `parallelism` gate-#87 `.dag` harness.
- Add the matching runner table row with claim name `cementing_regen_parallelism`.
- Prefer `LensOutputEquals` over `Compiles` if the current public `WorkflowParallelismReport` carrier is authorable as expected data; otherwise use the narrowest explicit placeholder with a dissolution trigger tied to typed pairwise noncommute evidence.
- Verify `cargo test -p v3-compiler r3_gate_87_regen_lens_registry_names_match_fixture_inventory` and the focused gate-#87 runner slice.

### G87-D2 — Cementing Residual Inventory Finalization

**Title dispatched:** `G87-D2 finalize cementing-family SG-0 residual inventory and blockers`

**Owned surfaces:**
- `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`
- `docs/briefs/r3-v-cluster-m-84-class-cementing-bulkport-worker.md`
- `src/v3/compiler/tests/integration/sg0_census_test.rs` comments only, if dispositions changed

**Acceptance:**
- Re-read `EXPECTED_HAND_AUTHORED_TEST` and confirm the complete cementing-family residual set.
- Classify each row as `DifferentialEquals`, `LensOutputEquals`, `SymbolicCostExprEquals`, `Compiles` plus paired receipt, or D2-routed blocker.
- Name the owning unblocker for every resistant row.
- Select the pilot rows for D3 and the bulk rows for D4 without creating a second inventory.

### G87-D3 — Cementing Pilot Port

**Title dispatched:** `G87-D3 port first unblocked cementing residual to .dag TestClaim`

**Owned surfaces:**
- One pilot row selected by G87-D2.
- The replacement `.dag` claim under `src/v3/compiler/tests/dag/`.
- The matching runner or dispatch table entry if the pilot is a `regen.dag` row.
- `src/v3/compiler/tests/integration/sg0_census_test.rs`.

**Acceptance:**
- Replace exactly one hand-Rust cementing residual with an equivalent `.dag` `TestClaim`.
- Remove the replaced hand-Rust census entry in the same PR.
- Preserve one-claim-per-test discipline and avoid stringified diagnostic assertions.
- Record the SG-0 hand-path delta in the PR body.

### G87-D4 — Cementing Bulk Port From Finalized Inventory

**Title dispatched:** `G87-D4 bulk-port remaining unblocked cementing residuals`

**Owned surfaces:**
- Remaining unblocked rows from G87-D2 after the D3 pilot.
- Replacement `.dag` claims and matching dispatch/runner surfaces.
- `src/v3/compiler/tests/integration/sg0_census_test.rs`.

**Acceptance:**
- Port all unblocked cementing-family residuals selected by G87-D2.
- Keep blocked rows out of the bulk PR and route them by their named owning unblocker.
- Land replacement artifacts and SG-0 census decrements together.
- Verify the relevant gate-#87 runner slices and `sg0_census`.

## Non-Goals

- Do not re-decompose gate #87 design or create a parallel cementing inventory.
- Do not classify `cost_lens_symbolic_consumer_test.rs` as a Band-C residual if D2 confirms it still belongs to gate #78 host-wrapper retirement.
- Do not edit unrelated lens application or T-LAS gates as part of the gate-#87 dispatch.
