> Part of: [lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md) (Lane 2) | Closes: [design-db18-workflow-effect-carrier.md](./design-db18-workflow-effect-carrier.md) §Open question 1 | Companion: [design-composed-effect-reshape.md](./design-composed-effect-reshape.md) (`CompositionVerdict` authority)

# Design DB-20 — Lane 2 Stage 2e: parallelism-as-lens (parallel composition safety)

**Status:** Implemented — `analyze_parallelism` in `src/v3/compiler/src/workflow_parallelism.rs`, report carrier in `src/v3/std/effects.dag`, lens stub `src/v3/lenses/parallelism.dag`.
**Scope boundary:** DB-20 covers **workflow** `ParallelEffect` + op-level commutativity only. Thesis Stage 2e items (dependency-graph parallelism, commutative fold reducibility) are **not** orphaned: see [ROADMAP.md](../ROADMAP.md) §Lane 2 Stage 2e — **Deferral: thesis graph-parallelism slice**, and [lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md) §Cross-cutting acceptance.
**Consumers:** CI fixtures (`lane2_stage_2e_parallelism_test.rs`); future `.dag` lens once `match` on user sums + `lane2_workflow` reflection land (same boundary as Stage 2b).

---

## Summary

DB-18 locks `WorkflowEffect::ParallelEffect { branches: NonSingletonList<WorkflowEffect> }` as the concurrent-product (⊗) introduction form. Stage 2e answers DB-18 §Open question 1: **whether safe parallel scheduling requires a stored commutativity witness on the carrier** versus **derivation from existing operation-level algebra**.

**Decision (path (b), preferred by substrate Q3):** do **not** add `commutativity: …` to `ParallelEffect`. Pairwise commutativity for concurrent branches is **derived** from `OperationEffect` / `EffectShape` / `KeySource` data already carried on each operation. The lens is the sole place that combines those facts into a judgment; nothing new is duplicated into the workflow substrate.

**Output shape:** `WorkflowParallelismReport = ParallelCompositionVerdict(CompositionVerdict) | ParallelismUnsupported(ParallelismUnsupportedDetail)` where `ParallelismUnsupportedDetail` carries a typed `ParallelismUnsupportedKind` (not Stage 2b’s `IdempotencyUnsupportedDetail`, which names an unsupported *workflow* variant). The **algebra verdict** stays `CompositionVerdict` only (PR #529 / DB-18 constraint — no parallel verdict carrier). Unsupported paths stay explicit and fail-closed (C-8), including “pairwise non-commute” and “non-`ParallelEffect` root.”

**Scope v1:** `ParallelEffect` whose **every branch is `LinearEffect`** (each branch is a non-empty list of `OperationEffect`). Nested `ParallelEffect` / `BranchEffect` / `LoopEffect` as direct parallel children return `ParallelismUnsupported` with a reason — not a silent `None`.

---

## DB-18 §Open question 1 — resolution

| Option | Verdict |
|--------|---------|
| (a) Additive `commutativity: CommutativityWitness` on `ParallelEffect` | **Rejected for Stage 2e.** Would duplicate facts derivable from per-op shapes (Q3), unless a future consumer proves a witness is *not* derivable from the op algebra alone. |
| (b) Derive from op-level algebra without a substrate field | **Adopted.** Same structural `KeySource` on two upserts/deletes is enough to treat them as commuting on one cell; **different** `KeySource` values (including different `PathParam` *names*) do **not** imply runtime key disjointness without a witness, so v1 is fail-closed there. |

---

## Commutativity model (derived)

For parallel branches that are each linear sequences of operations, **safe concurrent scheduling** requires that for **every** operation `a` taken from branch `i` and **every** operation `b` taken from branch `j` with `i ≠ j`, the two operations **commute** as state updates: reordering `a` before `b` does not change the observable outcome relative to `b` before `a`, under the effect algebra’s state model.

**Precedence:** if **any** operation is breaking (`IsBreaking`), parallel reorder is unsound regardless of pairwise checks — the report projects through `CompositionVerdict::BrokenBy { first_breaker: BreakingOperation }` (same breaker discovery order as linear `compose_effects`: branch order, then op order within each branch).

**Idempotent pairs** use a conservative partial function `idempotent_pair_commutes` on `IdempotentShape`:

- `ReadEffect` ∘ `ReadEffect` — commutes.
- `UpsertEffect` / `DeleteEffect` — **same** `KeySource` commutes (lattice meet on one key). **Distinct** `PathParam` parameter names do not prove disjoint runtime keys (`{id}` vs `{user_id}` can alias); inequality → **not** commute in v1.
- Mixed `Upsert`/`Delete` or `CompositeKey`/`InputField` combinations default to **not** commuting until a richer key-disjointness story exists (conservative).

This is **physics in the op algebra**, not a heuristic re-invented inside the lens body — the lens only combines facts already on `OperationEffect` (`feedback_lenses_not_passes`).

---

## Substrate principle audit (Q1–Q6)

**Q1 — Cardinality:** No change to `ParallelEffect.branches: NonSingletonList<…>`; empty or singleton parallel workflows remain unrepresentable at the type level.

**Q2 — Typed handles:** No new raw `PortId` surfaces; analysis uses `OperationEffect` and `WorkflowEffect` only.

**Q3 — Duplicated fact:** No commutativity field on `ParallelEffect`; derivation-only.

**Q4 — Coproduct dissolution:** No new coproduct; report sum `WorkflowParallelismReport` is a **lens boundary** sum (same class as `WorkflowIdempotencyReport`), not a second effect-algebra verdict type.

**Q5 — Construction authority:** Workflow shape remains from `Dag::try_register_lane2_workflow_effect`; the lens only reads.

**Q6 — Representation duality:** Single report path per `(dag, root)` analysis; unsupported carries reason text, not parallel “maybe” fields.

---

## `CompositionVerdict` projection map

| Situation | `ParallelCompositionVerdict` payload |
|-----------|--------------------------------------|
| All branches linear, no breaking op, all cross-branch pairs commute | `IdempotentComposition` |
| Any breaking op in any branch | `BrokenBy { first_breaker }` |
| Non-linear branch, wrong root variant, or pairwise non-commute | Not a `CompositionVerdict` — use `ParallelismUnsupported` with typed `kind` + `reason` |

---

## STOP-AND-ESCALATE (Lane C)

Escalate to a DB revision if:

- Extending `WorkflowEffect` beyond DB-18’s four variants becomes necessary (not done here).
- `ParallelEffect.branches` cardinality must change (not observed in Stage 2e v1).
- A stored witness field becomes **provably** non-derivable from `OperationEffect` (would reopen path (a)).

---

## Related paths

- Rust entry: `v3_compiler::analyze_parallelism` (re-exported from `lens_parallelism`).
- Tests: `src/v3/compiler/tests/lane2_stage_2e_parallelism_test.rs`.
