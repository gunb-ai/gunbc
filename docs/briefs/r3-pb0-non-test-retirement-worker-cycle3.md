---
status: "Mgr-authored worker brief (dispatch — PB-0 cycle 3, parallel with cycle 2)"
authority_parent: "R3 Debt-Paydown Mgr (zesty-boar-261)"
authoring_date: 2026-05-13
director_dispatch: "msg_5d7f3491-889b-4342-bf9f-8a97b8e55431"
program_anchor: "PB-0 cycle 3 — sustained EXPECTED_HAND_AUTHORED_NON_TEST retirement (same bars as cycles 1–2)"
predecessor_brief: "docs/briefs/r3-pb0-non-test-retirement-worker-cycle2.md"
taxonomy_anchor: "docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md"
---

# PB-0 cycle 3 — SG-0 NON_TEST hand-Rust retirement worker brief

## §0. Dispatch position (named scope)

**Director:** `msg_5d7f3491-889b-4342-bf9f-8a97b8e55431` — **operator-ratified parallel cycle** with **swift-bee-15** (cycle 2 / PR #3046). **Capacity:** this is **one of three** concurrent PB-0 retirement cycles until a cycle merges and frees headroom.

**Relationship to cycles 1–2:** Identical **acceptance geometry** and **retirement source priority** as [`r3-pb0-non-test-retirement-worker.md`](r3-pb0-non-test-retirement-worker.md) and [`r3-pb0-non-test-retirement-worker-cycle2.md`](r3-pb0-non-test-retirement-worker-cycle2.md). Cycle 3 is another **merged-PR slice** of the same standing program.

**Classification trail:** Prefer taxonomy **(a)** rows where available; this batch leads with the two remaining taxonomy **(a)** rows not assigned to cycle 2, then adds **narrow receipt / scaffold** hosts from the live census comments (same dissolution discipline as cycle 2 — **P5 / same-PR receipts** per `INVARIANTS.md` Dispatch-Discipline **(b)**).

### §0.1 Named `EXPECTED_HAND_AUTHORED_NON_TEST` paths (this worker only)

**Disjoint from cycle-2’s six paths** (`integration_rs_wiring_scan.rs`, `cementing_dispatch.rs`, `r3_gate_87_cementing_regen_runner_suites.rs`, `gunbc_ci.rs`, `lens_declaration_apply.rs`, `lens_t_las_carrier.rs`). Retire **these eight** paths in **this** PR (or fewer only if P5 forces a split — then PR body **(i–iii)** + bound follow-up):

| # | Path |
|---|------|
| 1 | `src/v3/compiler/src/r3_fc_lane2_loop_witness.rs` |
| 2 | `src/v3/compiler/src/wall_clock_ratchet_manifest.rs` |
| 3 | `src/v3/compiler/src/memory_peak_cost.rs` |
| 4 | `src/v3/compiler/src/omni_shape_b_openapi.rs` |
| 5 | `src/v3/compiler/src/process_exit.rs` |
| 6 | `src/v3/compiler/src/self_host_receipt_p0.rs` |
| 7 | `src/v3/compiler/src/r1c_e_gates.rs` |
| 8 | `src/v3/compiler/src/emit_rust_roundtrip_fixtures.rs` |

**Live census anchor:** `src/v3/compiler/tests/integration/sg0_census_test.rs` — grep for `const EXPECTED_HAND_AUTHORED_NON_TEST`.

---

**Dispatch.** Same **Dispatch** paragraph as cycle 2 brief (cadence **5–10** when **`before ≥ 5`**, tail bar, P5 **(i–iii)** split, `feedback_ratchet_only_down`). **This brief additionally pins** the eight paths in §0.1 as the **visible dependency tree** for operator accountability.

**Closure alignment.** Same as cycle 2 — §1.8 gate **#8** `sg0_non_test_zero`; **NON_TEST** slice unless an explicit fragments row is separately justified.

## Read first

- [`docs/audit/r3-sg0-trajectory-tracker.md`](../audit/r3-sg0-trajectory-tracker.md) §2 + §4.1.
- [`docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md`](../audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md) §0–§3 (classification; rows **(b)** in the table still require **named receipts** — do not orphan-drop).
- [`ROADMAP.md`](../../ROADMAP.md) T-PB-A row.
- [`INVARIANTS.md`](../../INVARIANTS.md) Dispatch-Discipline **(b)**.
- [`TESTING.md`](../../TESTING.md) gate **#87** context (distinguish transient cementing wiring vs authority retirement).

## Retirement source priority (Director-ordered)

Same ordered list as [`r3-pb0-non-test-retirement-worker-cycle2.md`](r3-pb0-non-test-retirement-worker-cycle2.md) §“Retirement source priority”. **Cycle-3 scope** is pre-filtered to §0.1; execute dissolution **in dependency order within the PR** when imports/call graphs demand it.

## Per-PR acceptance checklist

Same checklist as [`r3-pb0-non-test-retirement-worker-cycle2.md`](r3-pb0-non-test-retirement-worker-cycle2.md) §“Per-PR acceptance checklist”.

## STOP / escalate

Same as cycle 2 brief §“STOP / escalate”.

## Deliverable

One squash-ready PR: retire the §0.1 paths (count proof vs **all** `NON_TEST` lines, not only this batch), tests green, SG-0 discipline satisfied, each removed path paired with a **one-line dissolution receipt** in the PR body.
