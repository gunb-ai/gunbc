# R3 Gate 87 Lens-Cementing Test-Discipline Dispatch — 2026-05-13

**Owner:** Verification Mgr session `lively-raven-354`.

**Purpose:** decompose `lens_cementing_test_discipline_complete` into concrete child work that preserves the lens-completeness invariant while gate #87 stays tied to the `regen.dag` registry corpus.

This is a dispatch artifact, not a second gate authority. The acceptance authority remains `docs/r3-structure.md` §"T-Tests-As-Data-Completeness" and `docs/r3-program-plan.md` §1.8 row #87. Pattern details live in `docs/briefs/r3-v-cluster-m-87-cementing-worker.md` §7 and `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`.

## Dispatch Invariant

For every `LensRegistryEntry` in `src/v3/compiler/regen.dag`, the gate-87 corpus must have one visible cementing path:

- a `.dag` receipt under `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag`;
- runner inclusion through `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`;
- dispatch coverage through `src/v3/compiler/tests/dag/cementing_dispatch.dag`;
- any temporary Rust pin named in `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs` with an explicit dissolution trigger.

Rows outside `regen.dag` remain Band-C / #84 bulk-port scope. Do not use them to prove or reopen gate #87.

## Child Work Items

### G87-D1 — Registry-Invariant Audit

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

Use this checklist whenever you **add** a `LensRegistryEntry` to `src/v3/compiler/regen.dag` or change a lens capability row toward **`BEHAVIORALLY COMPLETE`** in `docs/v3-lens-capability-register.md` (including flips from `PARTIAL`, `STUB`, `PROXY`, or new rows that claim complete behavioral parity).

#### Merge gate (lens-completeness invariant)

A **COMPLETE flip** (register semantics that assert full v3 behavioral parity for an in-R3 generated lens) or a **new regen registry row** that participates in gate **#87** without the **full receipt stack below in the same PR** is **non-mergeable**. Splitting register prose from harnesses, runner inventory, dispatch data, or dissolution notes reopens silent drift against `lens_cementing_test_discipline_complete` and violates the single-authority surfaces in [`r3-cementing-discipline-pattern-2026-05-12.md`](r3-cementing-discipline-pattern-2026-05-12.md) §1.

#### Same-PR artifact bundle (all boxes required)

Work through in order; do not merge until every applicable row is satisfied.

| # | Artifact | Action |
|---|----------|--------|
| 1 | **`docs/v3-lens-capability-register.md`** | Update the lens row so capability class, v2 counterpart column, and `BEHAVIORALLY COMPLETE` intent match the regen row and the predicate choice in the table below. |
| 2 | **`src/v3/compiler/regen.dag`** | Add or adjust the `LensRegistryEntry` (`name`, lens `file`, and any metadata the registry workflow requires) so it matches the capability register row. |
| 3 | **`src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag`** | Add or extend the per-lens harness file; path must be exactly `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag` so it parses into `R3_GATE_87_CEMENTING_REGEN_SUITES` via [`r3_gate_87_cementing_regen_runner_suites.rs`](../../src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs). |
| 4 | **`R3_GATE_87_CEMENTING_REGEN_SUITES`** in `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs` | Add or update the `include_str!` row, harness path, suite id, and claim name list for the lens. This table is the **only** merge-visible inventory for which `t_pb_b_1_dag_runner_test` executes gate-#87 harnesses. |
| 5 | **`src/v3/compiler/tests/dag/cementing_dispatch.dag`** | Extend `cementing_band_c_v2_complete_receipts` (or the successor projection list) so `CementingDispatchMatchesProjection` continues to match capability register + `regen.dag` for every **real-v2-complete** row that requires a paired `TemporaryRustModule` receipt; for rows that cement entirely in `.dag`, ensure the `DagHarness` stem matches a runner row per [`cementing_dispatch.dag`](../../src/v3/compiler/tests/dag/cementing_dispatch.dag) module comments. |
| 6 | **`src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`** | If the `.dag` harness uses `Compiles` or a narrower predicate because the full expected carrier is not yet authorable as test data, add or update the **named Rust pin** and document a **dissolution trigger** (missing carrier or runner capability + owning lane). Silent placeholders are not allowed. |

#### Predicate choice by Band-C class

Pick one path per row; mixed or “best effort” without an explicit placeholder + pin is not sufficient.

| Register / regen situation | Required cementing shape |
|--------------------------|-------------------------|
| **Real v2 counterpart** (parity against shipped v2 lens or frozen oracle on a shared fixture) | Prefer **`DifferentialEquals`** on the shared minimal `Dag` / fixture. If the published v3 carrier differs from what `.dag` test data can express today, land a **reviewed frozen-v2 projection** (Rust or data) with lane-owner signoff and a named path to full `.dag` parity—same PR as the COMPLETE flip. |
| **v3-native** or full behavioral output authorable in `.dag` | **`LensOutputEquals`** (including Int projections where that is the published contract). |
| **Symbolic cost expression** as the published contract | **`SymbolicCostExprEquals`** when literals are authorable; otherwise treat as blocked and use the helper row below. |
| **Helper-only, intentionally narrower harness, or carrier blocked** | **`Compiles`** in the gate-#87 `.dag` harness **plus** paired Rust receipt in `r3_gate_87_lens_cementing_regen_receipts_test` (or other named module listed from dispatch) **plus** explicit **blocker** and **owning lane** in comments or [`r3-cementing-discipline-pattern-2026-05-12.md`](r3-cementing-discipline-pattern-2026-05-12.md) §2.1-style table updates in the same PR stack. |

Rows outside `regen.dag` do not satisfy this gate; do not use this checklist to justify skipping **#87** receipts for in-corpus lenses.

#### Acceptance (G87-D2)

- The checklist requires the **register row**, **regen row**, **per-lens `.dag` receipt**, **runner table**, **dispatch claim**, and **Rust-pin / dissolution note** (when applicable) to move together.
- Real v2 counterpart rows require **`DifferentialEquals`** or a **reviewed frozen-v2 projection** with explicit same-PR documentation.
- v3-native / helper rows require **`LensOutputEquals`**, **`SymbolicCostExprEquals`**, or an explicit **`Compiles`** placeholder **plus** named blocker and pin.
- A COMPLETE flip without the receipt stack is **non-mergeable** for the lens-completeness invariant (stated above).

Verification:

```bash
rg -n "COMPLETE|LensRegistryEntry|R3_GATE_87_CEMENTING_REGEN_SUITES|cementing_dispatch" \
  docs/briefs src/v3/compiler/regen.dag src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs
cargo test -p v3-compiler r3_gate_87
```

### G87-D3 — Placeholder-Dissolution Ledger

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

