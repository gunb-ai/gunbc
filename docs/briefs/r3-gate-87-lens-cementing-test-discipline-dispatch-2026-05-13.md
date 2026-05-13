# R3 Gate 87 Lens-Cementing Test-Discipline Dispatch — 2026-05-13

**Owner:** Verification Mgr session `lively-raven-354`.

**Purpose:** decompose `lens_cementing_test_discipline_complete` into concrete child work that preserves the lens-completeness invariant while gate #87 stays tied to the `regen.dag` registry corpus.

This is a dispatch artifact, not a second gate authority. The acceptance authority remains `docs/r3-structure.md` §"T-Tests-As-Data-Completeness" and `docs/r3-program-plan.md` §1.8 row #87. Pattern details live in `docs/briefs/r3-v-cluster-m-87-cementing-worker.md` §7 and `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`.

**Dispatch receipt:** session `fierce-boar-384` dispatched the five child work items below on 2026-05-13. The parent node `node://adhoc-b75b3d90-3d0` is blocked on these children until they close.

## Dispatch Invariant

For every `LensRegistryEntry` in `src/v3/compiler/regen.dag`, the gate-87 corpus must have one visible cementing path:

- a `.dag` receipt under `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag`;
- runner inclusion through `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`;
- dispatch coverage through `src/v3/compiler/tests/dag/cementing_dispatch.dag`;
- any temporary Rust pin named in `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs` with an explicit dissolution trigger.

Rows outside `regen.dag` remain Band-C / #84 bulk-port scope. Do not use them to prove or reopen gate #87.

## Child Work Items

### G87-D1 — Registry-Invariant Audit

Dashboard node: `node://adhoc-98651eef-7ed`.

Audit `src/v3/compiler/regen.dag`, `docs/v3-lens-capability-register.md`, `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`, and `src/v3/compiler/tests/dag/cementing_dispatch.dag` for exact row alignment.

Acceptance:

- Every registry lens name appears in the runner inventory.
- Every runner inventory lens has a corresponding `t_r3_gate_87_cementing_regen_<lens>.dag` file.
- `cementing_dispatch.dag` covers the same registry names and does not introduce a parallel hand list.
- Any mismatch is fixed in the same PR or escalated with the specific missing surface.

Verification:

```bash
cargo test -p v3-compiler r3_gate_87
```

### G87-D2 — COMPLETE-Flip Same-PR Checklist

Dashboard node: `node://adhoc-911287f8-aa4`.

Author a checklist for future work that changes a lens row to `BEHAVIORALLY COMPLETE` or adds a new `LensRegistryEntry`.

Acceptance:

- The checklist requires the register row, regen row, per-lens `.dag` receipt, runner table, dispatch claim, and Rust-pin dissolution note to move together.
- Real v2 counterpart rows require `DifferentialEquals` or a reviewed frozen-v2 projection.
- v3-native / helper rows require `LensOutputEquals`, `SymbolicCostExprEquals`, or an explicit `Compiles` placeholder plus named blocker.
- The checklist states that a COMPLETE flip without the receipt stack is non-mergeable for the lens-completeness invariant.

Verification:

```bash
rg -n "COMPLETE|LensRegistryEntry|R3_GATE_87_CEMENTING_REGEN_SUITES|cementing_dispatch" \
  docs/briefs src/v3/compiler/regen.dag src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs
```

### G87-D3 — Placeholder-Dissolution Ledger

Dashboard node: `node://adhoc-816059ea-94a`.

Audit all gate-87 receipts that still use `Compiles` or a host-side Rust pin because the exact expected carrier cannot yet be authored as `.dag` data.

Acceptance:

- Each placeholder names the missing carrier or runner capability.
- Each placeholder names the owning lane that can unblock it.
- No placeholder is treated as a silent exception to Band-C; it is either a temporary receipt or a non-gate-87 residual.
- The result updates the existing pattern/closure-audit docs rather than creating a new independent inventory.

Verification:

```bash
rg -n "Compiles|dissolve|dissolution|placeholder|Rust pin|blocked" \
  docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md \
  docs/briefs/r3-gate-87-lens-cementing-closure-audit.md \
  src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag \
  src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs
```

### G87-D4 — Runner and SG-0 Ratchet Receipt

Dashboard node: `node://adhoc-6d5e0931-e53`.

Verify that the executable tests enforce the gate-87 inventory rather than relying on prose.

Acceptance:

- `t_pb_b_1_dag_runner_test` executes the gate-87 suites through the shared runner table.
- `r3_gate_87_lens_cementing_regen_receipts_test` rejects registry / receipt drift.
- `sg0_census_test.rs` comments continue to point workers at the single cementing inventory and forbid parallel hand lists.
- Any failing or stale ratchet is fixed in the same PR.

Verification:

```bash
cargo test -p v3-compiler r3_gate_87
cargo test -p v3-compiler sg0_census
```

### G87-D5 — Band-C / #84 Handoff Classification

Dashboard node: `node://adhoc-5dbcdd61-f3a`.

Refresh the post-#87 handoff table for remaining hand-Rust cementing-looking tests so #84 workers do not consume gate-87 registry receipts incorrectly.

Acceptance:

- Every `src/v3/compiler/tests/integration/cementing/*.rs` row that remains in `EXPECTED_HAND_AUTHORED_TEST` is classified as gate-87 residual, Band-C bulk-port candidate, host-wrapper retirement, or T-LAS demonstration scope.
- The classification names the owning lane and the expected SG-0 census delta.
- The table stays in `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`; no duplicate inventory is introduced.

Verification:

```bash
rg -n "src/v3/compiler/tests/integration/cementing/|r3_gate_87_lens_cementing_regen_receipts_test|wiring_scanner_test" \
  src/v3/compiler/tests/integration/sg0_census_test.rs \
  docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md
```

## Dispatch Order

D1 and D4 are the fail-closed invariant checks and can run first. D2 and D3 can run in parallel once D1 confirms the current registry surface. D5 is the handoff slice for #84 and should consume D3's placeholder classifications where they overlap.

Completion of these children means gate #87 has a concrete, reviewable discipline package: the registry corpus stays complete, future COMPLETE flips have same-PR receipt requirements, placeholders have named dissolution paths, executable ratchets guard drift, and broader Band-C work is handed to #84 without duplicating authority.
