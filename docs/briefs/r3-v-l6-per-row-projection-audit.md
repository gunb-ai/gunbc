# R3 Verification - L6 Per-Row Projection Audit

**Status:** AUDIT RECEIPT - docs-only. This audits the Grounding-owned L6 coverage projection surface; it does not implement the conversion and does not reshape LanguageSpec carriers.

**Scope:** `src/v3/grounding_cross_target_meta/src/coverage.rs` and the current `MethodTemplateContract` row authorities in:

- `src/v3/std/rust_method_template_contracts.dag`
- `src/v3/std/python_method_template_contracts.dag`
- `src/v3/std/go_method_template_contracts.dag`

## Contract Restatement

L6 is a substrate-load-time cross-product completeness check, not an R3 corpus test and not a `Lens<C>` instance. The cross-product is:

```text
TypeConnective variant x Behavior variant x Shape A target
```

At current HEAD that is `6 x 5 x 3 = 90` cells, using the six `TypeConnective` discriminants, five L1 `Behavior` discriminants, and the Shape A targets Rust / Python / Go. `docs/r3-structure.md` and `docs/design-emission-model.md` place this in R2's T-Ground-CrossTarget-Meta lane; Verification is auditing the row projection shape only.

The load-bearing risk is already documented in `coverage.rs`:

```text
TODO: Land per-row projection before non-CardinalityxTransform rows enter
these lists ...
When rows span multiple L6 cells per target, replace list-non-empty checks with a
per-row projection that unions each row's target Cells instead of one bucket per list.
```

This audit decides whether that TODO can be satisfied by rewriting `coverage.rs` alone, or whether the LanguageSpec / row carriers need an owner-routed shape change first.

## Current Implementation Audit

`coverage.rs` declares the three target list names:

```rust
const TARGET_LISTS: &[(&str, ShapeATarget)] = &[
    ("rust_method_template_contracts", ShapeATarget::Rust),
    ("python_method_template_contracts", ShapeATarget::Python),
    ("go_method_template_contracts", ShapeATarget::Go),
];
```

`language_spec_emission_cells_covered` then performs a list-level projection:

```rust
if method_template_list_covers_bucket(dag, list_name) {
    covered.insert(Cell {
        connective: FormAxis::Cardinality,
        behavior: BehaviorAxis::Transform,
        target,
    });
}
```

`method_template_list_covers_bucket` only verifies that the named declaration exists, has a `ValueBody::List`, and that the list is non-empty. It does not inspect each `MethodTemplateContract` row.

This is honest for the current Phase 1 row set because the file header states every row maps to the same `Cardinality x Transform x target` bucket. It is not sufficient for future row-class diversity: a non-empty mixed list would still cover the old bucket, and a row belonging to a new cell would not be projected from its own structural facts.

## Carrier Shape Audit

### Current Row Shape

The current `MethodTemplateContract` carrier in `src/v3/std/emit_model.dag` is:

```dag
type MethodTemplateContract {
  dag_method: MethodRef
  runtime_template: String
  emit_template: MethodEmitTemplate
  wraps_result: Bool
  placeholder_convention: PlaceholderConvention
}
```

Target identity is intentionally carried by the containing list declaration (`rust_method_template_contracts`, `python_method_template_contracts`, `go_method_template_contracts`), not duplicated on each row.

`dag_method` points through `MethodRef { decl: DeclarationRef }` to `dsl/std/methods.dag`, but `MethodDeclaration` is identity-only:

```dag
type MethodDeclaration {
  name: String
}
```

The method registry header explicitly says no parameter lists, return types, size effects, profile fields, or render-template facts live there; those facts remain in other authorities.

### Sufficiency Verdict

**Carrier shape is not sufficient for future per-row projection.**

Current rows can be projected per row only by hardcoding out-of-band knowledge: "every row in these three lists is `Cardinality x Transform`." That is exactly the list-level assumption `coverage.rs` already uses. The row itself does not carry:

- `FormAxis` / `TypeConnective` cell identity,
- `BehaviorAxis` cell identity,
- a list of covered cells for rows that span multiple cells,
- or a typed route from `dag_method` to method signature / receiver form / behavior class that would derive the cell.

Therefore the per-row projection requirement is not expressible as a structural rewrite against the current row shape. It needs Grounding/Substrate routing before the code conversion can be made fail-closed for mixed row classes.

## Per-Row Projection Predicate Sketch

The projection should consume a structural tuple equivalent to:

```rust
struct EmissionPathProjection {
    row_identity: MethodRef,              // or another row key for non-method rows
    connective: FormAxis,                 // TypeConnective discriminant
    behavior: BehaviorAxis,               // L1 Behavior discriminant
    target: ShapeATarget,                 // carried by containing list or row
}
```

For rows that legitimately cover multiple cells, the row must expose `List<Cell>` / `List<EmissionPathProjection>` or be split into one row per cell. The walker then unions each row's cells:

```text
covered = union(row.projected_cells for each LanguageSpec emission-path row)
```

Required structural facts:

| Fact | Exists today? | Current authority |
|---|---:|---|
| Shape A target | Yes, list-level | `coverage.rs` target list mapping; per-target row-list declarations |
| Method row identity | Yes | `MethodTemplateContract.dag_method: MethodRef` |
| Method declaration identity | Yes, identity-only | `dsl/std/methods.dag::MethodDeclaration { name }` |
| `TypeConnective` cell | No row-level fact | Hardcoded prose in `coverage.rs` says Phase 1 rows are `Cardinality` |
| `Behavior` cell | No row-level fact | Hardcoded prose in `coverage.rs` says Phase 1 rows are `Transform` |
| Multi-cell row projection | No | TODO in `coverage.rs` names this future need |

## Conversion Cost Classification

| Classification | Verdict | Rationale |
|---|---|---|
| **(a) Verification-side / Rust rewrite-only** | **No** | Replacing `method_template_list_covers_bucket` with a per-row loop would still have no row-local cell facts to read. The conversion would either re-hardcode every current method name to `Cardinality x Transform` or infer from template strings / comments, both of which repeat the masking risk. |
| **(b) Requires Substrate carrier extension** | **Yes, if MethodTemplateContract remains the row authority for mixed L6 cells** | A typed cell projection must land as a row field, sibling row, or declared projection carrier that attaches L6 `Cell` facts to each emission-path row. This is a substrate fact because L6's axes are substrate discriminants and the completeness predicate consumes them structurally. |
| **(c) Requires Grounding-side row-class refactor** | **Likely yes, or at least a Grounding decision** | If Grounding does not want to extend `MethodTemplateContract` itself, it must split LanguageSpec row classes so each list is cell-homogeneous and expose that homogeneity structurally. The current single `*_method_template_contracts` list is safe only while every row stays in one bucket. |

## Verdict

**Routing needed before conversion.**

The audit is not rewrite-only. The current code is accurate for today's homogeneous Phase 1 rows, but per-row projection cannot be made structural without either:

1. a Substrate-owned carrier extension or sibling projection carrier that records `Cell` / `List<Cell>` per LanguageSpec emission-path row, or
2. a Grounding-owned row-class refactor that makes each row list structurally cell-homogeneous and discoverable by the coverage walker without prose assumptions.

Recommended routing:

- **Substrate / Grounding:** decide whether L6 cell projection attaches to `MethodTemplateContract`, to a sibling `EmissionPathProjection` row keyed by `MethodRef`, or to cell-specific LanguageSpec row lists.
- **Grounding CrossTarget-Meta:** after that carrier/refactor lands, convert `coverage.rs` from list-non-empty projection to per-row union.
- **Verification:** do not author the conversion PR until the projection facts are structural; otherwise the conversion would preserve the current hardcoded bucket in a more granular-looking form.

## Debt Receipt

This audit does not close a Debt-Paydown row directly.

Debt found + routed: `l6_method_template_per_row_projection` requires Substrate/Grounding routing for row-level L6 cell projection facts before `coverage.rs` can be converted safely. No new runtime or substrate code is authored in this PR.

## Test Plan

- `git diff --check`
