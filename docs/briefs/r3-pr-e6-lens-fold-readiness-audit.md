# R3 PR-E E6 Lens Fold Readiness Audit

**Status:** E6 implementation blocker after E1-E5 + Bind callable entry +
G0a/G0b/G0c. This is a docs-only receipt; it does not implement the
generic lens fold, `DimensionReport` construction, runner behavior,
substrate carriers, strategy/value widening, or a replacement lens
interpreter.

**Parent authorities:** [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
E6, [`r2-pr-e-lens-application-over-reflected-program-dag.md`](r2-pr-e-lens-application-over-reflected-program-dag.md),
[`r2-pr-c-reflection-dissolution-gates.md`](r2-pr-c-reflection-dissolution-gates.md),
[`../design-lens-framework.md`](../design-lens-framework.md),
[`r3-pr-e5-loop-readiness-audit.md`](r3-pr-e5-loop-readiness-audit.md),
[`r3-pr-e6-g0-first-gate-narrowing.md`](r3-pr-e6-g0-first-gate-narrowing.md),
[`r3-pr-e6-g0b-x1a-static-field-call-worker.md`](r3-pr-e6-g0b-x1a-static-field-call-worker.md),
[`x1b-evaluator-impact-audit.md`](x1b-evaluator-impact-audit.md), and
[`../design-prereq-x-ho-field-call.md`](../design-prereq-x-ho-field-call.md).

## Refresh — what changed since the original audit

This audit was written before the E6-G0 first-gate slices landed. The
relevant landings on `origin/main`:

- **E6-G0a — generic Conj field substitution** (PR #1640).
- **E6-G0b — Prereq-X1.a static call-on-field-access (parse + lower)**
  (PR #1699). Static `data v: WrapFn = { f: double }; v.f(x)` parses
  and lowers; `TransformTarget::Callable` is reused.
- **X1.b evaluator-side impact audit** (PR #1712). Docs-only, atomic
  S1 substrate migration is mandatory; STOP conditions and slice
  split documented.
- **E6-G0c — host evaluator `FieldProject` and `Callable` execution**
  (PR #1715). `eval_transform_node` now executes
  `TransformTarget::FieldProject` (record-carrier projection by
  label) and `TransformTarget::Callable` (Arrow `UserDefined` body
  via push-frame + `eval_callable_body_in_pushed_frame`). Both arms
  return typed `EvalError` on shape mismatch, no fallback to host
  Rust.
- **E1–E5 + `Bind` body coverage** is in `src/v3/compiler/src/lib.rs`
  on `origin/main`: `Behavior::Value`, `Transform`, `Branch`, `Loop`
  (Cardinality), and `Bind` all execute through the shared body
  evaluator with payload-frame discipline. Runtime `Value` carries
  `LiteralValue`, `RecordValue(Vec<NamedField>)`, `VariantValue`,
  `NodeRef`, and `CardinalityValue`.

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

`src/v3/std/dimensions.dag` declares the result carriers E6 must
consume verbatim:

```text
Witness<C> = Inhabits(C) | Violates { reason: String, at: Behavior }
OptionalDiagnostic = NoDiagnostic | SomeDiagnostic { value: Diagnostic }
DimensionReport<C>
  = DimensionOk { dimension_name, composed, witnesses }
  | DimensionFail { dimension_name, violations, witnesses }
```

The Rust compatibility seam `fold_lens_over_reflected_program` in
`src/v3/compiler/src/lens_apply.rs:337` remains. It still reflects
program nodes selected by `source_file`, prepends that reflected
carrier to caller inputs, and delegates to `apply_lens_declaration`,
returning `FieldValue`. **No dissolution path has landed.** It is
preserved as a compatibility seam pending generic
`fold_lens<C>: Lens<C> -> Dag -> DimensionReport<C>`.

## What G0a/G0b/G0c have unblocked (static top-level `data` lens path)

Capabilities now executable through the shared body evaluator.
The two static paths are distinct — keep them separate:

1. **Runtime field projection on record carriers (non-function
   fields).** `TransformTarget::FieldProject { field_label, .. }`
   evaluates by matching the operand `Value::RecordValue(fields)`
   and returning the named field's value (`lib.rs:556-578`). This
   is for *value* (non-Arrow) field access at runtime, e.g.
   reading a `name: String` or a numeric field from a record. It
   is **not** the path used to invoke a function-typed field on a
   static `data` binding.
2. **Static callable dispatch.** `TransformTarget::Callable(decl)`
   evaluates by resolving the Arrow `UserDefined` body's `Bind`,
   binding params from operands in a fresh pushed frame, evaluating
   the body, and popping deterministically (`lib.rs:579-617`). Arity
   mismatch and non-`UserDefined` Arrow bodies fail closed.
3. **Static function-field call (X1.a) lowers directly to
   `Callable`.** Per
   `design-prereq-x-ho-field-call.md` §L1.a (and PR #1699), a
   program of the form `data v: WrapFn = { f: double }; fn r(x:
   Int) -> Int = v.f(x)` projects `v.f` to
   `FieldValue::Reference(decl_id_of_double)` **at lowering time**
   and emits `TransformTarget::Callable(decl_id_of_double)`
   directly. There is no runtime `FieldProject` transform on this
   path — the field projection is compile-time, the carrier
   identity is preserved through the lowering boundary, and G0c's
   `Callable` arm does the rest. Runtime `FieldProject` (item 1)
   and static function-field call (item 3) are separate paths;
   conflating them is the X1.a/X1.b authority split.
4. **Generic Conj field substitution.** G0a's substitution surface
   means a `Lens<Int>` parametric instantiation that resolves to a
   concrete Conj at lowering time can have its field types
   substituted structurally rather than through a hand-coded shim.

What this means for E6: a **static top-level `data` lens instance**
of the shape

```text
data complexity_lens: Lens<Int> = {
  name: "complexity",
  read: complexity_read,           // fn ref to a top-level decl
  sequential: complexity_monoid,
  branch: complexity_branch,
  iterate: complexity_iterate,
  validate: complexity_validate,
}
```

is now executable through the body evaluator without an X1.b
runtime-callee dispatch step, *if and only if* every lens-field call
site is statically resolvable: the lens binding itself is a top-level
`data`, and each function-field call site (`complexity_lens.read`,
`complexity_lens.sequential.op`, `complexity_lens.branch`, etc.)
projects its field at lowering time to
`FieldValue::Reference(decl_id)` and emits
`TransformTarget::Callable(decl_id)` directly per X1.a §L1.a
— with **no** runtime `FieldProject` transform on the function-call
path and no `Indirect`-style runtime-port callee. Non-function
field reads on the lens record (e.g. `complexity_lens.name`) use
the runtime `FieldProject` path; function-field calls do not.

A first-slice **static-lens-scoped fold** can now be written against
this surface without bypassing the declared carriers.

## What still blocks generic parametric `fold_lens<C>`

Verified against `origin/main`:

- **Runtime-sourced callee dispatch (Prereq-X1.b).** The body of
  `fn fold_lens<C>(lens: Lens<C>, d: Dag) -> DimensionReport<C> = …`
  contains call sites of the form `lens.read(...)`,
  `lens.sequential.op(...)`, `lens.branch(...)`, `lens.iterate(...)`,
  `lens.validate(...)` where `lens` is a **function parameter**, not
  a top-level binding. Per `design-prereq-x-ho-field-call.md` §X1.b
  and `x1b-evaluator-impact-audit.md`, this requires the
  `TransformDispatch::Indirect(IndirectDispatch { callee:
  ArrowPortRef, args })` substrate carrier collapse, which has not
  landed. Until that lands and an evaluator slice (the audit's S3)
  implements `Indirect` evaluation, every parametric-`Lens<C>` call
  site fails closed at the lowering or evaluator boundary.
- **Structural program-scope authority.** The compatibility seam
  filters by `SourceSpan.file`. A generic
  `fold_lens<C>: Lens<C> -> Dag -> DimensionReport<C>` needs either
  a structural program-scope decision (the `Dag` argument carries
  the scope, with no `source_file` filter) or an explicit whole-Dag
  first slice. File-path filtering must not become the generic
  fold's authority by accident.
- **Typed `DimensionReport<C>` construction rules.** A first
  generic fold must lift `Witness<C>` outcomes into `DimensionOk` or
  `DimensionFail` verbatim. There is no evaluator-owned constructor
  for `DimensionReport<C>` yet; `apply_lens_declaration` returns
  `FieldValue`. The fold body must construct `DimensionReport<C>`
  through the declared substrate variants
  (`DimensionOk { dimension_name, composed, witnesses }` /
  `DimensionFail { dimension_name, violations, witnesses }`)
  through the body evaluator, not through a host-Rust struct. That
  requires the lens-instance program to declare and lower the
  variant-construction syntax for these substrate types — verifiable
  but unimplemented in any landed lens program.
- **Descent residual.** `LoopBound::Descent` remains fail-closed in
  the body evaluator. A first generic fold must either declare a
  cardinality-only program scope or wait for descent termination
  evidence; it must not silently treat descent as zero, one, or
  unknown.
- **Live `Lens<C>` data instances.** Representative lens instances
  (`data cost_lens: Lens<SymbolicCost>`,
  `data complexity_lens: Lens<...>`) remain explicitly deferred in
  `src/v3/lenses/cost.dag` and `src/v3/lenses/complexity.dag`.
  Authoring them requires class-5 structural data bodies and the
  `Witness<C>` / `OptionalDiagnostic` constructor surface.
- **Runtime values for substrate-shaped fold inputs and outputs.**
  The runtime `Value` mirror has five inhabitants
  (`LiteralValue`, `RecordValue`, `VariantValue`, `NodeRef`,
  `CardinalityValue`); there is no evaluator-owned lifting for
  substrate-shaped `Dag`, `Behavior`, `Witness<C>`,
  `OptionalDiagnostic`, `Diagnostic`, or `DimensionReport<C>`
  values. `RecordValue` plus `VariantValue` covers a lot — the
  question is which substrate types lower to which `Value` variants
  and whether any new mirror is required. Adding ad hoc Rust
  mirrors in E6 would create a second report/witness authority.

## Old blocker language to retire or narrow

The previous audit's "Current Blockers" section claimed the
following; each is now incorrect against `origin/main`:

- ❌ **"Callable field execution is not live in the body evaluator …
  `TransformTarget::Callable(_)` still fails closed."** Retire.
  G0c implemented it (`lib.rs:579-617`).
- ❌ **"Record/function-field projection is not live in the body
  evaluator … `TransformTarget::FieldProject` still fails closed."**
  Retire. G0c implemented it (`lib.rs:556-578`). Field projection
  on record carriers is structural; non-record carriers fail
  closed with a typed reason, which is the intended boundary, not
  a gap.
- ⚠️ **"E6 cannot call lens fields through `eval_node` / `eval_port`
  yet."** Narrow: this is true only for the **runtime-sourced
  callee** case (Prereq-X1.b). Static lens-instance call paths are
  now evaluable.
- ⚠️ **"`Lens<C>.sequential` must project `Monoid<C>.identity` /
  `Monoid<C>.op` … projecting those fields in host Rust would
  bypass the declared carrier."** Narrow: non-function nested
  field reads (e.g. a hypothetical numeric `Monoid<C>.identity`
  literal) project at runtime through `FieldProject` (item 1).
  Function-valued nested fields (`Monoid<C>.op`, the `Lens<C>`
  function fields themselves) do **not** route through runtime
  `FieldProject`; on the static path they project at lowering
  time and emit `Callable(decl_id)` directly per X1.a §L1.a.
  On the parametric path they need X1.b runtime-callee dispatch.
  Either way, host-Rust projection of the declared carrier is
  not required.

The remaining blockers in the previous audit
("No live `Lens<C>` data instances", "Descent remains a named
residual", "Program scope is not settled", "Runtime values cannot
yet represent the fold inputs and outputs") all still hold and are
preserved verbatim above.

## Immediate next executable receipt

The post-G0c static X1.a evaluator ratchet — closing the
G0b parser/lowerer surface with a body-evaluator round-trip
acceptance for `T1.1` (`data v: WrapFn = { f: double }; fn r(x: Int)
-> Int = v.f(x)` evaluates to `double(x)` via `eval_node`) — is
**assigned to `fierce-bear`** per the dispatch routing for E6-G0c.
This audit deliberately does not duplicate that work; it does not
author a worker brief or implementation slice for the static X1.a
evaluator ratchet.

## Conservative E6-G1 next-step recommendation

E6-G1 has two viable shapes. The recommendation is the **static-lens
first-slice** path, with parametric `fold_lens<C>` deferred to
post-X1.b.

### Recommended: G1.a — static-lens-scoped first fold

Author a single non-parametric fold against a top-level
`data complexity_lens: Lens<Int>` instance (or the smallest live
instance available; if none yet, this is in turn gated on landing
one in `src/v3/lenses/complexity.dag`). Scope:

- Lens binding is a **top-level `data`**, not a parameter.
- All field calls are statically resolvable
  (`TransformTarget::FieldProject` then
  `TransformTarget::Callable`); no runtime-port callee.
- Loop scope is **cardinality-only**; descent is excluded with a
  fail-closed test asserting `LoopBound::Descent` is rejected.
- Whole-Dag traversal; no `source_file` filter. The compatibility
  seam `fold_lens_over_reflected_program` is preserved unchanged
  as a parallel surface.
- Returns `DimensionReport<Int>` constructed verbatim through the
  declared substrate variants.

This is a deliberately scoped compromise: it does **not** validate
the full `fold_lens<C>: Lens<C> -> Dag -> DimensionReport<C>`
generic surface. It validates only that the static path is
end-to-end executable through G0a/G0b/G0c. **The audit explicitly
does not authorize claiming generic `fold_lens<C>` is
implementation-ready until X1.b's S1 substrate slice + S3
evaluator slice land.** A first slice that hard-codes lens fields in
host Rust to bypass X1.b is rejected.

### Deferred: G1.b — generic parametric `fold_lens<C>`

Blocked on the X1.b chain (per `x1b-evaluator-impact-audit.md`):
S1 substrate carrier collapse → S2 lowerer runtime-callee production
→ S3 evaluator `Indirect` evaluation. G1.b also blocked on
`DimensionReport<C>` construction conventions (above) and a
structural program-scope decision. Resume gating is the X1.b S3
slice merge plus a typed `DimensionReport<C>` construction receipt.

## Resume Gate (refreshed)

E6 implementation may resume per the recommended slice when:

- at least one representative `Lens<C>` instance is live as
  substrate data, with function fields sourced from the declared
  carrier rather than a Rust registry;
- the static-lens path uses G0c's `FieldProject` / `Callable`
  evaluation through the shared body evaluator, no second
  interpreter;
- report lifting consumes the existing `Witness<C>`,
  `OptionalDiagnostic`, and `DimensionReport<C>` carriers verbatim,
  including fail-closed `DimensionFail` construction when any
  witness violates or validation returns a diagnostic;
- the implementation names its structural program-scope authority
  or explicitly scopes the first fold to whole-Dag traversal;
- descent handling is either executable through termination evidence
  or explicitly excluded by the first implementation scope with
  tests proving `LoopBound::Descent` fails closed;
- `fold_lens_over_reflected_program` remains a compatibility seam
  with no claimed dissolution path until generic `fold_lens<C>`
  lands; do **not** narrow it speculatively.
- substrate-shaped runtime values: any `Dag` / `Behavior` /
  `Witness<C>` / `OptionalDiagnostic` / `Diagnostic` /
  `DimensionReport<C>` value the fold reads or constructs is
  represented through the existing five-inhabitant `Value` mirror
  (`LiteralValue` / `RecordValue` / `VariantValue` / `NodeRef` /
  `CardinalityValue`) without adding new mirror variants or
  host-Rust struct authorities. If a substrate type cannot be
  expressed through the existing mirror, the slice STOPs and
  routes through P1 — it does not extend `Value` ad hoc. This is
  the same prerequisite as the corresponding blocker; restated
  here so G1 readers do not miss the construction question.

For the parametric form (G1.b), additionally:
- X1.b S1 substrate slice merged (atomic
  `TransformDispatch` collapse per `x1b-evaluator-impact-audit.md`);
- X1.b S3 evaluator slice merged
  (`TransformDispatch::Indirect` evaluation in
  `eval_transform_node` and `lens_apply.rs:eval_transform`).

## Non-Goals

This refreshed readiness receipt does not authorize:

- a second `LensReport` / `DimensionReport` shape;
- new observable `Value` variants beyond
  `LiteralValue`/`RecordValue`/`VariantValue`/`NodeRef`/
  `CardinalityValue`;
- a Rust lens registry that substitutes for declared `Lens<C>`
  data;
- per-lens Rust callbacks for `read`, `branch`, `iterate`,
  `validate`, or sequential composition;
- strategy carrier widening, `NormalOrder`, `RightFirst`, or
  `EvalThunk`;
- substrate edits to `Lens<C>`, `Witness<C>`, `OptionalDiagnostic`,
  or `DimensionReport<C>`;
- runner, `test_runner.rs`, or `BinaryDimensionReportEquals`
  changes;
- any implementation slice that hard-codes lens fields in host
  Rust to bypass Prereq-X1.b runtime-callee dispatch;
- duplication of the post-G0c static X1.a evaluator ratchet
  assigned to `fierce-bear`;
- narrowing or retiring `fold_lens_over_reflected_program`
  speculatively.

— refreshed 2026-05-05 by sunny-hawk-622 (R3 Evaluator) at the
request of snappy-moth-795. Docs-only.
