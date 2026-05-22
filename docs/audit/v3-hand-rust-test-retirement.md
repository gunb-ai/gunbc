# v3 Hand-Rust Test Retirement — Classification

Audit of hand-Rust tests under `EXPECTED_HAND_AUTHORED_TEST` (`src/v3/compiler/tests/integration/sg0_census_test.rs:380`).
**Operator-direct (2026-05-22):** categorize by value; **aggressive deletion** over 1:1 TestClaim replacement. **Read-only** — no files deleted in this dispatch.

## Census reconciliation (HEAD 2026-05-22)

| Grain | Brief cited | Measured at HEAD |
|---|---|---|
| `EXPECTED_HAND_AUTHORED_TEST` path literals | 204 entries | **145** paths |
| `#[test]` functions in those paths | (implied) | **1240** tests |

The PM figure **204 does not match** the live census (145 path literals / **1240** `#[test]` functions on `origin/main` and this worktree). Likely explanations: (1) stale planning count from an older, longer `EXPECTED_HAND_AUTHORED_TEST` list; (2) conflation with **file-path** entries (~160–204) vs per-function tests; (3) intent to name a first **DELETE batch size** rather than the full ratchet. This audit classifies **every `#[test]`** in the const scope (1240 rows). Downstream retirement batches should use [`v3-hand-rust-test-retirement-inventory.jsonl`](v3-hand-rust-test-retirement-inventory.jsonl) unless the operator re-baselines to exactly 204 units.

**Operator DELETE-first wave (recommended):** **384** tests (31%) in the DELETE bucket — includes all pre-flagged v4 smoke/closeout (**78**), cementing Rust (**18**), and `m1_substrate_test.rs` (**115**) — without waiting for 1:1 TestClaim ports.

## Summary

- **DELETE:** 384 tests (31.0%)
- **REPLACE-VIA-TESTCLAIM:** 537 tests (43.3%)
- **KEEP-AS-RUST:** 319 tests (25.7%)
- **Total classified:** 1240

**Bias applied:** on-the-fence DELETE vs REPLACE → DELETE; on-the-fence vs KEEP → non-KEEP.

## T-19 coordination (quiet-moth-603 / smart-boar-330)

**Landed `TestgenConcept` arms** (`src/v4/TASKS.md` T-19): `TypeConstruction`, `AlgebraLaw`, `DiagnosticExhaustiveness`, `LensApplicability`, `BidirectionalRoundtrip`, `LanguageBehaviorEquivalence`.

**Already exercised without v3 integration Rust:** `scripts/check_t19_testgen_activation.py` on generated LBE corpus; manual anchors in `src/v4/test/claim/manual/`.

**REPLACE bucket T-19 mapping (aggregate):**
- `TypeConstruction`: 308 tests
- `LensApplicability`: 134 tests
- `LanguageBehaviorEquivalence`: 51 tests
- `DiagnosticExhaustiveness`: 20 tests
- `AlgebraLaw`: 9 tests
- `BidirectionalRoundtrip`: 8 tests
- `T-19-CATEGORY-MISSING: ModuleServiceParseSurface`: 7 tests

**Categories flagged MISSING** (need design before port, not an excuse to KEEP Rust):
- `WorkspaceManifestStructuralAudit` — v2_oracle G-1 manifest walks
- `RepoFileTreeNegativeBridgeAudit` — gate #62 filesystem include_str! audit
- `ModuleServiceParseSurface` — ctrl `module … service …` until parser models it
- `Gate73_ReportPredicateCarriers` — ComplexitySummary Band-C (blocks deleting `complexity_lens_behavioral_completion.rs` until .dag predicate lands OR Rust deleted after .dag green)

---

## Pre-flagged clusters — confirm / refute

### (a) `v2_oracle_*` family — **REFUTE naive DELETE; KEEP short-term**

PM assumed "v3 matches v2 output" oracle tests. **Actual:** `v2_oracle_no_remaining_test_consumers_test.rs` is **G-1 excision** — scans `src/**` Rust + workspace `Cargo.toml` deps for `v2-compiler` spellings (gate #41 / T-V2-Retirement). **Not** behavioral differential testing.

- **10 tests**, buckets: {'KEEP-AS-RUST': 10}
- **Disposition:** KEEP until `src/v2/` deletion closes gate #41; then **DELETE entire file** (or replace with manifest model in substrate — low ROI).

### (b) `v4_*_smoke_test.rs` family — **CONFIRM DELETE**

Eight smoke modules parse/tokenize v4 `.dag` from the v3 compiler test crate — inverted dependency while v4 self-host matures.

- **71 tests**, buckets: {'DELETE': 71}
- `v4_bin_main_dag_smoke_test.rs::v4_bin_main_dag_tokenizes_and_parses_with_trampoline_source_anchor` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_compile_public_terminal_smoke_test.rs::v4_compile_dag_tokenizes_and_parses` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_compile_public_terminal_smoke_test.rs::v4_compile_dag_module_path_is_compiler_compile` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_compile_public_terminal_smoke_test.rs::v4_compile_dag_declares_public_validate_then_compile` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_compile_public_terminal_smoke_test.rs::v4_compile_dag_declares_ratified_compile_core` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_compile_public_terminal_smoke_test.rs::v4_compile_dag_imports_target_carriers_not_emit_cycle` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_compile_public_terminal_smoke_test.rs::v4_compile_dag_does_not_import_specific_lens_modules` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_compile_public_terminal_smoke_test.rs::v4_emit_dag_does_not_import_compile_module` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_compile_public_terminal_smoke_test.rs::v4_target_carriers_dag_tokenizes_and_parses` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_compile_public_terminal_smoke_test.rs::v4_validate_then_compile_claim_tokenizes_and_parses` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_compile_public_terminal_smoke_test.rs::v4_validate_then_compile_claim_imports_public_terminal_helpers` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_find_witness_dag_tokenizes_and_parses` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_find_witness_dag_declares_find_witness_entrypoint` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_translate_dag_tokenizes_and_parses` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_translate_dag_module_path_is_compiler_translate` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_translate_dag_imports_coercion_fold_delegate` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_translate_dag_imports_fold_node_traversal` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_translate_dag_declares_coerce_grounded_node` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_translate_dag_declares_translate_node_and_translate` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_translate_dag_imports_find_witness_types_not_inline_fn` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_emit_dag_tokenizes_and_parses` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_emit_dag_module_path_is_compiler_emit` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_emit_dag_imports_translate_stage` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_emit_dag_declares_emit_entrypoint` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_emit_dag_does_not_import_find_witness` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_rust_language_model_declares_t11_translation_rules` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_java_language_model_declares_t11_translation_rules` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_mvp1_rust_add_claim_tokenizes_and_parses` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_compiler_emit_translate_smoke_test.rs::v4_mvp1_rust_add_claim_imports_translate_and_emit` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_file_system_dag_smoke_test.rs::v4_extdeps_file_system_dag_tokenizes_and_parses` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_file_system_dag_smoke_test.rs::v4_extdeps_file_system_dag_practice11_companion_has_no_node_import_or_binding` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_file_system_dag_smoke_test.rs::v4_extdeps_file_system_dag_file_path_is_posix_grounded_record` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_file_system_dag_smoke_test.rs::v4_extdeps_file_system_dag_wave2_c2_modeled_effects_and_receipt_shape` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_file_system_dag_smoke_test.rs::v4_extdeps_file_system_dag_legacy_consumer_exports_remain` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_compiles` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_use_memo_use_callback_dependencies_are_required_lists` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_effect_hooks_require_setup_ref` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_react_hook_site_roster_matches_pin` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_use_resource_is_react_use_call_site_not_hook_site` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_host_element_declares_key_and_ref` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_composite_element_declares_key_and_ref` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_fragment_arm_declares_key` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_element_children_are_list_create_element_child` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_react_element_partition_excludes_primitive_text` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_create_element_child_text_has_no_element_key` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_create_element_child_element_arm_wraps_react_element` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_extdeps_react_dag_smoke_test.rs::v4_extdeps_react_dag_context_binding_matches_create_context_surface` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_registry_dag_smoke_test.rs::v4_lens_registry_dag_tokenizes_and_parses` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_registry_dag_smoke_test.rs::v4_lens_registry_t23_closed_lens_ids_present` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_registry_dag_smoke_test.rs::v4_lens_registry_structural_resolution_bound_to_module` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_registry_dag_smoke_test.rs::v4_lens_registry_p9_owned_fn_surface_present` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_p9_registry_owner_claim_parses_and_checks_registry_exclusivity` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_wave0_modules_tokenize_and_parse` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_wave0_verification_manual_anchor_key_only` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_wave0_function_inventory_matches_wave0` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_wave0_nat_symbol_import_authority` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_wave0_outcome_return_surfaces` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_wave0_generator_t19_anchor_field_is_manual_key` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_wave0_concept_projection_matches_claim_t19_anchor` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_wave0_generator_carries_claim_classification_and_anchor` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_wave0_bootstrap_threads_claim_anchor_into_generator` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_nat_law_manual_claims_use_compiles_stub` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_nat_substrate_carries_algebra_law_subject_symbols` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_lens_testgen_dag_smoke_test.rs::v4_lens_testgen_testgen_carries_six_nat_algebra_law_scheduling_arms` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_std_model_core_dag_smoke_test.rs::v4_std_model_core_dag_tokenizes_and_parses` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_std_model_core_dag_smoke_test.rs::v4_std_model_core_declares_ratified_q1_carriers` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_std_model_core_dag_smoke_test.rs::v4_std_model_core_bundles_five_facets` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_std_model_core_dag_smoke_test.rs::v4_std_model_core_algebra_law_obligation_structural` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_std_model_core_dag_smoke_test.rs::v4_std_model_core_effect_partiality_ops_use_node_authority` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_std_model_core_dag_smoke_test.rs::v4_std_model_core_primitive_fact_bundle_uses_axis_keyed_spec_facts` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority
- `v4_std_model_core_dag_smoke_test.rs::v4_std_model_core_wave1_void_constructor_present` — Inverted-dependency: v3 tokenize/parse smoke of v4 .dag; v4 workflow/T-19 activation owns authority

**Exception:** none warrant KEEP; parse receipts belong in v4 CI + `check_t19_testgen_activation.py`.

### (c) `cementing/` (6 modules) — **CONFIRM DELETE** (Rust leg) with one carrier caveat

Band-C cementing Rust modules are **transitional** same-PR receipts for `regen.dag` COMPLETE rows; parallel `.dag` harnesses under `tests/dag/t_r3_gate_87_cementing_regen_*.dag` + `t_pb_b_1_dag_runner_test` are the migration target.

- **18 tests**, buckets: {'DELETE': 18}
- **`complexity_lens_behavioral_completion.rs`:** still cites `Gate73_ReportPredicateCarriers` for `ComplexitySummary` — **DELETE Rust** once `.dag` predicate exists OR after .dag harness passes (delete-bias: do not expand Rust).
- **Other five:** consumer/residual pins (`cost_lens_symbolic_consumer`, E-P descent, effect_enumeration, memory_peak, provenance origin) — **DELETE** when matching gate-87 `.dag` suite is green in CI.

### (d) `v4_test_bootstrap_infra_closeout_test.rs` — **CONFIRM DELETE**

- **7 tests**, buckets: {'DELETE': 7}
- Asserts parse + substring presence on v4 `testgen.dag`, `bootstrap.dag`, manual/generated claim files.
- **Structural reason:** duplicated by `scripts/check_t19_testgen_activation.py` (LBE + manifest) and v4-lane ownership; not v3 product behavior.
- **No remaining ratchet obligation** beyond v4 lane closeout gates already tracked in T-19/T-20/T-22 tasks.

---

## DELETE bucket (rollup by reason class)


### Imperative substrate walks (115 tests)

- `src/v3/compiler/tests/integration/m1_substrate_test.rs` (115 tests)

### Milestone / m0 / m2 parity obsolete (93 tests)

- `src/v3/compiler/tests/integration/four_fixture_regression_test.rs` (11 tests)
- `src/v3/compiler/tests/integration/m0_acceptance.rs` (12 tests)
- `src/v3/compiler/tests/integration/m2_feature_parity_test.rs` (70 tests)

### Host drivers for .dag TestClaims (21 tests)

- `src/v3/compiler/tests/integration/r3_free_consequences_first_batch_test.rs` (1 tests)
- `src/v3/compiler/tests/integration/r3_free_consequences_second_batch_test.rs` (4 tests)
- `src/v3/compiler/tests/integration/r3_lens_producer_retirement_executable_witness_test.rs` (1 tests)
- `src/v3/compiler/tests/integration/r3_pb_runtime_evaluator_corpus_seed_test.rs` (2 tests)
- `src/v3/compiler/tests/integration/r3_sg0_non_test_zero_test.rs` (1 tests)
- `src/v3/compiler/tests/integration/r3_substrate_gap_reflection_closure_test.rs` (1 tests)
- `src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs` (11 tests)

### v4 inverted-dependency smoke/closeout (78 tests)

- `src/v3/compiler/tests/integration/v4_bin_main_dag_smoke_test.rs` (1 tests)
- `src/v3/compiler/tests/integration/v4_compiler_compile_public_terminal_smoke_test.rs` (10 tests)
- `src/v3/compiler/tests/integration/v4_compiler_emit_translate_smoke_test.rs` (18 tests)
- `src/v3/compiler/tests/integration/v4_extdeps_file_system_dag_smoke_test.rs` (5 tests)
- `src/v3/compiler/tests/integration/v4_extdeps_react_dag_smoke_test.rs` (13 tests)
- `src/v3/compiler/tests/integration/v4_lens_registry_dag_smoke_test.rs` (4 tests)
- `src/v3/compiler/tests/integration/v4_lens_testgen_dag_smoke_test.rs` (13 tests)
- `src/v3/compiler/tests/integration/v4_std_model_core_dag_smoke_test.rs` (7 tests)
- `src/v3/compiler/tests/integration/v4_test_bootstrap_infra_closeout_test.rs` (7 tests)

### Cementing transitional Rust (18 tests)

- `src/v3/compiler/tests/integration/cementing/cementing_provenance_origin_integration_test.rs` (1 tests)
- `src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs` (2 tests)
- `src/v3/compiler/tests/integration/cementing/cost_lens_symbolic_consumer_test.rs` (3 tests)
- `src/v3/compiler/tests/integration/cementing/e_p_per_call_descent_lens_consumer_cementing.rs` (5 tests)
- `src/v3/compiler/tests/integration/cementing/effect_enumeration_lens_behavioral_completion.rs` (6 tests)
- `src/v3/compiler/tests/integration/cementing/memory_peak_cost_basis_demo.rs` (1 tests)

### Migration/freshness include_str! (23 tests)

- `src/v3/compiler/tests/integration/m2_lens_cost_migration_test.rs` (4 tests)
- `src/v3/compiler/tests/integration/m2_lens_idempotency_migration_test.rs` (2 tests)
- `src/v3/compiler/tests/integration/m2_lens_provenance_migration_test.rs` (5 tests)
- `src/v3/compiler/tests/integration/m2_lens_structural_resolution_migration_test.rs` (3 tests)
- `src/v3/compiler/tests/integration/m2_lens_unused_parameters_migration_test.rs` (6 tests)
- `src/v3/compiler/tests/integration/m2_lens_variant_payload_migration_test.rs` (3 tests)

### Bridge/meta/blocker ratchets (31 tests)

- `src/v3/compiler/tests/integration/bridge_lower_helpers_patch_zero_residual_test.rs` (1 tests)
- `src/v3/compiler/tests/integration/canonical_lens_bridge_ratchet_test.rs` (6 tests)
- `src/v3/compiler/tests/integration/idempotency_lens_instance_blocker_test.rs` (2 tests)
- `src/v3/compiler/tests/integration/lens_behavioral_parity_demonstration_test.rs` (12 tests)
- `src/v3/compiler/tests/integration/prereq_x_call_on_field_access_ratchet_test.rs` (7 tests)
- `src/v3/compiler/tests/integration/tc1_substrate_lens_eta_equivalence_deferred_test.rs` (2 tests)
- `src/v3/compiler/tests/integration/tc1_substrate_lens_eta_equivalence_strict_fire_test.rs` (1 tests)

---

## REPLACE-VIA-TESTCLAIM bucket

Port only where behavior is still load-bearing after DELETE waves. **Do not 1:1 port all 657** — operator intent is delete-first.

### Mapped to existing T-19 generators (top file rollups)

- `m2_substrate_inhabitance_test.rs` — 86 tests → **TypeConstruction**
- `int_literal_cardinality_test.rs` — 28 tests → **TypeConstruction**
- `thesis_validation_test.rs` — 24 tests → **TypeConstruction**
- `anthropic_schema_lockstep_test.rs` — 22 tests → **LanguageBehaviorEquivalence**
- `t_ci_workflow_as_data_demo_test.rs` — 22 tests → **TypeConstruction**
- `timing_lens_substrate_carrier_test.rs` — 22 tests → **LensApplicability**
- `sg2c1_parse_tables_authority_test.rs` — 19 tests → **TypeConstruction**
- `r3_gate_87_lens_cementing_regen_receipts_test.rs` — 17 tests → **LensApplicability**
- `m1_5_verification_test.rs` — 14 tests → **TypeConstruction**
- `lens_cost_target_realization_test.rs` — 13 tests → **LensApplicability**
- `lane2_stage_2b_db18_test.rs` — 12 tests → **TypeConstruction**
- `m1_5_omni_shape_b_openapi_test.rs` — 12 tests → **LanguageBehaviorEquivalence**
- `l1_5_fixed_point_test.rs` — 11 tests → **TypeConstruction**
- `bridge_ledger_carrier_test.rs` — 9 tests → **TypeConstruction**
- `lens_substrate_carrier_test.rs` — 9 tests → **LensApplicability**
- `thesis_parallelism_test.rs` — 9 tests → **LensApplicability**
- `cross_target_coverage_carrier_test.rs` — 8 tests → **BidirectionalRoundtrip**
- `extdeps_sql_transport_test.rs` — 8 tests → **TypeConstruction**
- `lane2_stage_2d_symbolic_cost_test.rs` — 8 tests → **AlgebraLaw**
- `lane2_stage_2e_parallelism_test.rs` — 8 tests → **LensApplicability**
- `lane3_stage_3b_db1_test.rs` — 8 tests → **TypeConstruction**
- `r3_gate_60_phase2_width_nat_parser_test.rs` — 7 tests → **TypeConstruction**
- `services_carrier_shape_test.rs` — 7 tests → **T-19-CATEGORY-MISSING: ModuleServiceParseSurface**
- `t_gate_106_show_correct_code_diagnostic_coverage_test.rs` — 7 tests → **DiagnosticExhaustiveness**
- `t_las_crdt_cost_basis_demo_test.rs` — 7 tests → **LensApplicability**

---

## KEEP-AS-RUST bucket

- `src/v3/compiler/tests/boundary/m1_3_emit_rust_test.rs` (65 tests) — Class-5 external-toolchain boundary; migrate via ExecuteCommand TestClaim per TESTING.md (not 1:1 port of every assertion)
- `src/v3/compiler/tests/integration/test_runner_test.rs` (45 tests) — Host TestRunner executes .dag TestClaims; shrinks as claims subsume runner-only checks
- `src/v3/compiler/tests/integration.rs` (33 tests) — Crate wiring / determinism infrastructure
- `src/v3/compiler/tests/boundary/m1_4_emit_python_test.rs` (26 tests) — Class-5 external-toolchain boundary; migrate via ExecuteCommand TestClaim per TESTING.md (not 1:1 port of every assertion)
- `src/v3/compiler/tests/boundary/m1_3_emit_go_test.rs` (20 tests) — Class-5 external-toolchain boundary; migrate via ExecuteCommand TestClaim per TESTING.md (not 1:1 port of every assertion)
- `src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs` (20 tests) — Host TestRunner executes .dag TestClaims; shrinks as claims subsume runner-only checks
- `src/v3/compiler/tests/integration/sg0_census_test.rs` (17 tests) — SG-0/SG-* census ratchet until EXPECTED_HAND_AUTHORED_TEST is empty (gate #84)
- `src/v3/compiler/tests/integration/common/wiring_scanner_test.rs` (11 tests) — Shared helpers — retire when last importing test file retires (not independent coverage)
- `src/v3/compiler/tests/integration/sg6_hand_authored_census_test.rs` (10 tests) — SG-0/SG-* census ratchet until EXPECTED_HAND_AUTHORED_TEST is empty (gate #84)
- `src/v3/compiler/tests/integration/v2_oracle_no_remaining_test_consumers_test.rs` (10 tests) — G-1 v2-consumer excision ratchet (gate #41); DELETE entire file when src/v2/ tree removed — not v3-vs-v2 output comparison
- `src/v3/compiler/tests/determinism_test.rs` (9 tests) — Crate wiring / determinism infrastructure
- `src/v3/compiler/tests/integration/pb1_bootstrap_full_snapshot_test.rs` (9 tests) — Bootstrap snapshot digest ratchet until PB-1 emits check from .dag
- `src/v3/compiler/tests/integration/sg1_tokenize_authority_test.rs` (8 tests) — SG-0/SG-* census ratchet until EXPECTED_HAND_AUTHORED_TEST is empty (gate #84)
- `src/v3/compiler/tests/integration/r3_gate_62_file_ingestion_negative_bridge_audit_test.rs` (5 tests) — Filesystem walk over dsl/**/*.dag — no substrate TestClaim walker (gate #62 supporting audit only)
- `src/v3/compiler/tests/integration/common/mod.rs` (4 tests) — Shared helpers — retire when last importing test file retires (not independent coverage)
- `src/v3/compiler/tests/integration/common/rust_comment_strip.rs` (4 tests) — Shared helpers — retire when last importing test file retires (not independent coverage)
- `src/v3/compiler/tests/integration/common/symbolic_cost_verification_fixture.rs` (4 tests) — Shared helpers — retire when last importing test file retires (not independent coverage)
- `src/v3/compiler/tests/boundary/l5_cross_target_consistency.rs` (3 tests) — Class-5 external-toolchain boundary; migrate via ExecuteCommand TestClaim per TESTING.md (not 1:1 port of every assertion)
- `src/v3/compiler/tests/boundary/m2_emit_multi_field_struct_variant_test.rs` (3 tests) — Class-5 external-toolchain boundary; migrate via ExecuteCommand TestClaim per TESTING.md (not 1:1 port of every assertion)
- `src/v3/compiler/tests/boundary/m1_5_emit_omni_demo_test.rs` (2 tests) — Class-5 external-toolchain boundary; migrate via ExecuteCommand TestClaim per TESTING.md (not 1:1 port of every assertion)
- `src/v3/compiler/tests/integration/common/cached_compile.rs` (2 tests) — Shared helpers — retire when last importing test file retires (not independent coverage)
- `src/v3/compiler/tests/integration/r3_v3_self_host_demonstration_dag_test.rs` (2 tests) — ExecuteCommand/CARGO_BIN_EXE splice for self_host_fixed_point (gate #71)
- `src/v3/compiler/tests/integration/sg2_parse_authority_test.rs` (2 tests) — SG-0/SG-* census ratchet until EXPECTED_HAND_AUTHORED_TEST is empty (gate #84)
- `src/v3/compiler/tests/integration/sg3_surface_reflection_consumer_test.rs` (2 tests) — SG-0/SG-* census ratchet until EXPECTED_HAND_AUTHORED_TEST is empty (gate #84)
- `src/v3/compiler/tests/integration/sg7_prep_variant_payload_freshness_test.rs` (2 tests) — SG-0/SG-* census ratchet until EXPECTED_HAND_AUTHORED_TEST is empty (gate #84)
- `src/v3/compiler/tests/integration/sg3_lower_parse_surface_stack_test.rs` (1 tests) — SG-0/SG-* census ratchet until EXPECTED_HAND_AUTHORED_TEST is empty (gate #84)

---

## Full per-test inventory

Machine-readable: [`v3-hand-rust-test-retirement-inventory.jsonl`](v3-hand-rust-test-retirement-inventory.jsonl) (1240 lines: `file`, `fn`, `bucket`, `reason`, `t19`).

### Per-file classification table (145 paths)

| Path | Tests | DELETE | REPLACE | KEEP |
|---|---:|---:|---:|---:|
| `src/v3/compiler/tests/boundary/l5_cross_target_consistency.rs` | 3 | 0 | 0 | 3 |
| `src/v3/compiler/tests/boundary/m1_3_emit_go_test.rs` | 20 | 0 | 0 | 20 |
| `src/v3/compiler/tests/boundary/m1_3_emit_rust_test.rs` | 65 | 0 | 0 | 65 |
| `src/v3/compiler/tests/boundary/m1_4_emit_python_test.rs` | 26 | 0 | 0 | 26 |
| `src/v3/compiler/tests/boundary/m1_5_emit_omni_demo_test.rs` | 2 | 0 | 0 | 2 |
| `src/v3/compiler/tests/boundary/m2_emit_multi_field_struct_variant_test.rs` | 3 | 0 | 0 | 3 |
| `src/v3/compiler/tests/determinism_test.rs` | 9 | 0 | 0 | 9 |
| `src/v3/compiler/tests/integration.rs` | 33 | 0 | 0 | 33 |
| `src/v3/compiler/tests/integration/anthropic_messages_callable_test.rs` | 6 | 0 | 6 | 0 |
| `src/v3/compiler/tests/integration/anthropic_messages_wire_demo_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/anthropic_operations_test.rs` | 4 | 0 | 4 | 0 |
| `src/v3/compiler/tests/integration/anthropic_schema_lockstep_test.rs` | 22 | 0 | 22 | 0 |
| `src/v3/compiler/tests/integration/anthropic_tool_result_wire_demo_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/bridge_ledger_carrier_test.rs` | 9 | 0 | 9 | 0 |
| `src/v3/compiler/tests/integration/bridge_lower_helpers_patch_zero_residual_test.rs` | 1 | 1 | 0 | 0 |
| `src/v3/compiler/tests/integration/canonical_lens_bridge_ratchet_test.rs` | 6 | 6 | 0 | 0 |
| `src/v3/compiler/tests/integration/cementing/cementing_provenance_origin_integration_test.rs` | 1 | 1 | 0 | 0 |
| `src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs` | 2 | 2 | 0 | 0 |
| `src/v3/compiler/tests/integration/cementing/cost_lens_symbolic_consumer_test.rs` | 3 | 3 | 0 | 0 |
| `src/v3/compiler/tests/integration/cementing/e_p_per_call_descent_lens_consumer_cementing.rs` | 5 | 5 | 0 | 0 |
| `src/v3/compiler/tests/integration/cementing/effect_enumeration_lens_behavioral_completion.rs` | 6 | 6 | 0 | 0 |
| `src/v3/compiler/tests/integration/cementing/memory_peak_cost_basis_demo.rs` | 1 | 1 | 0 | 0 |
| `src/v3/compiler/tests/integration/common/cached_compile.rs` | 2 | 0 | 0 | 2 |
| `src/v3/compiler/tests/integration/common/mod.rs` | 4 | 0 | 0 | 4 |
| `src/v3/compiler/tests/integration/common/rust_comment_strip.rs` | 4 | 0 | 0 | 4 |
| `src/v3/compiler/tests/integration/common/symbolic_cost_verification_fixture.rs` | 4 | 0 | 0 | 4 |
| `src/v3/compiler/tests/integration/common/wiring_scanner_test.rs` | 11 | 0 | 0 | 11 |
| `src/v3/compiler/tests/integration/coverage_defect_acceptance_dag_test.rs` | 1 | 0 | 1 | 0 |
| `src/v3/compiler/tests/integration/cross_target_coverage_carrier_test.rs` | 8 | 0 | 8 | 0 |
| `src/v3/compiler/tests/integration/ctrl_pr_digests_dag_smoke_test.rs` | 1 | 0 | 1 | 0 |
| `src/v3/compiler/tests/integration/dissolution_subsumption_carrier_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/e6_g1a_option3_static_lens_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/e_i_lane_induction_preflight_test.rs` | 1 | 1 | 0 | 0 |
| `src/v3/compiler/tests/integration/emission_provenance_lens_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/extdeps_rust_primitives_loader_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/extdeps_sql_transport_test.rs` | 8 | 0 | 8 | 0 |
| `src/v3/compiler/tests/integration/file_attachment_substrate_carrier_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/four_fixture_regression_test.rs` | 11 | 11 | 0 | 0 |
| `src/v3/compiler/tests/integration/idempotency_lens_instance_blocker_test.rs` | 2 | 2 | 0 | 0 |
| `src/v3/compiler/tests/integration/int_literal_cardinality_test.rs` | 28 | 0 | 28 | 0 |
| `src/v3/compiler/tests/integration/l1_5_fixed_point_test.rs` | 11 | 0 | 11 | 0 |
| `src/v3/compiler/tests/integration/lane2_stage_2a_effects_smoke.rs` | 4 | 0 | 4 | 0 |
| `src/v3/compiler/tests/integration/lane2_stage_2b_db18_test.rs` | 12 | 0 | 12 | 0 |
| `src/v3/compiler/tests/integration/lane2_stage_2c_db15_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/lane2_stage_2d_symbolic_cost_test.rs` | 8 | 0 | 8 | 0 |
| `src/v3/compiler/tests/integration/lane2_stage_2e_parallelism_test.rs` | 8 | 0 | 8 | 0 |
| `src/v3/compiler/tests/integration/lane3_stage_3b_db1_test.rs` | 8 | 0 | 8 | 0 |
| `src/v3/compiler/tests/integration/lens_application_substrate_carrier_test.rs` | 1 | 0 | 1 | 0 |
| `src/v3/compiler/tests/integration/lens_behavioral_parity_demonstration_test.rs` | 12 | 12 | 0 | 0 |
| `src/v3/compiler/tests/integration/lens_cost_target_realization_test.rs` | 13 | 0 | 13 | 0 |
| `src/v3/compiler/tests/integration/lens_register_correspondence_test.rs` | 5 | 0 | 5 | 0 |
| `src/v3/compiler/tests/integration/lens_substrate_carrier_test.rs` | 9 | 0 | 9 | 0 |
| `src/v3/compiler/tests/integration/m0_acceptance.rs` | 12 | 12 | 0 | 0 |
| `src/v3/compiler/tests/integration/m1_3_lens_cost_test.rs` | 4 | 0 | 4 | 0 |
| `src/v3/compiler/tests/integration/m1_3_lens_unused_parameters_test.rs` | 4 | 0 | 4 | 0 |
| `src/v3/compiler/tests/integration/m1_5_omni_shape_b_openapi_test.rs` | 12 | 0 | 12 | 0 |
| `src/v3/compiler/tests/integration/m1_5_testgen_test.rs` | 6 | 0 | 6 | 0 |
| `src/v3/compiler/tests/integration/m1_5_user_authored_lens_gate_test.rs` | 6 | 0 | 6 | 0 |
| `src/v3/compiler/tests/integration/m1_5_verification_test.rs` | 14 | 0 | 14 | 0 |
| `src/v3/compiler/tests/integration/m1_fn_external_body_reconciliation_test.rs` | 2 | 0 | 2 | 0 |
| `src/v3/compiler/tests/integration/m1_lens_structural_resolution_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/m1_substrate_test.rs` | 115 | 115 | 0 | 0 |
| `src/v3/compiler/tests/integration/m2_feature_parity_test.rs` | 70 | 70 | 0 | 0 |
| `src/v3/compiler/tests/integration/m2_field_access_binding_test.rs` | 2 | 0 | 2 | 0 |
| `src/v3/compiler/tests/integration/m2_lens_cost_migration_test.rs` | 4 | 4 | 0 | 0 |
| `src/v3/compiler/tests/integration/m2_lens_idempotency_emit_test.rs` | 2 | 0 | 2 | 0 |
| `src/v3/compiler/tests/integration/m2_lens_idempotency_migration_test.rs` | 2 | 2 | 0 | 0 |
| `src/v3/compiler/tests/integration/m2_lens_provenance_migration_test.rs` | 5 | 5 | 0 | 0 |
| `src/v3/compiler/tests/integration/m2_lens_structural_resolution_migration_test.rs` | 3 | 3 | 0 | 0 |
| `src/v3/compiler/tests/integration/m2_lens_unused_parameters_migration_test.rs` | 6 | 6 | 0 | 0 |
| `src/v3/compiler/tests/integration/m2_lens_variant_payload_migration_test.rs` | 3 | 3 | 0 | 0 |
| `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs` | 86 | 0 | 86 | 0 |
| `src/v3/compiler/tests/integration/method_registry_test.rs` | 4 | 0 | 4 | 0 |
| `src/v3/compiler/tests/integration/method_template_contract_test.rs` | 6 | 0 | 6 | 0 |
| `src/v3/compiler/tests/integration/method_template_projection_emit_shim_coherence_test.rs` | 1 | 0 | 1 | 0 |
| `src/v3/compiler/tests/integration/no_coercion_cost_dimension_ratchet_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/pb1_bootstrap_full_snapshot_test.rs` | 9 | 0 | 0 | 9 |
| `src/v3/compiler/tests/integration/pb_method_template_projection_test.rs` | 4 | 0 | 4 | 0 |
| `src/v3/compiler/tests/integration/pipe_desugar.rs` | 6 | 0 | 6 | 0 |
| `src/v3/compiler/tests/integration/prereq_x_call_on_field_access_ratchet_test.rs` | 7 | 7 | 0 | 0 |
| `src/v3/compiler/tests/integration/r1_release_acceptance_test.rs` | 4 | 4 | 0 | 0 |
| `src/v3/compiler/tests/integration/r2_b5_loop_construction_closure_test.rs` | 2 | 0 | 2 | 0 |
| `src/v3/compiler/tests/integration/r3_class_2_function_valued_data_test.rs` | 6 | 0 | 6 | 0 |
| `src/v3/compiler/tests/integration/r3_free_consequences_first_batch_test.rs` | 1 | 1 | 0 | 0 |
| `src/v3/compiler/tests/integration/r3_free_consequences_second_batch_test.rs` | 4 | 4 | 0 | 0 |
| `src/v3/compiler/tests/integration/r3_gate_60_phase2_width_nat_parser_test.rs` | 7 | 0 | 7 | 0 |
| `src/v3/compiler/tests/integration/r3_gate_62_file_ingestion_negative_bridge_audit_test.rs` | 5 | 0 | 0 | 5 |
| `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs` | 17 | 0 | 17 | 0 |
| `src/v3/compiler/tests/integration/r3_gate_90_lens_enforcement_carrier_landed_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/r3_lens_producer_retirement_executable_witness_test.rs` | 1 | 1 | 0 | 0 |
| `src/v3/compiler/tests/integration/r3_path_b_brief3_char_in_class_execution_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/r3_pb_runtime_evaluator_corpus_seed_test.rs` | 2 | 2 | 0 | 0 |
| `src/v3/compiler/tests/integration/r3_sg0_non_test_zero_test.rs` | 1 | 1 | 0 | 0 |
| `src/v3/compiler/tests/integration/r3_substrate_gap_reflection_closure_test.rs` | 1 | 1 | 0 | 0 |
| `src/v3/compiler/tests/integration/r3_v3_self_host_demonstration_dag_test.rs` | 2 | 0 | 0 | 2 |
| `src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs` | 11 | 11 | 0 | 0 |
| `src/v3/compiler/tests/integration/services_carrier_shape_test.rs` | 7 | 0 | 7 | 0 |
| `src/v3/compiler/tests/integration/sg0_census_test.rs` | 17 | 0 | 0 | 17 |
| `src/v3/compiler/tests/integration/sg1_tokenize_authority_test.rs` | 8 | 0 | 0 | 8 |
| `src/v3/compiler/tests/integration/sg2_parse_authority_test.rs` | 2 | 0 | 0 | 2 |
| `src/v3/compiler/tests/integration/sg2c1_parse_tables_authority_test.rs` | 19 | 0 | 19 | 0 |
| `src/v3/compiler/tests/integration/sg2c5_soft_keyword_ident_test.rs` | 1 | 0 | 1 | 0 |
| `src/v3/compiler/tests/integration/sg3_lower_parse_surface_stack_test.rs` | 1 | 0 | 0 | 1 |
| `src/v3/compiler/tests/integration/sg3_surface_reflection_consumer_test.rs` | 2 | 0 | 0 | 2 |
| `src/v3/compiler/tests/integration/sg6_hand_authored_census_test.rs` | 10 | 0 | 0 | 10 |
| `src/v3/compiler/tests/integration/sg7_prep_variant_payload_freshness_test.rs` | 2 | 0 | 0 | 2 |
| `src/v3/compiler/tests/integration/shape_a_target_source_filtering_authority_test.rs` | 2 | 0 | 2 | 0 |
| `src/v3/compiler/tests/integration/symbolic_cost_expr_equals_executable_ratchet_test.rs` | 1 | 0 | 1 | 0 |
| `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs` | 22 | 0 | 22 | 0 |
| `src/v3/compiler/tests/integration/t_gate_106_show_correct_code_diagnostic_coverage_test.rs` | 7 | 0 | 7 | 0 |
| `src/v3/compiler/tests/integration/t_gate_58_apply_lens_self_application_test.rs` | 2 | 0 | 2 | 0 |
| `src/v3/compiler/tests/integration/t_impossiblebugs_unenumerated_effects_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/t_las_complexity_contract_compile_error_test.rs` | 1 | 0 | 1 | 0 |
| `src/v3/compiler/tests/integration/t_las_crdt_cost_basis_demo_test.rs` | 7 | 0 | 7 | 0 |
| `src/v3/compiler/tests/integration/t_las_parallelism_iteration_gate95_demo_test.rs` | 2 | 0 | 2 | 0 |
| `src/v3/compiler/tests/integration/t_lens_application_carrier_test.rs` | 2 | 0 | 2 | 0 |
| `src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs` | 20 | 0 | 0 | 20 |
| `src/v3/compiler/tests/integration/tc1_substrate_lens_eta_equivalence_deferred_test.rs` | 2 | 2 | 0 | 0 |
| `src/v3/compiler/tests/integration/tc1_substrate_lens_eta_equivalence_strict_fire_test.rs` | 1 | 1 | 0 | 0 |
| `src/v3/compiler/tests/integration/tc2_church_rosser_strict_fire_test.rs` | 1 | 0 | 1 | 0 |
| `src/v3/compiler/tests/integration/tc3_strong_normalization_deferred_test.rs` | 1 | 0 | 1 | 0 |
| `src/v3/compiler/tests/integration/tc3_strong_normalization_strict_fire_test.rs` | 1 | 0 | 1 | 0 |
| `src/v3/compiler/tests/integration/test_runner_test.rs` | 45 | 0 | 0 | 45 |
| `src/v3/compiler/tests/integration/thesis_parallelism_test.rs` | 9 | 0 | 9 | 0 |
| `src/v3/compiler/tests/integration/thesis_validation_test.rs` | 24 | 0 | 24 | 0 |
| `src/v3/compiler/tests/integration/timing_lens_substrate_carrier_test.rs` | 22 | 0 | 22 | 0 |
| `src/v3/compiler/tests/integration/v2_oracle_no_remaining_test_consumers_test.rs` | 10 | 0 | 0 | 10 |
| `src/v3/compiler/tests/integration/v4_bin_main_dag_smoke_test.rs` | 1 | 1 | 0 | 0 |
| `src/v3/compiler/tests/integration/v4_compiler_compile_public_terminal_smoke_test.rs` | 10 | 10 | 0 | 0 |
| `src/v3/compiler/tests/integration/v4_compiler_emit_translate_smoke_test.rs` | 18 | 18 | 0 | 0 |
| `src/v3/compiler/tests/integration/v4_extdeps_file_system_dag_smoke_test.rs` | 5 | 5 | 0 | 0 |
| `src/v3/compiler/tests/integration/v4_extdeps_react_dag_smoke_test.rs` | 13 | 13 | 0 | 0 |
| `src/v3/compiler/tests/integration/v4_lens_registry_dag_smoke_test.rs` | 4 | 4 | 0 | 0 |
| `src/v3/compiler/tests/integration/v4_lens_testgen_dag_smoke_test.rs` | 13 | 13 | 0 | 0 |
| `src/v3/compiler/tests/integration/v4_std_model_core_dag_smoke_test.rs` | 7 | 7 | 0 | 0 |
| `src/v3/compiler/tests/integration/v4_test_bootstrap_infra_closeout_test.rs` | 7 | 7 | 0 | 0 |
| `src/v3/compiler/tests/integration/value_body_substrate_mirror_isomorphism_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/workflow_root_port_test.rs` | 3 | 0 | 3 | 0 |
| `src/v3/compiler/tests/integration/workflow_substrate_carriers_test.rs` | 4 | 0 | 4 | 0 |

---

## Recommended retirement batch order (for downstream dispatch)

1. **v4 smoke + v4 bootstrap closeout** (~DELETE cluster) — zero v3 behavioral loss.
2. **Cementing Rust modules** after gate-87 `.dag` CI green.
3. **`m1_substrate_test.rs` bulk DELETE** (115 tests) — largest ROI; retain ≤20 TestClaims.
4. **Host `.dag` claim drivers** (`r3_free_consequences_*`, corpus seeds, skeleton).
5. **m2_lens_*_migration_test** freshness duplicates.
6. **REPLACE wave** only for KEEP-adjacent load-bearing gates (anthropic wire, gate receipts still open).
7. **KEEP inventory** shrinks last (SG-0 census, TestRunner, boundary, gate #62/#71).
