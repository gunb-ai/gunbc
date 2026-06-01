# v4 TS TargetAtomRealization Worksheet

> **Status:** L0 complete on `main` (implementation PR #4189).  
> **Lane:** ALPHA/PREVIEW, not release-minimum.  
> **Shared carrier authority:** `src/v4/std/target_model.dag` owns `TargetAtomRealization`; `typescript.dag` owns only the TypeScript rows.

## Receipt

```text
SG class:
  TS-ATOM-REALIZATION

Representative failure:
  TypeScript type and value emission for kernel atoms could invent shapes
  independently when no per-target atom realization row existed.

Forbidden local patch:
  Add TypeScript string-keyed Symbol/Bool/String branches in translate or
  redefine the shared atom carrier in `typescript.dag`.

DFS path:
  std authority:
    src/v4/std/target_model.dag - TargetAtomRealization and catalog edge
  extdeps authority:
    src/v4/extdeps/languages/typescript.dag - TS rows and catalog
  consumer:
    src/v4/compiler/06_translate.dag - reads TargetModel realization data

Systemic fix:
  Add TypeScript TargetAtomRealization rows for:
    ts_target_atom_realization_symbol
    ts_target_atom_realization_bool
    ts_target_atom_realization_string
  and expose them through `ts_target_atom_realization_catalog`.
```

## PM Mapping

| Shared/Rust anchor | TypeScript realization | L0 disposition |
| --- | --- | --- |
| Symbol | ECMA `symbol`, produced by `Symbol(description?)`, not `new Symbol` | required |
| Bool | `boolean` | required |
| Char | no TS primitive Char; substitute structural String | required substitution |
| Int | split across `number` and `bigint` facts | deferred as atom row |

## L0 Acceptance

- `ts_target_atom_realization_symbol` uses `symbol_kernel_type_node()` and `ValueSymbolIdentityPassthrough`.
- `ts_target_atom_realization_bool` uses `bool_node()` and `ValueBoolLiteral`.
- `ts_target_atom_realization_string` uses the structural string carrier and `ValueStringLiteral`.
- Each row's `type_form` consumes `ts_type_expression_projection().atom_form`.
- The TypeScript catalog is exposed through the target model atom-realizations edge.
- The smoke test `v4_typescript_language_model_declares_target_atom_realization_rows` covers Symbol/Bool/String rows and rejects a copied Rust Char row.

## Dissolution Path

The host smoke remains temporary under T-PB-B / `pb_rust_tests_outside_residual_zero`. It dissolves when `.dag` TestClaim execution and generated target verification cover the same TypeScript TargetModel catalog facts.

