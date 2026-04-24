# T-PB-B-1 — First landable batch (Testgen coordination)

**Parallel to:** T-PB-A / SG-0 non-test work. **Child lane:** T-PB-B.

## Landed in repo (this PR path)

- **`src/v3/compiler/tests/dag/*.dag`** — `TestClaim` / `TestSuite` as `data` declarations (`module v3.compiler.tests.t_pb_b_1.*`), schema from `src/v3/std/verification.dag`.
- **`t_pb_b_1_tests_dag_smoke_test`** — `compile_to_dag` smoke only (no predicate evaluation, no `pb_*`).

**Compiler constraint (today):** M1(2.8) rejects standalone `data …: TestPredicate = PortHasState(…)` / `CostBounded(…)` bodies as opaque. Inline those predicates **inside** each `TestClaim` `data` record (see `t_pb_b_1_contract_port_cost.dag`) until class-5 data bodies lift the restriction — call this out when Testgen chooses how to factor shared predicates.

## Explicitly not landed (needs Testgen manager)

- Wiring these files to the **Testgen runner** and CI evaluation order.
- **Deleting** redundant Rust integration tests (deferred until runner + schema assumptions are approved — avoids large revert).
- **`pb_test_file_generated_from_dag`** / **`pb_rust_tests_outside_residual_zero`**.

## Coordination checklist (Testgen manager)

1. Runner accepts **v3 `.dag` modules** under `src/v3/compiler/tests/dag/` with the same `compile_to_dag` entrypoint as today’s integration harness, or requires a different path/layout — align before moving claims off Rust.
2. **`requires: []`** vs `empty()` — current `.dag` files use literal `[]` for `List<ResourceReference>`; confirm the runner’s parser/lowering agrees.
3. **First Rust deletion batch** — pick one module (e.g. a thin `Compiles`-only duplicate of `thesis_validation_test` or `pipe_desugar`) only after (1)–(2) are green.

## Inventory

See **`docs/briefs/t-pb-b-brief-d.md`** (D / G / A / B matrix).
