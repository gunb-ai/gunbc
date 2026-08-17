//! Regen READ set: the import closure of `src/v1` as recorded on origin/main.
//!
//! Namespace-cut sources have no `import` statements. A reference walk over
//! this tree cannot reconstruct the set main compiled (qualified-only 639,
//! unique-bare 1130, homonym 1203). This list is that closure, mapped onto
//! this tree's files, identity-joined at load (a listed path that is missing
//! refuses). Dissolve-on: the qualified `container.member` walk matches this
//! set identity-for-identity.
//!
//! Reconstructed 2026-08-17 from origin/main import lists. 142 paths.
//! `std.occurrence_identity` is present; `std.observation` is not.

pub const REGEN_READ_IMPORT_CLOSURE_PATHS: &[&str] = &[
    "dag/extdeps/container/oci/digest.dag",
    "dag/extdeps/external_authority.dag",
    "dag/extdeps/languages/dag/emit.dag",
    "dag/extdeps/languages/dag/syntax.dag",
    "dag/extdeps/languages/dag/types.dag",
    "dag/extdeps/languages/go/emit.dag",
    "dag/extdeps/languages/go/syntax.dag",
    "dag/extdeps/languages/go/types.dag",
    "dag/extdeps/languages/python/emit.dag",
    "dag/extdeps/languages/python/syntax.dag",
    "dag/extdeps/languages/python/types.dag",
    "dag/extdeps/languages/rust/emit.dag",
    "dag/extdeps/languages/rust/syntax.dag",
    "dag/extdeps/languages/rust/types.dag",
    "dag/extdeps/rust/cargo.dag",
    "dag/extdeps/rust/version.dag",
    "dag/extdeps/units/dimensionless.dag",
    "dag/extdeps/units/iec_80000_13.dag",
    "dag/extdeps/units/iso8601.dag",
    "dag/extdeps/units/iso_80000_3.dag",
    "dag/extdeps/uri.dag",
    "dag/extdeps/uri_path.dag",
    "dag/extdeps/version.dag",
    "dag/extdeps/version/semver.dag",
    "dag/gunbc/namespace_reference_derived_closure_admission.dag",
    "dag/gunbc/namespace_reference_derived_closure_contract.dag",
    "dag/gunbc/rust_decl_type_overlay.dag",
    "dag/gunbc/stage0_crate_layout_generated.dag",
    "dag/gunbc/stage0_crate_partition_generated.dag",
    "dag/std/algebra.dag",
    "dag/std/checked_arithmetic.dag",
    "dag/std/coercion.dag",
    "dag/std/computation.dag",
    "dag/std/constructors.dag",
    "dag/std/content_hash.dag",
    "dag/std/currency.dag",
    "dag/std/decl_ref.dag",
    "dag/std/disposition.dag",
    "dag/std/dissolution.dag",
    "dag/std/effects.dag",
    "dag/std/emit_model.dag",
    "dag/std/error_primitives.dag",
    "dag/std/execution_mode.dag",
    "dag/std/graph.dag",
    "dag/std/http_path.dag",
    "dag/std/induction.dag",
    "dag/std/integer.dag",
    "dag/std/interface_summary.dag",
    "dag/std/iteration.dag",
    "dag/std/keyed_roster.dag",
    "dag/std/keyed_row.dag",
    "dag/std/logic.dag",
    "dag/std/machine_constraints.dag",
    "dag/std/magnitude.dag",
    "dag/std/measure.dag",
    "dag/std/nat.dag",
    "dag/std/node.dag",
    "dag/std/occurrence_binding.dag",
    "dag/std/occurrence_binding_candidates.dag",
    "dag/std/occurrence_binding_resolve.dag",
    "dag/std/occurrence_identity.dag",
    "dag/std/pareto.dag",
    "dag/std/process_termination.dag",
    "dag/std/realization_schedule.dag",
    "dag/std/reference_binding_observation.dag",
    "dag/std/roster_frontier.dag",
    "dag/std/serialization.dag",
    "dag/std/source_annotation.dag",
    "dag/std/syntax.dag",
    "dag/std/termination.dag",
    "dag/std/trait_derive_shape.dag",
    "dag/std/types.dag",
    "dag/std/unicode/types.dag",
    "dag/std/witness_admission.dag",
    "src/v1/00_core.dag",
    "src/v1/01_tokenize.dag",
    "src/v1/02_parse.dag",
    "src/v1/03_normalize.dag",
    "src/v1/03_resolve.dag",
    "src/v1/04_access.dag",
    "src/v1/04_cycle.dag",
    "src/v1/04_emit_info.dag",
    "src/v1/04_env.dag",
    "src/v1/04_infer.dag",
    "src/v1/04_items.dag",
    "src/v1/04_lookup.dag",
    "src/v1/04_method.dag",
    "src/v1/04_occurrence_binding.dag",
    "src/v1/04_patterns.dag",
    "src/v1/04_resolve.dag",
    "src/v1/04_service.dag",
    "src/v1/04_sigs.dag",
    "src/v1/04_types.dag",
    "src/v1/05_emit.dag",
    "src/v1/05_emit_core_support.dag",
    "src/v1/05_emit_go.dag",
    "src/v1/05_emit_python.dag",
    "src/v1/05_emit_rust.dag",
    "src/v1/annotation_bind.dag",
    "src/v1/artifact.dag",
    "src/v1/closure_stub_v2_std_integer_rust.dag",
    "src/v1/closure_stub_v2_std_text_rust.dag",
    "src/v1/coercion.dag",
    "src/v1/compile.dag",
    "src/v1/compiler_tests_rust.dag",
    "src/v1/complexity.dag",
    "src/v1/dag_collect.dag",
    "src/v1/dag_collect_support.dag",
    "src/v1/effect_derivation.dag",
    "src/v1/frontend_observation.dag",
    "src/v1/gunbc/namespace_reference_derived_closure_production_observations.dag",
    "src/v1/gunbc/occurrence_binding_parser_walk.dag",
    "src/v1/languages.dag",
    "src/v1/ownership.dag",
    "src/v1/probe_emit_interp.dag",
    "src/v1/runtime_go.dag",
    "src/v1/runtime_rust.dag",
    "src/v1/stage0/tests/fixtures/fact_cardinality_split_brace.dag",
    "src/v1/stage0_crates.dag",
    "src/v1/tests/claim/caret_parse_smoke_test.dag",
    "src/v1/tests/claim/dag_parse_continuation_operator_witness_test.dag",
    "src/v1/tests/claim/emitter_ambiguous_variant_owner_witness_test.dag",
    "src/v1/tests/claim/namespace_reference_derived_closure_production_admissions_witness_test.dag",
    "src/v1/tests/claim/occurrence_binding_parser_walk_witness_test.dag",
    "src/v1/tests/claim/occurrence_identity_debt_receipt_test.dag",
    "src/v1/tests/claim/ordinary_frontend_observation_test.dag",
    "src/v1/tests/claim/pattern_binder_declaration_node_test.dag",
    "src/v1/tests/claim/required_expr_newline_continuation_test.dag",
    "src/v1/tests/claim/v1_annotation_binding_test.dag",
    "src/v1/tests/claim/v1_annotation_capture_test.dag",
    "src/v1/tests/claim/v1_annotation_erasure_test.dag",
    "src/v1/tests/claim/v1_annotation_round_trip_test.dag",
    "src/v1/tests/claim/v1_annotation_target_emission_test.dag",
    "src/v1/tests/claim/v1_complexity_eviction_hazard_test.dag",
    "src/v1/tests/claim/v1_match_pattern_identity_test.dag",
    "src/v1/tests/fixtures/non_ascii_perf.dag",
    "src/v1/tests/fixtures/whole_tree_wiring_enum/common.dag",
    "src/v1/tests/fixtures/whole_tree_wiring_enum/mod_a.dag",
    "src/v1/tests/fixtures/whole_tree_wiring_enum/mod_b.dag",
    "src/v1/trace.dag",
    "src/v1/trait_derive_emit.dag",
    "src/v1/workspace_members.dag",
];

#[cfg(test)]
mod tests {
    use super::REGEN_READ_IMPORT_CLOSURE_PATHS;
    use std::path::PathBuf;

    /// Identity join: every listed READ path exists in this checkout.
    /// CARGO_MANIFEST_DIR is `src/v1/stage0`.
    #[test]
    fn frozen_import_closure_paths_exist() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let mut missing = Vec::new();
        for rel in REGEN_READ_IMPORT_CLOSURE_PATHS {
            if !workspace.join(rel).is_file() {
                missing.push(*rel);
            }
        }
        assert!(
            missing.is_empty(),
            "regen READ import-closure paths missing: {missing:?}"
        );
    }
}
