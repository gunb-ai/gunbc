//! **Layer:** integration
//!
//! T-Ground-Engine Phase-1 unblock (Path 2): `dsl/extdeps/languages/rust/
//! primitives.dag` is loaded into the bootstrap Dag; `Dag::rust_pilot_primitives`
//! returns a walkable type-structure declaration. The 10-element pilot
//! enumeration is walkable as `ValueBody::List` after R2 T-Substrate's
//! 4th sub-lane lands the top-level list extension.

use v3_compiler::dag::{Dag, FieldValue, TypeConnective, ValueBody};

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
        vec![
            "target_name",
            "algebra",
            "carrier",
            "range_min_inclusive",
            "range_max_inclusive",
            "is_copy",
            "overflow"
        ],
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
fn rust_pilot_primitives_value_body_is_structural_list() {
    let dag = Dag::new();
    let decl = dag.rust_pilot_primitives().expect("loaded");
    let rust_primitive_decl_id = match &decl.connective {
        TypeConnective::Instantiation { arguments, .. } => arguments[0].value,
        other => panic!("rust_pilot_primitives must be List<RustPrimitive>, got {other:?}"),
    };
    let TypeConnective::Disj { variants } = &dag.declaration(rust_primitive_decl_id).connective
    else {
        panic!("RustPrimitive must be a Disj");
    };
    let body = decl
        .value_body
        .as_ref()
        .expect("data declarations carry a value_body");
    let ValueBody::List(elements) = body else {
        panic!("rust_pilot_primitives.value_body must lower to ValueBody::List, got {body:?}");
    };
    // R3 Phase B (Director Path A RATIFIED at gunbc#1739 #issuecomment-4392731264):
    // 10 IntegerPrimitive (i8..i64, i128, u8..u64, u128) + 2 NonIntegerPrimitive (bool, ()).
    // u128 row unblocked by Phase A `IntervalInt::ExactInterval` BigInt host repr widening.
    assert_eq!(elements.len(), 12);
    let constructors: Vec<&str> = elements
        .iter()
        .map(|element| {
            let FieldValue::Variant { constructor, .. } = element else {
                panic!("rust_pilot_primitives elements must be variants, got {element:?}");
            };
            variants
                .iter()
                .find(|variant| variant.ty == *constructor)
                .map(|variant| variant.label.as_str())
                .expect("variant constructor belongs to RustPrimitive")
        })
        .collect();
    assert_eq!(constructors.first().copied(), Some("IntegerPrimitive"));
    assert_eq!(constructors.last().copied(), Some("NonIntegerPrimitive"));
}

#[test]
fn dag_new_exposes_std_unicode_charclass() {
    let dag = Dag::new();
    let char_class = dag
        .declaration_by_name("CharClass")
        .expect("std.unicode::CharClass must load in Dag::new()");
    assert_eq!(char_class.span.file, "dsl/std/unicode.dag");
    let variants = match &char_class.connective {
        TypeConnective::Disj { variants } => variants,
        other => panic!("CharClass must be a Disj, got {other:?}"),
    };
    let labels: Vec<&str> = variants.iter().map(|field| field.label.as_str()).collect();
    assert!(
        labels.contains(&"Whitespace") && labels.contains(&"Digit"),
        "CharClass should expose tokenizer bootstrap variants, got {labels:?}"
    );
}
