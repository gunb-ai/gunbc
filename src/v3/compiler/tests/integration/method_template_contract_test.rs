//! **Layer:** integration
//!
//! Acceptance for `MethodTemplateContract` substrate carrier in
//! `src/v3/std/emit_model.dag`. Sibling type to §6a `MethodContract` in
//! `src/v3/std/algebra.dag`; this PR lands the type only — row population
//! and `MethodTranslation` / `SimpleMethodSpec` retirement are
//! Grounding-owned follow-ups.
//!
//! Three claims per Director dispatch:
//! - `method_template_contract_distinct_from_method_contract`
//! - `method_template_contract_per_target_dag_method_unique`
//!   (pending until Grounding lands target row-list authorities after the
//!   Substrate method registry)
//! - `method_template_contract_does_not_carry_cost_data`

use std::collections::{HashMap, HashSet};
use v3_compiler::dag::{Dag, DeclarationId, FieldValue, LiteralBits, TypeConnective, ValueBody};
use v3_compiler::generated_full_bootstrap_dag;

fn conj_field_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("`{name}` is not a Conj: {other:?}"),
    }
}

fn decl_id_by_name(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"))
        .id
}

fn conj_field_ty(dag: &Dag, name: &str, field: &str) -> DeclarationId {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Conj { children } => {
            children
                .iter()
                .find(|f| f.label == field)
                .unwrap_or_else(|| panic!("`{name}` missing `{field}` field"))
                .ty
        }
        other => panic!("`{name}` is not a Conj: {other:?}"),
    }
}

#[test]
fn method_template_contract_distinct_from_method_contract() {
    let dag = generated_full_bootstrap_dag();

    let template_id = decl_id_by_name(&dag, "MethodTemplateContract");
    let metadata_id = decl_id_by_name(&dag, "MethodContract");
    assert_ne!(
        template_id, metadata_id,
        "MethodTemplateContract and §6a MethodContract must be distinct \
         declarations (P2 single-authority)"
    );

    let template_fields: HashSet<String> = conj_field_labels(&dag, "MethodTemplateContract")
        .into_iter()
        .collect();
    let metadata_fields: HashSet<String> = conj_field_labels(&dag, "MethodContract")
        .into_iter()
        .collect();
    assert!(
        template_fields.is_disjoint(&metadata_fields),
        "MethodTemplateContract and §6a MethodContract field sets must be \
         disjoint — they are orthogonal sibling facts attached to method \
         declarations (P1 step 1). template={template_fields:?} \
         metadata={metadata_fields:?}"
    );
}

#[test]
fn method_template_contract_does_not_carry_cost_data() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "MethodTemplateContract")
        .into_iter()
        .collect();

    for forbidden in ["cost_shape", "size_effect", "callback_element_position"] {
        assert!(
            !labels.contains(forbidden),
            "MethodTemplateContract carries `{forbidden}` — that field \
             belongs on §6a MethodContract (target-agnostic cost/complexity \
             metadata). Template-contract carrier holds only render-template \
             facts (P1 step 2). actual fields={labels:?}"
        );
    }

    let expected: HashSet<&str> = [
        "dag_method",
        "runtime_template",
        "emit_template",
        "wraps_result",
        "placeholder_convention",
    ]
    .into_iter()
    .collect();
    let actual: HashSet<&str> = labels.iter().map(String::as_str).collect();
    assert_eq!(
        actual, expected,
        "MethodTemplateContract field set diverged from Director-locked shape"
    );
}

#[test]
fn method_template_contract_emit_template_uses_sum_not_nullable_bridge_fields() {
    let dag = generated_full_bootstrap_dag();
    let emit_template_ty = conj_field_ty(&dag, "MethodTemplateContract", "emit_template");
    assert_eq!(
        dag.declaration(emit_template_ty).name.as_deref(),
        Some("MethodEmitTemplate"),
        "MethodTemplateContract.emit_template must point at the sum carrier, not raw String"
    );

    let TypeConnective::Disj { variants } = &dag.declaration(emit_template_ty).connective else {
        panic!("MethodEmitTemplate must be a Disj");
    };
    let labels: HashSet<&str> = variants.iter().map(|v| v.label.as_str()).collect();
    assert_eq!(
        labels,
        ["SingleTemplate", "HigherOrderTemplates"]
            .into_iter()
            .collect(),
        "MethodEmitTemplate must keep the ordinary vs higher-order split explicit"
    );

    let contract_fields: HashSet<String> = conj_field_labels(&dag, "MethodTemplateContract")
        .into_iter()
        .collect();
    for forbidden in [
        "inline_template",
        "fn_ref_template",
        "wraps_in_sharing",
        "emit_template_inline",
        "emit_template_fn_ref",
    ] {
        assert!(
            !contract_fields.contains(forbidden),
            "MethodTemplateContract grew nullable/parallel higher-order field `{forbidden}` \
             instead of using MethodEmitTemplate"
        );
    }
}

/// Walk a per-target `List<MethodTemplateContract>` declaration's value body
/// and assert that every row's `dag_method: DeclarationRef` is unique within
/// the list. Empty lists vacuously pass (no rows to compare); once Substrate's
/// method-decl registry lands and rows reference real method declarations,
/// this check becomes load-bearing.
fn assert_per_target_list_dag_method_unique(dag: &Dag, list_name: &str) {
    let decl = dag
        .declaration_by_name(list_name)
        .unwrap_or_else(|| panic!("`{list_name}` missing from full bootstrap"));
    let body = decl.value_body.as_ref().unwrap_or_else(|| {
        panic!("`{list_name}` has no value body — must be a `data` declaration")
    });
    let ValueBody::List(rows) = body else {
        panic!(
            "`{list_name}` value body must be `ValueBody::List` \
             (declared as `List<MethodTemplateContract>`); got {body:?}"
        );
    };

    let mut seen: HashSet<DeclarationId> = HashSet::new();
    for (idx, row) in rows.iter().enumerate() {
        let fields = method_template_contract_row_fields(dag, row, list_name, idx);
        let (_, dag_method) = fields
            .iter()
            .find(|(label, _)| label == "dag_method")
            .unwrap_or_else(|| panic!("row {idx} in `{list_name}` missing `dag_method` field"));
        let FieldValue::Record(method_ref_fields) = dag_method else {
            panic!(
                "row {idx} in `{list_name}`: `dag_method` must be a \
                 `FieldValue::Record` (the `MethodRef {{ decl }}` shape); \
                 got {dag_method:?}"
            );
        };
        let (_, decl_field) = method_ref_fields
            .iter()
            .find(|(label, _)| label == "decl")
            .unwrap_or_else(|| {
                panic!("row {idx} in `{list_name}`: `MethodRef` missing `decl` field")
            });
        let FieldValue::Reference(decl_id) = decl_field else {
            panic!(
                "row {idx} in `{list_name}`: `MethodRef.decl` must be a \
                 `FieldValue::Reference(DeclarationId)` pointing at a \
                 `dsl/std/methods.dag` `MethodDeclaration`; got {decl_field:?}"
            );
        };
        assert!(
            seen.insert(*decl_id),
            "duplicate `dag_method` in `{list_name}` at row {idx} — per-target \
             MethodTemplateContract rows must be unique by `dag_method`"
        );
    }
}

fn method_template_contract_row_fields<'a>(
    dag: &'a Dag,
    row: &'a FieldValue,
    list_name: &str,
    row_index: usize,
) -> &'a [(String, FieldValue)] {
    match row {
        FieldValue::Record(fields) => fields,
        FieldValue::Reference(decl_id) => {
            let decl = dag.declaration_opt(decl_id).unwrap_or_else(|| {
                panic!(
                    "row {row_index} in `{list_name}` references missing declaration {decl_id:?}"
                )
            });
            let Some(ValueBody::Structural { fields }) = decl.value_body.as_ref() else {
                panic!(
                    "row {row_index} in `{list_name}` references {decl_id:?}, \
                     but it is not a structural MethodTemplateContract data declaration"
                );
            };
            fields
        }
        _ => {
            panic!(
                "row {row_index} in `{list_name}` is neither a `FieldValue::Record` \
                 nor a declaration ref to a MethodTemplateContract row"
            );
        }
    }
}

fn method_ref_decl_from_row<'a>(
    dag: &'a Dag,
    row: &'a FieldValue,
    row_context: &str,
) -> (&'a DeclarationId, &'a FieldValue, &'a FieldValue) {
    let fields = method_template_contract_row_fields(dag, row, row_context, 0);
    let (_, dag_method) = fields
        .iter()
        .find(|(label, _)| label == "dag_method")
        .unwrap_or_else(|| panic!("{row_context}: missing `dag_method` field"));
    let FieldValue::Record(method_ref_fields) = dag_method else {
        panic!("{row_context}: `dag_method` is not MethodRef record");
    };
    let (_, decl_field) = method_ref_fields
        .iter()
        .find(|(label, _)| label == "decl")
        .unwrap_or_else(|| panic!("{row_context}: MethodRef missing `decl` field"));
    let FieldValue::Reference(decl_id) = decl_field else {
        panic!("{row_context}: MethodRef.decl is not a reference");
    };
    let (_, emit_template) = fields
        .iter()
        .find(|(label, _)| label == "emit_template")
        .unwrap_or_else(|| panic!("{row_context}: missing `emit_template` field"));
    let (_, wraps_result) = fields
        .iter()
        .find(|(label, _)| label == "wraps_result")
        .unwrap_or_else(|| panic!("{row_context}: missing `wraps_result` field"));
    (decl_id, emit_template, wraps_result)
}

fn method_emit_template_variant_label(dag: &Dag, constructor: DeclarationId) -> &str {
    let method_emit_template = dag
        .declaration_by_name("MethodEmitTemplate")
        .expect("MethodEmitTemplate");
    let TypeConnective::Disj { variants } = &method_emit_template.connective else {
        panic!("MethodEmitTemplate must be a Disj");
    };
    variants
        .iter()
        .find(|variant| variant.ty == constructor)
        .unwrap_or_else(|| panic!("unknown MethodEmitTemplate constructor {constructor:?}"))
        .label
        .as_str()
}

fn rust_method_template_rows(dag: &Dag) -> &[FieldValue] {
    let decl = dag
        .declaration_by_name("rust_method_template_contracts")
        .expect("rust_method_template_contracts");
    let body = decl.value_body.as_ref().expect("value body");
    let ValueBody::List(rows) = body else {
        panic!("rust_method_template_contracts must lower to ValueBody::List");
    };
    rows
}

#[test]
fn rust_higher_order_method_template_contracts_are_present() {
    let dag = generated_full_bootstrap_dag();
    let rows = rust_method_template_rows(&dag);
    let mut seen = HashSet::new();
    let expected_wraps: HashMap<&str, bool> = [
        ("filter_method", true),
        ("any_method", false),
        ("all_method", false),
        ("flat_map_method", true),
    ]
    .into_iter()
    .collect();

    for (idx, row) in rows.iter().enumerate() {
        let (decl_id, emit_template, wraps_result) =
            method_ref_decl_from_row(&dag, row, &format!("rust row {idx}"));
        let method_name = dag
            .declaration(*decl_id)
            .name
            .as_deref()
            .unwrap_or("<unnamed>");
        if ![
            "all_method",
            "any_method",
            "filter_method",
            "flat_map_method",
        ]
        .contains(&method_name)
        {
            continue;
        }
        seen.insert(method_name.to_string());
        let FieldValue::Variant {
            constructor,
            payload,
        } = emit_template
        else {
            panic!("{method_name}: emit_template must be MethodEmitTemplate variant");
        };
        assert_eq!(
            method_emit_template_variant_label(&dag, *constructor),
            "HigherOrderTemplates",
            "{method_name} must use the higher-order template variant"
        );
        assert_eq!(
            payload.len(),
            2,
            "{method_name}: HigherOrderTemplates must carry inline and fn-ref templates only"
        );
        assert!(
            matches!(&payload[0], FieldValue::Literal(LiteralBits::String(s)) if s.contains("{param}") && s.contains("{iter}")),
            "{method_name}: inline_template should preserve legacy inline placeholders"
        );
        assert!(
            matches!(&payload[1], FieldValue::Literal(LiteralBits::String(s)) if s.contains("{arg}")),
            "{method_name}: fn_ref_template should preserve legacy fn-ref placeholder"
        );
        let FieldValue::Literal(LiteralBits::Bool(row_wraps)) = wraps_result else {
            panic!("{method_name}: wraps_result must be the row-level Bool wrapping authority");
        };
        assert!(
            *row_wraps == expected_wraps[method_name],
            "{method_name}: wraps_result must preserve legacy higher-order wrapping bit"
        );
    }

    assert_eq!(
        seen,
        [
            "all_method",
            "any_method",
            "filter_method",
            "flat_map_method"
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        "Rust MethodTemplateContract rows must include the four higher-order methods"
    );
}

#[test]
fn method_template_contract_per_target_dag_method_unique() {
    // Phase 1 (T-Ground-LanguageSpec scope E.1): per-target row lists landed
    // at `src/v3/std/{rust,python,go}_method_template_contracts.dag`,
    // populated with registry-backed `MethodRef` rows referencing
    // `dsl/std/methods.dag`. The walker is now load-bearing: each row
    // references a top-level `MethodDeclaration` and uniqueness within each
    // list is verified structurally.
    let dag = generated_full_bootstrap_dag();
    for list_name in EXPECTED_PER_TARGET_LISTS {
        assert_per_target_list_dag_method_unique(&dag, list_name);
    }
}

const EXPECTED_PER_TARGET_LISTS: &[&str] = &[
    "rust_method_template_contracts",
    "python_method_template_contracts",
    "go_method_template_contracts",
];

/// R3 Debt-Paydown — `diagnostics_empty_after_bootstrap` ratchet for the
/// method-template-contract bootstrap fixture authorities.
///
/// Per `ROADMAP.md:502` (`go_method_template_contracts` live diagnostic
/// mismatch dissolution) and `ROADMAP.md:504,576` (general
/// `diagnostics_empty_after_bootstrap` pattern), and the
/// `docs/debt/r3-debt-paydown-ledger-2026-05-02.md:82` row, the
/// dissolution requires a single combined ratchet that asserts, for
/// every method-template-contract data declaration:
///
///   1. The declaration is present in the full bootstrap Dag.
///   2. Its `value_body` lowers to `ValueBody::List(_)` (not `Map`,
///      not the no-body fallback).
///   3. The bootstrap Dag carries an empty `diagnostics()` table.
///
/// (1) and (2) overlap with `method_template_contract_per_target_dag_method_unique`,
/// but the existing test does not pin the diagnostics-empty axis;
/// (3) overlaps with `pb1_bootstrap_full_snapshot_test::generated_full_bootstrap_snapshots_have_no_diagnostics`,
/// but that test does not anchor on the per-contract authority. This
/// test combines both axes so that a future shape regression on any
/// of the 3 contracts cannot pass over a semantically diagnostic
/// bootstrap Dag (which is the failure mode the ROADMAP row describes).
#[test]
fn bootstrap_method_template_contracts_lower_to_list_with_empty_diagnostics() {
    let dag = generated_full_bootstrap_dag();

    // Axis 3 first: assert the bootstrap Dag is diagnostic-clean
    // before per-contract structural checks. A noisy bootstrap means
    // any per-contract assertion would be passing over a Dag the
    // compiler already flagged as broken — exactly the failure mode
    // ROADMAP.md:504 names.
    assert!(
        dag.diagnostics().is_empty(),
        "diagnostics_empty_after_bootstrap ratchet failed: bootstrap Dag carries diagnostics: {:?}",
        dag.diagnostics()
    );

    // Axes 1 + 2 per contract: each declaration must be present and
    // lower to ValueBody::List. assert_per_target_list_dag_method_unique
    // already covers this with the same panics; we re-invoke it here
    // so the combined gate is one test, one failure surface, one
    // dissolution receipt.
    for list_name in EXPECTED_PER_TARGET_LISTS {
        assert_per_target_list_dag_method_unique(&dag, list_name);
    }
}
