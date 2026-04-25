# T-PB-B-1 — `tests/dag/` convention (first landable batch)

**Owner:** T-PB-B (e.g. witty-fox-588). **Schema authority:** `src/v3/std/verification.dag` (`TestClaim`, `TestPredicate`, `requires: List<ResourceReference>`).

This directory holds **landed** v3 `.dag` modules that declare `TestClaim` / `TestSuite` values as `data` declarations (same shapes Rust tests build today, but as on-disk authority for the eventual Testgen runner).

## What is here vs what is not

- **Landed (runner path):** `t_pb_b_1_dag_runner_test` — `lower` keeps the empty-harness-diagnostics compile receipt for each on-disk `tests/dag/*.dag` file, then `TestRunner::run_suite` proves line-item (1) of the pre–Rust-deletion checklist in `docs/briefs/r1-testgen-manager.md`. The former standalone `t_pb_b_1_tests_dag_smoke_test` is **retired** (redundant; see Testgen *Decisions log*). Still not a `pb_*` gate by itself.
- **Not done here:** Further Rust test deletion or `pb_test_file_generated_from_dag` — follow-on work needs **Testgen** sign-off per the inventory in `docs/briefs/r1-testgen-manager.md`.
- **Compile-smoke vs `TestRunner` (Brief D vs landed):** **Brief D** `.v3` smoke (`t_pb_b_brief_d_fixture_smoke_test`) only checks that the declaring file lowers with empty **module** diagnostics — that is **not** by itself proof of `Compiles` / `FailsWithDiagnostic` on embedded `TestClaim.source` strings. **Landed** `tests/dag/*.dag` files use the same empty-harness check inside `t_pb_b_1_dag_runner_test::lower`, then `run_suite` evaluates the predicates on the embedded `source` — so `FailsWithDiagnostic` negatives, `PortHasState`, and `CostBounded` bounds are real witnesses, not compile-smoke proxies. See `docs/briefs/t-pb-b-1.md` (*Compile-smoke caveat*).

## Post-R2 residuals (unchanged)

Per `TESTING.md` §Post-R2 shape:

- **A:** `#[cfg(test)]` helpers under `src/v3/compiler/src/` stay Rust.
- **B:** Boundary tests (`tests/boundary/*`, `rustc` / `go` / `python`) stay Rust.

## Files

| File | Intent |
|------|--------|
| `t_pb_b_1_pipeline_smoke.dag` | `Compiles` claim (pipe unary) + `TestSuite` bundle. |
| `t_pb_b_1_contract_diagnostic_smoke.dag` | `FailsWithDiagnostic` (`TypeMismatch`). |
| `t_pb_b_1_contract_port_cost.dag` | `PortHasState` + `CostBounded` (`Eq 3`, the runner-evaluated witness for `1 + 1 + 1 + 1`; structure mirrors `m1_5_verification` smoke, which ships an unevaluated `Eq 8` placeholder). |

**M1(2.8) note:** standalone `data …: TestPredicate = …` bodies that use variant literals are still **opaque** in user `.dag` / test modules. Keep `TestPredicate` shapes **inlined** on each `TestClaim` `data` item (as in `t_pb_b_1_contract_port_cost.dag`) until class-5 data bodies unlock shared predicate values.

**Related:** `docs/briefs/t-pb-b-1.md`, `docs/briefs/t-pb-b-brief-d.md` (D/G/A/B matrix), Brief D `.v3` fixtures under `tests/fixtures/t_pb_b_brief_d/` (parallel draft track; this directory is the **landed** `.dag` home).

**Suite naming (`t-pb-b/…` vs `t-pb-b-1/…`):** Intentional. Brief D drafts use `TestSuite.name` values prefixed with `t-pb-b/` in the `.v3` fixtures; landed `.dag` modules use `t-pb-b-1/` so harness grep and humans can tell **draft fixture track** from **T-PB-B-1 on-disk authority** until the dissolution trigger in Brief D retires the `.v3` copies.
