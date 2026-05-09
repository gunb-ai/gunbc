# R3 Free Consequences - Second-Batch Witness Shapes

**Status:** PROPOSAL - research-only pre-authoring. This note documents the
runtime witness-construction shapes for the second five
T-Free-Consequences-Demonstration gates. It does not authorize substrate edits,
new `TestPredicate` variants, new lens carriers, runner changes, or fixture
rewrites.

**Parent:** [`r3-v-free-consequences-worker.md`](r3-v-free-consequences-worker.md);
fixture surface: `src/v3/compiler/tests/fixtures/r3_free_consequences_second_batch.dag`.

## Purpose

The second-batch fixture already pins the consumer envelopes:

- `auto_loop_parallelism_*` gates use `LensOutputEquals` and intentionally keep
  loop parallelism out of `DimensionReport<C>` per DB-3/DB-20.
- `cross_target_optimization_*` gates use `BinaryDimensionReportEquals` over
  paired `DimensionReport<CrossTargetOptimizationEvidence>` aliases.

This note pre-authors the runtime evidence those envelopes will consume once
R2-Evaluator, `Lens<C>` primitives, and cost facts can execute the reads.

## Authorities

- `docs/r3-structure.md`: auto-loop-parallelism is
  `Lens<Iteration-Independence> * Lens<Effect-Commutativity> * Lens<Cost>`;
  cross-target optimization is `Lens<Cost> * LanguageSpec`.
- `docs/design-db18-workflow-effect-carrier.md`: workflow and loop shape are
  substrate facts; lenses must not infer missing control-flow physics.
- `docs/design-db20-lane2-stage2e-parallelism-lens.md`: parallelism safety is
  ordinary lens data over `WorkflowParallelismReport`, not a dimension report.
- `docs/design-emission-model.md`: per-target `LanguageSpec` facts carry
  realization cost; cost composition is structural, not runtime measurement.
- `src/v3/std/verification.dag`: `LensOutputEquals` and
  `BinaryDimensionReportEquals` are the consumer envelopes.

## Auto-Loop-Parallelism Witness Shape

Applies to `auto_loop_parallelism_provable_independence_emits_parallel`,
`auto_loop_parallelism_unproven_falls_back_sequential`, and
`auto_loop_parallelism_dependence_emits_sequential`.

Runtime inputs are the same `(Dag, Behavior)` loop or loop-body cluster. Three
lens reads must be available:

- `Lens<Iteration-Independence>` produces an iteration-independence report:
  which loop iterations are independent and which carry loop-carried
  dependencies.
- `Lens<Effect-Commutativity>` produces the same observable-conflict report
  used by bind parallelism: effect pairs commute, are effect-free, or fail
  closed because ordering safety is unproven.
- `Lens<Cost>` produces cost evidence for the loop body and scheduling choice.

Witness composition mirrors first-batch auto-parallelism:

```
parallel_loop_eligible =
  iteration_independence.green
  && effect_commutativity.green
  && cost.green_and_parallelization_meaningful
```

The negative gates are the same composition with one input non-green:
unproven independence falls back to sequential, and explicit loop-carried
dependence forces sequential emission even if cost would otherwise favor
parallel work.

The eventual `LensOutputEquals` expected value should be a structural projection
from that composed report into the emitted-loop scheduling decision. Until the
real `src/v3/lenses/parallelism.dag` producer exposes DB-20 loop data, the
scalar placeholder remains fail-closed. Do not convert these gates to
`BinaryDimensionReportEquals`: loop parallelism is ordinary lens data, not a
dimension consumer.

## Cross-Target Optimization Witness Shape

Applies to `cross_target_optimization_constant_fold_consistent` and
`cross_target_optimization_cost_structurally_derived`.

Runtime inputs are emitted target programs plus the source `.dag` cost facts.
Two data families must be available:

- `Lens<Cost>` / `Lens<SymbolicCost>` produces pre- and post-emission cost
  reports for the certification corpus program.
- `LanguageSpec` provides per-target realization-cost facts for the target
  primitives selected by Rust, Python, and Go emitters.

The report producer must compose those facts into paired `DimensionReport<C>`
values consumed by `BinaryDimensionReportEquals`. The fixture's
`CrossTargetOptimizationEvidence` alias is a placeholder for that carrier role,
not a substrate claim. At gate-fire time the producer must name concrete `C`
values and produce paired observed/expected reports:

- constant-fold consistent: post-emission cost equals pre-emission cost minus
  the structurally folded subtree cost under each target's `LanguageSpec`;
- cost structurally derived: emitted target cost equals `.dag` algebra cost
  plus per-primitive realization cost, with no runtime measurement.

`BinaryDimensionReportEquals` is the correct envelope here because these are
cost-shaped `DimensionReport<C>` comparisons. The predicate must compare
structured reports, not serialized target output or benchmark results.

## Missing Producer Questions

Assumed producers, all routed through Substrate Manager / `INVARIANTS.md` P1 if
absent: iteration-independence report, DB-20-compatible effect-commutativity
projection for loop bodies, cost report over loop body and emitted target
artifacts, and `LanguageSpec` realization-cost rows for every target primitive
used in the certification corpus. Verification must not create new carriers or
predicate variants from this brief.

## Non-Claims

- No substrate edit is authorized here.
- No fixture rewrite is authorized here.
- No new lens carrier is proposed here; report names are descriptive roles.
- Auto-parallelism and auto-memoization are out of scope; they are covered by
  `r3-v-auto-parallelism-memoization-witness-shapes.md`.
