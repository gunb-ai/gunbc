# v4 TypeScript TargetTypeExpressionProjection Worksheet — SG-2 Alpha Lane

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — TypeScript RCA
> Manager ALPHA/PREVIEW (#4137 §11.8). **Scope:** L0 `TargetTypeExpressionProjection`
> row for TypeScript, including connective coverage beyond L0 row-2 (Conj):
> Instantiation, Arrow (positional + labeled wire), Disj.
> **Date:** 2026-06-03
> **Manager sequencing:**
> `docs/planning/v4-ts-rca-manager-worksheets-2026-06-03.md` Worksheet E.

---

## Mechanical dispatch rule

SG-2 rows are **per-language** `TargetTypeExpressionProjection` bundles wired as
`target_model_edge_type_expression_projection` on a dedicated TargetModel node
(`ts_sg2_type_expr_target_model`). MVP-1 bundle stays on ProjectionAbsent shim
until its own row lands — TS SG-2 uses a separate bundle node (no kind-match
fallback that silently re-emits projection-absent).

Acceptance:

1. `ts_type_expression_projection()` returns the full per-connective row.
2. Golden **emitted** type nodes exist for falsification probes (parse/serialize).
3. Behavioral contracts (arrow wire-shape, labeled serialize, mixed-wire rejection)
   live in `sg2_typescript_type_expression_projection.dag` manual claims.
4. Host Rust parse smoke (+0 SG-0) dissolves when manual-claim runner executes
   without `v4_std_target_realization_dag_smoke_test.rs` widening.

---

## Section 4 Connective Coverage

| Connective | Golden emitted fn | Manual claim focus |
|---|---|---|
| Conj (L0 row-2) | `ts_sg2_conj_xy_emitted` | baseline record shape |
| Instantiation | `ts_sg2_foobar_xy_emitted` | `FooBar<X, Y>` angle brackets |
| Arrow positional | `ts_sg2_arrow_xy_emitted` | `(X) => Y` fat arrow |
| Arrow labeled | `ts_sg2_arrow_labeled_xy_emitted` | `(X: X) => Y` |
| Arrow mixed wire | `ts_sg2_arrow_mixed_wire_emitted` | positional+labeled rejection |
| Disj | `ts_sg2_sum_xy_emitted` | `X \| Y` union pipe |

Lex extensions: `<`, `>`, ` | `, ` => ` snoc'd onto wave-1 lex for SG-2 bundle.

---

## Section 8 Modeling DFS checklist

- [x] `ts_type_expression_projection()` authored
- [x] `ts_type_expression_projection_bundle_node()` + `ts_sg2_type_expr_target_model()`
- [x] `target_model_edge_type_expression_projection` on bundle
- [x] Golden probes beyond Conj (Instantiation/Arrow/Disj)
- [x] Manual claims in `sg2_typescript_type_expression_projection.dag`
- [x] Parse-surface smoke `v4_typescript_language_model_declares_type_expression_projection_row`
- [ ] `06_translate.dag` consumer gate: delete ProjectionAbsent / MVP-1 shim when
      feature `sg-2-mvp1-projection-absent-shim` dissolves (cross-language, not TS-only)

---

## P5 / SG-0 receipt

`v4_std_target_realization_dag_smoke_test.rs` — +0 SG-0; same structural class as
Go/Rust SG-2 same-path expansions. Behavioral depth in manual claim module; PR
#4226 arrow labeled/mixed-wire contracts referenced in smoke comments.

Dissolve trigger: `run_test_claim` executes `sg2_typescript_type_expression_projection.dag`
without host Rust parse assertions.

---

## Falsification probes

- `ts_sg2_arrow_mixed_wire_emitted` — mixed positional+labeled params must not
  silently serialize as valid TS.
- `project_type_expression_node` + `target_serialize_source_from_model` manual
  claims pin wire decode for labeled arrows.
- Restoring `ProjectionAbsent` on `ts_sg2_type_expr_target_model` is forbidden
  (fail-closed bundle wiring).

---

## Related Artifacts

- `src/v4/extdeps/languages/typescript.dag` — SG-2 row §1386–1599
- `src/v4/compiler/06_translate.dag` — `sg-2-mvp1-projection-absent-shim` gate
- `src/v4/test/claim/manual/sg2_typescript_type_expression_projection.dag` — behavioral + serialize claims
- `src/v3/compiler/tests/integration/v4_std_target_realization_dag_smoke_test.rs`
- `docs/planning/v4-ts-rca-manager-worksheets-2026-06-03.md`
