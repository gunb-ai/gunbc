# T-PB-B-1 — First landable batch (Testgen coordination)

**Parallel to:** T-PB-A / SG-0 non-test work. **Child lane:** T-PB-B.

## Landed in repo (this PR path)

- **`src/v3/compiler/tests/dag/*.dag`** — `TestClaim` / `TestSuite` as `data` declarations (`module v3.compiler.tests.t_pb_b_1.*`), schema from `src/v3/std/verification.dag`.
- **`t_pb_b_1_tests_dag_smoke_test`** — `compile_to_dag` smoke only (no predicate evaluation, no `pb_*`).
- **`t_pb_b_1_dag_runner_test`** (PR #736) — runner evaluates each landed `TestSuite`'s claims via `TestRunner::run_suite`; covers `Compiles`, `FailsWithDiagnostic{TypeMismatch}`, `PortHasState`, `CostBounded`. See *Receipts for pre–Rust-deletion checklist* below for the per-item coverage.

**Compile-smoke caveat (smoke tests only):** `t_pb_b_1_tests_dag_smoke_test` does **not** call `TestRunner`; it only asserts the declaring module lowers with empty diagnostics. Its helpers accept both `Ok(dag)` and `Err(CompileError::Semantic(dag))` and then check empty **module** diagnostics — that proves the declaring `.dag` / `.v3` file lowers as a harness artifact, it does **not** evaluate `TestPredicate` (`Compiles`, `FailsWithDiagnostic`, …). Predicate evaluation for the `.dag` suites lives in `t_pb_b_1_dag_runner_test` instead: `FailsWithDiagnostic` is proven there by the runner on the embedded program, **not** by empty diagnostics on the outer file (which would invert the predicate). **`t_pb_b_1_contract_diagnostic_smoke.dag`** is the sharp example: it names `FailsWithDiagnostic` for embedded `source: "let x: Bool = 1"`, the smoke still requires the **wrapper** module to lower without diagnostics, and the runner test is what actually witnesses the embedded negative. Do not treat the smoke pass as semantic proof of the negative, and do not promote diagnostic-negative fixtures to CI-backed assertions from the smoke path alone (see also Brief D duplicate-authority / dissolution; the Brief D `.v3` `let`-binding path is still smoke-only).

**Compiler constraint (today):** M1(2.8) rejects standalone `data …: TestPredicate = PortHasState(…)` / `CostBounded(…)` bodies as opaque. Inline those predicates **inside** each `TestClaim` `data` record (see `t_pb_b_1_contract_port_cost.dag`) until class-5 data bodies lift the restriction — call this out when Testgen chooses how to factor shared predicates.

## Explicitly not landed (needs Testgen manager)

Runner wiring for `src/v3/compiler/tests/dag/*.dag` **is** landed (PR #736 — see the receipts section below for (1)/(2)/(3) coverage); CI evaluation order is the existing `cargo test -p v3-compiler` path. Still open here:

- **Deleting** redundant Rust integration tests (deferred until runner + schema assumptions are approved — avoids large revert).
- **`pb_test_file_generated_from_dag`** / **`pb_rust_tests_outside_residual_zero`**.

## Receipts for pre–Rust-deletion checklist (PR #736)

- **(1) Runner accepts the landed layout** — `src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs` loads each landed `.dag` via `compile_to_dag` and evaluates its `TestSuite` through `TestRunner::run_suite`, asserting every claim reaches `ClaimResult::Pass`. Covers `Compiles`, `FailsWithDiagnostic{TypeMismatch}`, `PortHasState`, and `CostBounded`.
- **(2) `requires: []` vs `empty()`** — **not equivalent in `data` bodies today.** `data …: TestClaim = { … requires: empty() }` is rejected by M1(2.8) class-5 ("data `…` field `requires` must be a list literal matching the declared `List<_>` type"). Receipt: `test_runner_data_bodies_reject_requires_empty_call_today` in `tests/integration/test_runner_test.rs` pins the diagnostic. The runner path (`src/v3/compiler/tests/dag/*.dag`) must therefore stay on the `[]` literal. `empty()` remains valid in `let` bindings (Brief D `.v3` fixtures), but those bindings are not `Declaration`s and so are not currently `run_suite`-consumable. When M1(2.8) class-5 lifts, the pinned test flips to a positive equivalence proof.
- **(3) M1(2.8) inline `PortHasState` / `CostBounded`** — proven for the Day-1 predicate shapes on `t_pb_b_1_contract_port_cost.dag`, which inlines both variant literals on each `data …: TestClaim = { … }` record. `t_pb_b_1_pipeline_smoke.dag` / `t_pb_b_1_contract_diagnostic_smoke.dag` likewise inline `Compiles` / `FailsWithDiagnostic`. **Still NYI:** standalone `data _: TestPredicate = PortHasState(…)` / `CostBounded(…)` bodies — same class-5 restriction. Shared-predicate factoring (`let pred_*: TestPredicate = …` in `m1_5_verification_test` style) does not flow through the runner today for the same `Declaration`-vs-`Bind` reason as (2).
- **(4) First Rust deletion batch** — **still open; prepped, not signed off** (2026-04-24). Candidate inventory (`thesis_validation_test.rs` `Compiles`/`PortHasState`/`CostBounded` subset; `pipe_desugar.rs` excluded — its asserts cover `Transform` target / arity / literal order / return shape, which the `.dag` pipeline smoke does not witness) and the four-point Testgen sign-off guard live in `docs/briefs/r1-testgen-manager.md` → *Hand-off points → Sideways to Self-hosting Manager* (single source). Deletion PRs must not open until a sign-off entry appears in that brief's *Decisions log* naming the specific `#[test]` → `.dag` `TestClaim` mapping.

**Cross-PR coordination — #735 (symbolic cost / `Lookup`).** The landed `CostBounded("answer", Eq, 3)` witness in `t_pb_b_1_contract_port_cost.dag` is the current `cost_of(&dag, &bind.value)` result for `let answer: Int = 1 + 1 + 1 + 1` (three `+` additions). If #735 changes cost-witness semantics (symbolic-cost `Lookup` reshape, operator costing, or the `CostLookup::Hit` path), the `Eq 3` bound must be re-derived against the post-#735 runner output. Same check applies to the Brief D `.v3` sibling (`tests/fixtures/t_pb_b_brief_d/contract_port_cost_smoke.v3`). The runner-backed `t_pb_b_1_dag_runner_test` will fail loudly at `cost N did not satisfy bound 3` if the witness drifts; treat that failure as the #735 re-derivation signal, not a bug in this test.

**Queue items parked pending runner seam / director signal.** Optional next-PR item (5) — running Brief D `.v3` fixtures through `TestRunner::run_suite` — needs either a new runner seam that consumes `let`-bound `TestSuite` / `TestClaim` values from `BindNode` ports (currently the runner only reads `Declaration.value_body` structural fields), or conversion of the `.v3` fixtures to `data`-body shape (which drops the `let`+`empty()` path the fixtures were written to exercise). Neither is a drop-in; flag to director before scoping.

## Coordination checklist (Testgen manager)

**Single source:** The operational pre–Rust-deletion checklist (runner
entrypoint for `src/v3/compiler/tests/dag/`, `requires: []` vs
`empty()`, M1(2.8) inlining, first Rust deletion batch) lives only in
**`docs/briefs/r1-testgen-manager.md`** → *Hand-off points* → **Sideways
to Self-hosting Manager**. Update that paragraph first; this brief
describes what is already landed in-repo above.

## Inventory

See **`docs/briefs/t-pb-b-brief-d.md`** (D / G / A / B matrix).
