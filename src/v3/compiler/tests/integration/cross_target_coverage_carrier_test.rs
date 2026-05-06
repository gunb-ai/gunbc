//! **Layer:** integration
//!
//! Slice-active ratchet for the L6 `EmissionPathProjection` substrate
//! carrier slice authored at `src/v3/std/cross_target_coverage.dag` per
//! Director Option 2 RATIFIED at gunbc#828 #issuecomment-4377533390 and the
//! worker brief
//! `docs/briefs/r3-l6-emission-path-projection-substrate-worker.md`.
//!
//! **What this slice asserts (load-bearing now):**
//!
//! 1. The six type declarations exist with the ratified field shapes
//!    (typed-substrate read; no string scan): `ShapeATarget`, `FormAxis`,
//!    `BehaviorAxis`, `MethodTemplateContractKey`, `EmissionCell`,
//!    `EmissionPathProjection`.
//! 2. `emission_path_projections == []` — the empty-state predicate. The
//!    slice ships with the data declaration empty by design; populated
//!    rows are scoped to Grounding's follow-up per §4.D=(b). The empty
//!    assertion is the slice's load-bearing claim that no row drift
//!    sneaks in via this PR.
//!
//! **What this slice DEFERS (per §4.D=(b)):**
//!
//! Per-row key bijection between `emission_path_projections` and the union
//! of `*_method_template_contracts` rows is the activation gate Grounding
//! flips on in the row-population follow-up PR. The bijection scaffold
//! lives there, not here — at HEAD `emission_path_projections: []` and the
//! source contract lists are non-empty, so a strict bijection cannot pass
//! while this slice ships empty. That's why the bijection check belongs
//! in Grounding's row-population PR (#1745).

use v3_compiler::dag::{TypeConnective, ValueBody};
use v3_compiler::generated_full_bootstrap_dag;

const SHAPE_A_TARGET: &str = "ShapeATarget";
const FORM_AXIS: &str = "FormAxis";
const BEHAVIOR_AXIS: &str = "BehaviorAxis";
const METHOD_TEMPLATE_CONTRACT_KEY: &str = "MethodTemplateContractKey";
const EMISSION_CELL: &str = "EmissionCell";
const EMISSION_PATH_PROJECTION: &str = "EmissionPathProjection";
const EMISSION_PATH_PROJECTIONS_DATA: &str = "emission_path_projections";

#[test]
fn cross_target_coverage_six_carrier_types_present() {
    let dag = generated_full_bootstrap_dag();
    for name in [
        SHAPE_A_TARGET,
        FORM_AXIS,
        BEHAVIOR_AXIS,
        METHOD_TEMPLATE_CONTRACT_KEY,
        EMISSION_CELL,
        EMISSION_PATH_PROJECTION,
    ] {
        dag.declaration_by_name(name)
            .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap dag"));
    }
}

#[test]
fn shape_a_target_is_disj_with_three_variants() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name(SHAPE_A_TARGET)
        .expect("ShapeATarget must exist");
    let TypeConnective::Disj { variants } = &decl.connective else {
        panic!(
            "ShapeATarget must be `Disj` (Rust | Python | Go); got {:?}",
            decl.connective
        );
    };
    assert_eq!(
        variants.len(),
        3,
        "ShapeATarget must carry exactly three variants (Rust / Python / Go); got {}",
        variants.len()
    );
}

#[test]
fn form_axis_is_disj_with_six_variants() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name(FORM_AXIS)
        .expect("FormAxis must exist");
    let TypeConnective::Disj { variants } = &decl.connective else {
        panic!("FormAxis must be `Disj`; got {:?}", decl.connective);
    };
    assert_eq!(
        variants.len(),
        6,
        "FormAxis must mirror v3_compiler::dag::TypeConnective's six discriminants; got {}",
        variants.len()
    );
}

#[test]
fn behavior_axis_is_disj_with_five_variants() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name(BEHAVIOR_AXIS)
        .expect("BehaviorAxis must exist");
    let TypeConnective::Disj { variants } = &decl.connective else {
        panic!("BehaviorAxis must be `Disj`; got {:?}", decl.connective);
    };
    assert_eq!(
        variants.len(),
        5,
        "BehaviorAxis must mirror v3_compiler::dag::Behavior's five discriminants; got {}",
        variants.len()
    );
}

#[test]
fn emission_path_projections_data_is_empty_list() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name(EMISSION_PATH_PROJECTIONS_DATA)
        .unwrap_or_else(|| {
            panic!(
                "`{EMISSION_PATH_PROJECTIONS_DATA}` data declaration missing from full bootstrap dag"
            )
        });
    let body = decl
        .value_body
        .as_ref()
        .expect("emission_path_projections must be a `data` declaration with a value body");
    let ValueBody::List(rows) = body else {
        panic!(
            "emission_path_projections must lower as `ValueBody::List` (declared as \
             `List<EmissionPathProjection>`); got {body:?}"
        );
    };
    assert!(
        rows.is_empty(),
        "Phase-1 carrier slice ships `emission_path_projections` EMPTY; \
         row population is Grounding's follow-up (#1745) per §4.D=(b). \
         Got {} row(s).",
        rows.len()
    );
}
