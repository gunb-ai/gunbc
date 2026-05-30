//! **Layer:** integration
//!
//! SG-1 receipt: `src/v4/std/target_realization.dag` — `TargetAtomRealization` canonical
//! carrier home; Rust rows in `extdeps/languages/rust.dag`; `06_translate.dag` consumer.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceField, SurfaceItem, SurfaceType};
use v3_compiler::tokenize_for_test;

const TARGET_REALIZATION_DAG: &str = include_str!("../../../../v4/std/target_realization.dag");
const TARGET_REALIZATION_PATH: &str = "src/v4/std/target_realization.dag";
const TRANSLATE_DAG: &str = include_str!("../../../../v4/compiler/06_translate.dag");
const TRANSLATE_PATH: &str = "src/v4/compiler/06_translate.dag";
const RUST_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/rust.dag");
const RUST_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/rust.dag";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn surface_declares_type(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::TypeSum { name: decl_name, .. }
                | SurfaceItem::TypeRecord { name: decl_name, .. }
                | SurfaceItem::TypeAlias { name: decl_name, .. }
                | SurfaceItem::TypeAtom { name: decl_name, .. }
                if decl_name == name
        )
    })
}

fn surface_declares_fn(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn { name: item_name, .. }
        | SurfaceItem::FnExternalBody { name: item_name, .. } => item_name == name,
        _ => false,
    })
}

fn surface_declares_data(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> bool {
    module.items.iter().any(|item| {
        matches!(item, SurfaceItem::Data { name: decl_name, .. } if decl_name == name)
    })
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
                        decimal,
                        ..
                    } => decimal.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{rendered}>")
        }
        other => format!("{other:?}"),
    }
}

fn type_record_fields(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Vec<(String, String)> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord {
                name: item_name,
                fields,
                ..
            } if item_name == name => Some(
                fields
                    .iter()
                    .map(|f: &SurfaceField| (f.name.clone(), surface_type_name(&f.ty)))
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

fn import_includes_name(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
    name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Import {
            path: import_path,
            names,
            ..
        } => {
            import_path.iter().map(String::as_str).collect::<Vec<_>>() == path
                && names.iter().any(|n| n == name)
        }
        _ => false,
    })
}

#[test]
fn v4_std_target_realization_dag_tokenizes_and_parses() {
    let _module = parse_module(TARGET_REALIZATION_DAG, TARGET_REALIZATION_PATH);
}

#[test]
fn v4_std_target_realization_declares_target_atom_realization_carrier() {
    let module = parse_module(TARGET_REALIZATION_DAG, TARGET_REALIZATION_PATH);
    assert!(
        surface_declares_type(&module, "TargetAtomRealization"),
        "{TARGET_REALIZATION_PATH}: must declare TargetAtomRealization"
    );
    let fields = type_record_fields(&module, "TargetAtomRealization");
    assert!(
        fields.iter().any(|(n, _)| n == "source_carrier"),
        "source_carrier must be Node-keyed authority"
    );
    assert!(
        fields.iter().any(|(n, ty)| n == "type_form" && ty.contains("TargetTypeExpression")),
        "type_form must use SG-2 TargetTypeExpression substrate"
    );
    assert!(
        fields.iter().any(|(n, ty)| n == "value_form" && ty.contains("TargetValueTemplate")),
        "value_form must be parametric TargetValueTemplate"
    );
}

#[test]
fn v4_std_target_realization_declares_catalog_lookup() {
    let module = parse_module(TARGET_REALIZATION_DAG, TARGET_REALIZATION_PATH);
    assert!(
        surface_declares_fn(&module, "target_atom_realization_lookup_in_catalog_node"),
        "catalog lookup must be structural over encoded row nodes"
    );
    assert!(
        surface_declares_fn(&module, "target_atom_type_spelling"),
        "type and value consumers share row authority via target_atom_type_spelling"
    );
    assert!(
        surface_declares_fn(&module, "target_atom_value_expression"),
        "value_form application must be row-driven"
    );
}

#[test]
fn v4_translate_dag_imports_target_atom_realization_consumer() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_realization"],
            "target_atom_realization_lookup_in_catalog_node"
        ),
        "{TRANSLATE_PATH}: translate must import catalog lookup from target_realization"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_atom_realizations"
        ),
        "{TRANSLATE_PATH}: translate must read atom_realizations bundle edge"
    );
    assert!(
        surface_declares_fn(&module, "translate_target_atom_realization_for_carrier"),
        "{TRANSLATE_PATH}: must declare translate_target_atom_realization_for_carrier"
    );
    assert!(
        surface_declares_fn(&module, "translate_symbol_atom_realization_type_spelling"),
        "{TRANSLATE_PATH}: type emit path must consult the same row as value emit"
    );
    assert!(
        surface_declares_fn(&module, "translate_symbol_atom_value_expression"),
        "{TRANSLATE_PATH}: value emit path must consult TargetAtomRealization row"
    );
}

#[test]
fn v4_rust_language_model_declares_target_atom_realization_rows() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    assert!(
        surface_declares_data(&module, "rust_target_atom_realization_symbol"),
        "{RUST_LANGUAGE_PATH}: Symbol row is greenfield (no parallel std_projection sentinel)"
    );
    assert!(
        surface_declares_data(&module, "rust_target_atom_realization_bool"),
        "{RUST_LANGUAGE_PATH}: Bool row must subsume rust_std_projection_bool sentinel"
    );
    assert!(
        surface_declares_data(&module, "rust_target_atom_realization_char"),
        "{RUST_LANGUAGE_PATH}: Char row must subsume rust_std_projection_char sentinel"
    );
    assert!(
        surface_declares_data(&module, "rust_target_atom_realization_catalog"),
        "{RUST_LANGUAGE_PATH}: per-language catalog prepares Python/Go parallel rows"
    );
    assert!(
        !RUST_LANGUAGE_DAG.contains("data rust_std_projection_bool:"),
        "rust_std_projection_bool sentinel must be absorbed, not left as third authority"
    );
    assert!(
        !RUST_LANGUAGE_DAG.contains("data rust_std_projection_char:"),
        "rust_std_projection_char sentinel must be absorbed, not left as third authority"
    );
}
