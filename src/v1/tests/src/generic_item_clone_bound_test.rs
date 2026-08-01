//! E0277: generic struct and enum items that #[derive] Clone/Debug/serde must emit
//! `N: Clone` on the item's generic parameter list when N appears in field or variant
//! payload types (bare generic fields, FreeMonoid<N>, nested applied types).
//!
//! Negative control: a generic param unused in field types must NOT receive a spurious bound.

use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

const FIXTURE: &str = concat!(
    "module generic_item.fixture\n",
    "import std.algebra { FreeMonoid, Empty }\n\n",
    "type ContainmentPath<N> {\n",
    "  ancestors: FreeMonoid<N>\n",
    "  terminal: N\n",
    "}\n\n",
    "type Holder<N> {\n",
    "  tag: Int\n",
    "}\n\n",
    "type OccurrenceBindingResult<N>\n",
    "  = OccurrenceBound { binding: OccurrenceBinding<N> }\n",
    "  | OccurrenceUnbound { occurrence: BindingOccurrence<N> }\n\n",
    "type OccurrenceBinding<N> {\n",
    "  occurrence: BindingOccurrence<N>\n",
    "  candidate: BindingCandidate<N>\n",
    "}\n\n",
    "type BindingOccurrence<N> {\n",
    "  containment: ContainmentPath<N>\n",
    "}\n\n",
    "type BindingCandidate<N> {\n",
    "  containment: ContainmentPath<N>\n",
    "}\n",
);

fn emit_host() -> String {
    compile_dag_named(
        "src/v1/generic_item_fixture.dag",
        FIXTURE,
        RenderTarget::Rust,
    )
    .files
    .iter()
    .map(|f| f.content.clone())
    .collect::<Vec<_>>()
    .join("\n")
}

#[test]
fn generic_struct_with_freemonoid_field_gets_clone_bound() {
    let emitted = emit_host();
    assert!(
        emitted.contains("struct ContainmentPath<N: Clone>"),
        "ContainmentPath with FreeMonoid<N> and bare N fields must emit N: Clone:\n{emitted}"
    );
}

#[test]
fn generic_enum_with_payload_gets_clone_bound() {
    let emitted = emit_host();
    assert!(
        emitted.contains("enum OccurrenceBindingResult<N: Clone>"),
        "enum with N in variant payloads must emit N: Clone:\n{emitted}"
    );
}

#[test]
fn unused_generic_param_has_no_clone_bound() {
    let emitted = emit_host();
    assert!(
        emitted.contains("struct Holder<N>") && !emitted.contains("struct Holder<N: Clone>"),
        "unused generic param N must not receive a spurious Clone bound:\n{emitted}"
    );
}
