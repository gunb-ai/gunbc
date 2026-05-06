# R3 Pattern-A — TC2 first executable slice (Church-Rosser / strategy-order) Worker Brief

**Status:** **PRE-AUTH DISPATCH-READY** — brief authored **ahead of** runtime triggers (Brian directive: pre-authored queue). **No strict-fire Implementation dispatch** until §Dependencies clear. **Independent of TC1 V1 Branch B hold** — TC2 is **strategy-order / Church-Rosser** obligation, not η-equivalence; do **not** block TC2 prep on TC1 Q-Reification unless shared **unified predicate** or **fold** substrate lands single PR (then coordinate sequencing only).

**Parent:** [`docs/briefs/r3-verification-manager.md`](r3-verification-manager.md) — absorbed formal-grounding / Pattern-A cluster (see [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure").

**Engineering analysis (research):** [`docs/briefs/r3-v-tc2-church-rosser-analysis.md`](r3-v-tc2-church-rosser-analysis.md) — coverage inputs for the **strategy-order** leg of unified `BinaryDimensionReportEquals` (Director Option 2 / [#828](https://github.com/gunb-ai/gunbc/issues/828)).

**Program plan (single operational authority):** [`docs/r3-program-plan.md`](../r3-program-plan.md) §10.3 — **Q-PAFS** Path **A** remains the **policy** anchor for Pattern-A ordering (TC1 first slice **policy**); **TC1 V1 implementation supersession** (Branch B) is **TC1-only**. **This brief** elaborates **gate #12** `tc2_church_rosser_executable` per [`docs/r3-structure.md`](../r3-structure.md) §"Acceptance" — **does not** override §10.3 table (**INVARIANTS** §P2).

**Consumer envelope:** `BinaryDimensionReportEquals` over two **typed** `DimensionReport<C>` reports produced under **two named executable evaluation strategies** for the **same** program + lens fold — roll-up: [`docs/briefs/r3-v-pattern-a-coverage-rollup.md`](r3-v-pattern-a-coverage-rollup.md).

## §1.8 closure predicate (this slice)

| Gate ID | Gate name (canonical) | Target transition |
| --- | --- | --- |
| **#12** | `tc2_church_rosser_executable` | **DECLARED → CONSUMER_LANDED** when executable `TestClaim` + runner path land per this brief; **PASSING** when strict-fire evaluates green on CI. |

**Do not** fold TC2 into **TC1 V1** PRs or deferred-claim widening — separate worker dispatch per [`r3-v-pattern-a-tc1-v1-worker.md`](r3-v-pattern-a-tc1-v1-worker.md) gate table.

## Worker pin (Verification Mgr partition)

| Preference | Worker | Condition |
| --- | --- | --- |
| **Primary** | **bold-crane-790** ([gunbc#1748](https://github.com/gunb-ai/gunbc/issues/1748)) | Route **TC2 V1** implementation PR(s) when §Dependencies **and** **V6** / Track A staffing allow — **after** Substrate+Evaluator preconditions land; same Track A pin as schedule §2 **A**. |
| **Alternate** | **New worker** | If bold-crane saturated — substitute per `feedback_idle_workers_dispatchable_directly`; do **not** invent parallel `lens_apply` interpretation. |

## Scope (in)

- Executable **TC2** slice: **≥2 closed `EvalStrategy` (or equivalent) inhabitants** + **strategy-keyed** `DimensionReport<C>` producers + **structural equality** via unified predicate **strategy-order modifier** (not a TC2-isolated `TestPredicate` name).
- Verification-owned: **TestClaim** wiring, integration tests, fixture naming for strict-fire slice once Substrate predicate shape exists.
- **Tractability default (research ordering):** prefer **second strategy = second `InputEvaluationOrder` under eager applicative** (e.g. `RightFirst` vs `LeftFirst`) before full **normal-order** / thunk path — per [`r3-v-tc2-church-rosser-analysis.md`](r3-v-tc2-church-rosser-analysis.md) §1.

## Scope (out) — STOP+PING

| Item | Discipline |
| --- | --- |
| **TC2-specific parallel predicate** | **STOP+PING** — consume **unified** `BinaryDimensionReportEquals` + **strategy-order modifier** only (Director Option 2). |
| **Deferred fixture rewrite** | **STOP+PING** — `tc2_evaluation_order_independence_deferred.dag` stays staging until prerequisites land ([`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md)). |
| **Vacuous strategy “equality”** | **STOP+PING** — both strategies must **actually** affect evaluation / memo keys where claimed; no constant-report shortcut (same spirit as TC1 Branch B, **strategy** axis). |

## Dependencies (hard)

Mapped from [`r3-v-tc2-church-rosser-analysis.md`](r3-v-tc2-church-rosser-analysis.md) §4 — all must be **green** before strict-fire **PASSING**:

| ID | Dependency | Owner | Why |
| --- | --- | --- | --- |
| P1 | PR-A.2 evaluator state carriers | Evaluator | `EvalFrame` / `EvalStateStack` substrate path |
| P2 | PR-A.3 strategy + memo **implemented** | Evaluator | closed `EvalStrategy` + memo keying |
| P3 | PR-B.1 eager body evaluator | Evaluator | baseline execution |
| P4 | **Second executable strategy** (or input order) | Evaluator | TC2 **bundle blocker** per formal-grounding TC bundle |
| P5 | `fold_lens<C>` (or equivalent) → `DimensionReport<C>` | Substrate + Evaluator | lens fold authority |
| P6 | Unified generalized predicate + **strategy-order modifier** | Substrate | evolves from `LensOutputEquals` per Director #828 |

## Dispatch triggers (mechanical)

1. **P4 + P6** land on `main` (or Director-visible equivalent) — **Evaluator + Substrate** issue/PR receipts linked from [#1743](https://github.com/gunb-ai/gunbc/issues/1743) / Substrate inbox as appropriate.
2. **Worker available** — **bold-crane-790** (or substitute) per partition.
3. **Verification Mgr** issues **sub-issue** under **#1748** + `addSubIssue` wire + inbox pointer (per Director workflow 2026-05-06).

## Implementation slices (suggested PR shape)

1. **Slice 1 — substrate/evaluator receipt:** second strategy + modifier hook produces two typed reports on a **fixed** representative program + lens set (finite, Director-visible).
2. **Slice 2 — executable `TestClaim`:** `tc2_church_rosser_executable` + integration **Pass**.
3. **Slice 3 — ledger / doc receipt:** §1.8 status column **DECLARED → CONSUMER_LANDED → PASSING**; cross-link [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) TC2 row.

**Substrate lands first** if carrier split — Verification PR must not invent predicate variant (**INVARIANTS** §P1).

## Cross-refs

- Analysis: [`r3-v-tc2-church-rosser-analysis.md`](r3-v-tc2-church-rosser-analysis.md)
- TC bundle: [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md)
- Deferred fixture: `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag`
- Evaluator: [`r2-evaluator-manager.md`](r2-evaluator-manager.md) (TC2 row); inbox [#1743](https://github.com/gunb-ai/gunbc/issues/1743)
- TC1 worker (ordering neighbor, different gate): [`r3-v-pattern-a-tc1-v1-worker.md`](r3-v-pattern-a-tc1-v1-worker.md)
