# R3 PB-Runtime Equivalence Corpus Seed Audit (docs-only, post-#1235)

**Status:** AUDIT artifact (docs-only). Authored 2026-04-30 by PB Manager continuation per dispatch on inbox #1149 — bounded post-#1235 planning slice that pre-stages the `pb_runtime_equivalent_to_evaluator_on_corpus` corpus *shape* without authoring the TestClaim, the `DifferentialEquals` declaration, any `DeclarationRef`s, or any implementation.

**Delta — #1849 merge receipt (2026-05-06):** PR [#1849](https://github.com/gunb-ai/gunbc/pull/1849) landed the author-now / fire-later Seed (1)-(2) receipt on `main` as squash merge `a001e90e`; this doc was refreshed against `origin/main` HEAD `62bec567114e`, and re-checked 2026-05-07T00:04Z against `origin/main` HEAD `2c7d82031`. The receipt added only fixture/test-scoped harness material:
- `src/v3/compiler/tests/fixtures/r3_pb_eval_corpus/seed_int_arithmetic.v3`
- `src/v3/compiler/tests/fixtures/r3_pb_eval_corpus/seed_list_fold.v3`
- `src/v3/compiler/tests/fixtures/r3_pb_runtime_evaluator_corpus_seeds.dag`
- `src/v3/compiler/tests/integration/r3_pb_runtime_evaluator_corpus_seed_test.rs`

The `.dag` fixture declares fixture-local `pb_runtime_evaluate` / `r2_evaluator_evaluate` producers that return `miss_int_lookup()`, and the Rust harness asserts the current `ClaimResult::NotYetImplemented(_)` receipt for those producers while byte-syncing `TestClaim.source` to the two `.v3` authority files. This is not Row 4 green, not the locked `pb_runtime_equivalent_to_evaluator_on_corpus` TestClaim, and not Seed (3) `Lens<C>` authoring.

**Delta — #1920 HEAD refresh (2026-05-07):** Verified against `origin/main` HEAD `2c7d82031` after #1900/#1908/#1909/#1911. The seed-table anchors still match current source: `LiteralBits` at `src/v3/std/substrate.dag:31`, `Value` at `src/v3/std/runtime.dag:44-49`, `fold` / `map` at `src/v3/std/list.dag:96` / `:132`, `LoopBound` at `src/v3/std/substrate.dag:398-400`, `ReleaseDeferredClaim` / `SubstrateResearchDeferredClaim` at `src/v3/std/verification.dag:281-285` / `:295-299`, and canonical lens examples at `src/v3/lenses/named_function_count.dag:24` / `src/v3/lenses/complexity.dag:60`. No scope expansion: Seed (3), final Row-4 `TestClaim` authoring, and real runtime producers remain future dispatch territory.

**Parent authorities:**
- [`docs/briefs/r2-pb-runtime-evaluator-convergence-matrix.md`](r2-pb-runtime-evaluator-convergence-matrix.md) — merged via #1235; Row 4 names the locked TestClaim `pb_runtime_equivalent_to_evaluator_on_corpus` and the three forward `DeclarationRef`s (`pb_runtime_evaluate`, `r2_evaluator_evaluate`, `corpus`).
- [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §7.1 — design lock for the corpus seed: "arithmetic on `Int`; `List` map/fold; one `Lens<C>` instance application."
- [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) — R2-Evaluator manager brief (PR-A through PR-E lane structure; owner of `r2_evaluator_evaluate`).

This audit does NOT introduce authority. Every row cites an existing locked authority; nothing is invented here.

## Scope

The merged convergence matrix (#1235) names the corpus as a single phrase. This audit expands that phrase into a per-category seed table so that, when the future Row-4 TestClaim worker dispatches, the corpus shape is already mapped against existing substrate authorities and the prerequisite gates per category are visible without re-deriving them at dispatch time.

**Not in scope (verbatim from dispatch):**
- The TestClaim `pb_runtime_equivalent_to_evaluator_on_corpus` itself.
- A `DifferentialEquals` declaration.
- A `ReleaseDeferredClaim` or `SubstrateResearchDeferredClaim` (#1235 established neither is a generic forward-ref staging carrier).
- Any implementation of PB-Runtime or R2-Evaluator.
- The `pb_runtime_evaluate`, `r2_evaluator_evaluate`, or `corpus` `DeclarationRef`s.
- Any new `TestPredicate` variant.
- Any fixture or test files beyond the post-audit #1849 Seed (1)-(2) receipt listed in the delta above.

## Corpus seed table (the three §7.1 categories)

| Seed | Purpose (what the equivalence-on-this-seed proves) | Required declarations / refs | Owning future lane | Prerequisite gates (cumulative) | STOP / report-instead-of-invent |
|---|---|---|---|---|---|
| **(1) Arithmetic on `Int`** — small programs over `Int` literals + primitive operators (e.g., `2 + 3`; `(a * b) + c`; `Int64::add` with overflow boundary). | Exercises `Behavior::Value(LiteralValue(Int))` evaluation + `Behavior::Transform(Operator(...))` dispatch on the `OrderedRing<Word64>` algebra (per `dsl/std/integer.dag` integer chain + [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Worked example: integer modeling) worked example). The smallest non-trivial closed `Value` shape: a `LiteralValue(LiteralBits)` returned from primitive operator dispatch. If PB-Runtime and R2-Evaluator agree here, the literal-value emission path + operator-dispatch path are converged. | **Substrate live on main:** `LiteralBits` at `src/v3/std/substrate.dag:31`; `Value::LiteralValue(LiteralBits)` at `src/v3/std/runtime.dag:44-45`; `Int = Int64 = OrderedRing<Word64>` per `dsl/std/integer.dag` + `dsl/std/algebra.dag`. **#1849 receipt live:** `seed_int_arithmetic.v3` plus deferred TestClaim `pb_eval_corpus_seed_int_arithmetic_deferred` in `r3_pb_runtime_evaluator_corpus_seeds.dag`. **Not yet declared outside fixture scope:** the final corpus row(s); the real `evaluate(...)` entry points on each side. | Inputs: PB-Runtime + R2-Evaluator both consume; Evaluator owns `r2_evaluator_evaluate` per `r2-evaluator-manager.md` §"Owned deliverables (through R2 close)" — "Runtime value model" sub-lane row; PB-Runtime owns `pb_runtime_evaluate` per Row 1+7 of merged matrix. **Author of final corpus rows:** PB-Runtime worker (R3-dispatch territory). | (a) Row 1 carrier is live on main via `src/v3/std/runtime.dag:44-49`; (b) R2-Evaluator's real `evaluate(...)` entry point still lands per PR-A through PR-E; (c) PB-Runtime's real `evaluate(...)` `.dag` declaration still lands in R3 per `design-pb-runtime-interpreter.md` §3.2. #1849 covers only a typed-deferred fixture receipt for this seed. | If a corpus row needs an integer-arithmetic semantic the runtime hasn't decided yet (e.g., overflow trap vs wrap on `Int64::add`), STOP — that's an Evaluator semantic-decision call, not a corpus-shape choice. Escalate to Evaluator Manager. |
| **(2) `List` map / fold** — small programs that build a `List<T>` and fold or map it (e.g., `fold(cons(1, cons(2, empty())), 0, +)`; `map(xs, f)`). | Exercises `Behavior::Loop` (the `LoopBound`-bounded recursive fold) + `Behavior::Transform(Callable(decl_id))` higher-order dispatch through user-defined `fn map` / `fn fold` declarations. The smallest non-trivial closed `Value` over multiple primitives: `LiteralValue` items inside a `RecordValue`/`VariantValue` shape (or whatever `List<T>`'s structural representation lowers to). Verifies the `Loop` iteration rule + per-iteration accumulator threading per §3.2. | **Substrate live on main:** `fn fold<T, U>` at `src/v3/std/list.dag:96`; `fn map<A, B>` at `src/v3/std/list.dag:132`; `LoopBound` at `src/v3/std/substrate.dag:398-400`; `cons` / `empty` constructors per `std.list`. **#1849 receipt live:** `seed_list_fold.v3` plus deferred TestClaim `pb_eval_corpus_seed_list_fold_deferred` in `r3_pb_runtime_evaluator_corpus_seeds.dag`. **Not yet declared outside fixture scope:** the final corpus row(s); PB-Runtime's real `Loop`-evaluation rule in `.dag`. | PB-Runtime worker (R3) authors the final corpus rows. The `Loop` evaluation rule itself is co-owned by Evaluator (semantics) and PB-Runtime (`.dag` rule expression mirroring §3.2). | (a) Same as Seed (1); plus (b) `Behavior::Loop` evaluation semantics agreed between Evaluator and PB-Runtime — per `design-pb-runtime-interpreter.md` §3.2 "Behavior::Loop(l) → fold over `l.bound`..." rule, but the `LoopBound` coproduct's runtime semantic must be settled (cardinality-bounded vs descent-bounded interactions). #1849 covers only a typed-deferred fixture receipt for this seed. | If the `LoopBound` coproduct exposes a runtime-semantic ambiguity at corpus authoring time (e.g., the `Descent { cluster: ClusterId, measure: PortId }` variant at `src/v3/std/substrate.dag:400` requires runtime descent-evidence checking the corpus would have to encode), STOP — that's a substrate-semantic decision in Evaluator's territory, not a PB corpus-shape choice. |
| **(3) One `Lens<C>` instance application** — apply a single existing lens (e.g., `lenses.named_function_count` at `src/v3/lenses/named_function_count.dag:24`; or `lenses.complexity::cost_of` at `:60`) to a small input program and compare results. | Exercises `Lens<C>` substrate composition (per the lens framework) end-to-end: lens declaration → `apply_lens_declaration`-shape walk → result `Value`. Verifies that the `Lens<C>` carrier (per R2-T-Substrate-Lens-Primitive in `r2-evaluator-manager.md` if cited there) produces the same fold-result through both runtimes. The smallest single-lens application that actually exercises the substrate-lens framework, not just primitive operators. | **Substrate live on main:** at least one canonical lens — `lenses.named_function_count` or `lenses.complexity` — both already used by `test_runner.rs` for L1/T-LaneE gates per the canonical-lens bridge disposition (#1183). **Choice of which canonical lens to seed with is the PB worker's call at dispatch** (the audit does not pre-select). **Not yet declared:** the corpus row's input program + lens-application invocation site. | PB-Runtime worker (R3) authors the corpus row. The lens framework itself is owned by the Substrate Manager / lens-framework lane (per `r2-pr-a-2-eval-frame-dependency-audit.md` references to `Lens<C>` substrate); PB consumes that authority. | (a) Same as Seeds (1) + (2); plus (b) `Lens<C>` substrate carrier live on main (verify at dispatch); plus (c) the chosen canonical lens declaration unchanged at dispatch time (or reauthor against whichever canonical lens has stabilized). | If `Lens<C>` substrate is not yet live or the canonical lenses are in flux when the corpus worker dispatches, STOP and either (i) defer Seed (3) until lens substrate stabilizes or (ii) escalate to Substrate Manager / lens-framework lane for a stable canonical lens to seed against. Do not invent a "seed lens" inside the corpus. |

## Cross-program dependency graph (cumulative; per-seed gates)

```
Seed (1) Int arithmetic   ← Row 1 (Value carrier) + Evaluator/PB-Runtime evaluate(...)
Seed (2) List map/fold     ← (1) + Behavior::Loop semantic agreed
Seed (3) Lens<C> instance  ← (1) + (2) + Lens<C> substrate live + canonical lens stable
```

Authoring order is the same as listed: smallest closed `Value` shape first, then user-defined-call composition, then full lens-framework participation. Each seed's gates are cumulative — Seed (3) requires (1) + (2) + lens-substrate, not just lens-substrate alone.

## ReleaseDeferredClaim / staging lesson (carried forward from #1235)

**No live generic forward-ref staging carrier exists.** Per the merged matrix's prerequisite-state row (#1235 line 45) and Row 4 prerequisite-gate cell:

- `ReleaseDeferredClaim` (`src/v3/std/verification.dag:281-285`) is R1 release-acceptance fixture-only — not usable as a generic staging mechanism for arbitrary `TestClaim`s.
- `SubstrateResearchDeferredClaim` (`src/v3/std/verification.dag:295-299`) is TC1 substrate-research fixture-only — same constraint.
- The Row-4 `TestClaim` (and therefore every per-seed sub-claim that composes into it) is **not authorable** until the three forward `DeclarationRef`s (`pb_runtime_evaluate`, `r2_evaluator_evaluate`, `corpus`) resolve.

**Implication for this audit:** the per-seed table above does *not* schedule `ReleaseDeferredClaim`-staged sub-claims. Each seed's corpus row authoring is a sequencing constraint, not a staging-variant choice. If a future authoring step concludes that a generic forward-ref staging variant is structurally needed (e.g., to incrementally land per-seed sub-corpora before all three runtime entry points resolve), that is a [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Procedure) substrate-fact-introduction event for Substrate Manager — out of this audit's scope.

## STOP / report-instead-of-invent

Per the dispatch directive: STOP if existing docs already fully cover this, or if corpus seed choices require Evaluator Manager authority before PB can document them. Audit conclusions:

- **Coverage check.** The merged matrix (#1235) names the corpus seed as the single design-doc phrase but does not expand it. `docs/design-pb-runtime-interpreter.md` §7.1 is the source phrase. `docs/briefs/r2-evaluator-manager.md` and `r2-pr-d-cross-target-equivalence-harness-primitives.md` cover the **L5 cross-target** corpus (R3 lane T-Verification-L5-Corpus, *different equivalence concern: Rust↔Python↔Go same-semantics*) — that is **not** this corpus. No existing doc expands the §7.1 PB-Runtime ↔ R2-Evaluator equivalence corpus into a per-category table. This audit fills that gap.
- **Authority check.** Each corpus seed's prerequisite gates name owning lanes explicitly. None of the seed *shape* choices require Evaluator Manager authority before PB can document them — the seed shapes come straight from §7.1's design lock. Per-seed *semantic* decisions (overflow on Int, LoopBound runtime semantics, Lens<C> substrate stability) ARE Evaluator / Substrate Manager authority and are routed there via the per-row STOP conditions, not pre-empted here.
- **No contradictions found.** The seed table is consistent with the merged matrix Rows 1, 4, and 7; with the `r2-evaluator-manager.md` PR-A through PR-E lane structure; with `design-pb-runtime-interpreter.md` §3.2 evaluation rules; and with the [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) escalation discipline.

No invented substrate; no `TestPredicate` changes; no evaluator/PB-Runtime implementation; no fixture/test files.

## Cross-refs

- Parent audit: [`docs/briefs/r2-pb-runtime-evaluator-convergence-matrix.md`](r2-pb-runtime-evaluator-convergence-matrix.md) (merged via #1235). Row 4 + prerequisite-state-table TestClaim row name the corpus phrase this audit expands.
- Design lock: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §7.1 (corpus phrase) + §3.2 (evaluation rules each seed exercises).
- Evaluator manager brief: [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) (PR-A through PR-E; owner of `r2_evaluator_evaluate`). Note: PR-D's L5 cross-target corpus is separate from this Row-4 corpus.
- PR-A design slice: [`docs/briefs/r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md) (Value carrier shape; consumed by Seed (1) onward).
- PR-A.1 / PR-A.2 audits: [`docs/briefs/r2-pr-a1-runtime-value-dependency-audit.md`](r2-pr-a1-runtime-value-dependency-audit.md), [`docs/briefs/r2-pr-a-2-eval-frame-dependency-audit.md`](r2-pr-a-2-eval-frame-dependency-audit.md).
- Substrate authorities cited: `dsl/std/integer.dag` (integer chain); `dsl/std/algebra.dag` (`OrderedRing<Word64>`); `src/v3/std/list.dag:96` (`fold`), `:132` (`map`); `src/v3/std/substrate.dag:31` (`LiteralBits`), `:398-400` (`LoopBound`); `src/v3/std/runtime.dag:44-49` (`Value`); canonical lenses at `src/v3/lenses/named_function_count.dag:24`, `src/v3/lenses/complexity.dag:60`.
- Substrate-fact-introduction procedure: [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Procedure).
- Sibling planning briefs (PB R3 lanes): [`docs/briefs/r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md), [`docs/briefs/r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md).
