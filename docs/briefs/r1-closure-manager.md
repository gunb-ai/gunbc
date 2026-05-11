# R1 Closure Manager Brief

**Status:** COMPLETE / DISSOLVED, ~2026-04-29.

R1 closed under path (a): release acceptance. The closure surface is
[`r1_release_acceptance.dag`](../../src/v3/compiler/tests/fixtures/r1_release_acceptance.dag),
which carries one strict PB gate (`pb_self_compile_fixed_point`) and five
Director-approved `ReleaseDeferredClaim` rows. R1C-B first landed the #1127
interim `p0_repeat_string_v2_oracle_rust_bridge`; the R1C-B T-P0 train (#1164)
retires that bridge and closes `p0_repeat_string_correct`,
`p0_no_fabrication_sentinel`, and `p0_rest_ops_aligned`. #1128 landed the
release-acceptance fixture; #1129 records the adjacent design locks.

## Orient before reading

- **R1 acceptance authority:** [`ROADMAP.md §"Release R1 Program"`](../../ROADMAP.md)
  remains the source for R1 gate semantics and the post-close ledger.
- **Strict-reading source:** [`THESIS.md`](../../THESIS.md) structural
  test-surface claim and [`docs/design-test-infra.md`](../design-test-infra.md)
  DB-15 authority remain the reason release gates are `.dag` data rather than
  host-only ratchets.
- **Closure shape:** R1 is closed by release acceptance, not by pretending every
  strict census gate is green. Strict receipts that remain RED are represented
  as explicit release deferrals with named target lanes.
- **Manager lifecycle:** this manager exists only to close R1 and hand off the
  remaining ledger. It is dissolved; post-close ownership follows R2 transition
  mechanics.

## Owned deliverables

| Lane | Size | Scope | Status |
|---|---|---|---|
| **R1C-A** | M-L | T-TestGen schema extensions: list-bodied `data` lowering, PB census predicate shapes, and the `MockBackedInvariant` minimal-demo gate. | **Closed on main.** PB census predicate dispatch landed in #939; the mock-backed gate evaluates through the runner. |
| **R1C-B** | S | T-P0 fixtures: structural `p0_repeat_string_correct`, `p0_no_fabrication_sentinel`, `p0_rest_ops_aligned`, and retirement of the #1127 interim bridge. | **Closed via #1164.** `p0_repeat_string_correct` is live in `r1_gates` through lower-time fold + `OutputEquals`; `p0_no_fabrication_sentinel` and `p0_rest_ops_aligned` are Day-1 `ExecuteCommand` receipts backed by `scripts/`; `p0_repeat_string_v2_oracle_rust_bridge` is retired per ROADMAP. |
| **R1C-C** | XS | T-Sub fixture for `sub_type_alias_where_lowers`. | **Closed.** PR #879 added the DB-11 witness gate and runner coverage. |
| **R1C-D** | M-L | T-PB census-as-`.dag`: six PB census gates in `tests/dag/t_r1c_d_pb_census_gates.dag` (retired on-disk alias `tests/fixtures/r1_pb_census_gates.dag` through 2026-04), plus release-acceptance disposition. | **Closed via #1128 under Director disposition.** `t_r1c_d_pb_census_gates.dag` is the live census surface: each gate lowers and evaluates as structural Pass/Fail against current SG-0 census authority. `r1_release_acceptance.dag` is the release surface: one strict gate plus five `ReleaseDeferredClaim` rows approved by Director disposition. |
| **R1C-E** | S | T-Emit `.dag` wrappers around existing host harnesses. | **Closed on main.** PR #978 and #1051 landed the rust-fixture, generic-bounds, and omni-demo gate wrappers. |
| **R1C-F** | S | T-Demo user-authored-lens rejection fixture. | **Closed.** PR #880 landed the rejecting-program demo gate. |

## Working state

Snapshot date: **2026-04-29, origin/main after #1164**. The live authority for
PB census counts is `src/v3/compiler/tests/integration/sg0_census_test.rs`; this
brief no longer treats older landing SHA `5f405cc8e` as HEAD.

| Surface | Gate set | Current state |
|---|---|---|
| R1C-B T-P0 receipts | `p0_repeat_string_correct`, `p0_no_fabrication_sentinel`, `p0_rest_ops_aligned` | **Closed on main via #1164.** `p0_repeat_string_correct` uses `r1_gates` lower-time fold + `OutputEquals`; the two former `[ext]` rows are Day-1 `ExecuteCommand` script receipts. |
| R1C-D live census | `t_r1c_d_pb_census_gates.dag` / suite `r1_pb_census_gates_suite` (`pb_hand_rust_at_shim_floor`, `lens_producer_files_remaining`, `pb_self_compile_fixed_point`, `pb_compiler_std_ratchet_zero`, `pb_test_file_generated_from_dag`, `pb_rust_tests_outside_residual_zero`) | **Runner-wired.** The suite proves every gate evaluates to structural Pass/Fail with no `NotYetImplemented`; RED counts are live SG-0 census facts, not stale brief data. |
| R1 release acceptance | `r1_release_acceptance.dag` | **Passes at HEAD.** `pb_self_compile_fixed_point` remains strict; the other five PB census gates are `ReleaseDeferredClaim` rows naming downstream authority docs / target lanes. |

## Remaining Ledger

| Item | Disposition |
|---|---|
| R1C-B T-P0 gates | Closed via #1164; no R1 Closure Manager ledger debt remains for `p0_repeat_string_correct`, `p0_no_fabrication_sentinel`, or `p0_rest_ops_aligned`. |
| PB census RED rows | No longer R1 program blockers after #1128. Their live counts belong to SG-0 census authority and downstream R2/R3 ledger rows named by `ReleaseDeferredClaim`. |

## Dissolution / Reporting

R1 hands off to R2 per [`docs/r2-structure.md` §"Transition mechanics"](../r2-structure.md#transition-mechanics):
R1 gates green by release acceptance, residual ledger rows receive R1-or-R2/R3
assignments, R1 manager scopes dissolve, and R2 standing managers own the
remaining work. The R1 Closure Manager role ends here; future updates should
land against the successor manager brief or the ROADMAP ledger row that owns the
remaining work.

| Lane | Gates (strict `.dag` receipts) | Status |
| --- | --- | --- |
| R1C-C | `sub_type_alias_where_lowers` (`DeclarationHasRefinement("PositiveInt")` on the DB-11 witness; `sub_type_alias_where_lowers_gate` in `r1_gates.template.dag` + `test_runner_runs_sub_type_alias_where_lowers_gate`) | **Closed — PR #879 merged** (`adda0eac`; all CI green before squash merge) |

## Cross-refs

- Parent authority: [`ROADMAP.md §"Release R1 Program"`](../../ROADMAP.md).
- R1C-B T-P0 receipts: [`r1_gates.dag`](../../src/v3/compiler/tests/fixtures/r1_gates.dag), [`scripts/r1_p0_no_fabrication_sentinel.sh`](../../scripts/r1_p0_no_fabrication_sentinel.sh), [`scripts/r1_p0_rest_ops_aligned.py`](../../scripts/r1_p0_rest_ops_aligned.py).
- R1C-D live census fixture: [`t_r1c_d_pb_census_gates.dag`](../../src/v3/compiler/tests/dag/t_r1c_d_pb_census_gates.dag) (runner receipt in `t_pb_b_1_dag_runner_test.rs`; Cluster M #84 pilot).
- R1 release-acceptance fixture: [`r1_release_acceptance.dag`](../../src/v3/compiler/tests/fixtures/r1_release_acceptance.dag).
- Census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs`.
