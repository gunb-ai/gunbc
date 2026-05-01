# R3 Verification — TC2 (Church-Rosser / evaluation-order independence): strict-claim activation analysis

**Status:** PROPOSAL — **research-only**. Does **not** assert TC2 strict-fire is dispatchable today; does **not** author substrate facts, new `TestPredicate` variants, or edits to the deferred fixture body.

**Authority bundle:** TC2 sits in the absorbed formal-grounding trio ([`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) §Scope table). Slice-0 hook: [`tc2_evaluation_order_independence_deferred.dag`](../../src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag) (`evaluation_order_independent_lens_results`, unified `BinaryDimensionReportEquals` consumer — runner equality NYI until `DimensionReport<C>` production lands).

**Unified predicate framing (Director ratification, [#828](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356050427)):** Cross-claim coordination is **unified substrate-introduction across TC1/TC2/TC3**, **Option 2 from [PR #1309](https://github.com/gunb-ai/gunbc/pull/1309)** — generalize [`LensOutputEquals`](../../src/v3/std/verification.dag) toward **binary structural equality over `DimensionReport<C>`** with **reflection-aware modifiers** (three legs: **η** for TC1, **strategy-order** for TC2, **evaluation-step** for TC3). This brief does **not** propose a parallel TC2-only predicate; it specifies **TC2 coverage requirements** for that unified predicate’s **strategy-order** modifier. TC1 analysis is merged in spirit via #1309; TC3 coverage is queued for a separate worker. Substrate-fact-introduction still routes **INVARIANTS §P1** when the unified shape lands; deferred-claim discipline unchanged.

---

## 1. What “≥2 executable evaluation strategies” means (TC2 coverage input)

The deferred claim text already names two **semantic** contrasts ([`tc2_evaluation_order_independence_deferred.dag`](../../src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag) L15–18): applicative vs normal order, and left-first vs right-first **n-ary Transform** input evaluation.

For the **body evaluator**, “strategy” must be a **closed substrate inhabitant** keyed through evaluator state/memo, not a string label ([`docs/briefs/r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) L70–95, L106–111). PR-B.1 explicitly lands **one** eager baseline and defers lazy/normal-order ([`docs/briefs/r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) L146–154).

**Tractability ordering (research judgment — feeds the strategy-order modifier design):**

1. **Second strategy = second `InputEvaluationOrder` under applicative eager** (e.g. `RightFirst` vs locked `LeftFirst`) — smaller lift than full normal-order: same eager skeleton, different argument evaluation schedule on multi-input transforms. Requires PR-A.3’s closed `EvalStrategy` / memo-key plumbing ([`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) L129–134; TC2 boundary L156–164).

2. **Second strategy = `NormalOrder` / call-by-need** — **EvalThunk + captured `EvalStateStack`** ([`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) L82–92); **R3 residual** relative to PR-B.1 ([`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) L146–148).

`EvalFrame` / `EvalStateStack` ([`src/v3/std/runtime.dag`](../../src/v3/std/runtime.dag) L60–75) support multi-strategy work once PR-A.3 keys memo by strategy ([`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) L101–111).

**TC2 coverage requirement for the unified predicate:** the **strategy-order modifier** must be able to name **≥2 executable strategies** through the same evaluator boundary and require **pairwise `DimensionReport<C>` equality** (see §2) for the same program + lens fold — not merely “left vs right” as documentation prose.

---

## 2. `DimensionReport<C>` equality under strategy variation (TC2 coverage input)

**Carrier shape today:** `DimensionReport<Carrier>` is `DimensionOk` / `DimensionFail` with `composed`, `witnesses`, `violations`, `dimension_name` ([`src/v3/std/dimensions.dag`](../../src/v3/std/dimensions.dag) L51–61). **Binary structural equality** over that sum — for the **same** generic `C` and the **same** dimension name — is the comparison core the unified predicate carries; TC1 #1309 converged here independently.

**Scaffold today:** [`LensOutputEquals`](../../src/v3/std/verification.dag) L169–173 compares a **single** lens application to an **expected** declaration — the **generalization target** is not a second TC2 predicate name but **one** unified envelope: `LensOutputEquals`-shaped evolution to **binary `DimensionReport<C>` equality** plus **modifiers** (Director Option 2). **`DifferentialEquals`** (L177–181) remains a different axis (subject vs oracle declarations); it does not replace the strategy-order story.

**TC2 coverage requirement for the unified predicate:** strict TC2 activation means the **strategy-order modifier** must (a) fix **which two strategies** are in play (e.g. eager+LeftFirst vs eager+RightFirst first; later eager vs normal-order when thunks land), (b) run `fold_lens<C>` (or equivalent Evaluator entry) under each, (c) assert **structural equality** of the resulting **`DimensionReport<C>`** values (composed + witness lists + failure partitions as appropriate — exact runner rules are authoring work for the unified proposal, not this note).

**Sequencing:** All three TC coverage analyses (TC1 in via #1309, **this TC2 note**, TC3 queued) should mature **before** unified-predicate authoring fires; this file is the **second leg** of that evidence base.

---

## 3. Prerequisites on the lens / fold side

TC2 is stated over **lens results** ([`tc2_evaluation_order_independence_deferred.dag`](../../src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag) L4–5, L18). **`fold_lens<C>`** → `DimensionReport<C>` appears **only in module commentary** today ([`src/v3/std/lens.dag`](../../src/v3/std/lens.dag) L6–8; the `Lens<C>` carrier is L70–77); there is **no** substrate `fn fold_lens` yet. Workflow-root design notes also reference `fold_lens<C>` ([`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag) L503–504, L535–536).

Q6/Q6.5/Q7 ([`docs/design-lens-framework.md`](../design-lens-framework.md) §Q6.5, §Q7) govern diagnostics into `DimensionFail.violations`; strategy-pair runs must not treat **diagnostic divergence** as success.

**Shared gap with TC3:** `Lens<C>` / T-Substrate-Lens-Primitive cadence ([`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) §Dependencies). Coordinate via Verification **#1276** when it blocks multiple TC legs.

---

## 4. Dispatch-precondition map (TC2 strict-fire authorable)

Rough **partial order** before TC2 strict-fire **evaluates** (beyond `BinaryDimensionReportEquals` shape validation):

| # | Precondition | Concrete anchor on `main` |
|---|----------------|---------------------------|
| P1 | PR-A.2 evaluator state carriers | [`src/v3/std/runtime.dag`](../../src/v3/std/runtime.dag) L60–75 (`EvalFrame`, `EvalStateStack`) |
| P2 | PR-A.3 strategy + memo carriers **implemented** | [`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) L126–134, L156–164 |
| P3 | PR-B.1 eager body evaluator | [`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) L142–154 |
| P4 | **Second** executable strategy (or input order) | R3 residual ([`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) L146–154) |
| P5 | `fold_lens<C>` (or equivalent) to `DimensionReport<C>` | [`src/v3/std/dimensions.dag`](../../src/v3/std/dimensions.dag) L51–61; [`src/v3/std/lens.dag`](../../src/v3/std/lens.dag) L6–8 |
| P6 | **Unified** generalized predicate + **strategy-order modifier** landed | Evolves from [`src/v3/std/verification.dag`](../../src/v3/std/verification.dag) L169–173 per Director Option 2 / #1309; **not** a TC2-isolated variant |

**Not TC2-primary:** T-LensProducer-Retirement ([`docs/r3-structure.md`](../r3-structure.md)) is TC1-η weighted in the bundle ([`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) L13); TC2’s named bundle blocker remains **second strategy** (L36).

---

## 5. Coordination / STOP flags

- **Unified predicate author** consumes §1–§2 as **TC2 coverage requirements** for the **strategy-order** modifier — not a competing predicate proposal.

- **TC3:** evaluation-step modifier + this note’s fold/strategy deps — track on **#1276**.

- **Fixture discipline:** do **not** rewrite [`tc2_evaluation_order_independence_deferred.dag`](../../src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag) until prerequisites land ([`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) L19).

## Cross-refs (read-only)

- Evaluator gate: [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) L127–128  
- TC2 bundle: [`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) L14–15, L54–55  
- Cadence matrix: [`docs/briefs/r2-evaluator-cadence-verification-matrix.md`](r2-evaluator-cadence-verification-matrix.md) (TC2 row)
