# v4 TS Algebra Inhabitance Widening Worksheet

> **Status:** L0 complete on `main` (implementation PR #4155).
> **Lane:** ALPHA/PREVIEW, not release-minimum.
> **Purpose:** provide stable TypeScript algebra fact IDs for leaf-model R2a/R2b claims, mirroring the Python #4117 pattern.

## Receipt

```text
Migration class:
  TS-ALGEBRA-INHABITANCE-WIDENING

Representative failure:
  TypeScript algebra inhabitance existed only as functions over `ts_facts_*`.
  Leaf-model claim rows could not name a stable subject without duplicating
  `AlgebraInhabitanceDecl` shapes.

Forbidden local patch:
  Add TypeScript-specific Subject coproduct arms or hand-copy algebra
  declarations into claim files.

DFS path:
  std authority:
    src/v4/std/model_core.dag - AlgebraInhabitanceDecl
  extdeps authority:
    src/v4/extdeps/languages/typescript.dag - exported fact rows
  claim consumers:
    src/v4/test/claim/language_model/typescript_r2a.dag
    src/v4/test/claim/language_model/typescript_r2b.dag

Systemic fix:
  Export top-level TypeScript fact rows from `typescript.dag`:
    ts_number_algebra_inhabitance_ts_facts_number
    ts_bigint_algebra_inhabitance_ts_facts_bigint
    ts_bool_algebra_inhabitance_ts_facts_boolean
    ts_string_algebra_inhabitance_ts_facts_string
```

## Claim Wiring

| Stable fact ID | Modeled algebra | L0 consumer |
| --- | --- | --- |
| `ts_number_algebra_inhabitance_ts_facts_number` | ECMA `number` / IEEE-754 approximate field | R2a |
| `ts_bigint_algebra_inhabitance_ts_facts_bigint` | ECMA `bigint` exact integer arithmetic | R2b |
| `ts_bool_algebra_inhabitance_ts_facts_boolean` | ECMA `boolean` Boolean algebra | follow-on claim surface |
| `ts_string_algebra_inhabitance_ts_facts_string` | UTF-16 string code-unit sequence | follow-on claim surface |

## L0 Acceptance

- `typescript_r2a.dag` references `ts_number_algebra_inhabitance_ts_facts_number`.
- `typescript_r2b.dag` references `ts_bigint_algebra_inhabitance_ts_facts_bigint`.
- No TypeScript-only Subject/Expectation coproduct was introduced.
- Downstream host receipts stay temporary under T-PB-B / `pb_rust_tests_outside_residual_zero`.
