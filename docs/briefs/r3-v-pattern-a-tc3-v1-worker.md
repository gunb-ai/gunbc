# R3 Pattern-A — TC3 first executable slice (Pattern-A second-mover / evaluation-step) Worker Brief

**Status:** **PRE-AUTH DISPATCH-READY** — brief authored **ahead of** runtime triggers (pre-authored queue). **No strict-fire Implementation dispatch** until §Dependencies clear for the **intended stage** (see **Two-stage gate** below). **TC1 V1 Branch B (η vacuity)** is **orthogonal** to TC3’s **evaluation-step / termination-evidence** axis — coordinate **only** if a **single PR** lands shared **unified `BinaryDimensionReportEquals`** + **fold** substrate touching both gates.

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md) — absorbed formal-grounding / Pattern-A cluster (see [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure").

**Conformance audit (structural envelope):** [`docs/briefs/r3-v-tc3-pattern-a-second-mover-conformance-audit.md`](r3-v-tc3-pattern-a-second-mover-conformance-audit.md) — second-mover consumer shape vs `BinaryDimensionReportEquals` standby.

**Program plan (single operational authority):** [`docs/r3-program-plan.md`](../r3-program-plan.md) §10.3 — **Q-PAFS** / Pattern-A **policy** lives in the table; **TC1 V1 supersession** does **not** block authoring **TC3** coverage requirements. **This brief** elaborates **gate #13** `tc3_pattern_a_second_mover_executable` per [`docs/r3-structure.md`](../r3-structure.md) §"Acceptance" — **does not** override §10.3 (**INVARIANTS** §P2).

**Bundle authority (two-stage):** [`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) — TC3 **stage (a)** vs **stage (b)**; **strict-fire PASSING** requires **(b) T-FixedPoint** horizon per PB ownership transition ([`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) §"Dispatch preconditions" for the Evaluator-gated dispatch floor; §"TC3 — Strong-normalization TestClaim (author-now-fire-later, PB → R3 Verification transition)" / §"Transition to R3 Verification" for the PB→Verification handoff and declarative TC3 scope).

**Consumer envelope:** `BinaryDimensionReportEquals` over **`DimensionReport<Dag>`** role pair — **baseline evaluation-step projection** vs **bounded-step / termination-evidence projection** (same carrier `C = Dag`; audit §Contract).

## §1.8 closure predicate (this slice)

| Gate ID | Gate name (canonical) | Target transition |
| --- | --- | --- |
| **#13** | `tc3_pattern_a_second_mover_executable` | **DECLARED → CONSUMER_LANDED** when executable `TestClaim` + runner path land per this brief; **PASSING** when strict-fire evaluates green on CI **including** bundle **stage (b)** when applicable. |

**Do not** fold TC3 into **TC1** or **TC2** implementation PRs unless Substrate explicitly batches unified-predicate landing — separate **Verification** receipt PRs preferred.

## Worker pin (Verification Mgr partition)

| Preference | Worker | Condition |
| --- | --- | --- |
| **Primary** | **bold-crane-790** ([gunbc#1748](https://github.com/gunb-ai/gunbc/issues/1748)) | Route **TC3 V1** implementation PR(s) when §Dependencies + Track A staffing allow — same **Track A** pin as [`r3-v-pattern-a-tc2-v1-worker.md`](r3-v-pattern-a-tc2-v1-worker.md). |
| **Alternate** | **New worker** | If bold-crane saturated — substitute per `feedback_idle_workers_dispatchable_directly`. |

## Scope (in)

- **Stage (a)** readiness: coverage requirements + fixture **shape** for the two `DimensionReport<Dag>` role producers land against **unified** predicate **evaluation-step modifier** (Substrate-owned predicate evolution per [#828](https://github.com/gunb-ai/gunbc/issues/828)).
- **Stage (b)** strict-fire: **T-FixedPoint** termination semantics + evaluator **evaluation-step / bounded-step** producer surface ([`r3-v-tc3-pattern-a-second-mover-conformance-audit.md`](r3-v-tc3-pattern-a-second-mover-conformance-audit.md) §Strict-Fire Preconditions).
- **Coverage decision** (Director-ratified shape when chosen): structural induction vs generated exhaustive producer over typed-fragment carrier vs **bounded representative harness** — must be **named** before strict-fire claims **PASSING** (audit §1 bullet 5).

## Scope (out) — STOP+PING

| Item | Discipline |
| --- | --- |
| **TC3-isolated `TestPredicate` / quantifier** | **STOP+PING** — unified `BinaryDimensionReportEquals` + **evaluation-step modifier** only (Director Option 2). |
| **Deferred fixture hard rewrite** | **STOP+PING** — `tc3_strong_normalization_deferred.dag` (stage-(a) path) stays staging until substrate-introduction PR authorizes ([`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md)). |
| **Fire-before-(b)** | **STOP+PING** — no **PASSING** strict-fire that pretends **T-FixedPoint** / termination horizon is satisfied when bundle says **(b)** is still open. |
| **Serialized / string / byte comparisons** | **STOP+PING** — per conformance audit **§Non-Drift Findings**. |

## Dependencies (hard)

Synthesized from [`r3-v-tc3-pattern-a-second-mover-conformance-audit.md`](r3-v-tc3-pattern-a-second-mover-conformance-audit.md) §Strict-Fire Preconditions + [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md):

| # | Dependency | Owner | Notes |
| --- | --- | --- | --- |
| D1 | **B5** loop construction-closure green | R2 Release / Substrate | bounded `Loop` construction |
| D2 | **T-Substrate-Lens-Primitive** + real `DimensionReport<Dag>` producers | Substrate + Evaluator | both role reports materialize |
| D3 | **T-FixedPoint** termination semantics (**stage (b)**) | PB Manager | bundle gate |
| D4 | **E5** `Descent` execution proof + evaluator **evaluation-step** producer | Evaluator + Substrate | audit §Strict-Fire items 4–7 narrative |
| D5 | **Unified predicate** + **evaluation-step modifier** | Substrate | INVARIANTS §P1 |
| D6 | **Coverage-shape ratification** (induction / exhaustive / bounded harness) | Director + Verification | named before PASSING |

## Dispatch triggers (mechanical)

1. **D5 + D2 (stage a)** land — substrate/evaluator receipts linked from [#1743](https://github.com/gunb-ai/gunbc/issues/1743) / Substrate / PB threads as appropriate.
2. **D3 + D4** land for **full strict-fire** — **no PASSING** without **(b)** unless Director narrows milestone (explicit §1.8 revision).
3. **Worker available** — **bold-crane-790** (or substitute).
4. **Sub-issue** under **#1748** + `addSubIssue` + inbox pointer (Director workflow).

## Implementation slices (suggested PR shape)

1. **Slice A — coverage + fixture shape:** Verification + Substrate land **stage (a)** authoring against unified predicate proposal (no premature strict-fire green if **(b)** open).
2. **Slice B — executable strict-fire:** `tc3_pattern_a_second_mover_executable` + integration **Pass** when **(a)+(b)** satisfied.
3. **Slice C — ledger/doc:** §1.8 status + bundle row update in [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md).

**Substrate lands first** on predicate splits — Verification does not invent carriers.

## Cross-refs

- Conformance audit: [`r3-v-tc3-pattern-a-second-mover-conformance-audit.md`](r3-v-tc3-pattern-a-second-mover-conformance-audit.md)
- TC bundle: [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md)
- PB declarative theorem: [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) §TC3
- TC1 / TC2 worker neighbors: [`r3-v-pattern-a-tc1-v1-worker.md`](r3-v-pattern-a-tc1-v1-worker.md), [`r3-v-pattern-a-tc2-v1-worker.md`](r3-v-pattern-a-tc2-v1-worker.md)
- Deferred fixture: `src/v3/compiler/tests/fixtures/tc3_strong_normalization_deferred.dag`
