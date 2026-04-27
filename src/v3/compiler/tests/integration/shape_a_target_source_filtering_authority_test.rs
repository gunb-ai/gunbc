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

/// Returns the `{ ... }` value block for a `data <name>: <Ty> = {` row starting
/// at `row_prefix` (must be unique in `spec`). Used so the canonical `internal`
/// binding is asserted inside the specific declaration, not anywhere in the file.
fn struct_value_block<'a>(spec: &'a str, row_prefix: &str) -> &'a str {
    let pos = spec.find(row_prefix).unwrap_or_else(|| {
        panic!("expected `{row_prefix}` in spec");
    });
    let tail = &spec[pos..];
    let open = tail
        .find('{')
        .unwrap_or_else(|| panic!("expected `{{` after `{row_prefix}`"));
    let from_brace = &tail[open..];
    let mut depth: i32 = 0;
    let mut end_byte = 0usize;
    for (i, c) in from_brace.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end_byte = i + c.len_utf8();
                    break;
                }
            }
            _ => {}
        }
    }
    assert!(end_byte > 0, "unclosed `{{` for `{row_prefix}`");
    &from_brace[..end_byte]
}

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
        let block = struct_value_block(spec, row);
        assert!(
            block.contains(CANONICAL_BINDING),
            "{label}: `{row}` value must bind `{CANONICAL_BINDING}` (got block: {block:?})"
        );
    }
}
