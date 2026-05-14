---
status: "Mgr-authored worker brief (dispatch — PB-0 cycle 4; **landed** PR #3048)"
authority_parent: "R3 Debt-Paydown Mgr (zesty-boar-261)"
authoring_date: 2026-05-13
director_dispatch: "msg_5d7f3491-889b-4342-bf9f-8a97b8e55431"
program_anchor: "PB-0 cycle 4 — sustained EXPECTED_HAND_AUTHORED_NON_TEST retirement (same bars as cycles 1–2)"
predecessor_brief: "docs/briefs/r3-pb0-non-test-retirement-worker-cycle2.md"
taxonomy_anchor: "docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md"
---

# PB-0 cycle 4 — SG-0 NON_TEST hand-Rust retirement worker brief

## §0. Dispatch position (named scope)

**Director:** `msg_5d7f3491-889b-4342-bf9f-8a97b8e55431` — **operator-ratified parallel cycle** with **swift-bee-15** (cycle 2 / PR #3046) and **cycle 3** (separate work item). **Capacity:** this is **one of three** concurrent PB-0 retirement cycles until a cycle merges and frees headroom.

**Relationship to cycles 1–2:** Identical **acceptance geometry** and **retirement source priority** as [`r3-pb0-non-test-retirement-worker.md`](r3-pb0-non-test-retirement-worker.md) and [`r3-pb0-non-test-retirement-worker-cycle2.md`](r3-pb0-non-test-retirement-worker-cycle2.md).

**Classification trail:** Taxonomy rows for several of these paths are **(b)** today — **named prereq / substrate bundles** in §3; this dispatch still **names** them for operator visibility. Workers **STOP** rather than orphan-drop if a path cannot lawfully retire in one PR under P5; use PR-body **(i–iii)** for a **largest strict subset** + bound follow-up when needed.

### §0.1 Named scope (**landed** — PR #3048)

**Bright-seal-412** merged **PR #3048** (2026-05-14). The **seven** paths below were the **dispatch-time** `NON_TEST` retirement targets (disjoint from cycle-2 + cycle-3 batches); they **no longer appear** in `EXPECTED_HAND_AUTHORED_NON_TEST` on `main`. Row-85 projection for method templates is now **`pb_method_template_projection_generated.rs`** (generated / `REGEN_OUTPUTS`); do **not** resurrect the deleted hand file name as a census path.

| # | Path (historical) |
|---|-------------------|
| 1 | `src/v3/compiler/src/complexity_lattice.rs` |
| 2 | `src/v3/compiler/src/cost_basis_declaration.rs` |
| 3 | `src/v3/compiler/src/enforced_lens_application.rs` |
| 4 | `src/v3/compiler/src/int_literal_ranges.rs` |
| 5 | `src/v3/compiler/src/pb_method_template_projection.rs` |
| 6 | `src/v3/compiler/src/emit/collection_ops_method_contract.rs` |
| 7 | `src/v3/compiler/src/emit_rust_bin_shim.rs` |

**Live census anchor:** `src/v3/compiler/tests/integration/sg0_census_test.rs` — grep for `const EXPECTED_HAND_AUTHORED_NON_TEST` (taxonomy §3 stays row-aligned with the **live** slice).

---

**Dispatch.** Same **Dispatch** paragraph as cycle 2 brief (cadence **5–10** when **`before ≥ 5`**, tail bar, P5 **(i–iii)** split, `feedback_ratchet_only_down`). **This brief additionally pins** the seven paths in §0.1 as the **visible dependency tree** for operator accountability.

**Closure alignment.** Same as cycle 2 — §1.8 gate **#8** `sg0_non_test_zero`; **NON_TEST** slice unless an explicit fragments row is separately justified.

## Read first

- [`docs/audit/r3-sg0-trajectory-tracker.md`](../audit/r3-sg0-trajectory-tracker.md) §2 + §4.1.
- [`docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md`](../audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md) §0–§3.
- [`docs/decisions/r3-row85-method-template-read-surface.md`](../decisions/r3-row85-method-template-read-surface.md) — **pb_method_template_projection** authority context.
- [`ROADMAP.md`](../../ROADMAP.md) T-PB-A row.
- [`INVARIANTS.md`](../../INVARIANTS.md) Dispatch-Discipline **(b)**.
- [`TESTING.md`](../../TESTING.md) gate **#87** context.

## Retirement source priority (Director-ordered)

Same ordered list as [`r3-pb0-non-test-retirement-worker-cycle2.md`](r3-pb0-non-test-retirement-worker-cycle2.md) §“Retirement source priority”. **Cycle-4 scope** is pre-filtered to §0.1.

## Per-PR acceptance checklist

Same checklist as [`r3-pb0-non-test-retirement-worker-cycle2.md`](r3-pb0-non-test-retirement-worker-cycle2.md) §“Per-PR acceptance checklist”.

## STOP / escalate

Same as cycle 2 brief §“STOP / escalate”.

## Deliverable

One squash-ready PR: retire the §0.1 paths (count proof vs **all** `NON_TEST` lines), tests green, SG-0 discipline satisfied, each removed path paired with a **one-line dissolution receipt** in the PR body.
