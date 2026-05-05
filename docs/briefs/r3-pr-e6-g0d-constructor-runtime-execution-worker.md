# R3 PR-E E6-G0d Constructor Runtime Execution Worker Brief

**Status:** worker brief / audit for the next Evaluator slice after E6-G0c.
This is docs-only. It does not implement constructor execution, edit the
parser/lowerer, widen `Value`, touch `lens_apply.rs`, add `fold_lens` logic, or
port Population A property tests.

**Parent authorities:** [`r3-pr-e6-lens-fold-readiness-audit.md`](r3-pr-e6-lens-fold-readiness-audit.md),
[`r3-pr-e6-post-blocker-gate-packet.md`](r3-pr-e6-post-blocker-gate-packet.md),
[`r3-pb-tv2-population-coverage-audit.md`](r3-pb-tv2-population-coverage-audit.md),
and the live evaluator / lowerer in `src/v3/compiler/src/lib.rs` and
`src/v3/compiler/src/lower.rs`.

**Current main context:** #1721 landed the clean E6-G0c receipt for static data
field projection plus Arrow/UserDefined `Callable` execution. #1723 landed PB's
Population A reclassification: Pop A remains blocked because constructor
expressions lower to non-Arrow `TransformTarget::Callable(target)` values that
the evaluator still rejects.

## Problem

E6-G0c made `TransformTarget::Callable(decl)` executable only when `decl`
resolves to an Arrow with a `UserDefined` body. The current evaluator branch
fails closed on every non-Arrow callable target:

- `src/v3/compiler/src/lib.rs:579-585` reads
  `dag.declaration(callee_decl).connective`, accepts only
  `TypeConnective::Arrow { body, .. }`, and otherwise returns
  `BadTransformOperands { reason: "Callable target declaration is not an Arrow type" }`.

That is correct for ordinary function dispatch, but the lowerer also uses
`TransformTarget::Callable(target_decl)` as the lowered representation for
constructor execution. Those constructor targets are declarations for record
and variant shapes, not Arrow functions, so they currently hit the G0c
fail-closed path.

PB's post-#1723 Population A reclassification is the consumer-facing symptom:
the substrate declarations exist, but behavioral tests cannot run because
values such as `Some { value: ... }`, `ErrorBound`, `ConstantShrink { steps:
... }`, and `StrictSubValue { field, factor }` require constructor runtime
values before user-defined Arrow bodies can complete.

## Target Shapes Reaching `Callable(non_arrow_decl)`

Two lowered constructor families reach `TransformTarget::Callable(target)`:

1. **Record constructors.**
   - `lower_record_literal_expr` resolves the expected record type to a
     `Conj`, orders inputs by the declaration's `children` labels, lowers each
     field expression against its field type, then calls
     `lower_constructor_invocation(dag, target_decl, inputs, span)` at
     `src/v3/compiler/src/lower.rs:7245-7306`.
   - `lower_constructor_invocation` emits
     `Behavior::Transform(TransformNode { target:
     TransformTarget::Callable(target), inputs, ... })` at
     `src/v3/compiler/src/lower.rs:7103-7118`.
   - The `target_decl` here is the record / Conj declaration, not an Arrow.

2. **Variant constructors.**
   - Nullary variants can enter through `SurfaceExpr::Var` when the expected
     type is a sum: `resolve_expected_variant_constructor(...)` followed by
     `lower_constructor_invocation(..., Vec::new(), ...)` at
     `src/v3/compiler/src/lower.rs:6574-6580`.
   - Named variant-record constructors enter through
     `lower_variant_record_expr`: the lowerer resolves the variant declaration,
     reads payload field labels/types via
     `variant_payload_fields_for_lowering`, orders payload inputs by those
     declared labels, and calls `lower_constructor_invocation(dag,
     variant_decl, inputs, ...)` at
     `src/v3/compiler/src/lower.rs:7317-7404`.
   - `variant_payload_fields_for_lowering` gets the payload fields by walking
     the variant declaration to a `Conj` and preserving child labels at
     `src/v3/compiler/src/lower.rs:6328-6347`.
   - The `target` here is the variant constructor declaration, not an Arrow.

This confirms the next Evaluator slice is not "broaden Callable" in general.
It is: when a lowered `Callable` target is a constructor declaration already
accepted by the lowerer, evaluate it into the existing runtime `Value` carrier.

## Runtime Output Contract

Use only the live evaluator carriers in `src/v3/compiler/src/lib.rs:71-87`:

```rust
Value::RecordValue(Vec<NamedField>)
Value::VariantValue {
    tag: DeclarationId,
    payload: Box<Value>,
}
```

Expected outputs:

- **Record constructor target:** return
  `Value::RecordValue(Vec<NamedField>)`, with one `NamedField { label, value }`
  per declared record field, in declaration order. Field labels come from the
  target declaration's `TypeConnective::Conj { children }`. Field values come
  from the already-evaluated transform operands, preserving the existing
  evaluator's operand evaluation order and arity discipline.
- **Variant constructor target with record payload:** return
  `Value::VariantValue { tag: variant_decl, payload:
  Box::new(Value::RecordValue(fields)) }`, with payload field labels from the
  variant declaration's payload `Conj` children and values from operands.
- **Nullary variant constructor target:** return
  `Value::VariantValue { tag: variant_decl, payload:
  Box::new(Value::RecordValue(Vec::new())) }`.

No new `Value` inhabitants are authorized by this slice. The evaluator must
not introduce host-only mirrors for `Option`, `CostBound`, `ShrinkFactor`,
`SubValueRelation`, `DimensionReport`, or any Population A family.

## Responsibility Split

- **Substrate authority remains in the Dag declarations.** Declaration
  connective shapes, variant membership, payload field labels, record field
  labels, generic substitution, and constructor identity belong to the
  substrate/lowerer authority that already produced the lowered
  `TransformTarget::Callable(target)` node.
- **Evaluator authority is runtime interpretation only.** Given a lowered
  constructor `Callable` target and already-evaluated operands, the evaluator
  may construct `Value::RecordValue` or `Value::VariantValue` using facts read
  from `dag.declaration(target)`. It may enforce arity and shape fail-closed.
- **PB / Population A stays a consumer.** Pop A property migration should wait
  until this constructor execution gate lands. This gate should not port Pop A
  tests or hard-code its property families.

## First Implementation Slice

Implement the non-Arrow constructor arm inside the existing
`TransformTarget::Callable(callee_decl)` branch in `eval_transform_node`.
Recommended flow:

1. Keep the current Arrow/UserDefined path unchanged.
2. Before returning `"Callable target declaration is not an Arrow type"`,
   recognize constructor target shapes:
   - `TypeConnective::Conj { children }` => build `RecordValue`.
   - Variant declaration whose payload walks to `Conj` => build
     `VariantValue` with `RecordValue` payload.
   - Nullary variant declaration => build `VariantValue` with empty
     `RecordValue` payload.
3. For every constructor path, compare `operands.len()` to the declared field
   count and return `TransformArityMismatch` on mismatch.
4. If declaration walking cannot identify a constructor shape, preserve the
   existing fail-closed reason for non-Arrow callables.

The implementation should reuse existing declaration-walking helpers where
they are already public to the module. If the only way forward is to duplicate
or recreate lowerer-only substitution/walking rules in a Rust-side registry,
STOP rather than adding a second constructor authority.

## Executable Ratchets

Add focused evaluator tests in the existing `#[cfg(test)]` module in
`src/v3/compiler/src/lib.rs`. Do not add new Rust test files.

At minimum:

1. **Variant constructor return executes.**
   - Use a small program such as:
     ```text
     fn pack(x: Int) -> Int? = Some { value: x }
     ```
     or `Some(x)` if that is the syntax accepted by the current parser for
     the selected fixture.
   - Evaluate the function body through the existing evaluator entry point.
   - Assert the result is `Value::VariantValue { tag: some_decl, payload:
     RecordValue([{ label: "value", value: LiteralValue(Int(...)) }]) }`.
   - This fails today at the non-Arrow `Callable` gate.

2. **Record constructor return executes.**
   - Use a minimal record-returning function with an expected record type.
   - Assert the result is `Value::RecordValue` with declared labels and operand
     values in declaration order, not source-order accident.

3. **Nullary variant constructor executes.**
   - Use an `Int?` / option-like `none` value or another existing nullary
     variant.
   - Assert the payload is an empty `RecordValue`, matching the existing
     runtime representation used by branch matching.

If match/unpack syntax is stable enough for the first PR, add one positive
composition test:

```text
fn pack(x: Int) -> Int? = Some { value: x }
fn unpack(x: Int) -> Int =
  match pack(x) {
    Some { value } => value
    none => 0
  }
```

This is useful because `eval_branch` already consumes
`Value::VariantValue`; however, it is not required for the first landing if
pattern syntax would pull the PR into parser/lowerer edits. A constructor-return
receipt is the first honest ratchet.

## STOP Conditions

Stop and surface to the Evaluator manager if the implementation needs any of
the following:

- A Rust-side mirror registry for constructor declarations, variant
  membership, or payload labels.
- New `Value` inhabitants or local report/option/cost/shrink carriers.
- Parser or lowerer surface changes.
- Changes to `lens_apply.rs`, `lens_testgen.rs`, `fold_lens`, X1.b /
  `TransformDispatch`, or Population A porting.
- Hard-coded Pop A property families, constructor names, or result codecs.
- New hand-authored Rust test files.
- Any substrate/carrier ambiguity where declaration/connective facts are not
  sufficient to identify constructor output shape.

## Recommendation

The next PR should be **implementation + ratchets**, not ratchet-only.

Rationale:

- The current failing boundary is already precise and ratcheted by the
  evaluator's non-Arrow `Callable` error; a ratchet-only PR would mostly
  duplicate the known failure from the PB reclassification.
- The implementation surface is narrow if it can read constructor labels from
  existing `Declaration` / `TypeConnective` facts and emit existing
  `Value::RecordValue` / `Value::VariantValue` carriers.
- The useful proof is executable: a constructor-returning body that fails today
  should pass in the same PR that teaches the evaluator to construct the value.

Risk / ETA: **S/M, one focused PR.** Low risk if declaration walking is already
available from the evaluator module or can be factored without changing
substrate authority. Escalate to M if variant payload walking requires generic
substitution logic that is currently lowerer-private; do not paper that over
with a mirror registry.

## Non-Goals

- No broad Population A migration.
- No parser or lowerer edits.
- No X1.b / `TransformDispatch` work.
- No `fold_lens` or `DimensionReport` construction.
- No changes to `lens_apply.rs` or `lens_testgen.rs`.
- No new substrate carrier design.
