//! Integration tests for the v2 self-hosted compiler.
//!
//! Tests call stage0 functions directly — no v1 interpreter, no Value wrapping.
//! Stage0 is a Rust crate generated from .dag source files by the v1 emitter.

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
mod bootstrap;
#[cfg(test)]
mod bug_sentinel_ratchet;
#[cfg(test)]
mod coproduct_reflection_conformance_test;
#[cfg(test)]
mod data_cache_scoping_test;
#[cfg(test)]
mod derive_bound_fail_closed_test;
#[cfg(test)]
mod enforce_host_marshal_probe_test;
#[cfg(test)]
mod diagnostics;
#[cfg(test)]
mod effects;
#[cfg(test)]
mod fn_as_value_test;
#[cfg(test)]
mod fold_list_generic_instantiation_test;
#[cfg(test)]
mod generator_match_arm_test;
#[cfg(test)]
mod html_markup_smoke_test;
#[cfg(test)]
mod infer_semantics;
#[cfg(test)]
mod int_pow_bounded_test;
#[cfg(test)]
mod interp_stats_test;
#[cfg(test)]
mod interpreted_parse_termination_test;
#[cfg(test)]
mod list_free_monoid_chokepoint_test;
#[cfg(test)]
mod map_lookup_dual_dispatch_test;
#[cfg(test)]
mod measure_field_access_test;
#[cfg(test)]
mod money_carrier_cost_witness_test;
#[cfg(test)]
mod nodefold_generic_instantiation_test;
#[cfg(test)]
mod parse;
#[cfg(test)]
mod pb_method_template_projection_consumability;
#[cfg(test)]
mod pd3_adversarial;
#[cfg(test)]
mod peano_materialization_cap_test;
#[cfg(test)]
mod pipeline;
#[cfg(test)]
mod r2_emit_add_named_test;
#[cfg(test)]
mod rc_probe_wire_decode_call_test;
#[cfg(test)]
mod render_repeat_test;
#[cfg(test)]
mod resolve_cross_process_cache_test;
#[cfg(test)]
mod resolve_typed_cache_equivalence_test;
#[cfg(test)]
mod source_audit;
#[cfg(test)]
mod sub_value_lattice_factor_test;
#[cfg(test)]
mod target_model_runtime_import_repro;
#[cfg(test)]
mod typescript_effect_io_receipt_test;
#[cfg(test)]
mod v2_compiler_lib_test;
#[cfg(test)]
mod value_carrier_swap_test;
#[cfg(test)]
mod width_nat_type_arg_test;
#[cfg(test)]
mod witness_option_bridge_test;
