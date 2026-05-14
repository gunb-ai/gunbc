---
status: "Mgr-authored worker brief (dispatch — PB-0 cycle 5; parallel with cycle 3 REDO + cycle 6)"
authority_parent: "R3 Debt-Paydown Mgr (zesty-boar-261)"
authoring_date: 2026-05-14
director_dispatch: "msg_b44f4320-0e6c-4c3b-93d7-c6ccbd08f7fe"
program_anchor: "PB-0 cycle 5 — sustained EXPECTED_HAND_AUTHORED_NON_TEST retirement (same bars as cycles 2–4)"
predecessor_brief: "docs/briefs/r3-pb0-non-test-retirement-worker-cycle4.md"
taxonomy_anchor: "docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md"
---

# PB-0 cycle 5 — SG-0 NON_TEST hand-Rust retirement worker brief

## §0. Dispatch position (named scope)

**Director:** `msg_b44f4320-0e6c-4c3b-93d7-c6ccbd08f7fe` — **parallel cycle** with **cycle 3 REDO** and **cycle 6**. **Capacity:** one of **three** concurrent PB-0 retirement cycles.

**Taxonomy queue honesty:** Track A §3 currently lists only **three** explicit **(a)** rows total; **two** of them are assigned to **cycle 3 REDO** (`r3_fc_lane2_loop_witness.rs`, `wall_clock_ratchet_manifest.rs`). This batch therefore **leads with** the **remaining** taxonomy **(a)** hub — `cementing_dispatch.rs` — and adds **five** **(b)** **core-host** rows that are **not** in the Director’s **Gap 13 / emit–coercion** deferral bucket and **not** in the **tier3 mirror / evaluator-gated** lane called out for deferral alongside **#3053**. Workers still owe **(a)/(b)/(c)** receipts per path per §5.1; **STOP** rather than orphan-drop.

**Anti-pattern:** Closed PR **#3047** — no paper census shrink or `REGEN_OUTPUTS`-only churn; cite **#3047** in the PR body as the negative lesson if touching regen-adjacent surfaces.

**Preference:** Path **(b)** codegen-driver shape per PR **#3048** when consolidation exists; path **(c)** when honest fallback is required.

**Director note (`msg_f05c2d68`):** `cementing_dispatch.rs` was the **consolidation target** in swift-bee-15 **PR #3046** (large absorb of former `NON_TEST` surfaces into one hub). Retiring that hub in this cycle is coherent with a **gate #87** cementing-class **closure cascade** once predicate / projection substrate owns the walk — if prereqs are unclear, **STOP-AND-PING** with **gate / Gap-tier / substrate** citation (not session-id).

### §0.1 Named `EXPECTED_HAND_AUTHORED_NON_TEST` paths + taxonomy class (this worker only)

**Disjoint from** cycle **3 REDO** (eight paths) and **cycle 6** (six paths). Retire **these six** in **this** PR (or strict subset + **(i–iii)** only when P5 demands):

| # | Path | Track A §3 class |
|---|------|------------------|
| 1 | `src/v3/compiler/src/cementing_dispatch.rs` | **(a)** |
| 2 | `src/v3/compiler/src/diagnostics.rs` | **(b)** |
| 3 | `src/v3/compiler/src/dimension.rs` | **(b)** |
| 4 | `src/v3/compiler/src/pipeline_authority.rs` | **(b)** |
| 5 | `src/v3/compiler/src/post_emit_verifier.rs` | **(b)** |
| 6 | `src/v3/compiler/src/infer.rs` | **(b)** |

**Live census anchor:** `src/v3/compiler/tests/integration/sg0_census_test.rs` — grep for `const EXPECTED_HAND_AUTHORED_NON_TEST`.

---

**Dispatch / closure / checklist / STOP / deliverable:** Same geometry as [`r3-pb0-non-test-retirement-worker-cycle2.md`](r3-pb0-non-test-retirement-worker-cycle2.md) and [`r3-pb0-non-test-retirement-worker-cycle4.md`](r3-pb0-non-test-retirement-worker-cycle4.md); **§5.1** classification per edited path at merge.

## Read first

- [`docs/audit/r3-sg0-trajectory-tracker.md`](../audit/r3-sg0-trajectory-tracker.md) §2 + §4.1.
- [`docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md`](../audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md) §0–§3.
- [`TESTING.md`](../../TESTING.md) gate **#87** — `cementing_dispatch.rs` remains a **CementingDispatchMatchesProjection** host until predicate substrate owns the walk; do **not** orphan-drop gate **#87** wiring.

## Retirement source priority (Director-ordered)

Same ordered list as cycle 2 brief. Scope is pre-filtered to §0.1.

## Per-PR acceptance checklist

Same as cycle 2 brief + **§5.1** per-path **(a)/(b)/(c)** in the PR body.

## STOP / escalate

Same as cycle 2 brief. **Gap-tier cite** for any deferred **(b)** work: `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id`.

## Deliverable

One squash-ready PR: retire §0.1 paths where lawfully batchable; count proof; tests green; SG-0 discipline satisfied; dissolution receipt per removed path.
