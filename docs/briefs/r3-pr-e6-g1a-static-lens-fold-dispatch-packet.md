# R3 PR-E E6-G1.a — Static top-level `Lens<C>` fold slice (dispatch packet)

**Status:** worker-ready design/dispatch receipt. Docs-only. Does **not**
implement a fold, parser/lowerer changes, runtime carriers,
`lens_apply.rs` / `lens_testgen.rs` edits, or generic
`fold_lens<C>(lens: Lens<C>, ...)`.

**Parent authorities:**
[`r3-pr-e6-lens-fold-readiness-audit.md`](./r3-pr-e6-lens-fold-readiness-audit.md)
§Conservative E6-G1 recommendation,
[`design-prereq-x-ho-field-call.md`](../design-prereq-x-ho-field-call.md)
§X1.a vs §X1.b,
[`x1b-evaluator-impact-audit.md`](./x1b-evaluator-impact-audit.md).

## Prerequisites (receipt or STOP)

This packet dispatches **G1.a** only while the earlier E6 gates below
still hold. Canonical narrative + landings:
[`r3-pr-e6-lens-fold-readiness-audit.md`](./r3-pr-e6-lens-fold-readiness-audit.md)
§Refresh and §What G0a/G0b/G0c have unblocked.

- **E6-G0c — body `FieldProject` + `Callable`:** Execution must remain
  in `src/v3/compiler/src/lib.rs` **`eval_transform_node`**
  (`Behavior::Transform`, `Value` carrier) at the cited line spans — not
  only under `lens_apply.rs`. If those match arms are removed, stubbed,
  or globally `UnsupportedTransformTarget`, **STOP** (G0c regression).
- **E6-G0b — Prereq-X1.a static field-call lowering:** Static lens
  function-field sites must still lower to `TransformTarget::Callable` per
  [`design-prereq-x-ho-field-call.md`](../design-prereq-x-ho-field-call.md)
  §L1.a. If static calls regress to a blocked HO form, **STOP** (G0b).

Implementers re-verify the `lib.rs` symbols at slice time; line numbers in
this file are a navigation snapshot only.

## Slice boundary (G1.a vs G1.b)

**In scope — G1.a:** A **single static top-level** substrate binding

```text
data <name>: Lens<C> = { ... }
```

whose **function-field** invocations (`lens.read`, `lens.sequential.op`,
`lens.branch`, `lens.iterate`, `lens.validate`, etc.) are lowered per
**Prereq-X1.a** to `TransformTarget::Callable(decl_id)` — static
field-call resolution, **no** runtime-port callee. Non-function fields
on the lens record (e.g. `name: String`) use runtime
`TransformTarget::FieldProject` on `RecordValue` carriers; that path is
orthogonal and already live.

**Explicitly out of scope — G1.b:** Generic

```text
fn fold_lens<C>(lens: Lens<C>, d: Dag) -> DimensionReport<C> = ...
```

where `lens` is a **parameter**. That requires **Prereq-X1.b**
(runtime-sourced callee dispatch / `Indirect` chain per
[`x1b-evaluator-impact-audit.md`](./x1b-evaluator-impact-audit.md)).
**Do not** implement or prototype
`fold_lens<C>` until X1.b lands; **do not** fake it with host-Rust
field dispatch.

**Authority:** Structural traversal and declared substrate types only.
`fold_lens_over_reflected_program` / `lens_apply.rs` remains a
**compatibility seam**, not the specification or implementation authority
for this slice (per readiness audit §Live Surfaces / §Non-Goals).

**Body evaluator vs lens seam:** The `FieldProject` / `Callable` receipts
below are **`eval_transform_node` on `Behavior::Transform` in
`src/v3/compiler/src/lib.rs`** (runtime `Value` carrier). That path is
where E6-G0c executes static field-call lowers. `lens_apply.rs` hosts a
separate `eval_transform` over `FieldValue` for the reflection
compatibility surface; it is **not** the authority cited in the table,
and must not be read as negating the `lib.rs` transform arms.

## Required source refs (verify before coding)

| Fact | Location |
|------|----------|
| `Lens<C>` six-field carrier | `src/v3/std/lens.dag` (`type Lens<C> { ... }`) |
| `Witness<C>`, `OptionalDiagnostic`, `DimensionReport<C>` | `src/v3/std/dimensions.dag` |
| Runtime `FieldProject` on records | `src/v3/compiler/src/lib.rs` `556-578` (`eval_transform_node`, `TransformTarget::FieldProject`) |
| Runtime `Callable` (Arrow `UserDefined`, pushed frame) | same file `579-613` (`eval_transform_node`, `TransformTarget::Callable`) |
| Cardinality loops live; `Descent` fail-closed | same file `356-386` (`eval_loop`; `LoopBound::Descent` at `367-373`) |
| Regression: descent bound fails closed | same file `1817-1868` (`eval_loop_descent_bound_fails_closed`) |
| X1.a lowering semantics (static reference → `Callable`) | `docs/design-prereq-x-ho-field-call.md` §L1.a |

`lib.rs` line numbers are a **snapshot** for navigation; they drift on
edits. Re-resolve symbols in-tree before implementing (same bar as
“verify before coding”).

## Acceptance gates (implementation slice)

1. **Static lens binding:** Fold logic consumes **one** top-level `data`
   `Lens<C>` instance declared in substrate `.dag`, not a parameter or
   Rust registry.
2. **Callable path:** Every lens **function** call site in the fold
   reaches the evaluator as `TransformTarget::Callable` resolved at
   lower time; no `Indirect` / X1.b shortcuts.
3. **Whole-program or explicitly scoped traversal:** Structural
   program-scope authority is named (e.g. whole-`Dag` walk). Do **not**
   inherit `source_file` filtering from `lens_apply.rs` as generic fold
   authority.
4. **Cardinality-first loops:** `lens.iterate` integration aligns with
   live `eval_loop` for `LoopBound::Cardinality`. Any fold that must
   execute `LoopBound::Descent` inherits the **E5 residual** — see
   STOP §Descent below (no stubbing descent as zero/one/unknown).
5. **Honest report construction:** `DimensionReport<C>`,
   `Witness<C>`, and `OptionalDiagnostic` values are produced **through
   declared substrate variant construction** in the executing program /
   evaluator-visible rules — **no** parallel host-Rust mirrors for
   reports, witnesses, dimensions, or lens carriers.

## STOP conditions (fail closed; do not bypass)

- **G0c / G0b regression:** Body `eval_transform_node` loses executable
  `FieldProject` / `Callable` for the shapes this slice needs, or static
  lens field-call lowering no longer reaches `Callable` — **STOP** (see
  Prerequisites).
- **Missing static lens value:** No live `data ... : Lens<C>` instance
  with the needed function bodies — stop and land authoring prerequisites
  (readiness audit still notes deferred instances in
  `src/v3/lenses/*.dag`).
- **Construction gap:** Cannot construct `Witness` / `DimensionReport` /
  `OptionalDiagnostic` honestly through substrate variants and existing
  `Value` mirror (`LiteralValue`, `RecordValue`, `VariantValue`,
  `NodeRef`, `CardinalityValue`) — **STOP** per readiness audit; do not
  add ad hoc `Value` variants or host structs.
- **X1.b creep:** Any requirement to call lens methods on a
  **non-static** carrier — STOP; that is G1.b blocked on X1.b.
- **Descent denial:** If the fold requires `LoopBound::Descent`
  execution and E5 descent evidence is absent — report dependency;
  cannot silently exclude or approximate.
- **Forbidden edits:** No parser/lowerer changes, no
  `lens_apply.rs` / `lens_testgen.rs` authority migration, no runtime
  carrier edits **for this dispatch packet** unless a separate
  authorized slice says otherwise.

## Non-goals (same PR / same worker turn)

- Generic `fold_lens<C>` / X1.b / `Indirect` evaluator.
- Using `lens_apply.rs` filtering or host helpers as the fold’s primary
  traversal or report authority.
- Runner / test harness churn beyond what a future implementation slice
  explicitly requires.

— packet for E6-G1.a static-lens fold; aligns merry-gull-128 dispatch
boundaries (2026-05-05).
