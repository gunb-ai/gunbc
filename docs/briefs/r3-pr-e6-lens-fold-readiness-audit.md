# R3 PR-E E6 Lens Fold Readiness Audit

**Status:** E6 implementation blocker after E1-E5 + Bind callable entry.
This is a docs-only receipt; it does not implement the generic lens fold,
`DimensionReport` construction, runner behavior, substrate carriers,
strategy/value widening, or a replacement lens interpreter.

**Parent authorities:** [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
E6, [`r2-pr-e-lens-application-over-reflected-program-dag.md`](r2-pr-e-lens-application-over-reflected-program-dag.md),
[`r2-pr-c-reflection-dissolution-gates.md`](r2-pr-c-reflection-dissolution-gates.md),
[`../design-lens-framework.md`](../design-lens-framework.md), and
[`r3-pr-e5-loop-readiness-audit.md`](r3-pr-e5-loop-readiness-audit.md).

## Dispatch Outcome

The 2026-05-02 E6 implementation dispatch was rechecked against current
`main` after E5 cardinality Loop execution and Bind callable entry landed.
The result is still STOP+PING for implementation: the body evaluator can now
execute the five L1 `Behavior` variants as program bodies, but it still cannot
faithfully execute `Lens<C>` function-valued fields or lift their outputs into
the existing `Witness<C>`, `OptionalDiagnostic`, and `DimensionReport<C>`
carriers without adding a second lens interpreter or new runtime carrier
authority.

## Live Surfaces

`src/v3/std/lens.dag` declares the live `Lens<C>` carrier:

```text
type Lens<C> {
  name: String
  read: fn(Dag, Behavior) -> Witness<C>
  sequential: Monoid<C>
  branch: fn(C, C) -> C
  iterate: fn(C, LoopBound) -> C
  validate: fn(Dag, C) -> OptionalDiagnostic
}
```

`src/v3/std/dimensions.dag` declares the result carriers E6 must consume
verbatim:

```text
Witness<C> = Inhabits(C) | Violates { reason: String, at: Behavior }
OptionalDiagnostic = NoDiagnostic | SomeDiagnostic { value: Diagnostic }
DimensionReport<C>
  = DimensionOk { dimension_name, composed, witnesses }
  | DimensionFail { dimension_name, violations, witnesses }
```

The landed Rust compatibility seam remains
`fold_lens_over_reflected_program` in `src/v3/compiler/src/lens_apply.rs`.
It reflects program nodes selected by `source_file`, prepends that reflected
carrier to caller inputs, and delegates to `apply_lens_declaration`. It
returns `FieldValue`, not `DimensionReport<C>`. That seam is reflect -> apply
compatibility; it is not the generic `fold_lens<C>: Lens<C> -> Dag ->
DimensionReport<C>` promised by the lens framework.

The body evaluator in `src/v3/compiler/src/lib.rs` now has the shared E1-E5
program-body spine:

- `Behavior::Value` evaluates to runtime `Value::LiteralValue`.
- `Behavior::Transform` executes the current eager arithmetic/comparison
  subset and fails closed for unsupported targets.
- `Behavior::Branch` executes resolved variant arms with payload-frame
  discipline.
- `Behavior::Loop` executes `LoopBound::Cardinality`; `LoopBound::Descent`
  remains `LoopBoundDescentResidual`.
- `Behavior::Bind` executes bounded callable/body entry by copying declared
  params into a fresh frame and restoring stack depth on success/error.

This is enough to execute ordinary program bodies in the landed E-slices. It is
not yet enough to execute a `Lens<C>` instance as data.

## Intended E6 Fold Shape

The first executable E6 implementation should consume the existing substrate
carriers rather than adding a parallel report or lens type:

```text
fn fold_lens<C>(
    dag: &Dag,
    lens: Lens<C>,
    strategy: &EvalStrategy,
) -> Result<DimensionReport<C>, EvalError>
```

Algorithmic obligations:

1. Select the program behavior set through structural `Dag` authority, not
   emitted bytes or runner-only observation.
2. For each `Behavior`, execute `Lens.read(dag, behavior)` through the shared
   body-evaluator authority. `Inhabits(c)` contributes carrier evidence;
   `Violates {..}` records a witness failure and prevents a fabricated
   `composed` value.
3. Compose successful carrier values with the lens-declared operations:
   `Lens.sequential.identity` for empty/unit sequential structure,
   `Lens.sequential.op` for Bind sequencing, `Lens.branch` for exclusive
   branch arms, and `Lens.iterate` for loops. `sequential` is a `Monoid<C>`
   carrier, not a direct Rust callback.
4. Execute `Lens.validate(dag, composed)` after successful composition.
   `SomeDiagnostic` yields `DimensionFail`; `NoDiagnostic` yields
   `DimensionOk`.
5. Return the existing `DimensionReport<C>` shape verbatim.

## Current Blockers

E6 still cannot honestly implement the full `fold_lens` /
`DimensionReport<C>` path on current `main`.

- **No live `Lens<C>` data instances to consume.** The substrate carrier exists,
  but representative lens instances such as `data cost_lens: Lens<SymbolicCost>`
  and `data complexity_lens: Lens<...>` are explicitly deferred in
  `src/v3/lenses/cost.dag` and `src/v3/lenses/complexity.dag`. The blockers are
  class-5 structural data bodies, function-valued field references, and the
  `Witness<C>` / `OptionalDiagnostic` constructors needed by `read` and
  `validate`. A Rust-side registry would be a parallel lens authority.
- **Callable field execution is not live in the body evaluator.** `Bind`
  callable entry is live, but `TransformTarget::Callable(_)` still fails
  closed as `UnsupportedTransformTarget { kind: "Callable" }` in
  `evaluator::eval_transform_node`. `Lens.read`, `Lens.branch`,
  `Lens.iterate`, `Lens.validate`, and `Lens.sequential.op` are function-valued
  fields; E6 cannot call them through `eval_node` / `eval_port` yet.
- **Record/function-field projection is not live in the body evaluator.**
  `Lens<C>.sequential` must project `Monoid<C>.identity` and `Monoid<C>.op`,
  and the fold must project `Lens<C>` fields. `TransformTarget::FieldProject`
  still fails closed in the evaluator. Projecting those fields in host Rust
  would bypass the declared carrier.
- **Runtime values cannot yet represent the fold inputs and outputs.** The
  runtime `Value` mirror has the five PR-A.1 inhabitants, but there is no
  evaluator-owned lifting for substrate-shaped `Dag`, `Behavior`,
  `Witness<C>`, `OptionalDiagnostic`, `Diagnostic`, or
  `DimensionReport<C>` values. Adding ad hoc Rust mirrors for these in E6 would
  create a second report/witness authority.
- **Descent remains a named residual.** Cardinality loops execute, but
  `LoopBound::Descent` is still fail-closed. A first generic fold must either
  declare a cardinality-only program scope or wait for descent evidence
  execution; it must not silently treat descent as zero, one, or unknown.
- **Program scope is not settled for the generic fold.** The compatibility seam
  filters by `SourceSpan.file`. Generic `fold_lens<C>: Lens<C> -> Dag ->
  DimensionReport<C>` needs a structural program-scope decision or an explicit
  whole-Dag first slice. File-path filtering must not become the generic fold's
  authority by accident.

These blockers mean a code slice today would either duplicate
`lens_apply.rs`'s `FieldValue` interpreter, bypass PB-Runtime body semantics,
or return something weaker than `DimensionReport<C>`. This receipt records the
gap instead of landing misleading partial behavior.

## Resume Gate

E6 implementation may resume when all of the following are true:

- at least one representative `Lens<C>` instance is live as substrate data, or
  an explicitly approved typed lens-instance handle exists, with function
  fields sourced from the declared carrier rather than a Rust registry;
- `TransformTarget::Callable` and required field projection execute through the
  shared evaluator boundary, so `Lens.read`, `Lens.sequential.op`,
  `Lens.branch`, `Lens.iterate`, and `Lens.validate` do not need a second
  interpreter;
- report lifting consumes the existing `Witness<C>`, `OptionalDiagnostic`, and
  `DimensionReport<C>` carriers verbatim, including fail-closed
  `DimensionFail` construction when any witness violates or validation returns
  a diagnostic;
- the implementation names its structural program-scope authority, or
  explicitly scopes the first fold to whole-Dag traversal;
- descent handling is either executable through termination evidence or
  explicitly excluded by the first implementation scope with tests proving
  `LoopBound::Descent` fails closed.

Required implementation tests:

- a live lens instance produces `DimensionOk` through the generic fold, not
  through `apply_lens_declaration` reflection plumbing;
- a validation lens produces `DimensionFail` with typed diagnostics and no
  fabricated `composed` value;
- empty/unit sequential structure consumes `Lens.sequential.identity`;
- Bind sequencing composes through `Lens.sequential.op`;
- branch and loop composition use the declared `Lens<C>.branch` and
  `Lens<C>.iterate` fields;
- callable/field-projection gaps fail closed if the fold is invoked before
  those evaluator capabilities are live;
- `fold_lens_over_reflected_program` remains either a compatibility seam or is
  narrowed with a named dissolution trigger once the generic fold path lands.

## Non-Goals

This readiness receipt does not authorize:

- a second `LensReport` / `DimensionReport` shape;
- new observable `Value` variants;
- a Rust lens registry that substitutes for declared `Lens<C>` data;
- per-lens Rust callbacks for `read`, `branch`, `iterate`, `validate`, or
  sequential composition;
- strategy carrier widening, `NormalOrder`, `RightFirst`, or `EvalThunk`;
- substrate edits to `Lens<C>`, `Witness<C>`, `OptionalDiagnostic`, or
  `DimensionReport<C>`;
- runner, `test_runner.rs`, or `BinaryDimensionReportEquals` changes.
