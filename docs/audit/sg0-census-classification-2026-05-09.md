# SG-0 Census Per-Entry Classification — 2026-05-09

**Authority**: PM dispatch at gunb-ai/gunbc#846 c#4413701937; Brian operator directive 2026-05-09 active posture; Debt-Paydown Mgr Phase 2 (3) standing authority.

**Methodology**: heuristic regex classification on preceding-comment-block + path; Mgr review converts heuristic → ratified classification + dissolution-trigger comment in `sg0_census_test.rs`.

**Heuristic rules** (in priority order): see `scripts/classify-sg0-census.py` `CLASS_RULES`.

---

## EXPECTED_HAND_AUTHORED_NON_TEST (53 entries)

| # | Path | Class | Heuristic basis | Comment present? |
|---|---|---|---|---|
| 1 | `src/v3/compiler/benches/tier3_mirror_perf.rs` | E | Tier3 mirror dissolution | ✓ |
| 2 | `src/v3/compiler/build.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 3 | `src/v3/compiler/src/bin/emit_method_template_projection.rs` | E | v2-retirement | ✓ |
| 4 | `src/v3/compiler/src/bin/r1c_e_emit_gates.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 5 | `src/v3/compiler/src/bin/regen_bootstrap.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 6 | `src/v3/compiler/src/bin/regen_lens.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 7 | `src/v3/compiler/src/bin/regen_parse.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 8 | `src/v3/compiler/src/bin/regen_parse_tables.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 9 | `src/v3/compiler/src/bin/regen_tokenize.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 10 | `src/v3/compiler/src/bin/regen_v3.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 11 | `src/v3/compiler/src/bin/self_host_fixed_point.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 12 | `src/v3/compiler/src/bootstrap.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 13 | `src/v3/compiler/src/bootstrap_regen_fresh.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 14 | `src/v3/compiler/src/complexity_lattice.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 15 | `src/v3/compiler/src/cost_basis_declaration.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 16 | `src/v3/compiler/src/dag.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 17 | `src/v3/compiler/src/dag/builder.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 18 | `src/v3/compiler/src/dag/cardinality_payload.rs` | G | Local/small bridge (default) | ✓ |
| 19 | `src/v3/compiler/src/dag/effects.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 20 | `src/v3/compiler/src/dag/ports.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 21 | `src/v3/compiler/src/diagnostics.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 22 | `src/v3/compiler/src/dimension.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 23 | `src/v3/compiler/src/emit.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 24 | `src/v3/compiler/src/emit/collection_ops_method_contract.rs` | G | Local/small bridge (default) | ✓ |
| 25 | `src/v3/compiler/src/emit/python_target.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 26 | `src/v3/compiler/src/emit/rust_target.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 27 | `src/v3/compiler/src/emit_rust.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 28 | `src/v3/compiler/src/emit_rust_bin_shim.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 29 | `src/v3/compiler/src/emit_rust_roundtrip_fixtures.rs` | G | Local/small bridge (default) | ✓ |
| 30 | `src/v3/compiler/src/enforced_lens_application.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 31 | `src/v3/compiler/src/infer.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 32 | `src/v3/compiler/src/int_literal_ranges.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 33 | `src/v3/compiler/src/lens_apply.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 34 | `src/v3/compiler/src/lens_t_las_carrier.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 35 | `src/v3/compiler/src/lens_testgen.rs` | E | T-LensProducer-Retirement | ✓ |
| 36 | `src/v3/compiler/src/lib.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 37 | `src/v3/compiler/src/lower.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 38 | `src/v3/compiler/src/memory_peak_cost.rs` | G | Local/small bridge (default) | ✓ |
| 39 | `src/v3/compiler/src/omni_shape_b_openapi.rs` | G | Local/small bridge (default) | ✓ |
| 40 | `src/v3/compiler/src/pb_method_template_projection.rs` | E | v2-retirement | ✓ |
| 41 | `…/v3/compiler/src/pb_method_template_projection_dag_emit.rs` | G | Local/small bridge (default) | ✓ |
| 42 | `src/v3/compiler/src/pipeline_authority.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 43 | `src/v3/compiler/src/post_emit_verifier.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 44 | `src/v3/compiler/src/process_exit.rs` | G | Local/small bridge (default) | ✓ |
| 45 | `src/v3/compiler/src/r1c_e_gates.rs` | G | Local/small bridge (default) | ✓ |
| 46 | `src/v3/compiler/src/regen_bootstrap_emit.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 47 | `src/v3/compiler/src/regen_parse_emit.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 48 | `src/v3/compiler/src/regen_parse_tables_emit.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 49 | `src/v3/compiler/src/regen_tokenize.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 50 | `src/v3/compiler/src/self_host_receipt_p0.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 51 | `src/v3/compiler/src/test_runner.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 52 | `src/v3/compiler/src/workflow_idempotency.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 53 | `src/v3/compiler/src/workflow_parallelism.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |

## EXPECTED_HAND_AUTHORED_TEST (105 entries)

| # | Path | Class | Heuristic basis | Comment present? |
|---|---|---|---|---|
| 1 | `src/v3/compiler/tests/boundary/m1_3_emit_go_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 2 | `src/v3/compiler/tests/boundary/m1_3_emit_rust_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 3 | `src/v3/compiler/tests/boundary/m1_4_emit_python_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 4 | `src/v3/compiler/tests/boundary/m1_5_emit_omni_demo_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 5 | `…/tests/boundary/m2_emit_multi_field_struct_variant_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 6 | `src/v3/compiler/tests/determinism_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 7 | `src/v3/compiler/tests/integration.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 8 | `…iler/tests/integration/anthropic_messages_callable_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 9 | `…v3/compiler/tests/integration/anthropic_operations_test.rs` | G | Local/small bridge (default) | ✓ |
| 10 | `…mpiler/tests/integration/anthropic_schema_lockstep_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 11 | `…3/compiler/tests/integration/bridge_ledger_carrier_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 12 | `…tegration/bridge_lower_helpers_patch_zero_residual_test.rs` | G | Local/small bridge (default) | ✓ |
| 13 | `…er/tests/integration/canonical_lens_bridge_ratchet_test.rs` | E | T-LensProducer-Retirement | ✓ |
| 14 | `…gration/cementing/cementing_lens_registry_dispatch_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 15 | `…gration/cementing/complexity_lens_behavioral_completion.rs` | G | Local/small bridge (default) | ✓ |
| 16 | `…tests/integration/cementing/memory_peak_cost_basis_demo.rs` | G | Local/small bridge (default) | ✓ |
| 17 | `src/v3/compiler/tests/integration/common/budgeted.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 18 | `src/v3/compiler/tests/integration/common/cached_compile.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 19 | `…/compiler/tests/integration/common/determinism_fixtures.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 20 | `…/v3/compiler/tests/integration/common/list_variant_tags.rs` | G | Local/small bridge (default) | ✓ |
| 21 | `src/v3/compiler/tests/integration/common/mod.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 22 | `src/v3/compiler/tests/integration/common/r1_gates_bridge.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 23 | `…v3/compiler/tests/integration/common/substrate_receipts.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 24 | `…iler/tests/integration/cost_lens_symbolic_consumer_test.rs` | G | Local/small bridge (default) | ✓ |
| 25 | `…er/tests/integration/cross_target_coverage_carrier_test.rs` | STRUCTURAL | Measurement infrastructure (§3.A grandfathered) | ✓ |
| 26 | `…piler/tests/integration/e6_g1a_option3_static_lens_test.rs` | G | Local/small bridge (default) | ✓ |
| 27 | `…ler/tests/integration/e_i_lane_induction_preflight_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 28 | `…ompiler/tests/integration/emission_provenance_lens_test.rs` | G | Local/small bridge (default) | ✓ |
| 29 | `…r/tests/integration/extdeps_rust_primitives_loader_test.rs` | G | Local/small bridge (default) | ✓ |
| 30 | `…compiler/tests/integration/four_fixture_regression_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 31 | `…ests/integration/idempotency_lens_instance_blocker_test.rs` | G | Local/small bridge (default) | ✓ |
| 32 | `…compiler/tests/integration/int_literal_cardinality_test.rs` | G | Local/small bridge (default) | ✓ |
| 33 | `src/v3/compiler/tests/integration/l1_5_fixed_point_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 34 | `…compiler/tests/integration/lane2_stage_2a_effects_smoke.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 35 | `…/v3/compiler/tests/integration/lane2_stage_2b_db18_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 36 | `…/v3/compiler/tests/integration/lane2_stage_2c_db15_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 37 | `…ler/tests/integration/lane2_stage_2d_symbolic_cost_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 38 | `…piler/tests/integration/lane2_stage_2e_parallelism_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 39 | `src/v3/compiler/tests/integration/lane3_stage_3b_db1_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 40 | `…ler/tests/integration/lens_cost_target_realization_test.rs` | D | Generated bridge with freshness gate | ✓ |
| 41 | `…ler/tests/integration/lens_register_correspondence_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 42 | `…/compiler/tests/integration/lens_substrate_carrier_test.rs` | D | Generated bridge with freshness gate | ✓ |
| 43 | `src/v3/compiler/tests/integration/m0_acceptance.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 44 | `src/v3/compiler/tests/integration/m1_3_lens_cost_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 45 | `…iler/tests/integration/m1_3_lens_unused_parameters_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 46 | `…mpiler/tests/integration/m1_5_omni_shape_b_openapi_test.rs` | G | Local/small bridge (default) | ✓ |
| 47 | `src/v3/compiler/tests/integration/m1_5_testgen_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 48 | `…ler/tests/integration/m1_5_user_authored_lens_gate_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 49 | `src/v3/compiler/tests/integration/m1_5_verification_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 50 | `…sts/integration/m1_fn_external_body_reconciliation_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 51 | `…er/tests/integration/m1_lens_structural_resolution_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 52 | `src/v3/compiler/tests/integration/m1_substrate_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 53 | `src/v3/compiler/tests/integration/m2_feature_parity_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 54 | `…compiler/tests/integration/m2_field_access_binding_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 55 | `…/compiler/tests/integration/m2_lens_cost_migration_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 56 | `…ompiler/tests/integration/m2_lens_idempotency_emit_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 57 | `…er/tests/integration/m2_lens_idempotency_migration_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 58 | `…ler/tests/integration/m2_lens_provenance_migration_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 59 | `…ntegration/m2_lens_structural_resolution_migration_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 60 | `…ts/integration/m2_lens_unused_parameters_migration_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 61 | `…ests/integration/m2_lens_variant_payload_migration_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 62 | `…ompiler/tests/integration/m2_substrate_inhabitance_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 63 | `src/v3/compiler/tests/integration/method_registry_test.rs` | D | Generated bridge with freshness gate | ✓ |
| 64 | `…ompiler/tests/integration/method_template_contract_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 65 | `…ion/method_template_projection_emit_shim_coherence_test.rs` | G | Local/small bridge (default) | ✓ |
| 66 | `…iler/tests/integration/p0_std_render_repeat_string_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 67 | `…iler/tests/integration/pb1_bootstrap_full_snapshot_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 68 | `…integration/pb_method_template_projection_dag_emit_test.rs` | G | Local/small bridge (default) | ✓ |
| 69 | `…er/tests/integration/pb_method_template_projection_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 70 | `src/v3/compiler/tests/integration/pipe_desugar.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 71 | `…/integration/prereq_x_call_on_field_access_ratchet_test.rs` | G | Local/small bridge (default) | ✓ |
| 72 | `…3/compiler/tests/integration/r1_release_acceptance_test.rs` | E | T-LensProducer-Retirement | ✓ |
| 73 | `…3/compiler/tests/integration/r1c_d_pb_census_gates_test.rs` | B | Pattern-A NYI | ✓ |
| 74 | `…v3/compiler/tests/integration/r1c_e_emit_gates_dag_test.rs` | G | Local/small bridge (default) | ✓ |
| 75 | `…mpiler/tests/integration/r1c_e_emit_gates_omni_dag_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 76 | `…/tests/integration/r2_b5_loop_construction_closure_test.rs` | G | Local/small bridge (default) | ✓ |
| 77 | `…tests/integration/r3_free_consequences_first_batch_test.rs` | G | Local/small bridge (default) | ✓ |
| 78 | `…ests/integration/r3_free_consequences_second_batch_test.rs` | G | Local/small bridge (default) | ✓ |
| 79 | `…ts/integration/r3_pb_runtime_evaluator_corpus_seed_test.rs` | E | T-LensProducer-Retirement | ✓ |
| 80 | `…ests/integration/r3_verification_l4_l7_l5_skeleton_test.rs` | B | Pattern-A NYI | ✓ |
| 81 | `…/compiler/tests/integration/services_carrier_shape_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 82 | `src/v3/compiler/tests/integration/sg0_census_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 83 | `…/compiler/tests/integration/sg1_tokenize_authority_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 84 | `…/v3/compiler/tests/integration/sg2_parse_authority_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 85 | `…ler/tests/integration/sg2c1_parse_tables_authority_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 86 | `…ompiler/tests/integration/sg2c5_soft_keyword_ident_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 87 | `…er/tests/integration/sg3_lower_parse_surface_stack_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 88 | `…/tests/integration/sg3_surface_reflection_consumer_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 89 | `…ompiler/tests/integration/sg6_hand_authored_census_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 90 | `…sts/integration/sg7_prep_variant_payload_freshness_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 91 | `…egration/shape_a_target_source_filtering_authority_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 92 | `…/integration/t_impossiblebugs_unenumerated_effects_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 93 | `…ntegration/t_las_complexity_contract_compile_error_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 94 | `…piler/tests/integration/t_las_crdt_cost_basis_demo_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 95 | `…/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 96 | `…ler/tests/integration/t_pb_b_brief_d_fixture_smoke_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 97 | `…ration/tc1_substrate_lens_eta_equivalence_deferred_test.rs` | B | Pattern-A NYI | ✓ |
| 98 | `…ion/tc1_substrate_lens_eta_equivalence_strict_fire_test.rs` | B | Pattern-A NYI | ✓ |
| 99 | `…ests/integration/tc3_strong_normalization_deferred_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 100 | `src/v3/compiler/tests/integration/test_runner_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 101 | `src/v3/compiler/tests/integration/thesis_parallelism_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 102 | `src/v3/compiler/tests/integration/thesis_validation_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 103 | `…er/tests/integration/timing_lens_substrate_carrier_test.rs` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |
| 104 | `…ntegration/value_body_substrate_mirror_isomorphism_test.rs` | C | typed-carrier+Rust-mirror | ✓ |
| 105 | `src/v3/compiler/tests/integration/workflow_root_port_test.rs` | G | Local/small bridge (default) | ✓ |

## EXPECTED_HAND_AUTHORED_FRAGMENTS (1 entries)

| # | Path | Class | Heuristic basis | Comment present? |
|---|---|---|---|---|
| 1 | `src/v3/compiler/parse_parser_body.txt` | G | (no comment — untagged; Mgr review needed) | — UNTAGGED |

---

## Summary

- **Class A**: 0 (0.0%)
- **Class B**: 4 (2.5%)
- **Class C**: 1 (0.6%)
- **Class D**: 3 (1.9%)
- **Class E**: 7 (4.4%)
- **Class F**: 0 (0.0%)
- **Class G**: 143 (89.9%)
- **Class STRUCTURAL**: 1 (0.6%)
- **Total entries**: 159
- **Untagged (no preceding comment)**: 115 (72.3%)

## Mgr next-action

1. Review heuristic classifications (especially Class G default + STRUCTURAL grandfathering).
2. For each Class A-F entry: confirm dissolution-trigger comment matches lane gate; surface drift to lane-Mgr if mismatch.
3. For each untagged entry: add explicit dissolution-trigger comment per option-(c) discipline OR escalate as "no clear path" to Director.
4. Classifications fold back into `docs/audit/r3-debt-sweep-2026-05-06.md` §1 Class A-G rows as per-entry sub-rows or row-aggregation.

