# R3 Pattern-A — TC1 first executable slice (V1) Worker Brief

**Status:** **HELD on Branch B η-non-vacuity** — Director η non-vacuity (Branch B) 2026-05-06 ([gunbc#828](https://github.com/gunb-ai/gunbc/issues/828)). **Scaffold landed at [PR #2184](https://github.com/gunb-ai/gunbc/pull/2184) with NotYetImplemented sentinel pending E3.c upgrade** per Director (C-modified) ratification at [gunbc#828](https://github.com/gunb-ai/gunbc/issues/828) on 2026-05-07; consumer wiring (η-pair callables + `BinaryDimensionReportEquals` predicate + integration test) cleanly authored under Option 3 hard bars (no `lens_apply` / `eval_substrate_reify` / `reflect_behavior` imports). Sentinel assertion is fail-closed-by-construction — actual implementation that runs WILL fail the NotYetImplemented assertion when Evaluator E3.c ([#1970](https://github.com/gunb-ai/gunbc/issues/1970)) lands, forcing fixture upgrade. §1.8 #11 stays **DECLARED** until E3.c merges and assertion upgrades; flip then is DECLARED → CONSUMER_LANDED → PASSING in one move. Original (pre-scaffold) HELD framing follows. **Q-Reification CLEARED 2026-05-07** ([Option A ratified](https://github.com/gunb-ai/gunbc/pull/2096), `Dag` IS the reflected program; no separate carrier). **Q-PAFS Path A** remains **ACCEPTED** (PR [#1824](https://github.com/gunb-ai/gunbc/pull/1824) merge record on `main`). **V1 (`tc1_eta_equivalence_executable`) unpairs** from Evaluator **E3 Option 3** narrow **argument-opaque** representative slice for **TC1 acceptance** — that shape yields **vacuous** `BinaryDimensionReportEquals` (constant `DimensionReport<C>`); it cannot honestly close the gate against [`r3-v-tc1-eta-equivalence-deeper-analysis.md`](r3-v-tc1-eta-equivalence-deeper-analysis.md) §What TC1 Asserts + §Strict-Fire Extension Surface. **Resume dispatch** only after lens fold consumes `Dag` via `.dag` body authority through Evaluator (real lens-over-Dag fold, non-vacuous η obligation) **or** Director-visible §1.8 / program-plan semantics revision (explicit "plumbing-only" TC1 milestone — **not** ratified 2026-05-06). **Q-Reification gate is cleared**; remaining hold is Branch B η non-vacuity only.

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md) — absorbed formal-grounding / Pattern-A cluster (not a fourth lane; see [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure").

**Ratified scope narrative (engineering brief):** [`docs/briefs/r3-v-tc1-eta-equivalence-deeper-analysis.md`](r3-v-tc1-eta-equivalence-deeper-analysis.md) §"DESIGN — Q-PAFS first executable slice (Pattern-A / TC1)" **Path A** — TC1 **static representative** via **E6-G1.a** (finite Director-visible lens set + fixed eta pair; `BinaryDimensionReportEquals` consumer envelope per [`docs/briefs/r3-v-pattern-a-coverage-rollup.md`](r3-v-pattern-a-coverage-rollup.md)).

**Program plan (single operational authority):** [`docs/r3-program-plan.md`](../r3-program-plan.md) §10.3 — rows **Q-PAFS** + **Q-EVAL-Lens-Fold-First-Slice** **Status** columns hold **policy ACCEPTED (Path A / G1.a)** **and** **implementation supersession**: TC1 V1 **HELD** until Q-Reification + non-vacuous η path (or explicit §1.8 plumbing-only revision — **not** ratified). **This brief elaborates worker mechanics only** — it **does not** override or restate dispatch authority independently of that table (**INVARIANTS** §P2 / modeling-discipline Practice 5).

**Director routing:** **Q-Reification** propagates via Director audit ([gunbc#828](https://github.com/gunb-ai/gunbc/issues/828)); Verification consumes outcome and patches this brief when the §10.3 table moves.

**PR #1844 caveat:** Director-confirmed **strict** argument-opacity (`read`/`validate` ignore `Dag`/`Behavior`; fixed witness) — Branch B stands. If Evaluator revises to **thread behavior-shaped evidence** before report formation despite opaque leaf bodies, re-open Verification analysis before dispatch.

## §1.8 closure predicate (this slice)

| Gate ID | Gate name (canonical) | Target transition |
| --- | --- | --- |
| **#11** | `tc1_eta_equivalence_executable` | **DECLARED → CONSUMER_LANDED** when executable `TestClaim` + runner path land per this brief; **PASSING** when strict-fire evaluates green on CI. |

Gates **#12–#14** (TC2 / TC3 / RustDagIsomorphism executables) stay **DECLARED** until their **separate** worker dispatches; **do not** fold them into this PR.

## Worker pin (Verification Mgr partition)

| Preference | Worker | Condition |
| --- | --- | --- |
| **Primary** | **bold-crane-790** ([gunbc#1748](https://github.com/gunb-ai/gunbc/issues/1748)) | **Track A** ( **V6** ledger audit, TC2/TC3/RustDagIso, partner-scope PRs per schedule): when session active **and** **V6** reaches a **clean checkpoint** acceptable to Verification Mgr — route **those** PRs through bold-crane. **`tc1_eta_equivalence_executable` / TC1 V1 slice excluded** until Branch B hold clears (above). |
| **Alternate** | **New worker** (spawn per `feedback_idle_workers_dispatchable_directly`) | If bold-crane saturated on **V6** or archived — substitute for **non-TC1-V1** Track A work; do **not** interpret as TC1 dispatch unblock. |
| **TC1 V1** | bold-crane (when **unheld**) | Same primary pin resumes **only** when lens fold over `Dag` (per Q-Reification Option A; `Dag` IS the reflected program) consumes `.dag` body authority through Evaluator with non-vacuous η obligation, **or** revised §1.8 TC1 semantics satisfies non-vacuous η per Director ratification. |

**cool-heron-521** remains on **V2 / V4 / V5** prep per partition; **not** the default home for V1 unless explicitly redirected.

## Scope (in)

- First **executable** TC1 slice under **Path A** only: static representative **E6-G1.a**; consumer remains **`BinaryDimensionReportEquals`** over two typed `DimensionReport<C>` reports once producers exist.
- Verification-owned: **TestClaim** wiring, integration tests, fixture **naming** for strict-fire slice, and **coverage** of the ratified representative set (no universal quantification in V1).
- **After** **Q-Reification** resolution + **non-vacuous** lens-over-Dag path: coordinated landing with **Evaluator E3** / producer shape ([#1743](https://github.com/gunb-ai/gunbc/issues/1743)) when interface satisfies §Strict-Fire minimum semantics — Verification does **not** own evaluator internals. **Not** paired with **argument-opaque** Option 3 slice for TC1 gate closure (Director Branch B).

## Scope (out) — STOP+PING

Hold **without** widening by inertia:

| Item | Discipline |
| --- | --- |
| **Path B** (TC1 generic **E6-G1.b / X1.b**) | **Deferred-not-blocked** — out of **V1** scope; separate ratchet after Path A is green or Director reprioritizes. |
| **`SubstrateResearchDeferredClaim` widening** | **STOP+PING** until a **separate** Substrate-routed PR explicitly authorizes carrier shape — V1 does **not** reuse deferred carrier as strict-fire. |
| **Deferred TC1 fixture hard activation** | **STOP+PING** — `tc1_substrate_lens_eta_equivalence_deferred.dag` stays the staging fixture until strict-fire fixture + routing land via implementation PR; no silent runner widening. |
| **New substrate / evaluator predicate shapes** | **STOP+PING** — INVARIANTS §P1 + Substrate Mgr authority; Verification consumes unified **`BinaryDimensionReportEquals`** surface per Director ratification at [#828](https://github.com/gunb-ai/gunbc/issues/828). |

## Dependencies (hard)

| Dependency | Owner | Why |
| --- | --- | --- |
| **E6-G1.a / E3 producer** | Evaluator Mgr | Fold must yield reports that **depend on reflected program shape** enough for **non-vacuous η-invariance** — **not** satisfied by **argument-opaque** `read`/`validate` (PR #1844 strict shape). **Q-Reification cleared 2026-05-07 (Option A: `Dag` IS the reflected program)**; remaining block is non-vacuous η obligation via lens fold consuming `Dag` body authority through Evaluator. TC1 V1 re-pairs on ratified producer contract once that fold lands. |
| **T-Substrate-Lens-Primitive + lens producer retirement progress** | Substrate / PB lanes | Path A assumes existing fold machinery; no parallel `lens_apply` interpretation for the fold receipts. |
| **Representative lens set + eta pair declaration refs** | Substrate (facts) + Verification (fixture refs) | Finite, Director-visible set enumerated in `.dag` / declarations — not “all `Lens<C>`” in V1. |

## Implementation slices (suggested PR shape)

1. **Slice 1 — wiring receipt:** Substrate + Evaluator land minimal producers/refs so both `DimensionReport<C>` sides are **typed** and **lifted** per Evaluator #1131 safe contract (no fixture-local producer identity).
2. **Slice 2 — executable `TestClaim`:** `tc1_eta_equivalence_executable` (or ratified final name per §1.8 ledger) + suite row; integration test exercises **Pass** on representative set.
3. **Slice 3 — ledger / doc receipt:** Update §1.8 **Status** column **DECLARED → CONSUMER_LANDED → PASSING** as CI proves; cross-link [`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) absorbed-responsibility audit row for TC1.

Single PR per `feedback_brief_pr_cadence` if possible; if Substrate and Verification diffs must split, **Substrate lands first** — Verification PR must not invent carrier shapes.

## Cross-refs

- Analysis + Path A: [`r3-v-tc1-eta-equivalence-deeper-analysis.md`](r3-v-tc1-eta-equivalence-deeper-analysis.md)
- Roll-up consumer envelope: [`r3-v-pattern-a-coverage-rollup.md`](r3-v-pattern-a-coverage-rollup.md)
- Formal-grounding bundle (audit cadence): [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md)
- Deferred staging fixture: `src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag`
- Evaluator coordination: [gunbc#1743](https://github.com/gunb-ai/gunbc/issues/1743)
