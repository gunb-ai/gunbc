//! **Layer:** integration
//!
//! T-Ground-Engine Phase-1 unblock (Path 2): `dsl/extdeps/languages/rust/
//! primitives.dag` is loaded into the bootstrap Dag; `Dag::rust_pilot_primitives`
//! returns a walkable type-structure declaration. The 10-element pilot
//! enumeration is NOT walkable as structured records until R2 T-Substrate's
//! 4th sub-lane lands the top-level `ValueBody::List`/aggregate extension;
//! this test pins the currently-available type-structure access and
//! explicitly records the `ValueBody::Unparsed` boundary.

use v3_compiler::dag::{Dag, TypeConnective, ValueBody};

#[test]
fn dag_new_exposes_rust_pilot_primitives_type_structure() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap must be clean for extdeps loader-close, got {:?}",
        dag.diagnostics()
    );

    let decl = dag
        .rust_pilot_primitives()
        .expect("rust_pilot_primitives must be loaded from EXTDEPS_BOOTSTRAP_FIXTURES");

    // Req 2: stable shape, walkable structurally.
    assert_eq!(
        decl.span.file, "dsl/extdeps/languages/rust/primitives.dag",
        "rust_pilot_primitives span must point at the authority file"
    );

    // Top-level declaration is `List<RustPrimitive>` — an Instantiation
    // connective parameterized over the RustPrimitive sum.
    let rust_primitive_decl_id = match &decl.connective {
        TypeConnective::Instantiation { arguments, .. } => {
            assert_eq!(
                arguments.len(),
                1,
                "rust_pilot_primitives: List<RustPrimitive> has exactly one template argument"
            );
            arguments[0].value
        }
        other => panic!(
            "rust_pilot_primitives must lower to an Instantiation (List<RustPrimitive>), got {:?}",
            other
        ),
    };

    // Walk through to the RustPrimitive sum and pin the two variants +
    // their field labels. This is the type-structure-only walk that
    // Engine sharpened-(b) Phase 1 consumes.
    let rust_primitive = dag.declaration(rust_primitive_decl_id);
    assert_eq!(rust_primitive.name.as_deref(), Some("RustPrimitive"));
    let variants = match &rust_primitive.connective {
        TypeConnective::Disj { variants } => variants,
        other => panic!("RustPrimitive must be a Disj, got {:?}", other),
    };
    let variant_labels: Vec<&str> = variants.iter().map(|f| f.label.as_str()).collect();
    assert_eq!(
        variant_labels,
        vec!["IntegerPrimitive", "NonIntegerPrimitive"],
        "pilot scope is integer/non-integer split per primitives.dag authority"
    );

    let integer_variant = dag.declaration(variants[0].ty);
    let integer_field_labels: Vec<&str> = match &integer_variant.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.as_str()).collect(),
        other => panic!(
            "IntegerPrimitive payload must be a Conj record, got {:?}",
            other
        ),
    };
    assert_eq!(
        integer_field_labels,
        vec!["target_name", "algebra", "carrier", "is_copy", "overflow"],
        "IntegerPrimitive field order is load-bearing for T-Ground L4-(C) witness consumption"
    );

    let non_integer_variant = dag.declaration(variants[1].ty);
    let non_integer_field_labels: Vec<&str> = match &non_integer_variant.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.as_str()).collect(),
        other => panic!(
            "NonIntegerPrimitive payload must be a Conj record, got {:?}",
            other
        ),
    };
    assert_eq!(
        non_integer_field_labels,
        vec!["target_name", "algebra", "carrier", "is_copy"],
        "NonIntegerPrimitive is structurally missing `overflow` (state-space discipline)"
    );
}

#[test]
fn rust_pilot_primitives_value_body_is_unparsed_until_r2_substrate_4th_sublane() {
    // Boundary pin: the 10-element pilot list stays unparsed at the
    // top-level `data ... = [...]` body because v3's `ValueBody` enum
    // lacks a top-level list/aggregate variant. When R2 T-Substrate's
    // 4th sub-lane (top-level `ValueBody::List`/aggregate extension)
    // lands, this assertion flips and Engine sharpened-(b) Phase 2 can
    // re-dispatch against structured records. See
    // `docs/r2-structure.md` 4th sub-lane scoping.
    let dag = Dag::new();
    let decl = dag.rust_pilot_primitives().expect("loaded");
    let body = decl
        .value_body
        .as_ref()
        .expect("data declarations carry a value_body");
    assert!(
        matches!(body, ValueBody::Unparsed(_)),
        "rust_pilot_primitives.value_body must stay Unparsed until ValueBody grows a \
         top-level list/aggregate variant; got {body:?}. If this test flips to failing \
         because the body is now structured, update Engine sharpened-(b) Phase 2 \
         consumers to walk the structured value and delete this assertion."
    );
}
