//! Behavioral consumer for `build_global_bare_variant_locals` / `merge_global_bare_variant_locals`
//! (sleek-wolf-190 / quiet-gull-833 cost-shape hoist).

use std::rc::Rc;

use im_rc::{vector, HashMap};
use v1_compiler::v1_compiler_infer::{
    build_global_bare_variant_locals, finish_global_bare_diagnostic_reconcile_refusal,
    merge_global_bare_variant_locals, take_merge_global_bare_per_module_scans, VariantFoldState,
};
use v1_compiler::v1_compiler_infer_env::{GlobalBareLookupState, TypeBinding};
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core::{
    has_child_named, leaf_node_with_span, make_span, Connective, Node, SubValueRelation,
};

fn fixture_disj_with_named_arm() -> (
    Rc<HashMap<String, Rc<GlobalBareLookupState>>>,
    Rc<HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>>,
    Rc<TypeBinding>,
) {
    let arm = leaf_node_with_span("Red".to_string(), make_span(20, 23));
    let disj = Rc::new(Node {
        name: "Color".to_string(),
        ident: None,
        span: make_span(0, 10),
        ident_span: Some(make_span(0, 10)),
        children: Rc::new(vector![arm]),
        connective: Connective::Disj,
        params: Rc::new(vector![]),
        inferred: None,
        return_cardinality: v1_compiler::v1_std_core::Cardinality::Required,
        uses: Rc::new(vector![]),
        body: None,
        transport: None,
        properties: Rc::new(vector![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(v1_compiler::v1_std_core::ExprData::NoExprData),
    });
    let binding = Rc::new(TypeBinding {
        name: "Red".to_string(),
        resolved: disj,
        provenance: Rc::new(SubValueRelation::SubValueUnknown),
    });
    let census = Rc::new(HashMap::from_iter([
        (
            "Red".to_string(),
            Rc::new(GlobalBareLookupState::GlobalBareUniqueBinding {
                binding: binding.clone(),
            }),
        ),
        (
            "Widget".to_string(),
            Rc::new(GlobalBareLookupState::GlobalBareUniqueBinding {
                binding: Rc::new(TypeBinding {
                    name: "Widget".to_string(),
                    resolved: leaf_node_with_span("Widget".to_string(), make_span(40, 46)),
                    provenance: Rc::new(SubValueRelation::SubValueUnknown),
                }),
            }),
        ),
        (
            "Ambiguous".to_string(),
            Rc::new(GlobalBareLookupState::GlobalBareAmbiguousBinding),
        ),
    ]));
    let source_indices = Rc::new(HashMap::new());
    (census, source_indices, binding)
}

#[test]
fn build_global_bare_variant_locals_only_admits_disj_with_named_arm() {
    let (census, source_indices, red_binding) = fixture_disj_with_named_arm();
    let variant_locals = build_global_bare_variant_locals(census.clone(), source_indices.clone());

    assert_eq!(
        variant_locals.len(),
        1,
        "only Disj types with a same-named arm enter the precomputed map"
    );
    let red = variant_locals
        .get("Red")
        .expect("Red must be present — Disj arm matches bare name");
    assert_eq!(red.name, red_binding.name);

    for (name, binding) in variant_locals.iter() {
        let census_binding = match census.get(name).map(|s| &**s) {
            Some(GlobalBareLookupState::GlobalBareUniqueBinding { binding }) => binding,
            _ => panic!("variant local {name} missing from unique census"),
        };
        assert_eq!(
            binding.resolved.connective,
            Connective::Disj,
            "{name} must be a coproduct owner"
        );
        assert!(
            has_child_named(
                census_binding.resolved.clone(),
                name.clone(),
                source_indices.clone()
            ),
            "{name} must pass has_child_named"
        );
    }

    assert!(
        !variant_locals.contains_key("Widget"),
        "non-Disj census entries must not enter variant_locals"
    );
    assert!(
        !variant_locals.contains_key("Ambiguous"),
        "ambiguous census entries must not enter variant_locals"
    );
}

#[test]
fn precomputed_merge_records_zero_has_child_named_per_module() {
    let (census, source_indices, _) = fixture_disj_with_named_arm();
    let precomputed = build_global_bare_variant_locals(census.clone(), source_indices.clone());
    let init = Rc::new(VariantFoldState {
        locals: v1_rt::rc_empty_map(),
        collision_errors: Rc::new(vector![]),
    });

    std::env::set_var("GUNBC_GLOBAL_BARE_RECEIPT_BASELINE_MERGE", "1");
    let _ = merge_global_bare_variant_locals(
        precomputed.clone(),
        init.clone(),
        source_indices.clone(),
        "mod_a".to_string(),
    );
    let _ = merge_global_bare_variant_locals(
        precomputed,
        init,
        source_indices.clone(),
        "mod_b".to_string(),
    );
    let rows = take_merge_global_bare_per_module_scans();
    std::env::remove_var("GUNBC_GLOBAL_BARE_RECEIPT_BASELINE_MERGE");

    assert_eq!(rows.len(), 2, "one receipt row per module merge");
    assert!(
        rows.iter().all(|(_, keys, has_child)| *keys == 1 && *has_child == 0),
        "precomputed merge visits hoisted keys only — has_child_named is module-invariant: {rows:?}"
    );
}

#[test]
fn baseline_merge_diagnostic_refuses_green_resolve_after_receipt() {
    std::env::set_var("GUNBC_GLOBAL_BARE_RECEIPT_BASELINE_MERGE", "1");
    let err = finish_global_bare_diagnostic_reconcile_refusal(3, None)
        .expect_err("diagnostic baseline merge must refuse green resolve after receipt");
    std::env::remove_var("GUNBC_GLOBAL_BARE_RECEIPT_BASELINE_MERGE");
    assert!(
        err.contains("GUNBC_GLOBAL_BARE_RECEIPT_BASELINE_MERGE"),
        "refusal must name the diagnostic env: {err}"
    );
}
