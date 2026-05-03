# R3 PR-E E6 Post-Blocker Gate Packet

**Status:** docs/test-plan handoff after the E6 blocker refresh. This packet
does not implement `fold_lens`, edit substrate, add runner predicates, widen
runtime `Value`, or introduce a parallel lens/report shape.

**Inputs:** [`r3-pr-e6-lens-fold-readiness-audit.md`](r3-pr-e6-lens-fold-readiness-audit.md),
[`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md) E6, live
`src/v3/std/lens.dag`, live `src/v3/std/dimensions.dag`, live
`dsl/std/algebra.dag` (`Monoid<T>` authority), and the body evaluator in
`src/v3/compiler/src/lib.rs`.

## Next Gates

The smallest actionable implementation sequence is:

- **E6-G0: evaluator field/call API gate.** The live evaluator entry points are
  `evaluate_body`, `eval_node`, and `eval_port` in
  `src/v3/compiler/src/lib.rs`. E6-G0 makes `TransformTarget::FieldProject` and
  `TransformTarget::Callable` execute through those existing entry points for
  projected lens fields. This gate must not add a second callable interpreter
  or bypass `EvalStateStack`.
- **E6-G1: lens field projection + report lifting over existing carriers.**
  Once E6-G0 exists, E6-G1 consumes one declared `Lens<C>` value as data,
  projects its fields and `Lens.sequential: Monoid<C>` subfields through the
  evaluator, and lifts returned `Witness<C>` / `OptionalDiagnostic` values into
  the existing `DimensionReport<C>` carrier.

E6-G1 is not a new substrate carrier. It is an implementation/API fact:

> The evaluator can consume one declared `Lens<C>` value as data, project its
> fields and `Lens.sequential: Monoid<C>` subfields, execute those projected
> function values through the live `evaluate_body` / `eval_node` / `eval_port`
> entry points after E6-G0 lands callable and field-projection support,
> and lift the returned `Witness<C>` / `OptionalDiagnostic` values into the
> existing `DimensionReport<C>` carrier without fabricating a carrier on failure.

The `Lens<C>` carrier itself is already live in `src/v3/std/lens.dag`; E6-G1
does not sequence declaring that carrier. The still-missing substrate/API fact
is an honest way to author or reference at least one `Lens<C>` **value** with
function-valued fields and a `Monoid<C>` witness sourced from the live carriers.

This is the narrowest useful unblock because the generic fold cannot prove even
the empty/unit or single-behavior cases until it can:

- project `Lens.name`, `Lens.read`, `Lens.sequential`, and `Lens.validate`;
- project `Monoid<C>.identity` and `Monoid<C>.op` from `Lens.sequential`;
- execute projected function values through the same evaluator entry points as
  ordinary program bodies after the E6-G0 callable/field API gate lands;
- distinguish `Witness::Inhabits(c)` from `Witness::Violates {..}`;
- distinguish `OptionalDiagnostic::NoDiagnostic` from
  `OptionalDiagnostic::SomeDiagnostic { value }`;
- construct `DimensionOk` and `DimensionFail` using the live
  `DimensionReport<C>` shape.

It deliberately does **not** require Descent execution. The first E6
implementation may be cardinality-only / no-descent, provided Descent is an
explicit typed residual and tests prove it fails closed. Descent evidence can
land as a later gate without changing the report or lens carriers.

## Why Not Implement Now

Current `main` still blocks E6-G1 in three concrete places:

- `TransformTarget::Callable(_)` is still fail-closed in
  `evaluator::eval_transform_node`, so projected `Lens.read`,
  `Lens.sequential.op`, `Lens.branch`, `Lens.iterate`, and `Lens.validate`
  functions cannot be invoked through PB-Runtime evaluator authority.
- `TransformTarget::FieldProject {..}` is still fail-closed in the evaluator,
  so `Lens<C>` fields and `Monoid<C>` fields cannot be read without host-side
  projection.
- Representative `Lens<C>` data instances are still deferred; for example,
  `src/v3/lenses/cost.dag` and `src/v3/lenses/complexity.dag` explicitly do
  not author `data cost_lens` / `data complexity_lens` while class-5 data
  bodies and function-valued field references are incomplete.

Any E6 code before those facts would either revive `lens_apply.rs`'s
`FieldValue` interpreter as a second authority or return a non-`DimensionReport`
placeholder. Both are forbidden by the E6 readiness audit.

## Acceptance Tests For E6-G1

The future E6-G1 implementation PR should add focused tests at the evaluator /
fold boundary. These tests should use the live `Lens<C>`, `Monoid<C>`,
`Witness<C>`, `OptionalDiagnostic`, and `DimensionReport<C>` shapes; they must
not compile a local MiniLens or MiniReport fixture.

1. **Monoid identity is consumed for an empty fold.**
   - Build or load a minimal declared `Lens<Int>` whose `sequential.identity`
     is a distinct value.
   - Fold over an explicitly empty behavior scope.
   - Assert `DimensionOk { composed: identity, witnesses: [] }`.
   - The test must fail if the implementation fabricates `0`, `None`, or any
     host default instead of reading `Lens.sequential.identity`.

2. **Monoid op is consumed for Bind sequencing.**
   - Build a two-step behavior scope whose `Lens.read` returns
     `Witness::Inhabits` values `a` and `b`.
   - Assert composed output equals the result of executing
     `Lens.sequential.op(a, b)`.
   - The test must fail if Rust hard-codes integer addition, symbolic-cost
     composition, list append, or any per-lens bespoke combine function.

3. **Witness violation lifts to `DimensionFail`.**
   - Make `Lens.read` return `Witness::Violates { reason, at }` for one
     behavior.
   - Assert the fold returns `DimensionFail { violations, witnesses }` with no
     `composed` value.
   - Assert the failing witness is retained in `witnesses`; do not parse
     `reason` text for control flow.

4. **Validation diagnostic lifts to `DimensionFail`.**
   - Make all `read` calls return `Inhabits`, compose successfully, then make
     `Lens.validate` return `SomeDiagnostic { value }`.
   - Assert `DimensionFail` carries the typed `Diagnostic` in `violations` and
     does not fabricate `DimensionOk`.

5. **Validation success lifts to `DimensionOk`.**
   - Same as the previous test, but `Lens.validate` returns `NoDiagnostic`.
   - Assert `DimensionOk` carries `dimension_name` from `Lens.name`, the
     composed carrier, and the full witness list.

6. **Callable and field projection gaps fail closed before E6-G0.**
   - Before `TransformTarget::Callable` and `FieldProject` are live, invoking
     the fold should return a typed evaluator diagnostic naming the unsupported
     target.
   - After E6-G0 lands, this test should be replaced by the positive
     projection/call tests above.

7. **Descent is explicitly residual in the first fold scope.**
   - If the first E6 implementation is cardinality-only, include a program
     behavior scope with `LoopBound::Descent`.
   - Assert a typed fail-closed residual, not `DimensionOk`, not
     `UnknownCost`, not a fabricated identity, and not a silently skipped loop.

## Required Non-Changes

E6-G1 must not:

- add `LensReport`, `WitnessResult`, or a new `DimensionReport` variant;
- add a Rust-only lens registry to stand in for declared `Lens<C>` data;
- add per-lens callbacks for cost, complexity, tenant flow, IFC, or tests;
- route through `test_runner.rs` or add a `TestPredicate`;
- compare `Witness.reason` strings for control flow;
- widen `EvalStrategy`, add `EvalThunk`, or add observable `Value` variants.

## Handoff Order

Recommended next dispatch order:

1. **Substrate/API:** land the smallest honest way to author or reference one
   `Lens<C>` value with function-valued fields and `Monoid<C>` witness fields.
   The 2026-05-03 substrate reassessment STOP receipt is
   [`r3-pr-e6-lens-value-authoring-stop.md`](r3-pr-e6-lens-value-authoring-stop.md):
   current `main` still lacks the honest function-valued structural data
   surface for that value.
2. **E6-G0 evaluator API:** land `FieldProject` and `Callable` execution for
   those fields through the existing `evaluate_body` / `eval_node` /
   `eval_port` entry points.
3. **E6-G1:** implement report lifting and the empty/single/small sequential
   generic fold tests above.
4. **Descent follow-on:** extend the fold to Descent only after termination
   evidence is executable; until then, keep Descent fail-closed.
