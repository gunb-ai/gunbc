---
status: "Mgr-authored worker brief (dispatch — PB-0 cycle 3 REDO; parallel with cycles 5–6)"
authority_parent: "R3 Debt-Paydown Mgr (zesty-boar-261)"
authoring_date: 2026-05-14
director_dispatch: "msg_b44f4320-0e6c-4c3b-93d7-c6ccbd08f7fe"
program_anchor: "PB-0 cycle 3 REDO — same eight census paths as prior cycle-3 dispatch; substantive (a)/(b)/(c) retirement only"
predecessor_brief: "docs/briefs/r3-pb0-non-test-retirement-worker-cycle3.md"
taxonomy_anchor: "docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md"
---

# PB-0 cycle 3 REDO — SG-0 NON_TEST hand-Rust retirement worker brief

## §0. Dispatch position (named scope)

**Director:** `msg_b44f4320-0e6c-4c3b-93d7-c6ccbd08f7fe` — **UNHOLD** after PR **#3049** §5.1 merge-default + Track A taxonomy authority on `main`. This cycle is **one of three** concurrent PB-0 retirement workers (with **cycle 5** and **cycle 6** briefs) until one merges and frees headroom.

**Anti-pattern (closed PR #3047):** The prior attempt at these paths was **rejected** as **paper census shrink** / `REGEN_OUTPUTS`-only churn without **substantive** retirement geometry. **Do not** repeat: no silent `NON_TEST` line removal paired only with fixture regen or comment edits. PR **#3047** is the canonical **what not to do** lesson; merge-tier discipline is now enforced by the **PR template** §5.1 ratchet (substrate audit axes + **(a)/(b)/(c)** classification per path).

**Track A bar:** For **each** path touched, the PR body names the taxonomy class **(a)** retire-now / **(b)** named prereq + receipt / **(c)** revert-and-restore — **not** **(d)** silent reclassify. Cite the taxonomy row in `docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md` §3.

**Preference:** Path **(b)** **codegen-driver** consolidation (bright-seal-412 / PR **#3048** precedent) where a real substrate hand-off exists; path **(c)** only when **(a)/(b)** are not viable in one lawful merge; **STOP** + P5 **(i–iii)** when needed.

### §0.1 Named `EXPECTED_HAND_AUTHORED_NON_TEST` paths + taxonomy class (this worker only)

**Disjoint from cycle 5 and cycle 6** brief path sets. Retire **these eight** paths in **this** PR (or fewer only if P5 forces a split — then PR body **(i–iii)** + bound follow-up):

| # | Path | Track A §3 class |
|---|------|------------------|
| 1 | `src/v3/compiler/src/r3_fc_lane2_loop_witness.rs` | **(a)** |
| 2 | `src/v3/compiler/src/wall_clock_ratchet_manifest.rs` | **(a)** |
| 3 | `src/v3/compiler/src/memory_peak_cost.rs` | **(b)** (gate **#94** / cost-lens substrate) |
| 4 | `src/v3/compiler/src/omni_shape_b_openapi.rs` | **(b)** (Shape B / `.dag` projection end-to-end) |
| 5 | `src/v3/compiler/src/process_exit.rs` | **(b)** (PB-1 Item 5 / `dsl/std/process.dag`) |
| 6 | `src/v3/compiler/src/self_host_receipt_p0.rs` | **(b)** (DB-8 / schema ownership) |
| 7 | `src/v3/compiler/src/r1c_e_gates.rs` | **(b)** (R1C-E / R1 close scaffold) |
| 8 | `src/v3/compiler/src/emit_rust_roundtrip_fixtures.rs` | **(b)** (R1C-E harness / `.dag` TestClaim migration) |

**Deferred (b) by Director dispatch — do not spend this cycle on:** emit / coercion-class rows owned by **Gap 13 / R3 Grounding** until that lane re-spawns post-deployment per `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id`; **complexity / lens-class** rows until PR **#3053** lands and the R3 Evaluator Mgr lane re-spawns. The eight paths above are **not** in those deferral buckets per current taxonomy §3.

**Live census anchor:** `src/v3/compiler/tests/integration/sg0_census_test.rs` — grep for `const EXPECTED_HAND_AUTHORED_NON_TEST`.

---

**Dispatch.** Same **Dispatch** cadence as [`r3-pb0-non-test-retirement-worker-cycle2.md`](r3-pb0-non-test-retirement-worker-cycle2.md) (strict ratchet, P5 **(i–iii)**, `feedback_ratchet_only_down`). **Mgr review-grep bridge:** treat `REGEN_OUTPUTS` + dissolution-trigger greps as first-class signals — receipts must show **compiler authority** or **generated replacement** moved, not census-only shrink.

**Closure alignment.** §1.8 gate **#8** `sg0_non_test_zero`; **NON_TEST** slice unless an explicit fragments row is separately justified.

## Read first

- [`docs/audit/r3-sg0-trajectory-tracker.md`](../audit/r3-sg0-trajectory-tracker.md) §2 + §4.1.
- [`docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md`](../audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md) §0–§3.
- [`docs/briefs/r3-pb0-non-test-retirement-worker-cycle2.md`](r3-pb0-non-test-retirement-worker-cycle2.md) + [`docs/briefs/r3-pb0-non-test-retirement-worker-cycle4.md`](r3-pb0-non-test-retirement-worker-cycle4.md) (landed path-(b) precedent).
- [`ROADMAP.md`](../../ROADMAP.md) T-PB-A row; [`INVARIANTS.md`](../../INVARIANTS.md) Dispatch-Discipline **(b)**; [`TESTING.md`](../../TESTING.md) gate **#87** context.

## Retirement source priority (Director-ordered)

Same ordered list as cycle 2 brief. **Cycle-3 REDO scope** is pre-filtered to §0.1; execute dissolution **in dependency order within the PR** when imports/call graphs demand it.

## Per-PR acceptance checklist

Same checklist as cycle 2 brief §“Per-PR acceptance checklist”, plus: **PR template §5.1** axes satisfied at merge (substrate audit + per-path **(a)/(b)/(c)** in the PR body).

## STOP / escalate

Same as cycle 2 brief §“STOP / escalate”. If the only “progress” is shrinking the census const without the §5.1 classification + receipts bar, **STOP** (same failure mode as closed **#3047**).

## Deliverable

One squash-ready PR: retire the §0.1 paths with **substantive** dissolution receipts; count proof vs **all** `NON_TEST` lines; tests green; SG-0 discipline satisfied.
