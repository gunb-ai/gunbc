# R3 Verification — TC2 (Church-Rosser / evaluation-order independence): strict-claim activation analysis

**Status:** PROPOSAL — **research-only**. Does **not** assert TC2 strict-fire is dispatchable today; does **not** propose substrate facts, `TestPredicate` variants, or edits to the deferred fixture body.

**Authority bundle:** TC2 sits in the absorbed formal-grounding trio ([`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) §Scope table). Slice-0 hook: [`tc2_evaluation_order_independence_deferred.dag`](../../src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag) (`evaluation_order_independent_lens_results`, predicate `Compiles` today).

**Coordination (Director-escalated, 2026-04-30):** TC1 deeper analysis in [PR #1309](https://github.com/gunb-ai/gunbc/pull/1309) (*loyal-ibex-851*) concludes TC1 likely **shares substrate equality ground with TC2** — both may need **binary structural equality over `DimensionReport<C>`**. Director is weighing **unified vs independent** substrate-introduction across TC1/TC2/TC3. **Do not** shop a **TC2-only** `TestPredicate` variant without Verification Manager coordination ([inbox #1276](https://github.com/gunb-ai/gunbc/issues/1276)). If this analysis agrees that equality-on-report is the shared lift, **signal #1276 explicitly** so the unified decision lands once.

---

## 1. What “≥2 executable evaluation strategies” means

The deferred claim text already names two **semantic** contrasts ([`tc2_evaluation_order_independence_deferred.dag`](../../src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag) L15–18): applicative vs normal order, and left-first vs right-first **n-ary Transform** input evaluation.

For the **body evaluator**, “strategy” must be a **closed substrate inhabitant** keyed through evaluator state/memo, not a string label ([`docs/briefs/r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) L70–95, L106–111). PR-B.1 explicitly lands **one** eager baseline and defers lazy/normal-order ([`docs/briefs/r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) L146–154).

**Tractability ordering (research judgment):**

1. **Second strategy = second `InputEvaluationOrder` under applicative eager** (e.g. `RightFirst` vs locked `LeftFirst`) — smaller lift than full normal-order: same “no thunk” eager skeleton, different argument evaluation schedule on multi-input transforms. Still requires PR-A.3’s closed `EvalStrategy` / memo-key plumbing so two schedules are **actually instantiated** at the evaluator boundary ([`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) L129–134; TC2 boundary L156–164).

2. **Second strategy = `NormalOrder` / call-by-need** — audit locks **EvalThunk + captured `EvalStateStack`** as co-requisites ([`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) L82–92). That is the faithful Church-Rosser-style contrast with applicative order but is explicitly **R3 residual** relative to PR-B.1 ([`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) L146–148).

`EvalFrame` / `EvalStateStack` are already the substrate home for closed-over runtime binding ([`src/v3/std/runtime.dag`](../../src/v3/std/runtime.dag) L51–75); they **support** multi-strategy experiments once PR-A.3 adds strategy identity into memo keys ([`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) L101–111). They do **not**, by themselves, satisfy TC2 without executable alternate reduction rules.

---

## 2. Strict-form output equality over `DimensionReport<C>`

Target outcome ([`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) L128): compare **two evaluator runs** (same program + lens fold boundary) for **equality at the report carrier**.

**Substrate today:** `DimensionReport<Carrier>` is a pass/fail sum with `composed`, `witnesses`, `violations`, and `dimension_name` ([`src/v3/std/dimensions.dag`](../../src/v3/std/dimensions.dag) L51–61). That shape is **adequate as the value being compared** once both strategies produce a report — no new carrier **fields** identified for TC2 **purely** from the equality statement.

**Predicate fit:**

- **`LensOutputEquals`** ([`src/v3/std/verification.dag`](../../src/v3/std/verification.dag) L169–173) pins **one** lens application against a **single** expected declaration — it does **not** encode “same lens/program, two strategies.”

- **`DifferentialEquals`** ([`src/v3/std/verification.dag`](../../src/v3/std/verification.dag) L177–181) compares **two declarations** (subject vs oracle) on shared input — closer metaphorically, but TC2’s contrast is **two evaluation schedules**, not two independent arrow declarations, unless the harness models strategies as separate declarations (awkward and easy to mis-read).

**Shared equality substrate (TC1 + TC2):** sibling TC1 analysis ([#1309](https://github.com/gunb-ai/gunbc/pull/1309)) points at the **same structural comparison problem**: proving equivalence of lens/dimension outputs likely reduces to **binary equality over `DimensionReport<C>`** (or a thin runner wrapper around it), not a TC2-exclusive predicate name. **Escalate to #1276** if implementation planning splits TC1 vs TC2 equality — Director owns unified vs per-TC introduction.

**Load-bearing escalation:** a **runner-visible** strict TC2 claim likely needs either (i) a **new** `TestPredicate` variant or (ii) a **deliberate reuse** pattern approved by Verification + Evaluator managers — both routes touch verification-runner authority and, if new substrate surface appears, **INVARIANTS §P1** / Substrate Manager. **Do not** author autonomously; **coordinate with #1276** before proposing TC2-specific variants given the TC1 overlap signal above.

---

## 3. Prerequisites on the lens / fold side

TC2 is stated over **lens results** ([`tc2_evaluation_order_independence_deferred.dag`](../../src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag) L4–5, L18). The generic consumer that yields `DimensionReport<C>` is **`fold_lens<C>`** ([`src/v3/std/lens.dag`](../../src/v3/std/lens.dag) L6–8, field contract L71–76); it is **documented but not authored** in substrate today.

Q6/Q6.5/Q7 affect **diagnostics and validate accumulation** into `DimensionFail.violations` ([`docs/design-lens-framework.md`](../design-lens-framework.md) §Q6.5, §Q7) — relevant so strategy runs don’t smuggle failures as carrier inhabitants. They do **not** remove the need for an **executable** `fold_lens<C>` + evaluator-backed `Lens<C>` interpretation path before TC2 strict-fire is meaningful.

**Shared gap with TC3:** both bundles assume **`Lens<C>` shape / T-Substrate-Lens-Primitive** cadence ([`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) §Dependencies summary). Surface shared substrate scheduling to Verification inbox **#1276** when it blocks either TC.

---

## 4. Dispatch-precondition map (strict-fire authorable)

Rough **partial order**; every item must be live before rewriting the deferred claim away from `Compiles`:

| # | Precondition | Concrete anchor on `main` |
|---|----------------|---------------------------|
| P1 | PR-A.2 evaluator state carriers | [`src/v3/std/runtime.dag`](../../src/v3/std/runtime.dag) L51–75 (`EvalFrame`, `EvalStateStack`) |
| P2 | PR-A.3 closed strategy + memo-key carriers **implemented** (not only audited) | Intended home named in [`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) L126–134; **TC2 explicitly waits until ≥2 strategies execute** ([`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md) L156–164) |
| P3 | PR-B.1 **eager** body evaluator — first executable schedule | Scope + exclusions [`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) L142–154 |
| P4 | **Second** executable strategy (or input order) through same Rust evaluator API | R3 residual per PR-B.1 out-of-scope list ([`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md) L146–154) |
| P5 | **Both** strategies can run `fold_lens<C>` (or an explicitly equivalent Evaluator entry) to `DimensionReport<C>` | Report shape [`src/v3/std/dimensions.dag`](../../src/v3/std/dimensions.dag) L51–61; fold authorship gap [`src/v3/std/lens.dag`](../../src/v3/std/lens.dag) L6–8 |
| P6 | Verification **predicate + runner** path that compares two reports | No suitable variant identified in §2; [`src/v3/std/verification.dag`](../../src/v3/std/verification.dag) L108–181 — **may unify with TC1 equality substrate per #1276 / #1309** |

**Not TC2-critical (avoid conflation):** T-LensProducer-Retirement / `lens_apply.rs` deletion ([`docs/r3-structure.md`](../r3-structure.md) acceptance gates) is load-bearing for **TC1 η** strict-fire per bundle scope table ([`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) L13), **not** listed as TC2’s primary blocker — TC2’s named blocker is the **second strategy** ([`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) L36). If Evaluator-hosted `fold_lens<C>` **replaces** legacy lens producers before retirement completes, revisit coupling in a joint Verification + PB note (signal **#1276**).

---

## 5. Coordination / STOP flags

- **Predicate / equality design (§2)** is the highest-risk scheduling dependency; **prefer #1276-coordinated unified equality** over TC2-only variants given TC1 [#1309](https://github.com/gunb-ai/gunbc/pull/1309).

- **TC3 parallel research:** shared dependency on **`Lens<C>` / fold machinery** — coordinate cross-claim gaps via parent Verification inbox **#1276**.

- **Fixture discipline:** do **not** alter [`tc2_evaluation_order_independence_deferred.dag`](../../src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag) deferred text until strict-fire prerequisites land ([`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) L19).

## Cross-refs (read-only)

- Evaluator acceptance gate prose: [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) L127–128  
- TC2 bundle row + acceptance name: [`docs/briefs/r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md) L14–15, L54–55  
- Cadence matrix TC2 row: [`docs/briefs/r2-evaluator-cadence-verification-matrix.md`](r2-evaluator-cadence-verification-matrix.md) §Verification matrix (TC2 / evaluation-order independence)
