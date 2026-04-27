//! **Layer:** integration
//!
//! Regen-clean ratchet: Shape-A Rust/Go/Python target specs must bind
//! `*_source_filtering.internal` to the single canonical
//! `data internal_source_filtering` in `computation_model.dag`, not fork
//! independent `excluded_prefixes` lists (internal/bootstrap declaration
//! prefix drift across targets).

const COMPUTATION_MODEL: &str = include_str!("../../../std/computation_model.dag");
const RUST_SPEC: &str = include_str!("../../../spec/rust.dag");
const GO_SPEC: &str = include_str!("../../../spec/go.dag");
const PYTHON_SPEC: &str = include_str!("../../../spec/python.dag");

const CANONICAL_BINDING: &str = "internal: internal_source_filtering";

#[test]
fn computation_model_declares_internal_source_filtering() {
    assert!(
        COMPUTATION_MODEL.contains("data internal_source_filtering: SourceFiltering"),
        "expected `data internal_source_filtering: SourceFiltering` in computation_model.dag"
    );
}

#[test]
fn shape_a_target_specs_reference_canonical_internal_source_filtering() {
    for (label, row, spec) in [
        (
            "rust.dag",
            "data rust_source_filtering: ShapeATargetSourceFiltering",
            RUST_SPEC,
        ),
        (
            "go.dag",
            "data go_source_filtering: ShapeATargetSourceFiltering",
            GO_SPEC,
        ),
        (
            "python.dag",
            "data python_source_filtering: ShapeATargetSourceFiltering",
            PYTHON_SPEC,
        ),
    ] {
        assert!(spec.contains(row), "{label}: expected `{row}`");
        assert!(
            spec.contains(CANONICAL_BINDING),
            "{label}: must bind `{CANONICAL_BINDING}` so internal/bootstrap prefixes stay canonical"
        );
    }
}
