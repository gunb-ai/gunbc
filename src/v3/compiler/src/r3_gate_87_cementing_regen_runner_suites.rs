//! R3 gate #87 — PB-B-1 runner table for `tests/dag/t_r3_gate_87_cementing_regen_*.dag` harnesses.
//!
//! **INVARIANTS P2 / single authority:** `R3_GATE_87_CEMENTING_REGEN_SUITES` is the only merge-visible
//! inventory for which harnesses `t_pb_b_1_dag_runner_test` executes. `cementing_dispatch` and
//! `r3_gate_87_lens_cementing_regen_receipts_test` consume the same table so `.dag` Band-C receipts
//! cannot pass `CementingDispatchMatchesProjection` without a matching runner row.
//!
//! **`Compiles` placeholders:** [`R3_GATE_87_CEMENTING_REGEN_PLACEHOLDER_DISSOLUTION_LEDGER`] is the
//! merge-visible dissolution ledger for harnesses still on source-compilation placeholders (P5
//! scaffold discipline — each row names the paired Rust receipt and the audit substring for the
//! harness `// Dissolution trigger:` comment).

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

/// Gate-#87 regen harness row still carried as a `Compiles` receipt in `.dag` (not yet a public
/// `LensOutputEquals` behavioral witness).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct R3Gate87CementingRegenPlaceholderDissolutionRow {
    /// `LensRegistryEntry.name` in `regen.dag` for this harness stem.
    pub lens_registry_name: &'static str,
    /// Harness path under the workspace root (matches [`R3_GATE_87_CEMENTING_REGEN_SUITES`]).
    pub harness_path: &'static str,
    /// `#[test]` fn name in `r3_gate_87_lens_cementing_regen_receipts_test` cited by the harness.
    pub temporary_rust_receipt_fn: &'static str,
    /// Must appear in the harness header after `// Dissolution trigger:` (substring ratchet).
    pub dissolution_trigger_substr: &'static str,
}

/// Dissolution ledger for gate-#87 `Compiles` cementing harnesses — must stay in lockstep with
/// `predicate: Compiles` rows in [`R3_GATE_87_CEMENTING_REGEN_SUITES`].
pub const R3_GATE_87_CEMENTING_REGEN_PLACEHOLDER_DISSOLUTION_LEDGER:
    &[R3Gate87CementingRegenPlaceholderDissolutionRow] = &[
    R3Gate87CementingRegenPlaceholderDissolutionRow {
        lens_registry_name: "infer_helpers",
        harness_path: "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_infer_helpers.dag",
        temporary_rust_receipt_fn: "r3_gate_87_infer_helpers_lens_source_compiles",
        dissolution_trigger_substr:
            "when an `infer_helpers` public output carrier is authorable as `.dag` data",
    },
    R3Gate87CementingRegenPlaceholderDissolutionRow {
        lens_registry_name: "lower_helpers",
        harness_path: "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_lower_helpers.dag",
        temporary_rust_receipt_fn: "r3_gate_87_lower_helpers_lens_source_compiles",
        dissolution_trigger_substr:
            "when a `lower_helpers` public output carrier is authorable as `.dag` data",
    },
    R3Gate87CementingRegenPlaceholderDissolutionRow {
        lens_registry_name: "variant_payload",
        harness_path: "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_variant_payload.dag",
        temporary_rust_receipt_fn: "r3_gate_87_variant_payload_lens_source_compiles",
        dissolution_trigger_substr:
            "when a stable variant-declaration fixture and `VariantPayloadShapeLookup`",
    },
];

#[cfg(test)]
mod r3_gate_87_placeholder_dissolution_ledger_tests {
    use super::*;

    /// Paired Rust receipts for gate-#87 placeholder harnesses (`INVARIANTS` P5).
    ///
    /// `temporary_rust_receipt_fn` in [`R3_GATE_87_CEMENTING_REGEN_PLACEHOLDER_DISSOLUTION_LEDGER`]
    /// must name a real `#[test]` in this file — not only a string mention in the `.dag` harness.
    const R3_GATE_87_LENS_CEMENTING_REGEN_RECEIPTS_TEST_RS: &str =
        include_str!("../tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs");

    fn suite_source_for_harness_path(path: &str) -> &'static str {
        R3_GATE_87_CEMENTING_REGEN_SUITES
            .iter()
            .find_map(|(source, p, _, _)| (*p == path).then_some(*source))
            .unwrap_or_else(|| {
                panic!("harness path not in R3_GATE_87_CEMENTING_REGEN_SUITES: {path}")
            })
    }

    fn assert_placeholder_rust_receipt_is_live_test_fn(fn_name: &str) {
        let needle = format!("fn {fn_name}(");
        let Some(fn_pos) = R3_GATE_87_LENS_CEMENTING_REGEN_RECEIPTS_TEST_RS.find(&needle) else {
            panic!(
                "ledger `temporary_rust_receipt_fn` `{fn_name}` must exist as a `fn` item in \
                 `r3_gate_87_lens_cementing_regen_receipts_test.rs` (a `.dag` comment citation alone \
                 is not a receipt ratchet)"
            );
        };
        let before = &R3_GATE_87_LENS_CEMENTING_REGEN_RECEIPTS_TEST_RS[..fn_pos];
        let Some(attr_pos) = before.rfind("#[test]") else {
            panic!(
                "ledger `temporary_rust_receipt_fn` `{fn_name}` must be a `#[test]` function in \
                 `r3_gate_87_lens_cementing_regen_receipts_test.rs`"
            );
        };
        let after_attr = attr_pos + "#[test]".len();
        let gap = &R3_GATE_87_LENS_CEMENTING_REGEN_RECEIPTS_TEST_RS[after_attr..fn_pos];
        assert!(
            gap.chars().all(|c| c.is_whitespace()),
            "ledger `temporary_rust_receipt_fn` `{fn_name}` must be decorated directly by `#[test]` \
             in `r3_gate_87_lens_cementing_regen_receipts_test.rs` (gap was non-whitespace-only: \
             {gap:?})"
        );
    }

    #[test]
    fn compiles_harness_paths_match_placeholder_dissolution_ledger() {
        let compiles_paths: BTreeSet<&'static str> = R3_GATE_87_CEMENTING_REGEN_SUITES
            .iter()
            .filter(|(source, _, _, _)| source.contains("predicate: Compiles"))
            .map(|(_, path, _, _)| *path)
            .collect();
        let ledger_paths: BTreeSet<&'static str> =
            R3_GATE_87_CEMENTING_REGEN_PLACEHOLDER_DISSOLUTION_LEDGER
                .iter()
                .map(|row| row.harness_path)
                .collect();
        assert_eq!(
            compiles_paths, ledger_paths,
            "`R3_GATE_87_CEMENTING_REGEN_PLACEHOLDER_DISSOLUTION_LEDGER` must list exactly the \
             `predicate: Compiles` harnesses in `R3_GATE_87_CEMENTING_REGEN_SUITES`"
        );
    }

    #[test]
    fn placeholder_ledger_rows_are_consistent_with_harness_files() {
        for row in R3_GATE_87_CEMENTING_REGEN_PLACEHOLDER_DISSOLUTION_LEDGER {
            assert_eq!(
                lens_name_from_gate_87_harness_path(row.harness_path),
                row.lens_registry_name,
                "ledger lens_registry_name must match harness stem for {}",
                row.harness_path
            );
            let source = suite_source_for_harness_path(row.harness_path);
            assert!(
                source.contains("predicate: Compiles"),
                "{} must remain a Compiles placeholder harness",
                row.harness_path
            );
            assert!(
                source.contains("// Dissolution trigger:"),
                "{} must carry a `// Dissolution trigger:` header",
                row.harness_path
            );
            assert!(
                source.contains(row.dissolution_trigger_substr),
                "{} must contain dissolution trigger substring {:?}",
                row.harness_path,
                row.dissolution_trigger_substr
            );
            assert!(
                source.contains(row.temporary_rust_receipt_fn),
                "{} must cite Rust receipt `{}`",
                row.harness_path,
                row.temporary_rust_receipt_fn
            );
            assert_placeholder_rust_receipt_is_live_test_fn(row.temporary_rust_receipt_fn);
        }
    }
}
