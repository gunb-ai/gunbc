# v4 TypeScript RCA Manager Worksheets - 2026-06-03

Scope: **#4137 §11.8 ALPHA/PREVIEW lane** (MW-D3 TypeScript tranche). This is **not**
the Wave F F3 release-minimum floor (Rust/Python/Go carry that). Goal for the
current iteration: **L0 complete per #4117 pattern** — modeled facts, fixture
pairs, `.dag` claim wiring, temporary host bridge, and dissolve path to T-22
`run_target_verification`, without claiming L1 static structural, L2
cross-target parity, L3 fixed point, or L4 self-compile.

## Dispatch Rule

The TypeScript lane advances by rungs:

1. **L0 (this iteration):** leaf-model facts from `typescript.dag` are wired to
   happy plus falsification fixture pairs; boundary host exercises `tsc` /
   `node` while modeled receipts remain `pending_verification`.
2. **L1 (deferred):** `tsc --noEmit` static structural lane on shared
   `TargetStaticAnalysis*` carriers (distinct from compile/runtime verdicts).
3. **L2 (deferred):** cross-target behavioral parity (Python worksheet C class).

Temporary Rust boundary tests and absent shell runners are **receipts only**
until T-22 executes the `TestClaim` rows. Authority remains in `.dag`.

## Worksheet A — R2a Number Algebra Operations

Authority: `docs/planning/v4-ts-leaf-model-algebra-inhabitance-widening-worksheet-2026-06-03.md`
§4 row R2a.

| Artifact | Path |
|---|---|
| Fact | `ts_number_algebra_inhabitance_ts_facts_number` in `typescript.dag` |
| Fixtures | `typescript_r2a_*` in `leaf_model_verification.dag` (lens) |
| Claims | `typescript_r2a.dag` |
| Bridge | `v4_leaf_model_typescript_r2_r3_external_test.rs` (R2a tests) |
| Host script (debt) | `scripts/v4-leaf-model-typescript-r2a-verify.sh` — **not landed** |

L0 close: `claim_typescript_r2a_fixture_pair_wired` + boundary `tsc` happy
(TS0) / falsification (TS2339 on `log2_exact`).

## Worksheet B — R2b BigInt Beyond Safe Integer

Authority: widening worksheet §4 row R2b.

| Artifact | Path |
|---|---|
| Fact | `ts_bigint_algebra_inhabitance_ts_facts_bigint` |
| Fixtures | `typescript_r2b_runtime_*` (Node runtime, not `tsc`) |
| Claims | `typescript_r2b.dag` |
| Bridge | same boundary file (R2b tests) |
| Host script (debt) | `scripts/v4-leaf-model-typescript-r2b-verify.sh` — **not landed** |

L0 close: `claim_typescript_r2b_fixture_pair_wired` + Node happy bigint add +
falsification demonstrating IEEE754 `number` lane divergence from `bigint`.

## Worksheet C — R3-External Symbol Atom Realization

Authority: widening worksheet §4 row R3-external.

| Artifact | Path |
|---|---|
| Fact | `ts_atom_realization_symbol` / `ts_target_atom_realization_symbol` |
| Fixtures | `typescript_r3_external_*` |
| Claims | `typescript_r3_external.dag` |
| Bridge | same boundary file (R3 tests) |
| Host script (debt) | `scripts/v4-leaf-model-typescript-r3-external-verify.sh` — **not landed** |

L0 close: `claim_typescript_r3_external_fixture_pair_wired` + `tsc` happy
`Symbol("x")` / falsification `new Symbol` (TS7009 or constructable rejection).

## Worksheet D — TargetAtomRealization Catalog (SG-1 alpha)

Authority:
`docs/planning/v4-ts-target-atom-realization-worksheet-2026-06-01.md`.

L0 close: Symbol/Bool/String rows + `ts_target_atom_realization_catalog` on
live TargetModel bundle (`target_model_edge_atom_realizations`); parse-surface
smoke in `v4_std_target_realization_dag_smoke_test.rs` (+0 SG-0). **No Char
row** — TS substitutes String for Rust Char.

Downstream dissolve: manual/generated harness executes catalog facts without
host Rust parse smoke.

## Worksheet E — TargetTypeExpressionProjection (SG-2 alpha)

Authority:
`docs/planning/v4-ts-target-type-expression-projection-worksheet-2026-06-03.md`.

L0 close: `ts_type_expression_projection()` row + bundle edge on
`ts_sg2_type_expr_target_model`; golden emitted nodes for Instantiation/Arrow/Disj
beyond L0 row-2 (Conj). Behavioral contracts in
`sg2_typescript_type_expression_projection.dag`; parse-surface widening in
`v4_std_target_realization_dag_smoke_test.rs` (+0 SG-0).

## Current Closeout State (manager read, 2026-06-03)

| Worksheet | Modeled `.dag` | Boundary / smoke | Host shell scripts | L0 manager sign-off |
|---|---|---|---|---|
| A R2a | yes | yes (9/9 tests pass) | missing | worksheets authored; scripts debt |
| B R2b | yes | yes | missing | same |
| C R3-external | yes | yes | missing | same |
| D TargetAtomRealization | yes | yes (+0 smoke) | n/a | worksheet authored |
| E TargetTypeExpression | yes | yes (+0 smoke + manual claims) | n/a | worksheet authored |

**Verification receipts** in `typescript_r2{a,b}.dag` /
`typescript_r3_external.dag` still default to `pending_verification` — expected
until T-22 or restored host scripts populate `LeafModelVerificationRunReceipt`.

## Worker Dispatch (next iteration)

1. Land the three `scripts/v4-leaf-model-typescript-r2*-verify.sh` runners
   (mirror Python #4117 test plan) without duplicating boundary assertions.
2. Optional: fold boundary tests into CI upsert roster when alpha lane graduates
   from SG-0 residual-only posture.
3. Do **not** expand into L1 `tsc --noEmit` static structural until operator
   promotes TS from alpha to release-minimum.

## Related Artifacts

- `docs/planning/v4-ts-leaf-model-algebra-inhabitance-widening-worksheet-2026-06-03.md`
- `docs/planning/v4-ts-target-atom-realization-worksheet-2026-06-01.md`
- `docs/planning/v4-ts-target-type-expression-projection-worksheet-2026-06-03.md`
- `src/v4/extdeps/languages/typescript.dag`
- `src/v4/lens/leaf_model_verification.dag`
- `src/v3/compiler/tests/boundary/v4_leaf_model_typescript_r2_r3_external_test.rs`
- `src/v3/compiler/tests/integration/v4_std_target_realization_dag_smoke_test.rs`
- `docs/planning/v4-python-rca-manager-worksheets-2026-06-01.md` (#4117 pattern reference)
- `docs/planning/v4-go-rca-manager-worksheets-2026-06-03.md` (release-minimum contrast)
