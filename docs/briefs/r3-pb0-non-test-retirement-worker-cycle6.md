---
status: "Mgr-authored worker brief (dispatch — PB-0 cycle 6; parallel with cycle 3 REDO + cycle 5)"
authority_parent: "R3 Debt-Paydown Mgr (zesty-boar-261)"
authoring_date: 2026-05-14
director_dispatch: "msg_b44f4320-0e6c-4c3b-93d7-c6ccbd08f7fe"
program_anchor: "PB-0 cycle 6 — sustained EXPECTED_HAND_AUTHORED_NON_TEST retirement (same bars as cycles 2–4)"
predecessor_brief: "docs/briefs/r3-pb0-non-test-retirement-worker-cycle5.md"
taxonomy_anchor: "docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md"
---

# PB-0 cycle 6 — SG-0 NON_TEST hand-Rust retirement worker brief

## §0. Dispatch position (named scope)

**Director:** `msg_b44f4320-0e6c-4c3b-93d7-c6ccbd08f7fe` — **parallel cycle** with **cycle 3 REDO** and **cycle 5**. **Capacity:** one of **three** concurrent PB-0 retirement cycles.

**Queue geometry:** This batch is an **all–(b)** **DAG substrate + CI bin** slice (six paths), **disjoint** from cycle **3 REDO** and cycle **5**. Taxonomy §3 ties each row to **substrate / Tier3 / effects** bundles — expect **P5** pressure; prefer **path (b)** consolidation into generated or substrate-owned surfaces where PR **#3048** precedent applies; **STOP** + **(i–iii)** when a single merge cannot lawfully finish a row.

**Excluded from this dispatch (Director deferral):** **`src/v3/compiler/src/emit/python_target.rs`**, **`emit/rust_target.rs`**, and other **emit / coercion-class** rows deferred to **Gap 13 / R3 Grounding** until lane re-spawn per `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id`; **complexity / lens-class** hosts deferred until PR **#3053** + Evaluator Mgr lane re-spawn. This cycle’s six paths are **not** those deferral buckets.

**Anti-pattern:** Closed PR **#3047** — no census-only shrink.

### §0.1 Named `EXPECTED_HAND_AUTHORED_NON_TEST` paths + taxonomy class (this worker only)

| # | Path | Track A §3 class |
|---|------|------------------|
| 1 | `src/v3/compiler/src/dag/builder.rs` | **(b)** |
| 2 | `src/v3/compiler/src/dag/cardinality_payload.rs` | **(b)** |
| 3 | `src/v3/compiler/src/dag/effects.rs` | **(b)** |
| 4 | `src/v3/compiler/src/dag/ports.rs` | **(b)** |
| 5 | `src/v3/compiler/src/bin/gunbc_ci.rs` | **(b)** |
| 6 | `src/v3/compiler/src/bin/r1c_e_emit_gates.rs` | **(b)** |

**Live census anchor:** `src/v3/compiler/tests/integration/sg0_census_test.rs` — grep for `const EXPECTED_HAND_AUTHORED_NON_TEST`.

### §0.2 Pre-structured STOP anchors (Gap-tier / program; not session-id)

Per Director `msg_f05c2d68` + `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id`: this batch is **100% taxonomy (b)**. If a path cannot lawfully retire in one PR, **STOP** with the **named blocker below** (and taxonomy §3 row cite) — structured signals are an acceptable work product when substrate is not yet landed.

| Path | Cite on STOP (taxonomy-anchored primary blocker) |
|------|--------------------------------------------------|
| `dag/builder.rs` | §1.8 **substrate bundle**; **Tier3 + effects + cardinality** owning program — not an isolated census drop (taxonomy §3). |
| `dag/cardinality_payload.rs` | Same **DAG / Tier3 / cardinality** program bundle as sibling `dag/*` rows. |
| `dag/effects.rs` | Same **DAG / Tier3 / effects** program bundle. |
| `dag/ports.rs` | Same **DAG / Tier3 / ports–effects** program bundle. |
| `bin/gunbc_ci.rs` | **§1.1** bootstrap/regen **Cluster M** canvas + **P5 atomic migration** (taxonomy §3 + §0 prose). |
| `bin/r1c_e_emit_gates.rs` | **§1.1 Cluster M** canvas **plus** **R1C-E / R1 close** / `.dag` **TestClaim** migration surface (taxonomy §3; couples to `emit_rust.rs` program row). |

**Gap 13 / emit–coercion** and **Gap 3 / evaluator–lens** deferrals apply elsewhere on the census; they are **not** the default cite for these six rows unless investigation proves a row-specific coupling (then cite **Gap tier + mechanism**, never a session id).

---

**Dispatch / closure / checklist / STOP / deliverable:** Same geometry as cycle 2 + cycle 4 briefs; **§5.1** per-path **(a)/(b)/(c)** at merge.

## Read first

- [`docs/audit/r3-sg0-trajectory-tracker.md`](../audit/r3-sg0-trajectory-tracker.md) §2 + §4.1.
- [`docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md`](../audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md) §0–§3 (especially DAG module rows + §1.1 cluster note for the two bins).
- [`docs/audit/r3-tier3-computation-mirror-consumer-prestage-2026-05-13.md`](../audit/r3-tier3-computation-mirror-consumer-prestage-2026-05-13.md) — DAG / Tier3 coupling context.

## Retirement source priority (Director-ordered)

Same ordered list as cycle 2 brief. Scope is pre-filtered to §0.1.

## Per-PR acceptance checklist

Same as cycle 2 brief + **§5.1** per-path classification.

## STOP / escalate

Same as cycle 2 brief. If work collapses into **§1.1 bootstrap/regen cluster** atomic migration without Cluster M canvas, **STOP** and return a bound follow-up per taxonomy §3 §1.1 note — do not paper-shrink census. **Every** STOP on a §0.1 path must echo the matching **§0.2** blocker row (Gap-tier / program vocabulary per `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id`).

## Deliverable

One squash-ready PR: retire §0.1 paths where lawfully batchable; count proof; tests green; SG-0 discipline satisfied; dissolution receipt per removed path.
