# R3 Free Consequences - First-Batch Witness Shapes

**Status:** PROPOSAL - research-only pre-authoring. This note documents the
runtime witness-construction shapes for the first five
T-Free-Consequences-Demonstration gates. It does not authorize substrate edits,
new `TestPredicate` variants, new lens carriers, runner changes, or fixture
rewrites.

**Parent:** [`r3-v-free-consequences-worker.md`](r3-v-free-consequences-worker.md)
and [`docs/design-free-consequences.md`](../design-free-consequences.md).

**Fixture surface:** `src/v3/compiler/tests/fixtures/r3_free_consequences_first_batch.dag`.

## Purpose

The first-batch fixture already pins the consumer envelopes:

- `auto_parallelism_*` gates use `LensOutputEquals` and intentionally keep
  workflow parallelism out of `DimensionReport<C>` per DB-3/DB-20.
- `auto_memoization_*` gates use `BinaryDimensionReportEquals` over paired
  `DimensionReport<AutoMemoizationEvidence>` aliases.

This note pre-authors the shape of the runtime evidence those envelopes will
consume once R2-Evaluator can execute `Lens<C>` reads. The goal is to make the
gate-fire moment a producer/evaluator wiring event, not a fixture-design event.

## Authorities

- `docs/r3-structure.md`: auto-parallelism is
  `Lens<Bind-Independence> * Lens<Effect-Commutativity> * Lens<Cost>`;
  auto-memoization is `Lens<Purity> * Lens<Cost>`.
- `docs/design-db18-workflow-effect-carrier.md`: workflow shape is substrate
  physics; lenses must not reconstruct it heuristically.
- `docs/design-db20-lane2-stage2e-parallelism-lens.md`: parallelism safety is
  operation-algebra commutativity over `WorkflowParallelismReport`, not a
  `DimensionReport<C>` instance.
- `docs/design-lens-framework.md`: `Lens.read` is pure over `(Dag, Behavior)`;
  runtime memoization of a lens fold is one instance of the auto-memoization
  consequence.
- `src/v3/std/verification.dag`: `LensOutputEquals` and
  `BinaryDimensionReportEquals` are the consumer envelopes.

## Auto-Parallelism Witness Shape

Applies to `auto_parallelism_independent_binds_emit_parallel`,
`auto_parallelism_dependent_binds_pending_lens_fail_closed`, and
`auto_parallelism_branch_arms_serialize`.

Runtime inputs are the same `(Dag, Behavior)` bind cluster. Three lens reads
must be available:

- `Lens<Bind-Independence>` produces a bind-independence report: which binds
  are dependency-independent and which binds carry ordering edges.
- `Lens<Effect-Commutativity>` produces an effect-commutativity report:
  operation pairs commute, are effect-free, or fail closed because observable
  conflict ordering is not proven.
- `Lens<Cost>` produces cost evidence for the same cluster. Existing cost
  authority is `Cost<Unit>` / `SymbolicCost` per `docs/design-emission-model.md`
  and `docs/design-lens-framework.md`; this note does not introduce a new cost
  carrier.

Witness composition is conjunctive:

```
parallel_emit_eligible =
  bind_independence.green
  && effect_commutativity.green
  && cost.green_and_parallelization_meaningful
```

The negative gates are the same composition with one input non-green:
dependent binds fail bind independence; branch arms do not expose parallel work
for one execution path; unproven effect commutativity stays sequential even if
graph independence is green.

The eventual `LensOutputEquals` expected value should be a structural projection
from that composed report into the emitted-schedule decision. Until the real
`src/v3/lenses/parallelism.dag` producer exists, the scalar placeholder remains
fail-closed. Do not convert these gates to `BinaryDimensionReportEquals`:
parallelism is ordinary lens data under DB-20, not a dimension consumer.

## Auto-Memoization Witness Shape

Applies to `auto_memoization_repeated_pure_call_cached` and
`auto_memoization_no_caching_for_one_shot`.

Runtime inputs are the call site or repeated-work cluster as `(Dag, Behavior)`.
Two lens reads must be available:

- `Lens<Purity>` produces a purity report: the work is effect-free and
  externally deterministic, or it fails closed with the operation that prevents
  memoization.
- `Lens<Cost>` produces repeated-work cost evidence: caching is beneficial for
  repeated pure work and not meaningful for one-shot work.

The report producer for these gates must compose purity and cost into a
`DimensionReport<C>` value consumed by `BinaryDimensionReportEquals`.
`AutoMemoizationEvidence` is a placeholder for that carrier role, not a
substrate claim. At gate-fire time, the producer must name concrete `C` values
and produce paired observed/expected reports: cached-work for repeated pure
calls, no-cache for one-shot calls.

`BinaryDimensionReportEquals` remains the correct envelope only because this
side of the lane is cost-shaped `DimensionReport<C>` evidence. The predicate
must compare structured reports, not serialized text.

## Missing Producer Questions

Assumed producers, all routed through Substrate Manager / `INVARIANTS.md` P1 if
absent: bind-independence report, DB-20-compatible effect-commutativity
projection, purity report for call/work clusters, and cost report for
repeated-work plus parallelization benefit without runtime measurement.
Verification must not create new carriers or predicate variants from this brief.

## Non-Claims

- No substrate edit is authorized here.
- No fixture rewrite is authorized here.
- No new lens carrier is proposed here; report names above are descriptive
  roles for existing or Substrate-owned producers.
- Auto-loop-parallelism and cross-target optimization are intentionally out of
  scope; they belong to the second-batch witness-shape dispatch.
