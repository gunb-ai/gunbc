# R1 Closure Manager Brief

**Status:** COMPLETE / DISSOLVED, ~2026-04-29.

R1 closed under path (a): release acceptance. The closure surface is
[`r1_release_acceptance.dag`](../../src/v3/compiler/tests/fixtures/r1_release_acceptance.dag),
which carries one strict PB gate (`pb_self_compile_fixed_point`) and five
Director-approved `ReleaseDeferredClaim` rows. R1C-B also landed the interim
`p0_repeat_string_v2_oracle_rust_bridge` in #1127, bridging the P0 repeat-string
gate to the v2 oracle until the structural receipt replaces it. #1128 landed
the release-acceptance fixture; #1129 records the adjacent design locks.

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
| **R1C-B** | S | T-P0 fixtures: structural `p0_repeat_string_correct`; interim `p0_repeat_string_v2_oracle_rust_bridge`; `[ext]` gates `p0_no_fabrication_sentinel` and `p0_rest_ops_aligned`. | **Interim closure via #1127.** `p0_repeat_string_v2_oracle_rust_bridge` is live in `r1_gates` and runner-backed. Remaining debt per ROADMAP: structural `p0_repeat_string_correct`, scaffold / `[ext]` `p0_no_fabrication_sentinel`, and scaffold / `[ext]` `p0_rest_ops_aligned`. |
| **R1C-C** | XS | T-Sub fixture for `sub_type_alias_where_lowers`. | **Closed.** PR #879 added the DB-11 witness gate and runner coverage. |
| **R1C-D** | M-L | T-PB census-as-`.dag`: six PB census gates in `r1_pb_census_gates.dag`, plus release-acceptance disposition. | **Closed via #1128 under Director disposition.** `r1_pb_census_gates.dag` is the live census surface: each gate lowers and evaluates as structural Pass/Fail against current SG-0 census authority. `r1_release_acceptance.dag` is the release surface: one strict gate plus five `ReleaseDeferredClaim` rows approved by Director disposition. |
| **R1C-E** | S | T-Emit `.dag` wrappers around existing host harnesses. | **Closed on main.** PR #978 and #1051 landed the rust-fixture, generic-bounds, and omni-demo gate wrappers. |
| **R1C-F** | S | T-Demo user-authored-lens rejection fixture. | **Closed.** PR #880 landed the rejecting-program demo gate. |

## Working state

Snapshot date: **2026-04-29, origin/main `041ed6780`**. The live authority for
PB census counts is `src/v3/compiler/tests/integration/sg0_census_test.rs`; this
brief no longer treats older landing SHA `5f405cc8e` as HEAD.

<<<<<<< HEAD
| Surface | Gate set | Current state |
|---|---|---|
| R1C-B interim bridge | `p0_repeat_string_v2_oracle_rust_bridge` | **Passes through runner.** This is an interim bridge, not the structural `p0_repeat_string_correct` receipt. |
| R1C-D live census | `r1_pb_census_gates.dag` (`pb_hand_rust_at_shim_floor`, `lens_producer_files_remaining`, `pb_self_compile_fixed_point`, `pb_compiler_std_ratchet_zero`, `pb_test_file_generated_from_dag`, `pb_rust_tests_outside_residual_zero`) | **Runner-wired.** The suite proves every gate evaluates to structural Pass/Fail with no `NotYetImplemented`; RED counts are live SG-0 census facts, not stale brief data. |
| R1 release acceptance | `r1_release_acceptance.dag` | **Passes at HEAD.** `pb_self_compile_fixed_point` remains strict; the other five PB census gates are `ReleaseDeferredClaim` rows naming downstream authority docs / target lanes. |
=======
| Lane | Size | Scope | Depends on | Status |
|---|---|---|---|---|
| **R1C-A** | M-L | T-TestGen schema extensions (three coupled sub-deliverables per the worker brief): (A) M1(2.8) list-body lowering for `data` declarations (compiler work in `lower.rs`); (B) scope predicate shapes for the 6 PB-census gates (T-PB-A `pb_hand_rust_at_shim_floor`, `lens_producer_files_remaining`, `pb_self_compile_fixed_point`, `pb_compiler_std_ratchet_zero` + T-PB-B `pb_test_file_generated_from_dag`, `pb_rust_tests_outside_residual_zero`); (C) `MockBackedInvariant` minimal-demo fixture closing `testgen_mock_backed_integration_safe`. Sized M-L (was M before audit revealed Sub-deliverable A is actual compiler work). | none | **Closed — 3/3 on main.** **Sub-A:** list-bodied `data` lowering (`ValueBody::List`), substrate/list path (e.g. `5bf0ec8d0`, follow-on lowers). **Sub-B:** PB census `TestPredicate` variants + runner dispatch — PR **#939** (`CensusBoundCheck`, `CensusSubsetCount`, `FixedPointConverges`, `RatchetZero`, `GeneratedFromDag` in `verification.dag` / `test_runner.rs`). **Sub-C:** mock-backed gate — **`3a18fa80b`** + [`src/v3/compiler/tests/fixtures/r1_mock_backed_invariant_gate.dag`](../../src/v3/compiler/tests/fixtures/r1_mock_backed_invariant_gate.dag); `test_runner_test.rs` asserts Pass on claim `testgen_mock_backed_integration_safe`. |
| **R1C-B** | S | T-P0 fixture authoring: structural `p0_repeat_string_correct` (`r1_gates` + lower-time fold); `p0_no_fabrication_sentinel` + `p0_rest_ops_aligned` as Day-1 **`ExecuteCommand`** host receipts (`scripts/`). Interim `p0_repeat_string_v2_oracle_rust_bridge` retired per ROADMAP. | none for landed predicates (ExecuteCommand + DB-15) | WORKER BRIEF AUTHORED — [`r1c-b-t-p0-fixtures-worker.md`](r1c-b-t-p0-fixtures-worker.md); R1C-B T-P0 train **PR #1164** |
| **R1C-C** | XS | T-Sub fixture authoring: 1 TestClaim fixture for `sub_type_alias_where_lowers` (PR #703 already landed the feature). | none | **R1C-C: 1/1 gates green** — PR #879 / merge `adda0eac` (`DeclarationHasRefinement("PositiveInt")` strict `.dag` receipt; `sub_type_alias_where_lowers_gate` runner green). |
| **R1C-D** | M-L | T-PB census-as-`.dag` wiring: 6 PB census gates as `.dag` TestClaims. Merges what was previously T-PB-A and T-PB-B into one lane because both share the predicate-shape work R1C-A scopes; splitting would duplicate the schema-consumer pattern. Authoritative ratchets remain `EXPECTED_HAND_AUTHORED_NON_TEST` + `EXPECTED_HAND_AUTHORED_FRAGMENTS` (T-PB-A subset) and `EXPECTED_HAND_AUTHORED_TEST` (T-PB-B subset) in `src/v3/compiler/tests/integration/sg0_census_test.rs`; the `.dag` TestClaims read those census values via the predicate shape R1C-A scopes. | R1C-A (**✓**) | WORKER BRIEF AUTHORED — [`r1c-d-t-pb-census-as-dag-worker.md`](r1c-d-t-pb-census-as-dag-worker.md); **dispatchable — R1C-A Sub-deliverable B landed (#939); awaiting fresh worker spawn (operator)** |
| **R1C-E** | S | T-Emit `.dag` TestClaim wrappers: 3 ExecuteCommand-based wrappers around the existing host harness (`emit_rust_fixtures_rustc_green` `[ext: ExecuteCommand]`; `emit_generic_bounds_survive` `[ext]`; `emit_omni_demo_fixtures_green` `[ext: ForAllTargets + ExecuteCommand]`). PB-Runtime `ExecuteCommand` runner landed PR #792, so this lane is dispatchable Day-1. Existing host harness in `tests/boundary/m1_3_emit_rust_test.rs` and `tests/boundary/m1_5_emit_omni_demo_test.rs` becomes the input to the `.dag` wrapper, not the gate authority. | none | **Closed — 3/3 on main.** PR **#978** (`emit_generic_bounds_survive`); PR **#1051** (`r1c_e_emit_gates.template.dag` / `rust-fixtures` + `r1c_e_emit_gates_omni.template.dag` / bin `omni-demo`). Brief: [`r1c-e-t-emit-dag-wrappers-worker.md`](r1c-e-t-emit-dag-wrappers-worker.md). |
| **R1C-F** | S | T-Demo user-authored-lens fixture: 1 TestClaim fixture for `demo_user_authored_lens_rejects_violating_program` (consumes `user_authored_lens_compiles` from T-LensAPI which is GREEN). | none | **R1C-F: 1/1 gates green** — PR #880 (`demo_user_authored_lens_rejects_violating_program_suite` Passes via `LensOutputEquals(named_function_count, …)`; demo blurb at [`docs/demos/r1c-f-user-authored-lens-rejection.md`](../demos/r1c-f-user-authored-lens-rejection.md)). |
>>>>>>> origin/main

## Remaining Ledger

| Item | Disposition |
|---|---|
| `p0_repeat_string_correct` | Remains as structural T-P0 debt. The #1127 bridge dissolves when modeled v3 evaluation can witness `repeat_string` directly. |
| `p0_no_fabrication_sentinel` | Remains `[ext]` T-P0 debt in [`ROADMAP.md §"Lane acceptance — .dag gates"`](../../ROADMAP.md) and the R1C-B worker brief. Successor lane: R1C-B Worker B / closure follow-up. Dissolves when a `.dag` `TestClaim` is authored, runner-dispatched, and evaluates `Pass` for the no-fabrication sentinel behavior. |
| `p0_rest_ops_aligned` | Remains `[ext]` T-P0 debt in [`ROADMAP.md §"Lane acceptance — .dag gates"`](../../ROADMAP.md) and the R1C-B worker brief. Successor lane: R1C-B Worker B / closure follow-up. Dissolves when a `.dag` `TestClaim` is authored, runner-dispatched, and evaluates `Pass` for REST operation alignment. |
| PB census RED rows | No longer R1 program blockers after #1128. Their live counts belong to SG-0 census authority and downstream R2/R3 ledger rows named by `ReleaseDeferredClaim`. |

## Dissolution / Reporting

R1 hands off to R2 per [`docs/r2-structure.md` §"Transition mechanics"](../r2-structure.md#transition-mechanics):
R1 gates green by release acceptance, residual ledger rows receive R1-or-R2/R3
assignments, R1 manager scopes dissolve, and R2 standing managers own the
remaining work. The R1 Closure Manager role ends here; future updates should
land against the successor manager brief or the ROADMAP ledger row that owns the
remaining work.

## Cross-refs

- Parent authority: [`ROADMAP.md §"Release R1 Program"`](../../ROADMAP.md).
- R1C-B interim bridge: [`r1_gates.dag`](../../src/v3/compiler/tests/fixtures/r1_gates.dag).
- R1C-D live census fixture: [`r1_pb_census_gates.dag`](../../src/v3/compiler/tests/fixtures/r1_pb_census_gates.dag).
- R1 release-acceptance fixture: [`r1_release_acceptance.dag`](../../src/v3/compiler/tests/fixtures/r1_release_acceptance.dag).
- Census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs`.
