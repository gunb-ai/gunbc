//! R3 gate #87 — PB-B-1 runner table for `tests/dag/t_r3_gate_87_cementing_regen_*.dag` harnesses.
//!
//! **INVARIANTS P2 / single authority:** `R3_GATE_87_CEMENTING_REGEN_SUITES` is the only merge-visible
//! inventory for which harnesses `t_pb_b_1_dag_runner_test` executes. `cementing_dispatch` and
//! `r3_gate_87_lens_cementing_regen_receipts_test` consume the same table so `.dag` Band-C receipts
//! cannot pass `CementingDispatchMatchesProjection` without a matching runner row.

use std::collections::BTreeSet;
use std::path::Path;

/// `LensRegistryEntry.name` values implied by [`R3_GATE_87_CEMENTING_REGEN_SUITES`] harness `file`
/// paths (`t_r3_gate_87_cementing_regen_<name>.dag`). **Single authority** for the gate-#87
/// inventory ratchet: `r3_gate_87_lens_cementing_regen_receipts_test` compares live `regen.dag`
/// names against this set so a new registry row cannot ship without a matching runner row.
pub fn r3_gate_87_cementing_regen_lens_names_for_runner_table() -> BTreeSet<String> {
    R3_GATE_87_CEMENTING_REGEN_SUITES
        .iter()
        .map(|(_, file, _, _)| lens_name_from_gate_87_harness_path(file))
        .collect()
}

/// Workspace-relative harness file paths enumerated in [`R3_GATE_87_CEMENTING_REGEN_SUITES`].
/// The gate-#87 receipt ratchet compares this set to the on-disk
/// `t_r3_gate_87_cementing_regen_*.dag` inventory so orphan receipts cannot land without runner
/// execution.
pub fn r3_gate_87_cementing_regen_runner_table_files() -> BTreeSet<String> {
    R3_GATE_87_CEMENTING_REGEN_SUITES
        .iter()
        .map(|(_, file, _, _)| (*file).to_string())
        .collect()
}

/// Module stems (`tests/dag/<stem>.dag`) enumerated in [`R3_GATE_87_CEMENTING_REGEN_SUITES`].
/// [`crate::cementing_dispatch::evaluate_cementing_dispatch_projection`] requires every
/// `kind == "dag"` receipt in `cementing_dispatch.dag` to use one of these stems so the dispatch
/// list cannot drift from the T-PB-B-1 runner.
pub fn r3_gate_87_cementing_regen_pb_b1_dag_module_stems() -> BTreeSet<String> {
    R3_GATE_87_CEMENTING_REGEN_SUITES
        .iter()
        .map(|(_, file, _, _)| {
            Path::new(file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_else(|| {
                    panic!("R3_GATE_87_CEMENTING_REGEN_SUITES: invalid harness path `{file}`")
                })
                .to_string()
        })
        .collect()
}

// R3 gate #87 — every `LensRegistryEntry` in `src/v3/compiler/regen.dag` has a
// `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<name>.dag` harness evaluated here.
const R3_GATE_87_CEMENTING_HARNESS_PATH_PREFIX: &str =
    "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_";
const R3_GATE_87_CEMENTING_HARNESS_PATH_SUFFIX: &str = ".dag";

fn lens_name_from_gate_87_harness_path(file: &str) -> String {
    file.strip_prefix(R3_GATE_87_CEMENTING_HARNESS_PATH_PREFIX)
        .and_then(|rest| rest.strip_suffix(R3_GATE_87_CEMENTING_HARNESS_PATH_SUFFIX))
        .unwrap_or_else(|| {
            panic!(
                "R3_GATE_87_CEMENTING_REGEN_SUITES: harness path must be \
                 `{prefix}<lens_name>{suffix}`, got `{file}`",
                prefix = R3_GATE_87_CEMENTING_HARNESS_PATH_PREFIX,
                suffix = R3_GATE_87_CEMENTING_HARNESS_PATH_SUFFIX,
            )
        })
        .to_string()
}

pub const R3_GATE_87_CEMENTING_REGEN_SUITES: &[(&str, &str, &str, &[&str])] = &[
    (
        include_str!("../tests/dag/t_r3_gate_87_cementing_regen_cost.dag"),
        "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost.dag",
        "r3_gate_87_cementing_regen_cost_suite",
        &[
            "cementing_regen_cost_merge_sort_lens_output_equals",
            "cementing_regen_cost",
        ],
    ),
    (
        include_str!("../tests/dag/t_r3_gate_87_cementing_regen_cost_symbolic.dag"),
        "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost_symbolic.dag",
        "r3_gate_87_cementing_regen_cost_symbolic_suite",
        &[
            "cementing_regen_cost_symbolic",
            "cementing_regen_cost_symbolic_countdown",
        ],
    ),
    (
        include_str!("../tests/dag/t_r3_gate_87_cementing_regen_cost_target_realization.dag"),
        "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost_target_realization.dag",
        "r3_gate_87_cementing_regen_cost_target_realization_suite",
        &["cementing_regen_cost_target_realization"],
    ),
    (
        include_str!("../tests/dag/t_r3_gate_87_cementing_regen_effect_enumeration.dag"),
        "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_effect_enumeration.dag",
        "r3_gate_87_cementing_regen_effect_enumeration_suite",
        &["cementing_regen_effect_enumeration"],
    ),
    (
        include_str!("../tests/dag/t_r3_gate_87_cementing_regen_infer_helpers.dag"),
        "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_infer_helpers.dag",
        "r3_gate_87_cementing_regen_infer_helpers_suite",
        &["cementing_regen_infer_helpers"],
    ),
    (
        include_str!("../tests/dag/t_r3_gate_87_cementing_regen_lower_helpers.dag"),
        "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_lower_helpers.dag",
        "r3_gate_87_cementing_regen_lower_helpers_suite",
        &["cementing_regen_lower_helpers"],
    ),
    (
        include_str!("../tests/dag/t_r3_gate_87_cementing_regen_parallelism.dag"),
        "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_parallelism.dag",
        "r3_gate_87_cementing_regen_parallelism_suite",
        &["cementing_regen_parallelism"],
    ),
    (
        include_str!("../tests/dag/t_r3_gate_87_cementing_regen_provenance.dag"),
        "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_provenance.dag",
        "r3_gate_87_cementing_regen_provenance_suite",
        &["cementing_regen_provenance"],
    ),
    (
        include_str!("../tests/dag/t_r3_gate_87_cementing_regen_structural_resolution.dag"),
        "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_structural_resolution.dag",
        "r3_gate_87_cementing_regen_structural_resolution_suite",
        &["cementing_regen_structural_resolution"],
    ),
    (
        include_str!("../tests/dag/t_r3_gate_87_cementing_regen_unused_parameters.dag"),
        "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_unused_parameters.dag",
        "r3_gate_87_cementing_regen_unused_parameters_suite",
        &["cementing_regen_unused_parameters"],
    ),
    (
        include_str!("../tests/dag/t_r3_gate_87_cementing_regen_variant_payload.dag"),
        "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_variant_payload.dag",
        "r3_gate_87_cementing_regen_variant_payload_suite",
        &["cementing_regen_variant_payload"],
    ),
];
