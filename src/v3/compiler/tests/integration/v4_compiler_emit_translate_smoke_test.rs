//! **Layer:** integration
//!
//! **T-10 / Wave-3-B:** `06_translate.dag` + `05_emit.dag` + `mvp1_rust_add_translate.dag`
//! tokenize/parse cleanly (M1(2.7) single-file path; full `compile_to_dag` deferred until
//! multi-module v4 load lands). Peers: `v4_bin_main_dag_smoke_test.rs`, `v4_extdeps_file_system_dag_smoke_test.rs`.
//!
//! **TESTING.md:** M1(2.7) `.dag` brace-bodied `fn` items surface as `FnExternalBody` (no
//! expression AST), so call-site contracts are checked via **import rows** and **declared `fn`
//! inventory** — not raw `str::contains` probes. Semantic substantiation deferred to T-22/T-14.
//!
//! **ROADMAP:** `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
//! **TASKS.md** T-10 (`src/v4/compiler/06_translate.dag`, `05_emit.dag`).
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` + INVARIANTS §SG-0 hand-authored integration test receipts row
//! land in the same PR.
//!
//! **Dissolution:** remove when translate/emit/MVP-1 surfaces are exercised only by `.dag`
//! `TestClaim` rows / a generated harness without this per-file Rust probe (or when
//! `compile_to_dag` over v4 compiler modules resolves imports without substrate collision).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{
    SurfaceExpr, SurfaceField, SurfaceItem, SurfaceType, TypeAngleArg,
};
use v3_compiler::tokenize_for_test;

const FIND_WITNESS_DAG: &str = include_str!("../../../../v4/std/find_witness.dag");
const FIND_WITNESS_PATH: &str = "src/v4/std/find_witness.dag";
const TRANSLATE_DAG: &str = include_str!("../../../../v4/compiler/06_translate.dag");
const TRANSLATE_PATH: &str = "src/v4/compiler/06_translate.dag";
const EMIT_DAG: &str = include_str!("../../../../v4/compiler/05_emit.dag");
const EMIT_PATH: &str = "src/v4/compiler/05_emit.dag";
const RUST_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/rust.dag");
const RUST_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/rust.dag";
const MVP1_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/mvp1_rust_add_translate.dag");
const MVP1_CLAIM_PATH: &str = "src/v4/test/claim/manual/mvp1_rust_add_translate.dag";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

#[test]
fn v4_find_witness_dag_tokenizes_and_parses() {
    let _module = parse_module(FIND_WITNESS_DAG, FIND_WITNESS_PATH);
}

#[test]
fn v4_find_witness_dag_declares_find_witness_entrypoint() {
    let module = parse_module(FIND_WITNESS_DAG, FIND_WITNESS_PATH);
    assert!(
        surface_declares_fn(&module, "find_witness"),
        "{FIND_WITNESS_PATH}: must declare find_witness primitive"
    );
}

#[test]
fn v4_translate_dag_tokenizes_and_parses() {
    let _module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
}

#[test]
fn v4_translate_dag_module_path_is_compiler_translate() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "compiler", "translate"]],
        "{TRANSLATE_PATH}: module authority path"
    );
}

#[test]
fn v4_translate_dag_imports_coercion_fold_delegate() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(&module, &["v4", "std", "coercion"], "coercion_fold"),
        "{TRANSLATE_PATH}: must import coercion_fold from v4.std.coercion (Practice 11)"
    );
}

#[test]
fn v4_translate_dag_imports_fold_node_traversal() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(&module, &["v4", "std", "node"], "fold_node"),
        "{TRANSLATE_PATH}: must import fold_node from v4.std.node"
    );
}

#[test]
fn v4_translate_dag_declares_coerce_grounded_node() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        surface_declares_fn(&module, "coerce_grounded_node"),
        "{TRANSLATE_PATH}: must declare coerce_grounded_node wrapper"
    );
}

#[test]
fn v4_translate_dag_declares_translate_node_and_translate() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        surface_declares_fn(&module, "translate_node"),
        "{TRANSLATE_PATH}: must declare translate_node fold entry"
    );
    assert!(
        surface_declares_fn(&module, "translate"),
        "{TRANSLATE_PATH}: must declare translate stage entry"
    );
}

#[test]
fn v4_translate_dag_imports_find_witness_types_not_inline_fn() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(&module, &["v4", "std", "find_witness"], "CandidateSet"),
        "{TRANSLATE_PATH}: may import find_witness carrier types"
    );
    assert!(
        !import_includes_name(&module, &["v4", "std", "find_witness"], "find_witness"),
        "{TRANSLATE_PATH}: must not import find_witness fn (delegates via coercion_fold)"
    );
}

#[test]
fn v4_emit_dag_tokenizes_and_parses() {
    let _module = parse_module(EMIT_DAG, EMIT_PATH);
}

#[test]
fn v4_emit_dag_module_path_is_compiler_emit() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "compiler", "emit"]],
        "{EMIT_PATH}: module authority path"
    );
}

#[test]
fn v4_emit_dag_imports_translate_stage() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert!(
        import_includes_name(&module, &["v4", "compiler", "translate"], "translate"),
        "{EMIT_PATH}: emit must import translate stage (serialize_target ∘ translate)"
    );
}

#[test]
fn v4_emit_dag_declares_emit_entrypoint() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert!(
        surface_declares_fn(&module, "emit"),
        "{EMIT_PATH}: must declare emit entrypoint"
    );
}

#[test]
fn v4_emit_dag_does_not_import_find_witness() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert!(
        !import_paths(&module)
            .iter()
            .any(|path| path.as_slice() == ["v4", "std", "find_witness"]),
        "{EMIT_PATH}: emit must not import find_witness (delegates via translate/coercion_fold)"
    );
}

#[test]
fn v4_rust_language_model_declares_t11_translation_rules() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_translation_rules"
        ),
        "{RUST_LANGUAGE_PATH}: Rust TargetModel must consume the shared translation-rules edge"
    );
    assert!(
        surface_declares_type(&module, "RustGrammarRelationRow"),
        "{RUST_LANGUAGE_PATH}: must declare the grammar relation row carrier"
    );
    assert!(
        surface_declares_fn(&module, "rust_mvp1_translation_rules_node"),
        "{RUST_LANGUAGE_PATH}: must project MVP-1 Rust translation rules into the target model"
    );
}

#[test]
fn v4_rust_integer_overflow_disposition_is_mode_aware_and_axis_bound() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    assert_eq!(
        type_record_fields(&module, "OverflowDisposition")
            .iter()
            .map(|f| (f.name.as_str(), surface_type_name(&f.ty)))
            .collect::<Vec<_>>(),
        vec![
            ("ir_carrier", "IRCarrier".to_string()),
            ("debug_default", "OverflowAction".to_string()),
            ("release_default", "OverflowAction".to_string()),
            ("overflow_checks_enabled", "OverflowAction".to_string()),
            ("overflow_checks_disabled", "OverflowAction".to_string()),
        ],
        "{RUST_LANGUAGE_PATH}: Rust overflow disposition must model debug/release defaults and explicit overflow-checks behavior"
    );
    assert_eq!(
        type_record_field_type(&module, "RustIntegerPrimitiveFacts", "overflow_disposition"),
        Some("OverflowDisposition<RustScalar>".to_string()),
        "{RUST_LANGUAGE_PATH}: integer primitive facts must carry the mode-aware overflow disposition"
    );
    assert!(
        function_body_contains_var(
            &module,
            "rust_primitive_bundle_from_integer_facts",
            "primitive_fact_axis_overflow_disposition"
        ),
        "{RUST_LANGUAGE_PATH}: integer primitive bundles must bind the overflow-disposition primitive fact axis"
    );
}

#[test]
fn v4_mvp1_rust_add_claim_tokenizes_and_parses() {
    let _module = parse_module(MVP1_CLAIM_DAG, MVP1_CLAIM_PATH);
}

#[test]
fn v4_mvp1_rust_add_claim_imports_translate_and_emit() {
    let module = parse_module(MVP1_CLAIM_DAG, MVP1_CLAIM_PATH);
    assert!(
        import_includes_name(&module, &["v4", "compiler", "translate"], "translate"),
        "{MVP1_CLAIM_PATH}: claim must import translate stage"
    );
    assert!(
        import_includes_name(&module, &["v4", "compiler", "emit"], "emit"),
        "{MVP1_CLAIM_PATH}: claim must import emit stage"
    );
}

fn module_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Module { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
}

fn import_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Import { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
}

fn import_includes_name(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
    name: &str,
) -> bool {
    module.items.iter().any(|item| {
        let SurfaceItem::Import {
            path: item_path,
            names,
            ..
        } = item
        else {
            return false;
        };
        item_path.len() == path.len()
            && item_path
                .iter()
                .zip(path.iter())
                .all(|(a, &b)| a.as_str() == b)
            && names.iter().any(|n| n == name)
    })
}

fn surface_declares_fn(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn {
            name: item_name, ..
        }
        | SurfaceItem::FnExternalBody {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

fn surface_declares_type(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::TypeAlias {
            name: item_name, ..
        }
        | SurfaceItem::TypeRecord {
            name: item_name, ..
        }
        | SurfaceItem::TypeSum {
            name: item_name, ..
        }
        | SurfaceItem::TypeAtom {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

fn type_record_fields<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> &'a [SurfaceField] {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord {
                name: item_name,
                fields,
                ..
            } if item_name == name => Some(fields.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type record `{name}`"))
}

fn type_record_field_type(
    module: &v3_compiler::parse_surface::SurfaceModule,
    record_name: &str,
    field_name: &str,
) -> Option<String> {
    type_record_fields(module, record_name)
        .iter()
        .find(|field| field.name == field_name)
        .map(|field| surface_type_name(&field.ty))
}

fn surface_type_name(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Named { name, .. } => name.clone(),
        SurfaceType::Parameterized { name, args, .. } => {
            let rendered_args = args
                .iter()
                .map(type_angle_arg_name)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{rendered_args}>")
        }
        SurfaceType::Optional { inner, .. } => format!("{}?", surface_type_name(inner)),
        SurfaceType::Arrow { inputs, output, .. } => {
            let rendered_inputs = inputs
                .iter()
                .map(surface_type_name)
                .collect::<Vec<_>>()
                .join(", ");
            format!("fn({rendered_inputs}) -> {}", surface_type_name(output))
        }
    }
}

fn type_angle_arg_name(arg: &TypeAngleArg) -> String {
    match arg {
        TypeAngleArg::TypeExpr { ty } => surface_type_name(ty),
        TypeAngleArg::WidthNatLiteral { decimal, .. } => decimal.clone(),
    }
}

fn function_body_contains_var(
    module: &v3_compiler::parse_surface::SurfaceModule,
    function_name: &str,
    var_name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn { name, body, .. } if name == function_name => {
            expr_contains_var(body, var_name)
        }
        _ => false,
    })
}

fn expr_contains_var(expr: &SurfaceExpr, var_name: &str) -> bool {
    match expr {
        SurfaceExpr::Var { name, .. } => name == var_name,
        SurfaceExpr::Path { segments, .. } => segments.iter().any(|segment| segment == var_name),
        SurfaceExpr::Call { args, .. }
        | SurfaceExpr::PathCall { args, .. }
        | SurfaceExpr::Operator { args, .. } => {
            args.iter().any(|arg| expr_contains_var(arg, var_name))
        }
        SurfaceExpr::VariantRecord { fields, .. } | SurfaceExpr::Record { fields, .. } => fields
            .iter()
            .any(|field| expr_contains_var(&field.value, var_name)),
        SurfaceExpr::Lambda { body, .. } => expr_contains_var(body, var_name),
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            expr_contains_var(cond, var_name)
                || expr_contains_var(then_branch, var_name)
                || expr_contains_var(else_branch, var_name)
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            expr_contains_var(scrutinee, var_name)
                || arms.iter().any(|arm| expr_contains_var(&arm.body, var_name))
        }
        SurfaceExpr::List { elements, .. } => {
            elements.iter().any(|element| expr_contains_var(element, var_name))
        }
        SurfaceExpr::Map { entries, .. } => entries
            .iter()
            .any(|entry| expr_contains_var(&entry.value, var_name)),
        SurfaceExpr::Literal { .. } => false,
    }
}
