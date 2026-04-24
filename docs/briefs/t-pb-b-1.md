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

**Single source:** The operational pre–Rust-deletion checklist (runner
entrypoint for `src/v3/compiler/tests/dag/`, `requires: []` vs
`empty()`, M1(2.8) inlining, first Rust deletion batch) lives only in
**`docs/briefs/r1-testgen-manager.md`** → *Hand-off points* → **Sideways
to Self-hosting Manager**. Update that paragraph first; this brief
describes what is already landed in-repo above.

## Inventory

See **`docs/briefs/t-pb-b-brief-d.md`** (D / G / A / B matrix).
