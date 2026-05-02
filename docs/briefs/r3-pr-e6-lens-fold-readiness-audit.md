# R3 PR-E E6 Lens Fold Readiness Audit

**Status:** E6 readiness / implementation blocker. This is a docs-only
receipt; it does not implement Transform, Branch, Loop, Bind, the generic lens
fold driver, `DimensionReport` construction, runner behavior, substrate
carriers, or strategy/value widening.

**Parent authorities:** [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
E6, [`r2-pr-e-lens-application-over-reflected-program-dag.md`](r2-pr-e-lens-application-over-reflected-program-dag.md),
[`r2-pr-c-reflection-dissolution-gates.md`](r2-pr-c-reflection-dissolution-gates.md),
[`../design-lens-framework.md`](../design-lens-framework.md), and
[`r3-pr-e5-loop-readiness-audit.md`](r3-pr-e5-loop-readiness-audit.md).

## Live Surfaces

`src/v3/std/lens.dag` declares the `Lens<C>` carrier:

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

`src/v3/std/dimensions.dag` declares the result carriers E6 must produce:

```text
Witness<C> = Inhabits(C) | Violates { reason: String, at: Behavior }
OptionalDiagnostic = NoDiagnostic | SomeDiagnostic { value: Diagnostic }
DimensionReport<C>
  = DimensionOk { dimension_name, composed, witnesses }
  | DimensionFail { dimension_name, violations, witnesses }
```

The landed Rust seam is narrower. `fold_lens_over_reflected_program` in
`src/v3/compiler/src/lens_apply.rs` reflects program nodes selected by
`source_file`, prepends that reflected carrier to caller inputs, and delegates
to `apply_lens_declaration`. It returns `FieldValue`, not
`DimensionReport<C>`. That slice is the reflect -> apply bridge; it is not the
generic `fold_lens<C>: Lens<C> -> Dag -> DimensionReport<C>` promised by the
lens framework.

E1 and E2 are live on `main`: the eager evaluator executes `Behavior::Value` and
has the `EvalStateStack` / `eval_port` frame discipline (`src/v3/compiler/src/lib.rs`,
`evaluator` module).

**E3 (Transform, eager LeftFirst) and E4 (`Behavior::Branch` with
`ResolvedVariant` arms and scrutinee discipline) are also live on `main` in the
same module** — see PR-E ordering in [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md).

**Still not live on the eager body-evaluator spine:** `Behavior::Loop` (PR-E
E5 — [`r3-pr-e5-loop-readiness-audit.md`](r3-pr-e5-loop-readiness-audit.md)) and
`Behavior::Bind` (callable / parameter binding into user bodies). Both remain
fail-closed as `UnsupportedBehavior` until their PR-E slices land, which blocks
executing typical `Lens<C>` function fields (`read`, monoid `op` / `identity`,
`validate`, …) through the shared `eval_node` / `eval_port` boundary without
reviving parallel `lens_apply.rs` frame machinery.

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

The algorithmic shape is:

1. Select the program behavior set through structural `Dag` authority, not
   emitted bytes or a convention-only runner observation.
2. For each `Behavior`, evaluate `Lens.read(dag, behavior)` through the same
   body-evaluator boundary as PR-E behavior execution. `Inhabits(c)` contributes
   carrier evidence; `Violates {..}` records a witness failure and prevents a
   fabricated `composed` value.
3. Compose successful carrier values according to the behavior structure:
   `sequential` for Bind composition, `branch` for exclusive branch arms, and
   `iterate` for loop bounds. For empty / unit sequential structure, the fold
   must evaluate and consume `Lens.sequential.identity`; for non-empty Bind
   composition it must evaluate and consume `Lens.sequential.op`. `sequential`
   is a `Monoid<C>` carrier, not a direct Rust callback; E6 must project and
   execute the declared monoid fields through evaluator authority rather than
   replacing them with per-lens Rust dispatch or fabricated unit values.
4. Evaluate `Lens.validate(dag, composed)` after successful composition.
   `SomeDiagnostic` yields `DimensionFail`; `NoDiagnostic` yields
   `DimensionOk`.
5. Return the existing `DimensionReport<C>` shape verbatim.

## Current Blockers

E6 cannot honestly implement the full `fold_lens` / `DimensionReport` path yet,
even after E3/E4: the fold algorithm still requires **Bind + callable entry**
and **Loop iteration** through the same `eval_node` authority, plus monoid
projection and report lifting rules below.

- **Body-evaluator coverage (lens function fields):** `Lens.read`,
  `Lens.sequential.op`, `Lens.sequential.identity`, `branch`, `iterate`, and
  `validate` must run as ordinary `.dag` bodies through `eval_node` /
  `eval_port`. **Bind and callable entry are still unsupported** on the eager
  evaluator, so these fields cannot be executed faithfully without duplicating
  `lens_apply.rs`'s hand `FieldValue` interpreter — forbidden here.
- **Program-side Transform / Branch (E3/E4) vs fold composition:** landed E3/E4
  let the evaluator execute `Transform` and `Branch` in **target programs**, but
  they do **not** by themselves satisfy `Lens<C>.branch` / monoid composition:
  those are **calls into lens-declared function values**, still blocked on Bind.
- **Loop / `Lens<C>.iterate` (E5):** `iterate` composes along `LoopBound` and
  requires executed loop semantics. **`Behavior::Loop` is still fail-closed** —
  this is a **hard dependency on PR-E E5** before `Lens.iterate` can participate
  in a generic fold without a second interpreter.
- **Sequential Monoid projection:** `Lens<C>.sequential` is a `Monoid<C>`, not
  a direct Rust callback. E6 must be able to project the declared monoid
  identity and composition operation, then execute that operation through the
  same body-evaluator / callable-entry authority as `read`, `branch`,
  `iterate`, and `validate`. A local Rust "sequential combine" helper would
  duplicate the algebra carrier and silently bypass the lens's declared
  sequential semantics.
- **Program scope:** the current reflect/apply seam selects behaviors by
  `SourceSpan.file`. A generic `fold_lens<C>: Lens<C> -> Dag ->
  DimensionReport<C>` needs a structural program scope or whole-Dag traversal
  decision. E6 must not canonize file-path filtering as the full fold's
  participation authority.
- **Report construction:** the carriers already exist, but the implementation
  still needs typed lifting rules from `Witness<C>` and `OptionalDiagnostic`
  into `DimensionOk` / `DimensionFail`. It must not fabricate a carrier when a
  witness violates or validation fails.

These blockers mean a small implementation in this slice would either duplicate
the old `lens_apply.rs` interpreter, bypass PB-Runtime body semantics, or return
something weaker than `DimensionReport<C>`. This receipt records the gap instead
of landing a misleading partial fold.

## Resume Gate

E6 implementation may resume when:

- **E3 and E4** have landed (done on `main` for program `Transform` / `Branch`);
  **E5** has landed for **loop iteration** through the shared evaluator boundary;
  **Bind + callable entry** has landed so lens function fields execute through
  `eval_node` / `eval_port` without local `FieldValue` frames;
- sequential composition consumes the declared `Lens.sequential` monoid witness:
  `identity` for empty/unit structure and `op` for Bind sequencing, with no
  host-fabricated unit or per-lens Rust sequencing;
- the implementation names its structural program-scope authority, or explicitly
  scopes the first fold to whole-Dag traversal;
- report construction uses the existing `Witness<C>`, `OptionalDiagnostic`, and
  `DimensionReport<C>` carriers verbatim.

Required implementation tests:

- a minimal complexity-style lens produces `DimensionOk` through the generic
  fold, not through `apply_lens_declaration` reflection plumbing;
- a validation lens produces `DimensionFail` with typed diagnostics and no
  fabricated `composed` value;
- the fold consumes Behavior tree structure through evaluator semantics rather
  than per-lens Rust dispatch;
- empty/unit sequential structure consumes `Lens.sequential.identity`;
- Bind sequencing composes through `Lens.sequential.op`;
- branch and loop composition use the declared `Lens<C>.branch` and
  `Lens<C>.iterate` fields;
- `fold_lens_over_reflected_program` remains either a compatibility seam or is
  narrowed with a named dissolution trigger once the generic fold path lands.

## Non-Goals

This readiness receipt does not authorize:

- **New** Transform, Branch, Loop, or Bind **implementation inside the lens fold
  driver** (or duplicate interpreters in `lens_apply.rs`) beyond what the PR-E
  body evaluator on `main` already exposes;
- a second `LensReport` / `DimensionReport` shape;
- new observable `Value` variants;
- strategy carrier widening, `NormalOrder`, `RightFirst`, or `EvalThunk`;
- substrate edits to `Lens<C>`, `Witness<C>`, `OptionalDiagnostic`, or
  `DimensionReport<C>`;
- runner, `test_runner.rs`, or `BinaryDimensionReportEquals` changes.
