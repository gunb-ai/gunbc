# CI Ratchet Exemption Audit — 2026-04-24

Source: `scripts/slow-test-exemptions.txt` on `session/swift-deer-46`.

This audit is the first T-Receipts pass over the slow-test exemption list. It
does **not** claim fresh local timing for every exempt test; it classifies the
current list from the annotated paydown reasons already present in the
exemption file and adds the missing meta-ratchet so the list cannot grow
silently.

## Summary

- Active exemptions: **43** non-comment entries.
- Stale exemptions removed in this pass: **0**. No entry was removed without a
  fresh timing run proving it is under the 2s budget.
- New enforcement: `scripts/check-test-timeout.sh` now requires the active
  exemption count to equal **43**. Future paydown PRs must lower that floor in
  the same change that deletes exemptions.
- Dominant category: **paydown backlog**. Most entries are compile-heavy
  integration, snapshot, emitted-rustc, or bootstrap-parity tests with explicit
  paydown triggers.

## Categories

| Category | Count | Meaning |
|---|---:|---|
| Paydown backlog | 36 | Slow for known test-shape reasons; should shrink through shared setup, cache warming, harness decomposition, or lane-specific speedups. |
| Structural for now | 7 | Intentionally compile/emission-heavy determinism matrix coverage; keep until the referenced determinism lane supplies a cheaper equivalent. |
| Stale | 0 | Not proven stale in this pass. |
| Unknown | 0 | Every active entry has a reason / owner path in the exemption file. |

## Exemption Classification

| Exemption | Category | Paydown / owner path |
|---|---|---|
| `bootstrap::tests::malformed_pipeline_stage_attaches_diagnostic` | Paydown backlog | Bootstrap fixture caching / test-infra bootstrap paydown. |
| `db8_rust_emit_avoids_time_paths_and_float_hooks_on_program_matrix` | Structural for now | DB-8 determinism matrix; replace only when the determinism lane supplies cheaper equivalent coverage. |
| `emit_matrix_module_go_is_deterministic` | Structural for now | DB-8 module re-emit matrix. |
| `emit_matrix_module_python_is_deterministic` | Structural for now | DB-8 module re-emit matrix. |
| `emit_matrix_module_rust_is_deterministic` | Structural for now | DB-8 module re-emit matrix. |
| `emit_matrix_program_go_is_deterministic` | Structural for now | DB-8 program re-emit matrix. |
| `emit_matrix_program_python_is_deterministic` | Structural for now | DB-8 program re-emit matrix. |
| `emit_matrix_program_rust_is_deterministic` | Structural for now | DB-8 program re-emit matrix. |
| `four_fixture_disk_sources_emit_deterministically` | Paydown backlog | Four-fixture disk corpus / shared compile setup. |
| `four_fixture_regression_test::four_fixture_suite_shares_one_reachability_shape` | Paydown backlog | Shared `compile_to_dag` suite warming / fixture pressure-corpus paydown. |
| `common::cached_compile::tests::cached_compile_any_returns_clean_dag_when_key_warmed_by_strict` | Paydown backlog | TESTING.md "Mocks over compile" paydown; real compile work remains. |
| `lane2_stage_2d_symbolic_cost_test::branch_reports_constant_when_both_arms_constant` | Paydown backlog | Shared bootstrap / shared cache key for symbolic-cost tests. |
| `lane2_stage_2d_symbolic_cost_test::cost_dag_compiles_cleanly` | Paydown backlog | Lane 2 Stage 2d authority compile smoke paydown. |
| `lane2_stage_2d_symbolic_cost_test::cost_generated_module_matches_checked_in_snapshot` | Paydown backlog | Snapshot compare / rustfmt harness paydown; inner wall-clock remains budgeted. |
| `m0_acceptance::compile_boundary_is_fail_closed` | Paydown backlog | TESTING.md test-shape / shared setup paydown. |
| `m0_acceptance::post_sweep_port_state_matches_diagnostic_table` | Paydown backlog | M0 acceptance compile-boundary shared setup. |
| `m1_substrate_test::referenced_port_walk_real_helper_stack_compiles` | Paydown backlog | Parse/lower cutover or helper-stack fixture restructuring. |
| `m1_3_emit_rust_test::rustc_roundtrip_emitted_module_compares_reflected_port_ids_in_list_contains` | Paydown backlog | Emitter/test-harness optimization. |
| `m1_3_emit_rust_test::rustc_roundtrip_emitted_module_invokes_reflected_dag_function` | Paydown backlog | Emitter/test-harness optimization. |
| `m1_3_emit_rust_test::rustc_roundtrip_emitted_module_matches_reflected_behavior_payloads` | Paydown backlog | Emitter/test-harness optimization. |
| `m1_3_emit_rust_test::rustc_roundtrip_emitted_module_returns_reflected_source_span_list` | Paydown backlog | Emitter/test-harness optimization. |
| `m1_3_emit_rust_test::rustc_roundtrip_emitted_module_returns_user_record_list_from_reflected_binds` | Paydown backlog | Emitter/test-harness optimization. |
| `m1_3_emit_rust_test::rustc_roundtrip_generic_list_fold_prints_one` | Paydown backlog | Emitter/test-harness optimization. |
| `m2_lens_cost_migration_test::complexity_dag_compiles_cleanly` | Paydown backlog | Lane 2 lens migration harness paydown. |
| `m2_lens_cost_migration_test::complexity_dag_runs_end_to_end_via_rustc_harness` | Paydown backlog | Lens migration emitted-oracle harness paydown. |
| `m2_lens_cost_migration_test::complexity_generated_module_matches_checked_in_snapshot` | Paydown backlog | Lens snapshot recompilation / rustfmt-check paydown. |
| `m2_lens_idempotency_migration_test::idempotency_emitted_analyze_matches_oracle` | Paydown backlog | Lane 2 emitted-oracle harness paydown. |
| `m2_lens_provenance_migration_test::lens_provenance_dag_runs_end_to_end_via_rustc_harness` | Paydown backlog | Lens migration emitted-oracle harness paydown. |
| `m2_lens_unused_parameters_migration_test::unused_parameters_dag_runs_end_to_end_via_rustc_harness` | Paydown backlog | Lens migration emitted-oracle harness paydown. |
| `m2_lens_unused_parameters_migration_test::unused_parameters_dag_self_analysis_reports_zero_findings` | Paydown backlog | Lens self-analysis harness paydown. |
| `pb1_bootstrap_full_snapshot_test::generated_full_bootstrap_snapshot_matches_runtime_full_bootstrap` | Paydown backlog | Shared PB-1 compile warming. |
| `pb1_bootstrap_full_snapshot_test::generated_full_bootstrap_without_parse_surface_matches_runtime_variant` | Paydown backlog | Shared PB-1 compile warming. |
| `pb1_bootstrap_std_snapshot_test::generated_std_bootstrap_snapshot_matches_runtime_std_bootstrap` | Paydown backlog | Shared PB-1 compile warming. |
| `pb1_bootstrap_full_snapshot_test::full_bootstrap_extends_std_snapshot` | Paydown backlog | Shared PB-1 compile warming; optional streaming snapshot compare. |
| `bootstrap::tests::kernel_bool_path_a_attaches_diagnostic_when_boolean_algebra_unresolvable` | Paydown backlog | Bool / `BooleanAlgebra` bootstrap bridge dissolution and shared `Dag::new` warming. |
| `p0_std_render_repeat_string_test::std_render_repeat_string_and_indent_text_match_interpreter` | Paydown backlog | Shared v2-oracle bootstrap cache / TESTING.md shared setup. |
| `sg1_tokenize_authority_test::tokenize_generated_module_matches_checked_in_snapshot` | Paydown backlog | SG-1 tokenize authority snapshot speedup. |
| `sg2_parse_authority_test::parse_generated_module_matches_checked_in_snapshot` | Paydown backlog | SG-2 parser-staging snapshot paydown. |
| `sg2_parse_authority_test::parse_surface_dag_compiles_cleanly_for_regen_parse` | Paydown backlog | SG-2 parser-staging shared bootstrap paydown. |
| `sg2c1_parse_tables_authority_test::parse_tables_generated_module_matches_checked_in_snapshot` | Paydown backlog | SG-2c parse-tables authority compile / rustfmt paydown. |
| `sg3_lower_authority_test::lower_generated_module_matches_regen_snapshot` | Paydown backlog | SG-3 lowering regen snapshot / CLI cost paydown. |
| `sg6_hand_authored_census_test::sg6_regen_lens_cli_smoke_regenerates_named_entry_without_drift` | Paydown backlog | SG-6 regen CLI smoke cost paydown. |
| `thesis_validation_test::kf_1_structural_list_operation_ordering_holds` | Paydown backlog | TESTING.md thesis-validation test-shape decomposition. |

## Follow-Up

This pass intentionally lands the count meta-ratchet before the full timing
audit. The remaining CI-ratchet debt is:

1. Re-measure each exempt test on the CI-shaped command and remove entries that
   now run under 2s.
2. Add per-exempt duration budgets once fresh timings exist. Do not invent
   budgets from stale comments.
3. Lower the `TEST_TIMEOUT_MAX_EXEMPTIONS` default in
   `scripts/check-test-timeout.sh` whenever exemptions are removed.
