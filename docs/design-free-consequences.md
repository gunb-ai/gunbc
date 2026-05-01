# Design: Free Consequences

**Status:** PROPOSAL - Lane 3 closure deliverable for
T-Free-Consequences-Demonstration. This document records the guarantee framing;
it does not introduce substrate facts, new `TestPredicate` variants, or new lens
carriers.

**User directive:** "what guarantees does the compiler ACTUALLY provide - i
expect a small doc/testcases."

The testcase half is the 10-gate suite in
`src/v3/compiler/tests/fixtures/r3_free_consequences_first_batch.dag` and
`src/v3/compiler/tests/fixtures/r3_free_consequences_second_batch.dag`. This
document is the prose half: each consequence below is a structural guarantee
only when its named lens inputs exist and produce witnesses. If a witness is
missing, the compiler must fall back to the conservative behavior named by the
gate; it must not infer a heuristic pass.

The common discipline is `feedback_lenses_not_passes`: analyses are lenses over
declared substrate physics, not ad hoc compiler passes. `docs/design-db18-workflow-effect-carrier.md`
rejects deriving workflow shape heuristically inside each lens, and
`docs/design-db20-lane2-stage2e-parallelism-lens.md` frames parallelism safety
as operation-algebra physics rather than a lens-body guess.

## 1. Auto-Parallelism

**Lens algebra:** `Lens<Bind-Independence> * Lens<Effect-Commutativity> * Lens<Cost>`.

The compiler may emit a parallel schedule for independent bind sequences only
when three facts line up:

- `Lens<Bind-Independence>` proves the binds have no dependency edge requiring
  sequential order;
- effect/commutativity evidence proves the operations commute or have no
  observable conflict; and
- `Lens<Cost>` proves parallelizing the work is meaningful rather than pure
  ceremony.

This is not a `DimensionReport<C>` consumer. Per the DB-3/DB-20 split, workflow
parallelism is ordinary lens data rooted in DB-20's `WorkflowParallelismReport`.
The current first-batch fixture keeps that distinction by using
`LensOutputEquals` over `auto_parallelism_pending_lens`, with a dissolution
comment pointing to `src/v3/lenses/parallelism.dag` once the real producer
lands.

**Testcase anchors:**

- `auto_parallelism_independent_binds_emit_parallel`
- `auto_parallelism_dependent_binds_emit_sequential`
- `auto_parallelism_branch_arms_serialize`

The first gate is the positive shape; the second and third are the safety
shapes. Dependent binds and branch arms remain sequential unless the substrate
can prove a parallel structure is legal.

## 2. Auto-Loop-Parallelism

**Lens algebra:** `Lens<Iteration-Independence> * Lens<Effect-Commutativity> * Lens<Cost>`.

Loop iteration parallelism follows the same rule as bind parallelism, but the
independence witness is per-iteration. The default is sequential. Parallel loop
emission is opt-in through `Lens<Iteration-Independence>` and still needs the
same effect/commutativity and cost witnesses. There is no heuristic "looks safe"
loop parallelizer.

This mirrors `Lens<Bind-Independence>` rather than creating a new dimension. The
second-batch fixture therefore uses the same ordinary `LensOutputEquals` shape
as auto-parallelism and explicitly keeps `LoopIterationEvidence` out of
`DimensionReport<C>`. Its current scalar placeholder is intentionally
mismatched (`0 != 1`), so the gates fail closed while still proving the ordinary
lens-output path is expressible. The positive loop gate becomes substantive only
when `src/v3/lenses/parallelism.dag` exposes the real DB-20
`WorkflowParallelismReport` producer for `Lens<Iteration-Independence>`.

**Testcase anchors:**

- `auto_loop_parallelism_provable_independence_emits_parallel`
- `auto_loop_parallelism_unproven_falls_back_sequential`
- `auto_loop_parallelism_dependence_emits_sequential`

Together these define the guarantee: provable iteration independence can emit a
parallel loop; missing or negative evidence emits sequential code.

## 3. Auto-Memoization

**Lens algebra:** `Lens<Purity> * Lens<Cost>`.

The compiler may memoize repeated work when the work is pure and the cost lens
shows a useful repeated computation. Purity is load-bearing: a call that reads
external mutable state, performs I/O, or depends on hidden runtime context is
not eligible. Cost is also load-bearing: one-shot work does not receive
memoization scaffolding just because it is pure.

`docs/design-lens-framework.md` locks the `Lens.read` purity invariant:
`Lens.read` is a pure function of declared `(Dag, Behavior)` inputs and may not
depend on external mutable state. Runtime memoization of a lens fold is one
instance of this same free consequence, not a separate "lens framework caching"
feature.

**Testcase anchors:**

- `auto_memoization_repeated_pure_call_cached`
- `auto_memoization_no_caching_for_one_shot`

The first-batch fixture keeps these cost-related claims on
`BinaryDimensionReportEquals` over paired `DimensionReport<C>` aliases, because
this side of the lane consumes cost-shaped reports.

## 4. Cross-Target Optimization

**Lens algebra:** `Lens<Cost> * LanguageSpec`.

The guarantee is not byte-identical output and not identical syntax across Rust,
Python, and Go. Each target may choose target-native forms. The guarantee is
that the cost and semantic relation is structurally preserved through the
target's `LanguageSpec` facts.

Constant folding is the minimal example: the folded subtree disappears in a
target-specific way, but the symbolic cost reading changes by the same
structural amount under each target's realization-cost table. The broader
`cost_structurally_derived` gate states that emitted target cost is derived from
the composition of `.dag` algebra-level cost and per-primitive realization cost,
not runtime measurement or target-specific heuristic policy.

**Testcase anchors:**

- `cross_target_optimization_constant_fold_consistent`
- `cross_target_optimization_cost_structurally_derived`

Both second-batch gates use `BinaryDimensionReportEquals` because cost is a
`DimensionReport<C>` consumer.

## 5. Space-Bound CX

Space-bound CX is a reference item in this lane, not a new derivation. The R1
closure surface assigns the complexity-lens family to R1/T-LaneE authority:
`ROADMAP.md` names `complexity_merge_sort_is_nlogn`,
`complexity_merge_sort_v3_matches_v2_oracle`, and
`lane_e_bundled_witness_host_emit_parity`; `docs/briefs/r1-closure-manager.md`
records R1 closure by release acceptance; and
`docs/thesis/r2-r3-thesis-mapping.md` maps "Space bound proofs from CX" to
R1/T-LaneE complexity-lens gates.

Lane 3 does not re-prove space bounds and does not claim a live space lens.
`docs/thesis-validation-plan.md` still marks proven space bound per function as
deferred, and `docs/briefs/r3-v-free-consequences-worker.md` explicitly keeps
space-bound CX as status/reference only. The actual Lane 3 guarantee is that
the other free consequences above stay structural and lens-derived; space-bound
proofs remain governed by their R1/R2 authority until a modeled space lens
exists.

## Closure Shape

T-Free-Consequences-Demonstration closes only when this document and the
10-gate suite are both accepted. The gates are not heuristic demos: they pin the
consumer shapes that must become substantive as the relevant `Lens<C>` producers
and R2-Evaluator witness construction land.
