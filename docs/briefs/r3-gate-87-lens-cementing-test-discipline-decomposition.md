# R3 Gate 87 Lens-Cementing Test Discipline Decomposition

**Date:** 2026-05-13

**Owner:** Verification Mgr decomposition session `swift-fox-199`.

**Status:** Dispatch artifact for follow-through work under the already-landed gate #87 pattern. This document does not reopen `lens_cementing_test_discipline_complete`; it decomposes concrete child work needed to keep the Band-C lens-completeness invariant cemented as later lens/test rows move.

## Authorities

- [`docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`](r3-cementing-discipline-pattern-2026-05-12.md) — current pattern and single-authority surfaces.
- [`docs/briefs/r3-v-cluster-m-87-cementing-worker.md`](r3-v-cluster-m-87-cementing-worker.md) — historical as-shipped #87 index.
- [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §5 / §C5 — predicate taxonomy for cementing.
- [`TESTING.md`](../../TESTING.md) "Cementing tests (Band C — lens subsumption)" — process rule.
- [`docs/v3-lens-capability-register.md`](../v3-lens-capability-register.md) — human-readable mirror.
- `src/v3/std/verification.dag` — `TestPredicate` variants and `lens_capability_register_rows`.
- `src/v3/compiler/regen.dag` — generated-lens registry surface for gate #87.
- `src/v3/compiler/tests/integration/sg0_census_test.rs` — only live Rust-test census authority.

## Invariant To Cement

When a lens row claims behavioral completeness, the test surface must contain an executable cementing receipt that would fail on silent semantic drift:

- real v2 counterpart: `DifferentialEquals` or a documented frozen-oracle projection over the same fixture;
- v3-native / `N/A`: `LensOutputEquals` against the published v3 contract, or a narrow `Compiles` helper receipt only with an explicit dissolution trigger;
- all `regen.dag` registry movements update the runner table, per-lens `.dag` receipt, and dispatch list together.

The source of truth for broad SG-0 Rust-test retirement remains `EXPECTED_HAND_AUTHORED_TEST`; do not create parallel inventories.

## Dispatch Items

### G87-D1 — Regen Registry Authority Sweep

**Goal:** Prove the gate #87 registry corpus remains internally aligned after recent mainline movement.

**Scope:**
- Compare `src/v3/compiler/regen.dag` `LensRegistryEntry` rows with `R3_GATE_87_CEMENTING_REGEN_SUITES`.
- Compare those names with `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag`.
- Compare the Band-C receipt projection in `src/v3/compiler/tests/dag/cementing_dispatch.dag`.
- Fix any drift in the same PR, using the single-authority update rule from the pattern brief.

**Acceptance:** `cargo test -p v3-compiler r3_gate_87` passes, or the PR records the smallest failing command and concrete blocker.

### G87-D2 — Placeholder-To-Behavioral Upgrade Sweep

**Goal:** Tighten any gate #87 `Compiles` placeholder receipts that now have authorable expected carriers.

**Scope:**
- Inspect all `t_r3_gate_87_cementing_regen_*.dag` files whose comments mention `Compiles`, helper-only scope, or a dissolution trigger.
- For each row, decide one of:
  - convert to `LensOutputEquals` / `DifferentialEquals` with concrete expected data;
  - keep `Compiles` and refresh the blocker wording to name the missing carrier or parser surface;
  - route the row to the owning substrate/compiler blocker if it cannot be made behavioral.
- Do not weaken a receipt by replacing behavioral equality with compilation.

**Acceptance:** Every remaining placeholder has a named dissolution trigger, and every newly authorable row has a behavioral predicate.

### G87-D3 — V2-Counterpart Differential Receipt Hardening

**Goal:** Ensure complete rows with a real v2 counterpart are cemented against the v2/frozen-oracle axis, not only by v3 self-consistency.

**Scope:**
- Audit real-v2 rows in the register/regen corpus, including cost-family rows.
- Confirm the receipt is `DifferentialEquals` where the carrier is directly comparable, or a documented frozen projection where the carrier differs.
- Where a Rust frozen-oracle pin remains because `.dag` data cannot express the carrier, name the exact expected-data blocker and owning lane.

**Acceptance:** A reviewer can trace every real-v2 complete row to a same-fixture v2/frozen-oracle receipt and a named `.dag`-port dissolution path if Rust remains.

### G87-D4 — SG-0 Cementing Residual Disposition Refresh

**Goal:** Refresh the live residual list for cementing-family Rust tests without duplicating the SG-0 census.

**Scope:**
- Read `EXPECTED_HAND_AUTHORED_TEST` at dispatch time.
- Reclassify only paths under the cementing family and adjacent gate #87 receipt pins.
- Update the disposition table in `r3-cementing-discipline-pattern-2026-05-12.md` if a row moved, became unblocked, or stopped being cementing-family scope.
- Do not hardcode a total count; cite the census path as authority.

**Acceptance:** The pattern brief has current per-row dispositions for cementing residuals, including blocker owner and expected SG-0 delta on port.

### G87-D5 — First Unblocked Cementing Residual Port

**Goal:** Convert one unblocked cementing-family Rust residual into a `.dag` or generated-test receipt using the Band-C pattern.

**Scope:**
- Consume the refreshed D4 disposition.
- Pick the smallest unblocked row with an expressible predicate.
- Add the replacement `TestClaim` / generated-test artifact.
- Remove the replaced Rust path from `EXPECTED_HAND_AUTHORED_TEST` in the same PR.
- Preserve one-claim-per-test discipline and lane-owner signoff for behavioral fidelity.

**Acceptance:** PR body includes `SG-0 hand-path delta: -1` and the relevant targeted runner/check passes.

## Dispatch Notes

These items are intentionally split so D1-D4 can run in parallel. D5 should consume D4 if the refreshed disposition changes the candidate set; if D4 confirms the current table and an unblocked row is already obvious, D5 may proceed with that cited basis.

Workers must not create GitHub sub-issues for these items. Use dashboard work items and keep branch PRs scoped to the item.
