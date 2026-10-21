use im::{HashMap, Vector};
use std::rc::Rc;
use v1_compiler::v1_compiler_emit_rust::{
    anonymous_record_struct_candidates, find_struct_name_by_fields,
};
use v1_compiler::v1_compiler_infer_emit_info::{TypeRepr, TypeSummary, TypeSummaryIndex};

fn summary(name: &str, fields: &[(&str, &str)]) -> Rc<TypeSummary> {
    let mut field_type_map = HashMap::new();
    for (field, ty) in fields {
        field_type_map.insert((*field).to_string(), (*ty).to_string());
    }
    Rc::new(TypeSummary {
        name: name.to_string(),
        key: format!("fixture.{name}"),
        repr: Rc::new(TypeRepr::StructRepr),
        field_summaries: Rc::new(HashMap::new()),
        field_type_map: Rc::new(field_type_map),
        field_import_surface_names: Rc::new(Vector::new()),
        variant_name_set: Rc::new(HashMap::new()),
        generic_param_names: Rc::new(Vector::new()),
        has_fn_fields: false,
    })
}

// Summaries are keyed by declaring identity, with the leaf index beside them.
fn summaries(rows: &[Rc<TypeSummary>]) -> Rc<TypeSummaryIndex> {
    let mut by_key = HashMap::new();
    let mut keys_by_leaf: HashMap<String, Rc<Vector<String>>> = HashMap::new();
    for row in rows {
        by_key.insert(row.key.clone(), row.clone());
        let mut keys = keys_by_leaf
            .get(&row.name)
            .map(|k| (**k).clone())
            .unwrap_or_default();
        keys.push_back(row.key.clone());
        keys_by_leaf.insert(row.name.clone(), Rc::new(keys));
    }
    Rc::new(TypeSummaryIndex {
        by_key: Rc::new(by_key),
        keys_by_leaf: Rc::new(keys_by_leaf),
    })
}

#[test]
fn alphabetical_pick_mutation_control_refuses_identical_shapes() {
    let table = summaries(&[
        summary("Alpha", &[("left", "Int"), ("right", "String")]),
        summary("Zulu", &[("left", "Int"), ("right", "String")]),
    ]);
    let fields = Rc::new(Vector::from(vec!["left".to_string(), "right".to_string()]));
    let hints = Rc::new(HashMap::new());

    assert_eq!(
        anonymous_record_struct_candidates(fields.clone(), hints.clone(), table.clone()).len(),
        2
    );
    assert_eq!(find_struct_name_by_fields(fields, hints, table), None);
}

#[test]
fn field_type_hints_still_select_one_struct() {
    let table = summaries(&[
        summary("Ints", &[("left", "Int"), ("right", "Int")]),
        summary("Strings", &[("left", "String"), ("right", "String")]),
    ]);
    let fields = Rc::new(Vector::from(vec!["left".to_string(), "right".to_string()]));
    let mut hints = HashMap::new();
    hints.insert("left".to_string(), "String".to_string());
    hints.insert("right".to_string(), "String".to_string());

    assert_eq!(
        find_struct_name_by_fields(fields, Rc::new(hints), table),
        Some("Strings".to_string())
    );
}

// Lane G (P1.5 step 1, v1-interface-cull-audit@63a464ed) deletes the shape-only fallback that
// used to run after a failed hint-aware search. The deletion rests on one case analysis, fixed
// here as specimens: a failed hint-aware search implies the shape roster is not a singleton,
// because |shape| == 1 returns before any hint filtering, and a singleton hint-aware set returns
// Present. So after a failed hint-aware search the shape-only search could only answer Absent,
// and absence stays distinct from ambiguity on the shared candidate set.

#[test]
fn shape_one_selects_before_hint_filtering() {
    // The early return that makes the deleted fallback dead: one shape candidate is selected
    // even when the hint would filter it out.
    let table = summaries(&[summary("Alpha", &[("left", "Int"), ("right", "String")])]);
    let fields = Rc::new(Vector::from(vec!["left".to_string(), "right".to_string()]));
    let mut hint_map = HashMap::new();
    hint_map.insert("left".to_string(), "String".to_string());
    let hints = Rc::new(hint_map);

    assert_eq!(
        find_struct_name_by_fields(fields.clone(), hints.clone(), table.clone()),
        Some("Alpha".to_string())
    );
    assert_eq!(
        anonymous_record_struct_candidates(fields.clone(), hints.clone(), table.clone()).len(),
        1
    );
}

#[test]
fn shape_many_hints_narrow_to_zero_keeps_absence_distinct() {
    // Shape roster is many; the hint matches no candidate. The shared helper returns the shape
    // roster unchanged in that case, so the hint-aware search yields no name, and the deleted
    // fallback -- a shape-only search over this same many roster -- could only answer Absent
    // (|shape| != 1). Many candidates remain the refusal signal, never a silent pick.
    let table = summaries(&[
        summary("Alpha", &[("left", "Int"), ("right", "String")]),
        summary("Zulu", &[("left", "Int"), ("right", "String")]),
    ]);
    let fields = Rc::new(Vector::from(vec!["left".to_string(), "right".to_string()]));
    let mut hint_map = HashMap::new();
    hint_map.insert("left".to_string(), "String".to_string());
    let hints = Rc::new(hint_map);

    assert_eq!(
        find_struct_name_by_fields(fields.clone(), hints.clone(), table.clone()),
        None
    );
    // The shared candidate set is the shape roster: the refusal signal the emitter arms read.
    assert_eq!(
        anonymous_record_struct_candidates(fields.clone(), hints.clone(), table.clone()).len(),
        2
    );
    // The shape roster the deleted fallback would have re-searched is this same many set;
    // its shape-only search answers Absent -- the arm that was deleted.
    let no_hints = Rc::new(HashMap::new());
    assert_eq!(
        anonymous_record_struct_candidates(fields.clone(), no_hints.clone(), table.clone()).len(),
        2
    );
    assert_eq!(
        find_struct_name_by_fields(fields.clone(), no_hints.clone(), table.clone()),
        None
    );
}

#[test]
fn shape_many_hints_narrow_to_many_keeps_refusal() {
    // Shape roster is many and the hint is satisfied by every candidate, so the narrowed set
    // stays many: no name is chosen, and the shared candidate count is the refusal signal the
    // emitter arms consume instead of a second search.
    let table = summaries(&[
        summary("Alpha", &[("left", "Int"), ("right", "String")]),
        summary("Zulu", &[("left", "Int"), ("right", "String")]),
    ]);
    let fields = Rc::new(Vector::from(vec!["left".to_string(), "right".to_string()]));
    let mut hint_map = HashMap::new();
    hint_map.insert("left".to_string(), "Int".to_string());
    let hints = Rc::new(hint_map);

    assert_eq!(
        find_struct_name_by_fields(fields.clone(), hints.clone(), table.clone()),
        None
    );
    assert_eq!(
        anonymous_record_struct_candidates(fields.clone(), hints.clone(), table.clone()).len(),
        2
    );
}

#[test]
fn absent_nominal_candidate_remains_distinct_from_ambiguity() {
    let table = summaries(&[summary("Other", &[("only", "Int")])]);
    let fields = Rc::new(Vector::from(vec!["left".to_string(), "right".to_string()]));
    let hints = Rc::new(HashMap::new());

    assert!(
        anonymous_record_struct_candidates(fields.clone(), hints.clone(), table.clone()).is_empty()
    );
    assert_eq!(find_struct_name_by_fields(fields, hints, table), None);
}
