# v4 TypeScript TargetAtomRealization Worksheet — SG-1 Alpha Lane

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — TypeScript RCA
> Manager ALPHA/PREVIEW (#4137 §11.8). **Scope:** L0 `TargetAtomRealization` rows
> for Symbol, Bool, and String on the live TS TargetModel bundle.
> **Date:** 2026-06-01 (authority date cited by integration smoke)
> **Manager sequencing:**
> `docs/planning/v4-ts-rca-manager-worksheets-2026-06-03.md` Worksheet D.

---

## Mechanical dispatch rule

SG-1 atom realization rows connect kernel carriers to per-language surface
spellings and value templates. For TypeScript alpha lane:

- Rows are **authored in `typescript.dag`**, not in translate shims.
- Each row's `type_form` consumes `ts_type_expression_projection().atom_form`
  (SG-2 dependency — no parallel atom spelling table).
- **String substitutes for Rust Char** — do not add `ts_target_atom_realization_char`.

Acceptance is parse-surface presence of row data + catalog edge on TargetModel
until `.dag` TestClaims execute catalog facts without host Rust smoke.

---

## Section 4 Row Catalog

| Row | Data name | Source carrier | Surface spelling | Value template |
|---|---|---|---|---|
| Symbol | `ts_target_atom_realization_symbol` | `symbol_kernel_type_node()` | `ts_surface_spelling_symbol` | `ValueSymbolIdentityPassthrough` |
| Bool | `ts_target_atom_realization_bool` | `bool_node()` | `ts_surface_spelling_boolean` | `ValueBoolLiteral` |
| String | `ts_target_atom_realization_string` | `ts_std_text_string_type_node()` | `ts_surface_spelling_string` | `ValueStringLiteral` |

Catalog: `ts_target_atom_realization_catalog` lists all three. Live bundle must
expose `target_model_edge_atom_realizations` on the TS TargetModel node used by
translate consumers.

Claim subject fact (R3-external leaf-model): `ts_atom_realization_symbol` mirrors
`rust_atom_realization_symbol` pattern; realization row is
`ts_target_atom_realization_symbol`.

---

## Section 8 Modeling DFS checklist

- [x] `ts_target_atom_realization_{symbol,bool,string}` authored
- [x] `ts_target_atom_realization_catalog` lists rows
- [x] `target_model_edge_atom_realizations` on live bundle
- [x] `type_form` uses `ts_type_expression_projection().atom_form`
- [x] No Char row (TS L0 substitutes String)
- [ ] Int/Number primitive TargetAtomRealization rows (deferred — shared
      `TargetValueTemplateKind` integer literal arm, same class as Go Int row)

---

## P5 / SG-0 receipt

`v4_std_target_realization_dag_smoke_test.rs`:
`v4_typescript_language_model_declares_target_atom_realization_rows` — +0 SG-0
paths; dissolves when substrate `run_test_claim` covers catalog without host Rust.

ROADMAP: `ROADMAP.md` § **Public Operational Lanes** / T-PB-B
(`pb_rust_tests_outside_residual_zero`).

---

## Falsification probes

- Adding `ts_target_atom_realization_char` must fail smoke (Rust Char is not TS L0).
- Removing `target_model_edge_atom_realizations` must fail translate wiring smoke.
- Atom row with `type_form` not tied to SG-2 projection row must fail closed review.

---

## Related Artifacts

- `src/v4/extdeps/languages/typescript.dag` — rows §1534–1560, symbol §2234–2241
- `src/v4/std/target_model.dag` — `TargetAtomRealization` carrier
- `src/v3/compiler/tests/integration/v4_std_target_realization_dag_smoke_test.rs`
- `docs/planning/v4-ts-rca-manager-worksheets-2026-06-03.md`
