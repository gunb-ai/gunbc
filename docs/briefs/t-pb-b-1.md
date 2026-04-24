# T-PB-B-1 — First landable batch (Testgen coordination)

**Parallel to:** T-PB-A / SG-0 non-test work. **Child lane:** T-PB-B.

## Landed in repo (this PR path)

- **`src/v3/compiler/tests/dag/*.dag`** — `TestClaim` / `TestSuite` as `data` declarations (`module v3.compiler.tests.t_pb_b_1.*`), schema from `src/v3/std/verification.dag`.
- **`t_pb_b_1_tests_dag_smoke_test`** — `compile_to_dag` smoke only (no predicate evaluation, no `pb_*`).

**Compile-smoke caveat (until runner):** The smoke helpers accept both `Ok(dag)` and `Err(CompileError::Semantic(dag))`, then assert empty **module** diagnostics. That proves the declaring `.dag` / `.v3` file lowers as a harness artifact — it does **not** evaluate `TestPredicate` (`Compiles`, `FailsWithDiagnostic`, …). When Testgen runs claims for real, `FailsWithDiagnostic` must be proven by the runner on the embedded program, **not** by empty diagnostics on the outer file (which would invert the predicate). **`t_pb_b_1_contract_diagnostic_smoke.dag`** is the sharp example: it names `FailsWithDiagnostic` for embedded `source: "let x: Bool = 1"`, while smoke still requires the **wrapper** module to lower without diagnostics. Do not treat that as semantic proof of the negative, and do not promote diagnostic-negative fixtures to CI-backed assertions without the runner (see also Brief D duplicate-authority / dissolution).

**Compiler constraint (today):** M1(2.8) rejects standalone `data …: TestPredicate = PortHasState(…)` / `CostBounded(…)` bodies as opaque. Inline those predicates **inside** each `TestClaim` `data` record (see `t_pb_b_1_contract_port_cost.dag`) until class-5 data bodies lift the restriction — call this out when Testgen chooses how to factor shared predicates.

## Explicitly not landed (needs Testgen manager)

- Wiring these files to the **Testgen runner** and CI evaluation order.
- **Deleting** redundant Rust integration tests (deferred until runner + schema assumptions are approved — avoids large revert).
- **`pb_test_file_generated_from_dag`** / **`pb_rust_tests_outside_residual_zero`**.

## Receipts for pre–Rust-deletion checklist (PR #736)

- **(1) Runner accepts the landed layout** — `src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs` loads each landed `.dag` via `compile_to_dag` and evaluates its `TestSuite` through `TestRunner::run_suite`, asserting every claim reaches `ClaimResult::Pass`. Covers `Compiles`, `FailsWithDiagnostic{TypeMismatch}`, `PortHasState`, and `CostBounded`.
- **(2) `requires: []` vs `empty()`** — **not equivalent in `data` bodies today.** `data …: TestClaim = { … requires: empty() }` is rejected by M1(2.8) class-5 ("data `…` field `requires` must be a list literal matching the declared `List<_>` type"). Receipt: `test_runner_data_bodies_reject_requires_empty_call_today` in `tests/integration/test_runner_test.rs` pins the diagnostic. The runner path (`src/v3/compiler/tests/dag/*.dag`) must therefore stay on the `[]` literal. `empty()` remains valid in `let` bindings (Brief D `.v3` fixtures), but those bindings are not `Declaration`s and so are not currently `run_suite`-consumable. When M1(2.8) class-5 lifts, the pinned test flips to a positive equivalence proof.
- **(3) M1(2.8) inline `PortHasState` / `CostBounded`** — proven for the Day-1 predicate shapes on `t_pb_b_1_contract_port_cost.dag`, which inlines both variant literals on each `data …: TestClaim = { … }` record. `t_pb_b_1_pipeline_smoke.dag` / `t_pb_b_1_contract_diagnostic_smoke.dag` likewise inline `Compiles` / `FailsWithDiagnostic`. **Still NYI:** standalone `data _: TestPredicate = PortHasState(…)` / `CostBounded(…)` bodies — same class-5 restriction. Shared-predicate factoring (`let pred_*: TestPredicate = …` in `m1_5_verification_test` style) does not flow through the runner today for the same `Declaration`-vs-`Bind` reason as (2).
- **(4) First Rust deletion batch** — **still open.** Not addressed in PR #736; Testgen manager's call once (1)–(3) receipts are reviewed.

## Coordination checklist (Testgen manager)

**Single source:** The operational pre–Rust-deletion checklist (runner
entrypoint for `src/v3/compiler/tests/dag/`, `requires: []` vs
`empty()`, M1(2.8) inlining, first Rust deletion batch) lives only in
**`docs/briefs/r1-testgen-manager.md`** → *Hand-off points* → **Sideways
to Self-hosting Manager**. Update that paragraph first; this brief
describes what is already landed in-repo above.

## Inventory

See **`docs/briefs/t-pb-b-brief-d.md`** (D / G / A / B matrix).
