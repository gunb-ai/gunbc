# v4 TS TargetTypeExpressionProjection Worksheet

> **Status:** L0 complete on `main` (implementation PR #4176).  
> **Lane:** ALPHA/PREVIEW, not release-minimum.  
> **Shared carrier authority:** `src/v4/std/target_model.dag` owns `TargetTypeExpressionProjection`; `typescript.dag` owns only the TypeScript row.

## Receipt

```text
SG class:
  TS-TYPE-EXPRESSION-PROJECTION

Representative failure:
  Generic TypeScript type emission could fall back to name-keyed compiler
  branches for carriers such as Outcome<T> or Witness<C>.

Forbidden local patch:
  Add Outcome/Witness/FooBar special cases to `06_translate.dag` for TypeScript.

DFS path:
  std authority:
    src/v4/std/target_model.dag - TargetTypeExpressionProjection
    src/v4/std/node.dag - Node connective vocabulary
  extdeps authority:
    src/v4/extdeps/languages/typescript.dag - ts_type_expression_projection()
  compiler consumer:
    src/v4/compiler/06_translate.dag - projection readers and type-expression emit

Systemic fix:
  Author `ts_type_expression_projection()` and attach its bundle edge to the
  TypeScript TargetModel so type-expression emission derives syntax from row
  data instead of target-name branches.
```

## Connective Mapping

| Connective | L0 TypeScript surface | Notes |
| --- | --- | --- |
| Atom | identifier or keyword primitive | `number`, `string`, `boolean`, `bigint`, `symbol` |
| Conj | `{ ... }` generic record surface | field-label fidelity remains gated |
| Disj | `A | B` | TypeScript union surface |
| Arrow | `(T) => U` structural function type | parameter-name fidelity remains gated |
| Cardinality | `Head<args>` generic collection | uses angle-bracket generic apply |
| Instantiation | `Foo<Bar, Baz>` | no Rust `::<>` syntax |

## L0 Acceptance

- `ts_type_expression_projection()` defines per-connective forms in `typescript.dag`.
- `ts_type_expression_projection_bundle_node()` serializes the row as named edges for the TargetModel bundle.
- `ts_sg2_type_expr_target_model_node()` attaches `target_model_edge_type_expression_projection`.
- `v4_std_target_realization_dag_smoke_test.rs` covers the shared carrier, Rust precedent, and TypeScript same-path expansion.
- No name-keyed Outcome/Witness/Refined TypeScript branch is introduced.

## Known Gated Gaps

- `target-function-type-named-param-binding`: the shared function-type carrier lacks parameter-name slots, so L0 does not claim faithful `(a: T) => U`.
- `target-record-field-label-binding`: the shared record carrier and serializer do not yet emit named field labels, so L0 does not claim faithful `{ id: T }`.

These gaps are explicitly bounded in `src/v4/extdeps/languages/typescript.dag` and must dissolve through shared carrier changes, not TypeScript-only row redefinition.

