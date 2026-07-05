#![allow(
    clippy::disallowed_macros,
    clippy::absurd_extreme_comparisons,
    dead_code
)]

pub mod helpers;

#[cfg(test)]
mod a4_opacity;
#[cfg(test)]
mod b1_hash_primitive_test;
#[cfg(test)]
mod body_producer_infer_perf_witness_test;
#[cfg(test)]
mod bug_sentinel_ratchet;
#[cfg(test)]
mod cache_purity_oracle_test;
#[cfg(test)]
mod constructor_owner_ruling_test;
#[cfg(test)]
mod consumed_input_closure_drift_test;
#[cfg(test)]
mod coproduct_reflection_conformance_test;
#[cfg(test)]
mod coverage_completeness_lens_test;
#[cfg(test)]
mod cross_representation_equality_test;
#[cfg(test)]
mod dag_collect_fingerprint_witness_test;
#[cfg(test)]
mod dag_comment_wall_test;
#[cfg(test)]
mod data_cache_scoping_test;
#[cfg(test)]
mod data_def_brand_alias_type_test;
#[cfg(test)]
mod dependency_pool_index_compile_test;
#[cfg(test)]
mod derive_bound_fail_closed_test;
#[cfg(test)]
mod eval_measurement_purity_test;
#[cfg(test)]
mod faithful_string_element_char_witness_test;
#[cfg(test)]
mod fn_as_value_test;
#[cfg(test)]
mod fold_unused_element_clone_elision_test;
#[cfg(test)]
mod func_env_scope_chain_test;
#[cfg(test)]
mod func_env_semantic_equivalence_test;
#[cfg(test)]
mod generator_match_arm_test;
#[cfg(test)]
mod generic_return_clone_bound_test;
#[cfg(test)]
mod gunbhub_serve_program_test;
#[cfg(test)]
mod html_markup_smoke_test;
#[cfg(test)]
mod int_pow_bounded_test;
#[cfg(test)]
mod interp_dry_run_test;
#[cfg(test)]
mod interp_stats_test;
#[cfg(test)]
mod interp_string_family_cast_test;
#[cfg(test)]
mod interp_wire_serialize_test;
#[cfg(test)]
mod interpreted_parse_termination_test;
#[cfg(test)]
mod ir_fixture_seam_soundness_test;
#[cfg(test)]
mod kernel_shadow_seams_test;
#[cfg(test)]
mod list_free_monoid_chokepoint_test;
#[cfg(test)]
mod map_literal_string_key_test;
#[cfg(test)]
mod map_lookup_dual_dispatch_test;
#[cfg(test)]
mod measure_alias_ctor_test;
#[cfg(test)]
mod measure_field_access_test;
#[cfg(test)]
mod measure_grounded_deref_test;
#[cfg(test)]
mod measure_periphery_emit_test;
#[cfg(test)]
mod measure_value_arg_unit_collapse_test;
#[cfg(test)]
mod module_authority_resolution_test;
#[cfg(test)]
mod nested_list_alias_emit_test;
#[cfg(test)]
mod nodefold_generic_instantiation_test;
#[cfg(test)]
mod optional_carrier_signature_test;
#[cfg(test)]
mod optional_consumer_fail_closed_test;
#[cfg(test)]
mod optional_receiver_method_unwrap_test;
#[cfg(test)]
mod parse;
#[cfg(test)]
mod parse_table_memo_amortization_test;
#[cfg(test)]
mod pd3_adversarial;
#[cfg(test)]
mod peano_materialization_cap_test;
#[cfg(test)]
mod pipeline;
#[cfg(test)]
mod resolve_cross_process_cache_test;
#[cfg(test)]
mod resolve_expr_types_retraversal_guard_test;
#[cfg(test)]
mod resolve_typed_cache_equivalence_test;
#[cfg(test)]
mod resolved_graph_cache_size_bound_test;
#[cfg(test)]
mod route_a_emit_fresh_cargo_green_test;
#[cfg(test)]
mod route_a_final_six_test;
#[cfg(test)]
mod shell_transport_stdin_wet_test;
#[cfg(test)]
mod source_root_ingest_manifest_host_test;
#[cfg(test)]
mod sub_value_lattice_factor_test;
#[cfg(test)]
mod target_model_runtime_import_repro;
#[cfg(test)]
mod type_alias_phantom_param_test;
#[cfg(test)]
mod type_env_scope_chain_test;
#[cfg(test)]
mod type_param_casing_test;
#[cfg(test)]
mod typescript_effect_io_receipt_test;
#[cfg(test)]
mod typescript_field_access_typecheck_test;
#[cfg(test)]
mod typescript_program_emit_run_test;
#[cfg(test)]
mod union_resolve_receipts_test;
#[cfg(test)]
mod v1_compiler_lib_test;
#[cfg(test)]
mod value_carrier_swap_test;
#[cfg(test)]
mod variant_export_surface_witness_test;
#[cfg(test)]
mod variant_owner_disambiguation_test;
#[cfg(test)]
mod wet_hermetic_equivalence_test;
#[cfg(test)]
mod whole_tree_wiring_enum_test;
#[cfg(test)]
mod width_nat_type_arg_test;
#[cfg(test)]
mod witness_option_bridge_test;
