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
//! land in the same PR. **This PR expansion (+0 census paths):** interim ratchet rows for
//! `v4_rust_language_model_declares_t11_translation_rules` and
//! `v4_java_language_model_declares_t11_translation_rules` (T-4 `java.dag`) in INVARIANTS.md.
//!
//! **Dissolution:** remove when translate/emit/MVP-1 surfaces are exercised only by `.dag`
//! `TestClaim` rows / a generated harness without this per-file Rust probe (or when
//! `compile_to_dag` over v4 compiler modules resolves imports without substrate collision).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{
    SurfaceExpr, SurfaceField, SurfaceItem, SurfaceLiteral, SurfaceType,
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
const JAVA_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/java.dag");
const JAVA_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/java.dag";
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
fn v4_java_language_model_declares_t11_translation_rules() {
    let module = parse_module(JAVA_LANGUAGE_DAG, JAVA_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_translation_rules"
        ),
        "{JAVA_LANGUAGE_PATH}: Java TargetModel must consume the shared translation-rules edge"
    );
    assert!(
        surface_declares_type(&module, "JavaGrammarRelationRow"),
        "{JAVA_LANGUAGE_PATH}: must declare the grammar relation row carrier"
    );
    assert!(
        surface_declares_fn(&module, "java_mvp1_translation_rules_node"),
        "{JAVA_LANGUAGE_PATH}: must project MVP-1 Java translation rules into the target model"
    );
}

#[test]
fn v4_rust_integer_primitive_facts_declares_range_source_carriers() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    let fields: Vec<(String, String)> = type_record_fields(&module, "RustIntegerPrimitiveFacts")
        .iter()
        .map(|field| (field.name.clone(), surface_type_name(&field.ty)))
        .collect();
    assert_eq!(
        fields,
        vec![
            ("surface_spelling".to_string(), "Symbol".to_string()),
            ("signedness".to_string(), "RustIntKind".to_string()),
            ("width".to_string(), "RustIntWidth".to_string()),
            ("range_min".to_string(), "RustIntegerRangeBound".to_string()),
            ("range_max".to_string(), "RustIntegerRangeBound".to_string()),
            ("overflow_release".to_string(), "OverflowAction".to_string()),
            ("std_projection".to_string(), "Symbol".to_string()),
        ],
        "{RUST_LANGUAGE_PATH}: RustIntegerPrimitiveFacts must carry declared range source facts before downstream axis binding"
    );
}

#[test]
fn v4_rust_integer_primitive_rows_populate_range_bounds() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    for (row, min, max) in [
        ("rust_facts_i8", "rust_range_min_i8", "rust_range_max_i8"),
        (
            "rust_facts_i128",
            "rust_range_min_i128",
            "rust_range_max_i128",
        ),
        ("rust_facts_u64", "rust_range_min_u64", "rust_range_max_u64"),
        (
            "rust_facts_u128",
            "rust_range_min_u128",
            "rust_range_max_u128",
        ),
        (
            "rust_facts_isize",
            "rust_range_min_isize",
            "rust_range_max_isize",
        ),
        (
            "rust_facts_usize",
            "rust_range_min_usize",
            "rust_range_max_usize",
        ),
    ] {
        let expr = data_expr(&module, row);
        assert_eq!(
            record_field_var(expr, "range_min"),
            Some(min),
            "{RUST_LANGUAGE_PATH}: {row}.range_min must point at the declared Rust Reference bound carrier"
        );
        assert_eq!(
            record_field_var(expr, "range_max"),
            Some(max),
            "{RUST_LANGUAGE_PATH}: {row}.range_max must point at the declared Rust Reference bound carrier"
        );
    }
}

#[test]
fn v4_rust_integer_range_bounds_carry_reference_values() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    assert!(
        surface_declares_type(&module, "RustIntegerRangeBound"),
        "{RUST_LANGUAGE_PATH}: Rust integer ranges must be modeled as value-carrying facts, not bare Symbols"
    );
    for (bound, value) in [
        ("rust_range_min_i8", "-128"),
        ("rust_range_max_i8", "127"),
        (
            "rust_range_min_i128",
            "-170141183460469231731687303715884105728",
        ),
        (
            "rust_range_max_i128",
            "170141183460469231731687303715884105727",
        ),
        ("rust_range_min_u64", "0"),
        ("rust_range_max_u64", "18446744073709551615"),
        ("rust_range_min_u128", "0"),
        (
            "rust_range_max_u128",
            "340282366920938463463374607431768211455",
        ),
        ("rust_range_min_isize", "pointer_width_signed_min"),
        ("rust_range_max_isize", "pointer_width_signed_max"),
        ("rust_range_min_usize", "0"),
        ("rust_range_max_usize", "pointer_width_unsigned_max"),
    ] {
        let expr = data_expr(&module, bound);
        assert_eq!(
            record_field_string(expr, "value"),
            Some(value),
            "{RUST_LANGUAGE_PATH}: range bound `{bound}` must carry the Rust Reference value, not only an opaque label"
        );
    }
}

#[test]
fn v4_rust_integer_fact_bundle_binds_range_axis() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "model_core"],
            "primitive_fact_axis_range"
        ),
        "{RUST_LANGUAGE_PATH}: Rust integer primitive bundles must import the canonical range axis"
    );
    assert!(
        surface_declares_fn(&module, "rust_integer_range_node"),
        "{RUST_LANGUAGE_PATH}: range axis must bind a dedicated range fact node, not reuse width/signedness heuristics"
    );
    assert!(
        surface_declares_fn(&module, "rust_primitive_bundle_from_integer_facts"),
        "{RUST_LANGUAGE_PATH}: integer primitive bundles must remain the producer for axis-keyed facts"
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
        .unwrap_or_else(|| panic!("missing type record {name}"))
}

fn surface_type_name(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Named { name, .. } => name.clone(),
        SurfaceType::Parameterized { name, args, .. } => {
            let rendered = args
                .iter()
                .map(|arg| match arg {
                    v3_compiler::parse_surface::TypeAngleArg::TypeExpr { ty } => {
                        surface_type_name(ty)
                    }
                    v3_compiler::parse_surface::TypeAngleArg::WidthNatLiteral {
                        decimal, ..
                    } => decimal.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{rendered}>")
        }
        SurfaceType::Optional { inner, .. } => format!("?{}", surface_type_name(inner)),
        SurfaceType::Arrow { .. } => "fn".to_string(),
    }
}

fn data_expr<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> &'a SurfaceExpr {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Data {
                name: item_name,
                body: Some(body),
                ..
            } if item_name == name => Some(body),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing data declaration body {name}"))
}

fn data_body_var<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Option<&'a str> {
    match data_expr(module, name) {
        SurfaceExpr::Var { name, .. } => Some(name.as_str()),
        _ => None,
    }
}

fn record_field_var<'a>(expr: &'a SurfaceExpr, field_name: &str) -> Option<&'a str> {
    let fields = match expr {
        SurfaceExpr::Record { fields, .. } | SurfaceExpr::VariantRecord { fields, .. } => fields,
        _ => return None,
    };
    fields
        .iter()
        .find(|field| field.name == field_name)
        .and_then(|field| match &field.value {
            SurfaceExpr::Var { name, .. } => Some(name.as_str()),
            _ => None,
        })
}

fn record_field_string<'a>(expr: &'a SurfaceExpr, field_name: &str) -> Option<&'a str> {
    let fields = match expr {
        SurfaceExpr::Record { fields, .. } | SurfaceExpr::VariantRecord { fields, .. } => fields,
        _ => return None,
    };
    fields
        .iter()
        .find(|field| field.name == field_name)
        .and_then(|field| match &field.value {
            SurfaceExpr::Literal {
                value: SurfaceLiteral::String(value),
                ..
            } => Some(value.as_str()),
            _ => None,
        })
}
